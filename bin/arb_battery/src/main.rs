use arb_types::{
    HistoricalCase, AttributionResult, CandidateOpportunity, RoutePath, TokenAddress, QuoteSizeBucket, RouteLeg, GraphEdge, EventStamp, PoolUpdate, ReserveSnapshot, PoolId, PoolKind,
    ExecutionPlan, ExecutionPath, ExecutionLeg, ExpectedOutcome, SlippageGuard, MinOutConstraint, BuiltTransaction, SubmissionMode, SubmissionResult
};
use arb_config::Config;
use arb_execute::{builder::TxBuilder, submitter::Submitter, signer::Wallet};
use arb_sim::LocalSimulator;
use arb_state::StateEngine;
use arb_metrics::MetricsRegistry;
use alloy_signer_local::PrivateKeySigner;
use alloy_provider::{ProviderBuilder, Provider};
use alloy_primitives::{U256, Address, B256};
use std::fs;
use std::sync::Arc;
use std::str::FromStr;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;
use hex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Phase 13 Historical Battery Runner...");

    // Load Phase 12 env parameters
    let config = Config::load();
    let rpc_url = config.local_rpc_url.clone().expect("ANVIL_RPC_URL must be specified in .env");
    let test_pk = config.test_private_key.clone().expect("TEST_PRIVATE_KEY must be specified in .env");

    let cases_json = fs::read_to_string("fixtures/historical_cases.json")
        .expect("Failed to read fixtures/historical_cases.json. Run generator first.");
    let cases: Vec<HistoricalCase> = serde_json::from_str(&cases_json)?;

    let executor_address = Address::from_str("0x5FbDB2315678afecb367f032d93F642f64180aa3")
        .expect("Invalid default executor address");

    let signer: PrivateKeySigner = test_pk.parse()?;
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

    let provider = ProviderBuilder::new().on_http(rpc_url.parse()?);

    let mut attributions = Vec::new();
    let mut nonce = provider.get_transaction_count(executor_address).await.unwrap_or(0);

    for case in cases {
        info!("--- RUNNING CASE: {} ---", case.case_id);
        info!("Reason: {}", case.notes);

        // 1. Fetch Real Historical State from RPC
        let state_engine = Arc::new(StateEngine::new(metrics.clone()));
        
        let pool_address = Address::from_str(case.pool_ids.first().unwrap())?;
        let kind = *case.pool_kinds.first().unwrap();

        let update = match kind {
            PoolKind::ReserveBased => {
                // Fetch V2 Reserves: getReserves() -> 0x0902f1ac
                let data = hex::decode("0902f1ac")?;
                let res_raw = provider.call(&alloy_rpc_types_eth::TransactionRequest::default().to(pool_address).input(data.into())).await?;
                
                let reserve0 = u128::from_be_bytes(res_raw[16..32].try_into()?);
                let reserve1 = u128::from_be_bytes(res_raw[48..64].try_into()?);

                PoolUpdate {
                    pool_id: PoolId(pool_address.to_string()),
                    kind,
                    token0: Some(case.path_tokens[0].clone()),
                    token1: Some(case.path_tokens[1].clone()),
                    fee_bps: Some(30),
                    reserves: Some(ReserveSnapshot { reserve0, reserve1 }),
                    cl_snapshot: None,
                    cl_full_state: None,
                    stamp: EventStamp { block_number: case.fork_block_number, log_index: 0 },
                }
            }
            PoolKind::ConcentratedLiquidity => {
                // Fetch V3 slot0: slot0() -> 0x3850c7bd
                let slot0_data = hex::decode("3850c7bd")?;
                let slot0_raw = provider.call(&alloy_rpc_types_eth::TransactionRequest::default().to(pool_address).input(slot0_data.into())).await?;
                
                let sqrt_price_x96 = U256::from_be_slice(&slot0_raw[0..32]);
                let tick = i32::from_be_bytes(slot0_raw[60..64].try_into()?);

                // Fetch Liquidity: liquidity() -> 0x1a686597
                let liq_data = hex::decode("1a686597")?;
                let liq_raw = provider.call(&alloy_rpc_types_eth::TransactionRequest::default().to(pool_address).input(liq_data.into())).await?;
                let liquidity = u128::from_be_bytes(liq_raw[16..32].try_into()?);

                PoolUpdate {
                    pool_id: PoolId(pool_address.to_string()),
                    kind,
                    token0: Some(case.path_tokens[0].clone()),
                    token1: Some(case.path_tokens[1].clone()),
                    fee_bps: Some(5),
                    reserves: None,
                    cl_snapshot: Some(arb_types::CLSnapshot { sqrt_price_x96, liquidity: alloy_primitives::U128::from(liquidity), tick }),
                    cl_full_state: None,
                    stamp: EventStamp { block_number: case.fork_block_number, log_index: 0 },
                }
            }
            _ => panic!("Unsupported PoolKind in battery"),
        };

        state_engine.apply(update).await;
        let simulator = LocalSimulator::new(state_engine);
        
        // 2. Real Candidate -> Simulation
        let candidate = CandidateOpportunity {
            path: RoutePath {
                root_asset: case.root_asset.clone(),
                legs: vec![RouteLeg {
                    edge: GraphEdge {
                        pool_id: PoolId(pool_address.to_string()),
                        kind,
                        token_in: case.path_tokens[0].clone(),
                        token_out: case.path_tokens[1].clone(),
                        fee_bps: if kind == PoolKind::ReserveBased { 30 } else { 5 },
                        is_stale: false,
                    }
                }],
            },
            bucket: QuoteSizeBucket::Small,
            amount_in: case.amount_in,
            estimated_amount_out: U256::ZERO, // Will be filled by simulator
            estimated_gross_profit: U256::ZERO,
            estimated_gross_bps: 0,
            is_fresh: true,
        };

        let sim_result = simulator.validate_candidate(candidate.clone()).await;
        let sim_out = sim_result.sim_result.expected_amount_out.unwrap_or(U256::ZERO);
        let sim_profit = sim_result.sim_result.expected_profit.unwrap_or(U256::ZERO);

        // 3. Execution Plan Assembly
        let (min_amount_out, min_profit_wei) = match &case.guard_overrides {
            Some(guards) => (
                guards.min_amount_out.unwrap_or(U256::ZERO),
                guards.min_profit_wei.unwrap_or(U256::ZERO)
            ),
            None => (U256::ZERO, U256::ZERO),
        };

        let plan = ExecutionPlan {
            target_token: case.path_tokens[1].clone(),
            path: ExecutionPath {
                legs: vec![ExecutionLeg {
                    pool_id: PoolId(pool_address.to_string()),
                    token_in: case.path_tokens[0].clone(),
                    token_out: case.path_tokens[1].clone(),
                    zero_for_one: *case.leg_directions.first().unwrap(),
                }],
            },
            outcome: ExpectedOutcome {
                amount_in: case.amount_in,
                expected_amount_out: sim_out,
                expected_profit: sim_profit,
            },
            guard: SlippageGuard {
                min_out: MinOutConstraint {
                    min_amount_out,
                },
            },
            flash_loan: None,
        };

        // 4. Signed Tx Generation
        let builder = TxBuilder::new(executor_address, 31337);
        let built_tx = match builder.build_tx(&plan, nonce, 10_000_000_000, 100_000_000, 2100000) {
            Ok(tx) => tx,
            Err(e) => {
                warn!("Failed to build TX for case {}: {}", case.case_id, e);
                continue;
            }
        };

        // 5. Local Submit
        info!("Submitting built transaction for {}...", case.case_id);
        let result = submitter.submit(built_tx).await;

        // 6. Receipt processing and Honest Attribution
        let mut actual_out = U256::ZERO;
        let mut actual_profit = U256::ZERO;
        let mut gas_used = 0;
        let mut success = false;
        let mut revert_reason = None;

        if let SubmissionResult::Success { tx_hash } = result {
            nonce += 1;
            loop {
                let receipt = provider.get_transaction_receipt(B256::from_str(&tx_hash)?).await?;
                if let Some(r) = receipt {
                    gas_used = r.gas_used.try_into().unwrap_or(0);
                    success = r.status() == true;
                    if !success {
                        revert_reason = Some("Reverted on-chain".to_string());
                    } else {
                        // Honest extraction: look for Swap/Sync logs or Transfer to executor
                        // For simplicity in Phase 13 single-leg, find the last Transfer event to executor_address
                        let transfer_topic = B256::from_slice(&hex::decode("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef")?);
                        for log in r.inner.logs() {
                            if log.topics().first() == Some(&transfer_topic) {
                                // check if topic2 (to) matches executor
                                if log.topics().get(2).map(|t| Address::from_word(*t)) == Some(executor_address) {
                                    actual_out = U256::from_be_slice(log.data().data.as_ref());
                                }
                            }
                        }
                        if actual_out > case.amount_in {
                            actual_profit = actual_out - case.amount_in;
                        }
                    }
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        } else {
            success = false;
            revert_reason = Some(format!("{:?}", result));
        }

        let abs_err = if sim_profit > actual_profit { sim_profit - actual_profit } else { actual_profit - sim_profit };
        let rel_err = if sim_profit > U256::ZERO { 
            (abs_err.to::<u128>() as f64) / (sim_profit.to::<u128>() as f64)
        } else if actual_profit > U256::ZERO {
            1.0
        } else {
            0.0
        };

        let attribution = AttributionResult {
            case_id: case.case_id.clone(),
            predicted_amount_out: sim_out,
            predicted_profit: sim_profit,
            actual_amount_out: actual_out,
            actual_profit: actual_profit,
            gas_used,
            success_or_revert: success,
            revert_reason,
            absolute_error: abs_err,
            relative_error: rel_err,
        };
        attributions.push(attribution.clone());
        
        info!("Outcome for {}: success={}, gas={}, predicted_profit={}, actual_profit={}, rel_err={:.4}", 
            case.case_id, success, gas_used, sim_profit, actual_profit, rel_err);
        info!("--- END CASE ---\n");
    }

    // Phase 13 Reporting
    info!("========================================");
    info!("HISTORICAL FORK REPLAY BATTERY REPORT");
    info!("========================================");
    
    let mut total_success = 0;
    let mut total_reverts = 0;
    let mut total_gas = 0;

    for attr in &attributions {
        if attr.success_or_revert {
            total_success += 1;
        } else {
            total_reverts += 1;
        }
        total_gas += attr.gas_used;
        
        let status_str = if attr.success_or_revert { "SUCCESS" } else { "REVERT" };
        info!("[{}] {} -> predicted_profit={}, actual_profit={}, gas={}, diff_err={}", 
            attr.case_id, status_str, attr.predicted_profit, attr.actual_profit, attr.gas_used, attr.absolute_error);
        if let Some(r) = &attr.revert_reason {
            info!("  -> Reason: {}", r);
        }
    }

    info!("----------------------------------------");
    info!("Aggregate Stats:");
    info!("Cases Run: {}", attributions.len());
    info!("Success Count: {}", total_success);
    info!("Revert Count: {}", total_reverts);
    if attributions.len() > 0 {
        info!("Average Gas: {}", total_gas / attributions.len() as u64);
    }
    info!("========================================");
    
    Ok(())
}
