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
use reqwest::Url;
use alloy_primitives::{U256, Address, B256};
use std::fs;
use std::sync::Arc;
use std::str::FromStr;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- BATTERY RUNNER START ---");
    let config = Config::load();
    let rpc_url = config.local_rpc_url.clone().expect("ANVIL_RPC_URL must be specified in .env");
    let test_pk = config.test_private_key.clone().expect("TEST_PRIVATE_KEY must be specified in .env");

    let cases_path = env::var("HISTORICAL_CASES_PATH").unwrap_or_else(|_| "fixtures/historical_cases.json".to_string());
    let cases_json = fs::read_to_string(&cases_path).map_err(|e| format!("Failed to read {}: {}", cases_path, e))?;
    let cases: Vec<HistoricalCase> = serde_json::from_str(&cases_json).map_err(|e| e.to_string())?;

    let url = rpc_url.parse::<Url>().map_err(|e| e.to_string())?;
    let provider = ProviderBuilder::new().on_http(url);
    let chain_id = provider.get_chain_id().await.map_err(|e| format!("provider.get_chain_id failed: {}", e))?;
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
        false, // require_preflight
        false, // require_eth_call
        false, // require_gas_estimate
        None,  // tenderly_config
        false, // canary_live_mode_enabled
        10000, // gas_limit_multiplier_bps
        21000, // gas_limit_min
        5_000_000, // gas_limit_max
        1000,  // receipt_poll_interval_ms
        60000, // receipt_timeout_ms
    );

    let mut attributions = Vec::new();
    let fork_url = config.anvil_fork_url.clone().expect("ANVIL_FORK_URL must be specified in .env");
    
    let artifact_json = fs::read_to_string("contracts/out/ArbExecutor.sol/ArbExecutor.json").map_err(|e| e.to_string())?;
    let artifact: serde_json::Value = serde_json::from_str(&artifact_json).map_err(|e| e.to_string())?;
    let bytecode_hex = artifact["deployedBytecode"]["object"].as_str().unwrap_or("").trim_start_matches("0x");
    let bytecode = hex::decode(bytecode_hex).map_err(|e| e.to_string())?;

    for case in &cases {
        println!("[{}] Replaying...", case.case_id);
        let reset_res: Result<serde_json::Value, _> = provider.raw_request("anvil_reset".into(), (serde_json::json!({ "forking": { "jsonRpcUrl": &fork_url } }),)).await;
        if reset_res.is_err() { println!("[{}] Warning: anvil_reset failed, attempting to continue...", case.case_id); }
        let _ : serde_json::Value = provider.raw_request("evm_setAutomine".into(), (true,)).await.unwrap_or_default();
        let nonce = provider.get_transaction_count(signer_address).await.unwrap_or(0);

        let bytecode_hex_str = format!("0x{}", hex::encode(&bytecode));
        let _ : serde_json::Value = provider.raw_request("anvil_setCode".into(), (executor_address, &bytecode_hex_str)).await.unwrap_or_default();
        
        let mut owner_slot_val = [0u8; 32];
        owner_slot_val[12..].copy_from_slice(signer_address.as_slice());
        let _ : serde_json::Value = provider.raw_request(
            "anvil_setStorageAt".into(),
            (executor_address, "0x0", format!("0x{}", hex::encode(owner_slot_val))),
        ).await.unwrap_or_default();

        let mut call_req = alloy_rpc_types_eth::TransactionRequest::default();
        call_req.to = Some(executor_address.into());
        call_req.input = alloy_rpc_types_eth::TransactionInput::new(hex::decode("8da5cb5b").unwrap().into()); // owner()
        if let Ok(owner_res) = provider.call(&call_req).await {
            println!("[{}] Verified owner() slot 0 against ArbExecutor: 0x{}", case.case_id, hex::encode(&owner_res));
        }

        let hundred_eth = "0x56bc75e2d63100000";
        let _ : serde_json::Value = provider.raw_request("anvil_setBalance".into(), (signer_address, hundred_eth)).await.unwrap_or_default();
        let _ : serde_json::Value = provider.raw_request("anvil_setBalance".into(), (executor_address, hundred_eth)).await.unwrap_or_default();

        let state_engine = Arc::new(StateEngine::new(metrics.clone()));
        for (i, pool_id_str) in case.pool_ids.iter().enumerate() {
            let target_pool_address = Address::from_str(pool_id_str).map_err(|e| e.to_string())?;
            let kind = case.pool_kinds[i];
            
            let update = match kind {
                PoolKind::ReserveBased => {
                    let mut req = alloy_rpc_types_eth::TransactionRequest::default();
                    req.to = Some(target_pool_address.into());
                    req.input = alloy_rpc_types_eth::TransactionInput::new(hex::decode("0902f1ac")?.into());
                    let res_val = match provider.call(&req).await {
                        Ok(v) => v.to_vec(),
                        Err(_) => vec![0u8; 64],
                    };
                    let (r0, r1) = if res_val.len() < 64 { (0, 0) } else {
                        (u128::from_be_bytes(res_val[16..32].try_into().unwrap_or([0u8; 16])), u128::from_be_bytes(res_val[48..64].try_into().unwrap_or([0u8; 16])))
                    };
                    PoolUpdate {
                        pool_id: PoolId(target_pool_address.to_string().to_lowercase()), kind,
                        token0: Some(TokenAddress(case.path_tokens[i].0.to_lowercase())), token1: Some(TokenAddress(case.path_tokens[i+1].0.to_lowercase())),
                        fee_bps: Some(30), reserves: Some(ReserveSnapshot { reserve0: r0, reserve1: r1 }), cl_snapshot: None, cl_full_state: None,
                        stamp: EventStamp { block_number: case.fork_block_number, log_index: 0 },
                    }
                }
                PoolKind::ConcentratedLiquidity => {
                    let mut s0_req = alloy_rpc_types_eth::TransactionRequest::default();
                    s0_req.to = Some(target_pool_address.into());
                    s0_req.input = alloy_rpc_types_eth::TransactionInput::new(hex::decode("3850c7bd")?.into());
                    let s0 = provider.call(&s0_req).await.unwrap_or_default();
                    let sp = if s0.len() >= 32 { U256::from_be_slice(&s0[0..32]) } else { U256::ZERO };
                    let t = if s0.len() >= 64 { i32::from_be_bytes(s0[60..64].try_into().unwrap_or([0u8; 4])) } else { 0 };
                    let mut liq_req = alloy_rpc_types_eth::TransactionRequest::default();
                    liq_req.to = Some(target_pool_address.into());
                    liq_req.input = alloy_rpc_types_eth::TransactionInput::new(hex::decode("1a686597")?.into());
                    let liq_res = provider.call(&liq_req).await.unwrap_or_default();
                    let l = if liq_res.len() >= 32 { u128::from_be_bytes(liq_res[16..32].try_into().unwrap_or([0u8; 16])) } else { 0 };
                    PoolUpdate {
                        pool_id: PoolId(target_pool_address.to_string().to_lowercase()), kind,
                        token0: Some(TokenAddress(case.path_tokens[i].0.to_lowercase())), token1: Some(TokenAddress(case.path_tokens[i+1].0.to_lowercase())),
                        fee_bps: Some(5), cl_snapshot: Some(CLSnapshot { sqrt_price_x96: sp, liquidity: alloy_primitives::U128::from(l), tick: t }),
                        cl_full_state: None, stamp: EventStamp { block_number: case.fork_block_number, log_index: 0 }, reserves: None,
                    }
                }
                _ => continue,
            };
            state_engine.apply(update).await;
        }

        let mut path_legs = Vec::new();
        for i in 0..case.pool_ids.len() {
            let pool_kind = case.pool_kinds[i];
            path_legs.push(RouteLeg {
                edge: GraphEdge {
                    pool_id: PoolId(case.pool_ids[i].to_lowercase()), kind: pool_kind,
                    token_in: TokenAddress(case.path_tokens[i].0.to_lowercase()), token_out: TokenAddress(case.path_tokens[i+1].0.to_lowercase()),
                    fee_bps: if pool_kind == PoolKind::ReserveBased { 30 } else { 5 }, is_stale: false,
                }
            });
        }

        let simulator = LocalSimulator::new(state_engine);
        let candidate = CandidateOpportunity {
            path: RoutePath { legs: path_legs, root_asset: TokenAddress(case.root_asset.0.to_lowercase()) },
            bucket: QuoteSizeBucket::Small, amount_in: case.amount_in, estimated_amount_out: U256::ZERO, estimated_gross_profit: U256::ZERO, estimated_gross_bps: 0, is_fresh: true, route_family: arb_types::RouteFamily::Unknown,
        };

        let sim_result = simulator.validate_candidate(candidate).await;
        let sim_out = sim_result.sim_result.expected_amount_out.unwrap_or(U256::ZERO);
        let sim_profit = sim_result.sim_result.expected_profit.unwrap_or(U256::ZERO);
        let m_out = case.guard_overrides.as_ref().and_then(|g| g.min_amount_out).unwrap_or(U256::ZERO);
        let m_profit = case.guard_overrides.as_ref().and_then(|g| g.min_profit_wei).unwrap_or(U256::ZERO);

        let mut execution_legs = Vec::new();
        for i in 0..case.pool_ids.len() {
            let leg_out = sim_result.sim_result.leg_amounts_out.get(i).cloned().unwrap_or(U256::ZERO);
            if case.case_id == "case_4_mixed_v2_v3_success" {
                println!("[case 4] leg {} sim out: {}", i, leg_out);
            }
            execution_legs.push(ExecutionLeg {
                pool_id: PoolId(case.pool_ids[i].clone()), pool_kind: case.pool_kinds[i],
                token_in: case.path_tokens[i].clone(), token_out: case.path_tokens[i+1].clone(),
                zero_for_one: case.leg_directions[i], amount_out: leg_out,
            });
        }

        let plan = ExecutionPlan {
            target_token: case.path_tokens.last().cloned().unwrap(), path: ExecutionPath { legs: execution_legs },
            outcome: ExpectedOutcome { amount_in: case.amount_in, expected_amount_out: sim_out, expected_profit: sim_profit },
            guard: SlippageGuard { min_out: MinOutConstraint { min_amount_out: m_out }, min_profit_wei: m_profit }, flash_loan: None,
        };

        let builder = TxBuilder::new(executor_address, chain_id).with_force_legacy(true);
        let built_tx = builder.build_tx(&plan, nonce, 10_000_000_000, 100_000_000, 2100000)?;
        let root_token_addr = Address::from_str(&case.path_tokens[0].0)?;
        // Use a known V3 pool to prevent V2 pool reserve corruption while guaranteeing USDC
        let rich_address = Address::from_str("0xd0b53d9277642d899df5c87a3966a349a798f224").unwrap();
        let hundred_eth = "0x56bc75e2d63100000";
        let _ : serde_json::Value = provider.raw_request("anvil_setBalance".into(), (rich_address, hundred_eth)).await.unwrap_or_default();
        let _ : serde_json::Value = provider.raw_request("anvil_impersonateAccount".into(), (rich_address,)).await.unwrap_or_default();

        let mut transfer_data = hex::decode("a9059cbb")?;
        let mut to_padded = [0u8; 32]; to_padded[12..].copy_from_slice(executor_address.as_slice());
        let mut amount_padded = [0u8; 32]; amount_padded.copy_from_slice(&case.amount_in.to_be_bytes::<32>());
        transfer_data.extend_from_slice(&to_padded); transfer_data.extend_from_slice(&amount_padded);
        
        let mut tx_req = alloy_rpc_types_eth::TransactionRequest::default();
        tx_req.from = Some(rich_address); tx_req.to = Some(root_token_addr.into()); tx_req.gas = Some(100_000); tx_req.gas_price = Some(1_000_000_000);
        tx_req.input = alloy_rpc_types_eth::TransactionInput::new(transfer_data.into());
        let _ = provider.send_transaction(tx_req).await?;
        let _ : serde_json::Value = provider.raw_request("anvil_stopImpersonatingAccount".into(), (rich_address,)).await.unwrap_or_default();

        let mut bal_req = alloy_rpc_types_eth::TransactionRequest::default();
        bal_req.to = Some(root_token_addr.into());
        bal_req.input = alloy_rpc_types_eth::TransactionInput::new([hex::decode("70a08231")?, to_padded.to_vec()].concat().into());
        let bal_before = U256::from_be_slice(&provider.call(&bal_req).await?);
        let result = submitter.submit(built_tx.clone()).await;
        let _ : serde_json::Value = provider.raw_request("anvil_mine".into(), (1,)).await.unwrap_or_default();

        if let SubmissionResult::Success { tx_hash, .. } = result {
            let mut retries = 0;
            loop {
                if let Some(r) = provider.get_transaction_receipt(B256::from_str(&tx_hash)?).await? {
                    if r.inner.status() {
                        let bal_after = U256::from_be_slice(&provider.call(&bal_req).await?);
                        let actual_profit = if bal_after >= bal_before { bal_after - bal_before } else { U256::ZERO };
                        let abs_err = if actual_profit > sim_profit { actual_profit - sim_profit } else { sim_profit - actual_profit };
                        let rel_err = if !sim_profit.is_zero() { (abs_err.to_string().parse::<f64>().unwrap_or(0.0)) / (sim_profit.to_string().parse::<f64>().unwrap_or(1.0)) } else { 0.0 };
                        attributions.push(AttributionResult {
                            case_id: case.case_id.clone(), actual_status: "success".into(),
                            predicted_amount_out: sim_out, predicted_profit: sim_profit,
                            actual_amount_out: Some(bal_after), actual_profit: Some(actual_profit),
                            gas_used: r.gas_used as u64, success_or_revert: true, revert_reason: None,
                            absolute_error: abs_err, relative_error: rel_err,
                        });
                    } else {
                    let mut status = "revert".to_string();
                    let mut call_req = alloy_rpc_types_eth::TransactionRequest::default();
                    call_req.from = Some(signer_address); call_req.to = Some(executor_address.into()); call_req.input = alloy_rpc_types_eth::TransactionInput::new(built_tx.data.clone().into());
                    if let Ok(rd) = provider.call(&call_req).await {
                         println!("[{}] Revert data hex: 0x{}", case.case_id, hex::encode(&rd));
                         if rd.len() >= 4 {
                             let s = &rd[0..4];
                             if s == [0x71, 0xc4, 0xef, 0xed] { status = "slippage_revert".into(); }
                             else if s == [0x88, 0x21, 0x5f, 0x9c] { status = "no_profit_revert".into(); }
                         }
                    } else if let Err(e) = provider.call(&call_req).await {
                         let es = e.to_string();
                         println!("[{}] Call reverted with error: {}", case.case_id, es);
                         if es.contains("0x71c4efed") { status = "slippage_revert".into(); }
                         else if es.contains("0x88215f9c") { status = "no_profit_revert".into(); }
                    }
                        attributions.push(AttributionResult {
                            case_id: case.case_id.clone(), actual_status: status, predicted_amount_out: sim_out, predicted_profit: sim_profit,
                            actual_amount_out: None, actual_profit: None, gas_used: r.gas_used as u64, success_or_revert: false,
                            revert_reason: Some("On-chain Revert".into()), absolute_error: sim_profit, relative_error: 1.0,
                        });
                    }
                    break;
                }
                retries += 1; if retries > 50 { break; }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        } else {
            attributions.push(AttributionResult {
                case_id: case.case_id.clone(), actual_status: "submission_failed".into(),
                predicted_amount_out: sim_out, predicted_profit: sim_profit, actual_amount_out: None, actual_profit: None,
                gas_used: 0, success_or_revert: false, revert_reason: Some(format!("{:?}", result)),
                absolute_error: sim_profit, relative_error: 1.0,
            });
        }
    }

    println!("==========================================================================================================");
    println!("HISTORICAL BATTERY REPORT");
    println!("{:<28} | {:<16} | {:<7} | {:<12} | {:<12} | {:<8} | {:<12}", "Case ID", "Actual Status", "Match", "Pred.Profit", "Act.Profit", "Gas", "Error(%)");
    println!("----------------------------------------------------------------------------------------------------------");
    for (i, attr) in attributions.iter().enumerate() {
        let matched = if attr.actual_status == cases[i].expected_outcome { "TRUE" } else { "FALSE" };
        println!("{:<28} | {:<16} | {:<7} | {:<12} | {:<12} | {:<8} | {:<12}", attr.case_id, attr.actual_status, matched, attr.predicted_profit.to_string(), attr.actual_profit.map(|p| p.to_string()).unwrap_or("N/A".into()), if attr.gas_used > 0 { attr.gas_used.to_string() } else { "N/A".into() }, format!("{:.2}%", attr.relative_error * 100.0));
    }
    println!("==========================================================================================================");
    println!("Total Summary: {}/{} Successful Replays", attributions.iter().zip(cases.iter()).filter(|(a, c)| a.actual_status == c.expected_outcome).count(), cases.len());

    let results_json = serde_json::to_string_pretty(&attributions)?;
    fs::write("fixtures/fork_verification_results.json", results_json)?;
    println!("Results saved to fixtures/fork_verification_results.json");
    
    Ok(())
}
