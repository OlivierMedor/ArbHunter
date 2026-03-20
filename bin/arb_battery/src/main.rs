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
use alloy_eips::BlockId;
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

    let executor_address = Address::from_str("0xcf7ed3acca5a467e9e704c703e8d87f634fb0fc9").unwrap();
    let signer = test_pk.parse::<PrivateKeySigner>().map_err(|e| e.to_string())?;
    let signer_address = signer.address();
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
    let mut nonce = provider.get_transaction_count(signer_address).await.unwrap_or(0);

    for case in &cases {
        let state_engine = Arc::new(StateEngine::new(metrics.clone()));
        let target_pool_address = Address::from_str(&case.pool_ids[0]).map_err(|e| e.to_string())?;
        let kind = case.pool_kinds[0];

        let update = match kind {
            PoolKind::ReserveBased => {
                let req = alloy_rpc_types_eth::TransactionRequest::default().to(target_pool_address).input(alloy_rpc_types_eth::TransactionInput::new(alloy_primitives::Bytes::from(hex::decode("0902f1ac").unwrap())));
                let res_val = provider.call(&req).await.map_err(|e| e.to_string())?;
                if res_val.len() < 64 { 
                    println!("[{}] SKIPPED: ReserveBased call returned too short", case.case_id);
                    continue; 
                }
                let res_bytes = res_val;
                let reserve0 = u128::from_be_bytes(res_bytes[16..32].try_into().unwrap());
                let reserve1 = u128::from_be_bytes(res_bytes[48..64].try_into().unwrap());
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
                let (sqrt_price_x96, tick, liquidity) = if let Some(seed) = &case.seed_data {
                    let parts: Vec<&str> = seed.split(':').collect();
                    if parts.len() == 2 {
                        let slot0_bytes = hex::decode(&parts[0][2..]).unwrap_or_default();
                        let liq_bytes = hex::decode(&parts[1][2..]).unwrap_or_default();
                        if slot0_bytes.len() >= 64 {
                            let sp = U256::from_be_slice(&slot0_bytes[0..32]);
                            let t = i32::from_be_bytes(slot0_bytes[60..64].try_into().unwrap_or_default());
                            let l = u128::from_be_bytes(liq_bytes[16..32].try_into().unwrap_or_default());
                            (sp, t, l)
                        } else {
                            println!("[{}] SKIPPED: seed data too short", case.case_id);
                            continue;
                        }
                    } else {
                        println!("[{}] SKIPPED: seed data parts != 2", case.case_id);
                        continue;
                    }
                } else {
                    let slot0_req = alloy_rpc_types_eth::TransactionRequest::default().to(target_pool_address).input(alloy_rpc_types_eth::TransactionInput::new(alloy_primitives::Bytes::from(hex::decode("3850c7bd").unwrap())));
                    let slot0_res = provider.call(&slot0_req).await.map_err(|e| e.to_string())?;
                    if slot0_res.len() < 32 { 
                        println!("[{}] SKIPPED: slot0 call failure", case.case_id);
                        continue; 
                    }
                    let sp = U256::from_be_slice(&slot0_res[0..32]);
                    let t = i32::from_be_bytes(slot0_res[60..64].try_into().unwrap());
                    
                    let liq_req = alloy_rpc_types_eth::TransactionRequest::default().to(target_pool_address).input(alloy_rpc_types_eth::TransactionInput::new(alloy_primitives::Bytes::from(hex::decode("1a686597").unwrap())));
                    let liq_res = provider.call(&liq_req).await.map_err(|e| e.to_string())?;
                    if liq_res.len() < 32 { 
                        println!("[{}] SKIPPED: liquidity call failure", case.case_id);
                        continue; 
                    }
                    let l = u128::from_be_bytes(liq_res[16..32].try_into().unwrap());
                    (sp, t, l)
                };

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
        for i in 0..case.pool_ids.len() {
            let pool_kind = case.pool_kinds[i];
            path_legs.push(RouteLeg {
                edge: GraphEdge {
                    pool_id: PoolId(case.pool_ids[i].clone()),
                    kind: pool_kind,
                    token_in: case.path_tokens[i].clone(),
                    token_out: case.path_tokens[i+1].clone(),
                    fee_bps: if pool_kind == PoolKind::ReserveBased { 30 } else { 5 },
                    is_stale: false,
                }
            });
        }

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

        let mut execution_legs = Vec::new();
        for i in 0..case.pool_ids.len() {
            execution_legs.push(ExecutionLeg {
                pool_id: PoolId(case.pool_ids[i].clone()),
                token_in: case.path_tokens[i].clone(),
                token_out: case.path_tokens[i+1].clone(),
                zero_for_one: case.leg_directions[i],
            });
        }

        let plan = ExecutionPlan {
            target_token: case.path_tokens.last().cloned().unwrap(),
            path: ExecutionPath {
                legs: execution_legs,
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
            Err(e) => {
                println!("[{}] build_tx failed: {}", case.case_id, e);
                continue;
            }
        };

        let root_token_addr = Address::from_str(&case.path_tokens[0].0).map_err(|e| e.to_string())?;
        
        // Seed tokens for the executor:
        // We'll impersonate a known rich address for the root token.
        // For USDC/DAI/WETH on Base, the pools themselves are rich.
        let rich_address = Address::from_str(&case.pool_ids[0]).map_err(|e| e.to_string())?; 
        
        let _: serde_json::Value = provider.raw_request("anvil_impersonateAccount".into(), (rich_address,)).await.map_err(|e| e.to_string())?;
        
        // ERC20 Transfer: transfer(to, amount) -> 0xa9059cbb...
        let mut transfer_data = hex::decode("a9059cbb").unwrap();
        let mut to_padded = [0u8; 32];
        to_padded[12..].copy_from_slice(executor_address.as_slice());
        let mut amount_padded = [0u8; 32];
        amount_padded.copy_from_slice(&case.amount_in.to_be_bytes::<32>());
        transfer_data.extend_from_slice(&to_padded);
        transfer_data.extend_from_slice(&amount_padded);
        
        let transfer_req = alloy_rpc_types_eth::TransactionRequest::default()
            .from(rich_address)
            .to(root_token_addr)
            .input(alloy_rpc_types_eth::TransactionInput::new(alloy_primitives::Bytes::from(transfer_data)));
        
        provider.send_transaction(transfer_req).await.map_err(|e| e.to_string())?;
        let _: serde_json::Value = provider.raw_request("anvil_mine".into(), (1,)).await.map_err(|e| e.to_string())?;
        let _: serde_json::Value = provider.raw_request("anvil_stopImpersonatingAccount".into(), (rich_address,)).await.map_err(|e| e.to_string())?;

        let mut bal_data = hex::decode("70a08231").map_err(|e| e.to_string())?;
        let mut addr_padded = [0u8; 32];
        addr_padded[12..].copy_from_slice(executor_address.as_slice());
        bal_data.extend_from_slice(&addr_padded);
        
        let req = alloy_rpc_types_eth::TransactionRequest::default().to(root_token_addr).input(alloy_rpc_types_eth::TransactionInput::new(alloy_primitives::Bytes::from(bal_data)));
        let bal_before_raw = provider.call(&req).await.map_err(|e| e.to_string())?;
        let bal_before = U256::from_be_slice(&bal_before_raw);
        println!("[{}] bal_before={}", case.case_id, bal_before);

        let result = submitter.submit(built_tx).await;
        println!("[{}] submission_result={:?}", case.case_id, result);
        if let SubmissionResult::Success { tx_hash } = result {
            nonce += 1;
            loop {
                let receipt = provider.get_transaction_receipt(B256::from_str(&tx_hash).map_err(|e| e.to_string())?).await.map_err(|e| e.to_string())?;
                if let Some(r) = receipt {
                    if r.inner.status() {
                        let bal_after_raw = provider.call(&req).await.map_err(|e| e.to_string())?;
                        let bal_after = U256::from_be_slice(&bal_after_raw);
                        
                        let gas_price = 100_000_000_u128; // fallback
                        let gas_cost = U256::from(r.gas_used as u128 * gas_price);
                        
                        // Honest attribution math:
                        // Bal_after = Bal_before - amount_in + amount_out - gas_cost
                        // => amount_out = Bal_after - Bal_before + amount_in + gas_cost
                        let actual_amount_out = if bal_after + gas_cost + case.amount_in >= bal_before {
                            bal_after + gas_cost + case.amount_in - bal_before
                        } else {
                            U256::ZERO
                        };
                        
                        let actual_profit = if actual_amount_out > case.amount_in {
                            actual_amount_out - case.amount_in
                        } else {
                            U256::ZERO
                        };

                        let abs_err = if actual_profit > sim_profit {
                            actual_profit - sim_profit
                        } else {
                            sim_profit - actual_profit
                        };
                        
                        let rel_err = if !sim_profit.is_zero() {
                            let a: f64 = abs_err.to_string().parse().unwrap_or(0.0);
                            let s: f64 = sim_profit.to_string().parse().unwrap_or(1.0);
                            a / s
                        } else {
                            0.0
                        };

                        attributions.push(AttributionResult {
                            case_id: case.case_id.clone(),
                            predicted_amount_out: sim_out,
                            predicted_profit: sim_profit,
                            actual_amount_out: Some(actual_amount_out),
                            actual_profit: Some(actual_profit),
                            gas_used: r.gas_used as u64,
                            success_or_revert: true,
                            revert_reason: None,
                            absolute_error: abs_err,
                            relative_error: rel_err,
                        });
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
                            absolute_error: sim_profit, // Entire profit is the error
                            relative_error: 1.0,
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
