use arb_types::{
    HistoricalCase, AttributionResult, CandidateOpportunity, RoutePath, TokenAddress, QuoteSizeBucket, RouteLeg, GraphEdge, EventStamp, PoolUpdate, ReserveSnapshot, PoolId, PoolKind,
    ExecutionPlan, ExecutionPath, ExecutionLeg, ExpectedOutcome, SlippageGuard, MinOutConstraint, BuiltTransaction, SubmissionMode, SubmissionResult, GuardOverrides, CLSnapshot
};
use arb_config::Config;
use arb_execute::{builder::TxBuilder, submitter::Submitter, signer::Wallet};
use arb_sim::LocalSimulator;
use arb_state::StateEngine;
use arb_metrics::MetricsRegistry;
use alloy_signer_local::PrivateKeySigner;
use alloy_provider::{ProviderBuilder, Provider};
use reqwest::Url;
use alloy_primitives::{U256, Address, B256};
use std::fs;
use std::sync::Arc;
use std::str::FromStr;
use hex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::load();
    let rpc_url = config.local_rpc_url.clone().expect("ANVIL_RPC_URL must be specified in .env");
    let test_pk = config.test_private_key.clone().expect("TEST_PRIVATE_KEY must be specified in .env");

    let cases_json = fs::read_to_string("fixtures/historical_cases.json").map_err(|e| e.to_string())?;
    let cases: Vec<HistoricalCase> = serde_json::from_str(&cases_json).map_err(|e| e.to_string())?;

    let executor_address = Address::from_str("0x5FbDB2315678afecb367f032d93F642f64180aa3").map_err(|e| e.to_string())?;

    let signer = test_pk.parse::<PrivateKeySigner>().map_err(|e| e.to_string())?;
    let wallet = Wallet { signer };
    let metrics = Arc::new(MetricsRegistry::new());
    
    let submitter = Submitter::new(
        wallet,
        SubmissionMode::Broadcast,
        metrics.clone(),
        Some(rpc_url.clone()),
        false, 
        false, 
        false
    );

    let url = rpc_url.parse::<Url>().map_err(|e| e.to_string())?;
    let provider = ProviderBuilder::new().on_http(url);
    let mut attributions = Vec::new();
    let mut nonce = provider.get_transaction_count(executor_address).await.unwrap_or(0);

    for case in &cases {
        let state_engine = Arc::new(StateEngine::new(metrics.clone()));
        let target_pool_address = Address::from_str(&case.pool_ids[0]).map_err(|e| e.to_string())?;
        let kind = case.pool_kinds[0];

        let update = match kind {
            PoolKind::ReserveBased => {
                let log_block_hex = format!("0x{:x}", case.fork_block_number);
                let call_data = serde_json::json!({"to": target_pool_address, "data": "0x0902f1ac"});
                let res_val: serde_json::Value = provider.raw_request("eth_call".into(), (call_data, &log_block_hex)).await.map_err(|e| e.to_string())?;
                let res_str = res_val.as_str().unwrap_or("0x");
                if res_str.len() < 130 { continue; }
                let res_bytes = hex::decode(&res_str[2..]).map_err(|e| e.to_string())?;
                let reserve0 = u128::from_be_bytes(res_bytes[16..32].try_into().map_err(|e: std::array::TryFromSliceError| e.to_string())?);
                let reserve1 = u128::from_be_bytes(res_bytes[48..64].try_into().map_err(|e: std::array::TryFromSliceError| e.to_string())?);
                PoolUpdate {
                    pool_id: PoolId(target_pool_address.to_string()),
                    kind,
                    token0: Some(case.path_tokens[0].clone()),
                    token1: Some(case.path_tokens.get(1).cloned().unwrap_or(case.path_tokens[0].clone())),
                    fee_bps: Some(30),
                    reserves: Some(ReserveSnapshot { reserve0, reserve1 }),
                    cl_snapshot: None,
                    cl_full_state: None,
                    stamp: EventStamp { block_number: case.fork_block_number, log_index: 0 },
                }
            }
            PoolKind::ConcentratedLiquidity => {
                let log_block_hex = format!("0x{:x}", case.fork_block_number);
                let slot0_data = serde_json::json!({"to": target_pool_address, "data": "0x3850c7bd"});
                let slot0_val: serde_json::Value = provider.raw_request("eth_call".into(), (slot0_data, &log_block_hex)).await.map_err(|e| e.to_string())?;
                let slot0_str = slot0_val.as_str().unwrap_or("0x");
                if slot0_str.len() < 66 { continue; }
                let slot0_bytes = hex::decode(&slot0_str[2..]).map_err(|e| e.to_string())?;
                let sqrt_price_x96 = U256::from_be_slice(&slot0_bytes[0..32]);
                let tick = i32::from_be_bytes(slot0_bytes[60..64].try_into().map_err(|e: std::array::TryFromSliceError| e.to_string())?);

                let liq_data = serde_json::json!({"to": target_pool_address, "data": "0x1a686597"});
                let liq_val: serde_json::Value = provider.raw_request("eth_call".into(), (liq_data, &log_block_hex)).await.map_err(|e| e.to_string())?;
                let liq_str = liq_val.as_str().unwrap_or("0x");
                if liq_str.len() < 66 { continue; }
                let liq_bytes = hex::decode(&liq_str[2..]).map_err(|e| e.to_string())?;
                let liquidity = u128::from_be_bytes(liq_bytes[16..32].try_into().map_err(|e: std::array::TryFromSliceError| e.to_string())?);

                PoolUpdate {
                    pool_id: PoolId(target_pool_address.to_string()),
                    kind,
                    token0: Some(case.path_tokens[0].clone()),
                    token1: Some(case.path_tokens.get(1).cloned().unwrap_or(case.path_tokens[0].clone())),
                    fee_bps: Some(5),
                    reserves: None,
                    cl_snapshot: Some(CLSnapshot { sqrt_price_x96, liquidity: alloy_primitives::U128::from(liquidity), tick }),
                    cl_full_state: None,
                    stamp: EventStamp { block_number: case.fork_block_number, log_index: 0 },
                }
            }
            _ => continue,
        };

        state_engine.apply(update).await;
        let simulator = LocalSimulator::new(state_engine);
        
        let mut path_legs = Vec::new();
        path_legs.push(RouteLeg {
            edge: GraphEdge {
                pool_id: PoolId(target_pool_address.to_string()),
                kind,
                token_in: case.path_tokens[0].clone(),
                token_out: case.path_tokens.get(1).cloned().unwrap_or(case.path_tokens[0].clone()),
                fee_bps: if kind == PoolKind::ReserveBased { 30 } else { 5 },
                is_stale: false,
            }
        });

        let candidate = CandidateOpportunity {
            path: RoutePath {
                legs: path_legs,
                root_asset: case.root_asset.clone(),
            },
            bucket: QuoteSizeBucket::Small,
            amount_in: case.amount_in,
            estimated_amount_out: U256::ZERO,
            estimated_gross_profit: U256::ZERO,
            estimated_gross_bps: 0,
            is_fresh: true,
        };

        let sim_result = simulator.validate_candidate(candidate.clone()).await;
        let sim_out = sim_result.sim_result.expected_amount_out.unwrap_or(U256::ZERO);
        let sim_profit = sim_result.sim_result.expected_profit.unwrap_or(U256::ZERO);

        let m_out = case.guard_overrides.as_ref().and_then(|g| g.min_amount_out).unwrap_or(U256::ZERO);
        let _m_profit = case.guard_overrides.as_ref().and_then(|g| g.min_profit_wei).unwrap_or(U256::ZERO);

        let plan = ExecutionPlan {
            target_token: case.path_tokens.get(1).cloned().unwrap_or_else(|| case.path_tokens[0].clone()),
            path: ExecutionPath {
                legs: vec![ExecutionLeg {
                    pool_id: PoolId(target_pool_address.to_string()),
                    token_in: case.path_tokens[0].clone(),
                    token_out: case.path_tokens.get(1).cloned().unwrap_or_else(|| case.path_tokens[0].clone()),
                    zero_for_one: case.leg_directions.get(0).cloned().unwrap_or(true),
                }],
            },
            outcome: ExpectedOutcome {
                amount_in: case.amount_in,
                expected_amount_out: sim_out,
                expected_profit: sim_profit,
            },
            guard: SlippageGuard { min_out: MinOutConstraint { min_amount_out: m_out } },
            flash_loan: None,
        };

        let builder = TxBuilder::new(executor_address, 31337);
        let built_tx = match builder.build_tx(&plan, nonce, 10_000_000_000, 100_000_000, 2100000) {
            Ok(tx) => tx,
            Err(_) => continue,
        };

        let root_token_addr = Address::from_str(&case.path_tokens[0].0).map_err(|e| e.to_string())?;
        let mut bal_data = hex::decode("70a08231").map_err(|e| e.to_string())?;
        let mut addr_padded = [0u8; 32];
        addr_padded[12..].copy_from_slice(executor_address.as_slice());
        bal_data.extend_from_slice(&addr_padded);
        
        let req = alloy_rpc_types_eth::TransactionRequest::default().to(root_token_addr).input(alloy_primitives::Bytes::from(bal_data).into());
        let bal_before_raw = provider.call(&req).await.map_err(|e| e.to_string())?;
        let bal_before = U256::from_be_slice(&bal_before_raw);

        let result = submitter.submit(built_tx).await;
        if let SubmissionResult::Success { tx_hash } = result {
            nonce += 1;
            loop {
                let receipt = provider.get_transaction_receipt(B256::from_str(&tx_hash).map_err(|e| e.to_string())?).await.map_err(|e| e.to_string())?;
                if let Some(r) = receipt {
                    if r.inner.status() {
                        let bal_after_raw = provider.call(&req).await.map_err(|e| e.to_string())?;
                        let bal_after = U256::from_be_slice(&bal_after_raw);
                        if bal_after > bal_before {
                            let profit = bal_after - bal_before;
                            attributions.push(AttributionResult {
                                case_id: case.case_id.clone(),
                                predicted_amount_out: sim_out,
                                predicted_profit: sim_profit,
                                actual_amount_out: Some(case.amount_in + profit),
                                actual_profit: Some(profit),
                                gas_used: r.gas_used as u64,
                                success_or_revert: true,
                                revert_reason: None,
                                absolute_error: U256::ZERO,
                                relative_error: 0.0,
                            });
                        }
                    } else {
                        attributions.push(AttributionResult {
                            case_id: case.case_id.clone(),
                            predicted_amount_out: sim_out,
                            predicted_profit: sim_profit,
                            actual_amount_out: None,
                            actual_profit: None,
                            gas_used: r.gas_used as u64,
                            success_or_revert: false,
                            revert_reason: Some("Reverted".to_string()),
                            absolute_error: U256::ZERO,
                            relative_error: 0.0,
                        });
                    }
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }

    println!("========================================");
    println!("HISTORICAL BATTERY REPORT");
    for attr in &attributions {
        println!("[{}] success={} actual_p={:?}", attr.case_id, attr.success_or_revert, attr.actual_profit);
    }
    println!("Total Success: {}/{}", attributions.iter().filter(|a| a.success_or_revert).count(), cases.len());
    
    Ok(())
}
