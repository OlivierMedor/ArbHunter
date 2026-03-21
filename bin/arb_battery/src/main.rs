use arb_types::{
    HistoricalCase, AttributionResult, CandidateOpportunity, RoutePath, TokenAddress, QuoteSizeBucket, RouteLeg, GraphEdge, EventStamp, PoolUpdate, ReserveSnapshot, PoolId, PoolKind,
    ExecutionPlan, ExecutionPath, ExecutionLeg, ExpectedOutcome, SlippageGuard, MinOutConstraint, BuiltTransaction, SubmissionMode, SubmissionResult, GuardOverrides, CLSnapshot
};
use arb_config::Config;
use arb_execute::{builder::TxBuilder, submitter::Submitter, signer::Wallet};
use arb_sim::LocalSimulator;
use arb_state::StateEngine;
use arb_metrics::MetricsRegistry;
use alloy_signer::Signer;
use alloy_signer_local::PrivateKeySigner;
use alloy_provider::{ProviderBuilder, Provider};
use alloy_eips::BlockId;
use reqwest::Url;
use alloy_primitives::{U256, Address, B256};
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use std::str::FromStr;
use tokio::time::timeout;
use hex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- BATTERY RUNNER START ---");
    let config = Config::load();
    println!("Config loaded successfully.");
    let rpc_url = config.local_rpc_url.clone().expect("ANVIL_RPC_URL must be specified in .env");
    let test_pk = config.test_private_key.clone().expect("TEST_PRIVATE_KEY must be specified in .env");

    let cases_json = fs::read_to_string("fixtures/historical_cases.json").map_err(|e| e.to_string())?;
    let cases: Vec<HistoricalCase> = serde_json::from_str(&cases_json).map_err(|e| e.to_string())?;

    let url = rpc_url.parse::<Url>().map_err(|e| e.to_string())?;
    let provider = ProviderBuilder::new().on_http(url);
    let chain_id = provider.get_chain_id().await.map_err(|e| format!("provider.get_chain_id failed: {}", e))?;
    println!("Detected Chain ID: {}", chain_id);
    let signer = test_pk.parse::<PrivateKeySigner>().map_err(|e| e.to_string())?.with_chain_id(Some(chain_id));
    let signer_address = signer.address();
    let wallet = Wallet { signer };
    let executor_address = Address::from_str("0xcf7ed3acca5a467e9e704c703e8d87f634fb0fc9").unwrap();
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

    let fork_url = config.anvil_fork_url.clone().expect("ANVIL_FORK_URL must be specified in .env");
    
    // Read bytecode from artifact
    let artifact_json = fs::read_to_string("contracts/out/ArbExecutor.sol/ArbExecutor.json").map_err(|e| e.to_string())?;
    let artifact: serde_json::Value = serde_json::from_str(&artifact_json).map_err(|e| e.to_string())?;
    let bytecode_hex = artifact["deployedBytecode"]["object"].as_str().unwrap_or("").trim_start_matches("0x");
    let bytecode = hex::decode(bytecode_hex).map_err(|e| e.to_string())?;

    for case in &cases {
        println!("[{}] Starting replay setup...", case.case_id);
        // Reset Anvil fork
        let _: serde_json::Value = provider.raw_request("anvil_reset".into(), (serde_json::json!({ "forking": { "jsonRpcUrl": &fork_url, "blockNumber": case.fork_block_number } }),)).await.map_err(|e| format!("anvil_reset failed: {}", e))?;
        let _: serde_json::Value = provider.raw_request("evm_setAutomine".into(), (true,)).await.map_err(|e| format!("evm_setAutomine failed: {}", e))?;
        println!("[{}] Reset complete and automine enabled.", case.case_id);
        let nonce = provider.get_transaction_count(signer_address).await.unwrap_or(0);

        // Inject bytecode
        let bytecode_hex_str = format!("0x{}", hex::encode(&bytecode));
        let _: serde_json::Value = provider.raw_request(
            "anvil_setCode".into(),
            (executor_address, &bytecode_hex_str)
        ).await.map_err(|e| format!("anvil_setCode failed: {}", e))?;
        println!("[{}] Code injected.", case.case_id);

        // Set owner in storage (slot 0)
        let signer_hex = hex::encode(signer_address.as_slice());
        let owner_padded = format!("0x{:0>64}", signer_hex);
        let _: serde_json::Value = provider.raw_request(
            "anvil_setStorageAt".into(),
            (executor_address, "0x0", &owner_padded)
        ).await.map_err(|e| format!("anvil_setStorageAt failed: {}", e))?;
        println!("[{}] Storage set.", case.case_id);

        // Fund signer and executor
        let hundred_eth = "0x56bc75e2d63100000"; // 100 ETH in wei
        let _: serde_json::Value = provider.raw_request("anvil_setBalance".into(), (signer_address, hundred_eth)).await.map_err(|e| format!("anvil_setBalance (signer) failed: {}", e))?;
        let _: serde_json::Value = provider.raw_request("anvil_setBalance".into(), (executor_address, hundred_eth)).await.map_err(|e| format!("anvil_setBalance (executor) failed: {}", e))?;
        println!("[{}] Accounts funded.", case.case_id);

        let state_engine = Arc::new(StateEngine::new(metrics.clone()));
        let target_pool_address = Address::from_str(&case.pool_ids[0]).map_err(|e| e.to_string())?;
        let kind = case.pool_kinds[0];

        let update = match kind {
            PoolKind::ReserveBased => {
                let res_val = if let Some(seed) = &case.seed_data {
                    hex::decode(seed.trim_start_matches("0x")).unwrap_or_default()
                } else {
                    println!("[{}] Fetching reserves...", case.case_id);
                    let req = alloy_rpc_types_eth::TransactionRequest::default().to(target_pool_address).input(alloy_rpc_types_eth::TransactionInput::new(alloy_primitives::Bytes::from(hex::decode("0902f1ac").unwrap())));
                    provider.call(&req).await.map_err(|e| format!("provider.call(reserves) failed: {}", e))?.to_vec()
                };

                if res_val.len() < 64 { 
                    println!("[{}] SKIPPED: ReserveBased call returned too short (got {})", case.case_id, res_val.len());
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
                    println!("[{}] Fetching slot0...", case.case_id);
                    let slot0_req = alloy_rpc_types_eth::TransactionRequest::default().to(target_pool_address).input(alloy_rpc_types_eth::TransactionInput::new(alloy_primitives::Bytes::from(hex::decode("3850c7bd").unwrap())));
                    let slot0_res = provider.call(&slot0_req).await.map_err(|e| format!("provider.call(slot0) failed: {}", e))?;
                    if slot0_res.len() < 32 { 
                        println!("[{}] SKIPPED: slot0 call failure", case.case_id);
                        continue; 
                    }
                    let sp = U256::from_be_slice(&slot0_res[0..32]);
                    let t = i32::from_be_bytes(slot0_res[60..64].try_into().unwrap());
                    
                    println!("[{}] Fetching liquidity...", case.case_id);
                    let liq_req = alloy_rpc_types_eth::TransactionRequest::default().to(target_pool_address).input(alloy_rpc_types_eth::TransactionInput::new(alloy_primitives::Bytes::from(hex::decode("1a686597").unwrap())));
                    let liq_res = provider.call(&liq_req).await.map_err(|e| format!("provider.call(liquidity) failed: {}", e))?;
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
        let m_profit = case.guard_overrides.as_ref().and_then(|g| g.min_profit_wei).unwrap_or(U256::ZERO);

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
            guard: SlippageGuard { 
                min_out: MinOutConstraint { min_amount_out: m_out },
                min_profit_wei: m_profit,
            },
            flash_loan: None,
        };

        let builder = TxBuilder::new(executor_address, chain_id).with_force_legacy(true);
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
        println!("[{}] Impersonating rich account...", case.case_id);
        let _: serde_json::Value = provider.raw_request("anvil_setBalance".into(), (rich_address, U256::from(100_000_000_000_000_000_000_u128))).await.map_err(|e| format!("anvil_setBalance(rich) failed: {}", e))?;
        let _: serde_json::Value = provider.raw_request("anvil_impersonateAccount".into(), (rich_address,)).await.map_err(|e| format!("anvil_impersonateAccount failed: {}", e))?;
        
        // ERC20 Transfer: transfer(to, amount) -> 0xa9059cbb...
        let mut transfer_data = hex::decode("a9059cbb").unwrap();
        let mut to_padded = [0u8; 32];
        to_padded[12..].copy_from_slice(executor_address.as_slice());
        let mut amount_padded = [0u8; 32];
        amount_padded.copy_from_slice(&case.amount_in.to_be_bytes::<32>());
        transfer_data.extend_from_slice(&to_padded);
        transfer_data.extend_from_slice(&amount_padded);
        
        let mut transfer_req = alloy_rpc_types_eth::TransactionRequest::default();
        transfer_req.from = Some(rich_address);
        transfer_req.to = Some(alloy_primitives::TxKind::Call(root_token_addr));
        transfer_req.gas = Some(100_000);
        transfer_req.gas_price = Some(1_000_000_000);
        transfer_req.input = alloy_rpc_types_eth::TransactionInput::new(alloy_primitives::Bytes::from(transfer_data));

        println!("[{}] Sending seed tokens (impersonated)...", case.case_id);
        let fut = provider.raw_request::<_, serde_json::Value>("eth_sendTransaction".into(), (transfer_req.clone(),));
        match tokio::time::timeout(std::time::Duration::from_secs(30), fut).await {
            Ok(Ok(_)) => println!("[{}] Seed tokens sent.", case.case_id),
            Ok(Err(e)) => return Err(format!("eth_sendTransaction(seed) failed: {}", e).into()),
            Err(_) => println!("[{}] Seed tokens timed out (continuing anyway)...", case.case_id),
        }
        let _: serde_json::Value = provider.raw_request::<_, serde_json::Value>("anvil_stopImpersonatingAccount".into(), (rich_address,)).await.map_err(|e| format!("anvil_stopImpersonatingAccount failed: {}", e))?;

        println!("[{}] Checking balance before...", case.case_id);
        let mut bal_data = hex::decode("70a08231").map_err(|e| e.to_string())?;
        let mut addr_padded = [0u8; 32];
        addr_padded[12..].copy_from_slice(executor_address.as_slice());
        bal_data.extend_from_slice(&addr_padded);
        
        let req = alloy_rpc_types_eth::TransactionRequest::default().to(root_token_addr).input(alloy_rpc_types_eth::TransactionInput::new(alloy_primitives::Bytes::from(bal_data)));
        let bal_before_raw = provider.call(&req).await.map_err(|e| format!("provider.call(bal_before) failed: {}", e))?;
        let bal_before = U256::from_be_slice(&bal_before_raw);
        println!("[{}] Balance before: {}. Submitting trade (nonce: {}, gas: {})...", case.case_id, bal_before, built_tx.nonce, built_tx.gas_limit);
        let result = submitter.submit(built_tx).await;
        println!("[{}] Submission result: {:?}. Mining result block...", case.case_id, result);
        let _: serde_json::Value = provider.raw_request::<_, serde_json::Value>("anvil_mine".into(), (1,)).await.map_err(|e| format!("anvil_mine failed: {}", e))?;

        if let SubmissionResult::Success { tx_hash } = result {
            let mut retries = 0;
            loop {
                let receipt = provider.get_transaction_receipt(B256::from_str(&tx_hash).map_err(|e| format!("B256::from_str(tx_hash) failed: {}", e))?).await.map_err(|e| format!("provider.get_transaction_receipt failed: {}", e))?;
                if let Some(r) = receipt {
                    if r.inner.status() {
                        let bal_after_raw = provider.call(&req).await.map_err(|e| format!("provider.call(bal_after) failed: {}", e))?;
                        let bal_after = U256::from_be_slice(&bal_after_raw);
                        
                        let gas_price = 100_000_000_u128; // fallback
                        let gas_cost = U256::from(r.gas_used as u128 * gas_price);
                        
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
                            revert_reason: Some("On-chain Revert".to_string()),
                            absolute_error: sim_profit,
                            relative_error: 1.0,
                        });
                    }
                    break;
                }
                retries += 1;
                if retries > 50 { 
                    println!("[{}] Receipt timeout after 50 retries.", case.case_id);
                    break; 
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        } else {
            let reason = match result {
                SubmissionResult::Failed(arb_types::SubmissionFailureReason::ExecutionReverted(msg)) => msg,
                SubmissionResult::Failed(arb_types::SubmissionFailureReason::PreflightFailed(msg)) => msg,
                _ => format!("{:?}", result),
            };
            attributions.push(AttributionResult {
                case_id: case.case_id.clone(),
                predicted_amount_out: sim_out,
                predicted_profit: sim_profit,
                actual_amount_out: None,
                actual_profit: None,
                gas_used: 0,
                success_or_revert: false,
                revert_reason: Some(reason),
                absolute_error: sim_profit,
                relative_error: 1.0,
            });
        }
    }

    println!("========================================");
    println!("HISTORICAL BATTERY REPORT");
    println!("{:<25} | {:<7} | {:<12} | {:<12} | {:<20}", "Case ID", "Success", "Profit", "Error(%)", "Note/Revert");
    println!("--------------------------------------------------------------------------------------------------");
    for attr in &attributions {
        let success_str = if attr.success_or_revert { "TRUE" } else { "FALSE" };
        let profit_str = attr.actual_profit.map(|p| p.to_string()).unwrap_or_else(|| "N/A".to_string());
        let error_str = format!("{:.2}%", attr.relative_error * 100.0);
        let note = attr.revert_reason.as_deref().unwrap_or("");
        println!("{:<25} | {:<7} | {:<12} | {:<12} | {:<20}", attr.case_id, success_str, profit_str, error_str, note);
    }
    println!("========================================");
    println!("Total Summary: {}/{} Successful Replays", attributions.iter().filter(|a| a.success_or_revert).count(), cases.len());
    
    Ok(())
}
