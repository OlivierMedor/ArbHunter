use std::str::FromStr;
use std::sync::Arc;
use std::io::Write;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error};
use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;

use arb_config::Config;
use arb_metrics::MetricsRegistry;
use arb_ingest::DexDecoder;
use arb_state::StateEngine;
use arb_route::{RouteGraph, CandidateGenerator};
use alloy_primitives::{U256 as AlloyU256, U128 as AlloyU128};
use arb_types::{
    EventStamp, IngestEvent, PoolId, PoolKind, PoolUpdate, TokenAddress, QuoteSizeBucket,
    HistoricalReplayResult, HistoricalReplaySummary, HistoricalRecheckResult, HistoricalDriftSummary,
    ForkVerificationResult, PendingLogEvent, RoutePath, ShadowRecheckResult, CLSnapshot, ReserveSnapshot, RouteLeg
};
use ethers::prelude::*;
use warp::Filter;

abigen!(
    IERC20Pool,
    r#"[
        function token0() external view returns (address)
        function token1() external view returns (address)
        function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)
    ]"#
);

abigen!(
    IUniswapV3Pool,
    r#"[
        function slot0() external view returns (uint160 sqrtPriceX96, int24 tick, uint16 observationIndex, uint16 observationCardinality, uint16 observationCardinalityNext, uint8 feeProtocol, bool unlocked)
        function liquidity() external view returns (uint128)
        function fee() external view returns (uint24)
        function token0() external view returns (address)
        function token1() external view returns (address)
    ]"#
);

#[derive(Debug, Clone)]
struct ReplayStats {
    total_logs: u64,
    candidates_considered: u64,
    would_trade: u64,
    rechecks_total: u64,
    still_profitable: u64,
}

impl ReplayStats {
    fn new() -> Self {
        Self {
            total_logs: 0,
            candidates_considered: 0,
            would_trade: 0,
            rechecks_total: 0,
            still_profitable: 0,
        }
    }
}

fn eth_u256_to_alloy(eth: ethers::types::U256) -> AlloyU256 {
    let mut bytes = [0u8; 32];
    eth.to_big_endian(&mut bytes);
    AlloyU256::from_be_bytes(bytes)
}

fn eth_u128_to_alloy(eth_u128: u128) -> AlloyU128 {
    AlloyU128::from(eth_u128)
}

fn normalize_addr(addr: ethers::types::Address) -> String {
    format!("{:?}", addr).to_lowercase()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    let config = Config::load();
    if !config.enable_historical_shadow_replay {
        info!("Historical Shadow Replay is disabled.");
        return Ok(());
    }

    info!("Starting Phase 18 Quoter Execution Calibration Replay...");

    let metrics = Arc::new(MetricsRegistry::new());
    let rpc_url = &config.quicknode_http_url;
    let provider = Arc::new(Provider::<Http>::try_from(rpc_url)?);

    let metrics_state = metrics.clone();
    let metrics_port = config.historical_replay_metrics_port;
    tokio::spawn(async move {
        let metrics_route = warp::path("metrics")
            .map(move || metrics_state.gather_metrics());
        warp::serve(metrics_route).run(([0, 0, 0, 0], metrics_port)).await;
    });

    let (start_block, end_block) = if let (Some(s), Some(e)) = (config.historical_replay_start_block, config.historical_replay_end_block) {
        (s, e)
    } else {
        let latest = provider.get_block_number().await?.as_u64();
        (latest.saturating_sub(1000), latest)
    };

    info!("Replaying {}..{} ({} blocks)", start_block, end_block, end_block - start_block + 1);

    let decoder = DexDecoder::new(metrics.clone());
    let state_engine = Arc::new(StateEngine::new(metrics.clone()));
    let generator = CandidateGenerator::new(state_engine.clone());
    
    // Normalize root asset
    let trimmed_root = config.root_asset.trim();
    info!("Root asset from config: '{}'", trimmed_root);
    let root_addr = ethers::types::Address::from_str(trimmed_root)
        .map_err(|e| anyhow::anyhow!("Invalid ROOT_ASSET '{}': {:?}", trimmed_root, e))?;
    let root_asset = TokenAddress(normalize_addr(root_addr));
    info!("Root asset (normalized): {}", root_asset.0);
    
    let mut stats = ReplayStats::new();
    let mut pending_rechecks: BTreeMap<u64, Vec<HistoricalReplayResult>> = BTreeMap::new();
    let mut all_replay_results: Vec<HistoricalReplayResult> = Vec::new();
    let mut pool_tokens: HashMap<ethers::types::Address, (TokenAddress, TokenAddress)> = HashMap::new();

    let v2_sync_sig = H256::from_slice(&hex::decode("1c91f030eb7c0a042c0211d40a5440311dec3b1285bc035ede49007f502574e4").expect("Valid v2_sync_sig hex"));
    let aero_swap_sig = H256::from_slice(&hex::decode("d78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822").expect("Valid aero_swap_sig hex"));
    let v3_swap_sig = H256::from_slice(&hex::decode("c42079f94a6350d7e5735f2399b6d8de98a486f7af7d160cfd333044e7c75db1").expect("Valid v3_swap_sig hex"));

    let chunk_size = 500;
    let mut current_block_ptr = start_block;

    // Phase 18: Enhanced Warm-up Discovery (1,000 block lookback)
    let warmup_start = start_block.saturating_sub(1000);
    info!("Running warmup discovery from {} to {}...", warmup_start, start_block);
    
    let mut warmup_ptr = warmup_start;
    while warmup_ptr < start_block {
        let warmup_end = (warmup_ptr + chunk_size - 1).min(start_block - 1);
        for sig in [v2_sync_sig, aero_swap_sig, v3_swap_sig] {
            let filter = ethers::types::Filter::new()
                .from_block(warmup_ptr)
                .to_block(warmup_end)
                .topic0(ValueOrArray::Value(sig));
            if let Ok(logs) = provider.get_logs(&filter).await {
                for log in logs {
                    // Fast discovery (reserves only, no graph build)
                    let pool_addr = log.address;
                    if !pool_tokens.contains_key(&pool_addr) {
                        let v2_pool = IERC20Pool::new(pool_addr, provider.clone());
                        let t0_res = v2_pool.token_0().call().await;
                        let t1_res = v2_pool.token_1().call().await;
                        if let (Ok(t0_raw), Ok(t1_res_raw)) = (t0_res, t1_res) {
                            let t0_ta = TokenAddress(normalize_addr(t0_raw));
                            let t1_ta = TokenAddress(normalize_addr(t1_res_raw));
                            pool_tokens.insert(pool_addr, (t0_ta.clone(), t1_ta.clone()));
                            
                            // Apply minimal state with initial reserves/liquidity
                            let mut update = PoolUpdate {
                                pool_id: PoolId(normalize_addr(pool_addr)),
                                kind: if log.topics[0] == v3_swap_sig { PoolKind::ConcentratedLiquidity } else { PoolKind::ReserveBased },
                                token0: Some(t0_ta),
                                token1: Some(t1_ta),
                                fee_bps: Some(30),
                                reserves: None,
                                cl_snapshot: None,
                                cl_full_state: None,
                                stamp: EventStamp { block_number: log.block_number.unwrap().as_u64(), log_index: 0 },
                            };
                            
                            if update.kind == PoolKind::ReserveBased {
                                if let Ok((r0, r1, _)) = v2_pool.get_reserves().call().await {
                                    update.reserves = Some(ReserveSnapshot { reserve0: r0, reserve1: r1 });
                                }
                            } else {
                                let v3_pool = IUniswapV3Pool::new(pool_addr, provider.clone());
                                let s0_res = v3_pool.slot_0().call().await;
                                let liq_res = v3_pool.liquidity().call().await;
                                if let (Ok(s0), Ok(liq)) = (s0_res, liq_res) {
                                    update.cl_snapshot = Some(CLSnapshot {
                                        sqrt_price_x96: eth_u256_to_alloy(s0.0),
                                        liquidity: eth_u128_to_alloy(liq),
                                        tick: s0.1,
                                    });
                                }
                            }
                            state_engine.apply(update).await;
                        }
                    }
                }
            }
        }
        warmup_ptr += chunk_size;
        info!("  Warmup Progress: block {}/{} ({} pools discovered)", warmup_ptr, start_block, pool_tokens.len());
    }
    info!("Warmup discovery finished. Total pools discovered: {}", pool_tokens.len());
    
    let mut all_replay_results = Vec::new();
    let mut pending_rechecks: BTreeMap<u64, Vec<HistoricalReplayResult>> = BTreeMap::new();
    let generator = CandidateGenerator::new(state_engine.clone());
    
    let mut target_export_file = if let Some(path) = std::env::var("EXPORT_CANDIDATES_PATH").ok() {
        use std::io::Write;
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        info!("Streaming candidates to {}", path);
        Some(std::io::BufWriter::new(file))
    } else {
        None
    };

    while current_block_ptr <= end_block {
        let chunk_end = (current_block_ptr + chunk_size - 1).min(end_block);
        
        let mut logs = Vec::new();
        for sig in [v2_sync_sig, aero_swap_sig, v3_swap_sig] {
            let filter = ethers::types::Filter::new()
                .from_block(current_block_ptr)
                .to_block(chunk_end)
                .topic0(ValueOrArray::Value(sig));
            if let Ok(l) = provider.get_logs(&filter).await {
                logs.extend(l);
            }
        }
        
        let mut logs_by_block: BTreeMap<u64, Vec<ethers::types::Log>> = BTreeMap::new();
        for log in logs {
            if let Some(bn) = log.block_number {
                logs_by_block.entry(bn.as_u64()).or_default().push(log);
            }
        }

        for block_num in current_block_ptr..=chunk_end {
            let mut block_trades = 0;
            if let Some(block_logs) = logs_by_block.get(&block_num) {
                stats.total_logs += block_logs.len() as u64;
                for log in block_logs {
                    let pool_addr = log.address;
                    let topic0 = log.topics.get(0).cloned();
                    
                    if let Some(t0) = topic0 {
                        let is_sync = t0 == v2_sync_sig;
                        let is_v3_swap = t0 == v3_swap_sig;
                        let is_aero_swap = t0 == aero_swap_sig;

                        if is_sync || is_v3_swap || is_aero_swap {
                            if !pool_tokens.contains_key(&pool_addr) {
                                let v2_pool = IERC20Pool::new(pool_addr, provider.clone());
                                let call_t0 = v2_pool.token_0();
                                let call_t1 = v2_pool.token_1();
                                match tokio::join!(call_t0.call(), call_t1.call()) {
                                    (Ok(t0_raw), Ok(t1_raw)) => {
                                        let t0_ta = TokenAddress(normalize_addr(t0_raw));
                                        let t1_ta = TokenAddress(normalize_addr(t1_raw));
                                        pool_tokens.insert(pool_addr, (t0_ta.clone(), t1_ta.clone()));
                                        
                                        if is_sync || is_aero_swap {
                                            let call_r = v2_pool.get_reserves();
                                            if let Ok((r0, r1, _)) = call_r.call().await {
                                                state_engine.apply(PoolUpdate {
                                                    pool_id: PoolId(normalize_addr(pool_addr)),
                                                    kind: PoolKind::ReserveBased,
                                                    token0: Some(t0_ta),
                                                    token1: Some(t1_ta),
                                                    fee_bps: Some(30),
                                                    reserves: Some(ReserveSnapshot { reserve0: r0, reserve1: r1 }),
                                                    cl_snapshot: None,
                                                    cl_full_state: None,
                                                    stamp: EventStamp { block_number: block_num - 1, log_index: 0 },
                                                }).await;
                                            }
                                        } else if is_v3_swap {
                                            let v3_pool = IUniswapV3Pool::new(pool_addr, provider.clone());
                                            let call_s = v3_pool.slot_0();
                                            let call_l = v3_pool.liquidity();
                                            let call_f = v3_pool.fee();
                                            match tokio::join!(call_s.call(), call_l.call(), call_f.call()) {
                                                (Ok(slot0), Ok(liq), Ok(fee)) => {
                                                    state_engine.apply(PoolUpdate {
                                                        pool_id: PoolId(normalize_addr(pool_addr)),
                                                        kind: PoolKind::ConcentratedLiquidity,
                                                        token0: Some(t0_ta),
                                                        token1: Some(t1_ta),
                                                        fee_bps: Some(fee as u32),
                                                        reserves: None,
                                                        cl_snapshot: Some(CLSnapshot {
                                                            sqrt_price_x96: eth_u256_to_alloy(slot0.0),
                                                            liquidity: eth_u128_to_alloy(liq),
                                                            tick: slot0.1,
                                                        }),
                                                        cl_full_state: None,
                                                        stamp: EventStamp { block_number: block_num - 1, log_index: 0 },
                                                    }).await;
                                                }
                                                _ => {},
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            if let Some((t0_a, t1_a)) = pool_tokens.get(&pool_addr) {
                                let pl = PendingLogEvent {
                                    address: normalize_addr(pool_addr),
                                    topics: log.topics.iter().map(|t| format!("{:?}", t).to_lowercase()).collect(),
                                    data: format!("0x{}", hex::encode(&log.data)),
                                    transaction_hash: format!("{:?}", log.transaction_hash.unwrap_or_default()).to_lowercase(),
                                    block_number: log.block_number.unwrap_or_default().as_u64(),
                                    log_index: log.log_index.unwrap_or_default().as_u32(),
                                };
                                if let Some(mut update) = decoder.decode_log(&pl) {
                                    update.token0 = Some(t0_a.clone());
                                    update.token1 = Some(t1_a.clone());
                                    state_engine.apply(update).await;
                                }
                            }
                        }
                    }
                }
            }

            if let Some(to_recheck) = pending_rechecks.remove(&block_num) {
                for mut res in to_recheck {
                    stats.rechecks_total += 1;
                    metrics.inc_hist_rechecks();

                    let mut current_amount = res.amount_in;
                    let mut possible = true;
                    for leg in &res.route.legs {
                        let zero_for_one = leg.edge.token_in.0 < leg.edge.token_out.0;
                        let next_opt = match leg.edge.kind {
                            PoolKind::ReserveBased => state_engine.quote_v2(&leg.edge.pool_id, current_amount, zero_for_one).await,
                            PoolKind::ConcentratedLiquidity => state_engine.quote_v3(&leg.edge.pool_id, current_amount, zero_for_one).await,
                            _ => None,
                        };
                        match next_opt {
                            Some(nxt) if !nxt.is_zero() => current_amount = nxt,
                            _ => { possible = false; break; }
                        }
                    }
                    
                    let rechecked_amount_out = if possible { current_amount } else { AlloyU256::ZERO };
                    let rechecked_profit = if rechecked_amount_out > res.amount_in {
                        rechecked_amount_out - res.amount_in
                    } else {
                        AlloyU256::ZERO
                    };

                    let still_profitable = rechecked_profit > AlloyU256::ZERO;
                    if still_profitable {
                        stats.still_profitable += 1;
                        metrics.inc_hist_still_profitable();
                    }

                    res.recheck = Some(HistoricalRecheckResult {
                        block_number: block_num,
                        rechecked_amount_out,
                        rechecked_profit,
                        drift_summary: HistoricalDriftSummary {
                            profit_drift_wei: 0,
                            amount_out_drift_wei: 0,
                            is_still_profitable: still_profitable,
                        },
                        invalidated_reason: if !possible { Some("LiquidityVanished".to_string()) } else { None },
                    });
                    all_replay_results.push(res);
                }
            }

            if true { // Run every block for Phase 18 density calibration
                let snapshots = state_engine.get_all_pools().await;
                let mut graph = RouteGraph::new();
                if !snapshots.is_empty() {
                    graph.build_from_snapshots(snapshots.clone());
                    
                    let node_count = graph.node_count();
                    let has_root = graph.has_token(&root_asset);
                    if block_num % 100 == 0 {
                        info!("  Graph Check: nodes={}, has_root={}", node_count, has_root);
                    }

                    if has_root {
                            let buckets = vec![
                                arb_types::QuoteSizeBucket::Custom(1_000_000_000_000_000),  // 0.001 ETH
                                arb_types::QuoteSizeBucket::Custom(5_000_000_000_000_000),  // 0.005 ETH
                                arb_types::QuoteSizeBucket::Custom(10_000_000_000_000_000), // 0.01 ETH
                                arb_types::QuoteSizeBucket::Custom(50_000_000_000_000_000), // 0.05 ETH
                                arb_types::QuoteSizeBucket::Custom(100_000_000_000_000_000),// 0.1 ETH
                            ];
                            let candidates = generator.generate_candidates(&graph, &root_asset, &buckets).await;
                        
                        stats.candidates_considered += candidates.len() as u64;
                        for _ in 0..candidates.len() { metrics.inc_hist_candidates(); }

                        for cand in candidates {
                            if cand.estimated_gross_profit > alloy_primitives::U256::from(0) {
                                stats.would_trade += 1;
                                block_trades += 1;
                                metrics.inc_hist_would_trade(if cand.path.legs.len() > 2 { "multi" } else { "direct" });

                                // Phase 18 Metric: Size Bucket
                                let bucket = if cand.estimated_gross_profit >= alloy_primitives::U256::from(50_000_000_000_000_000u128) { "0.05+" }
                                            else if cand.estimated_gross_profit >= alloy_primitives::U256::from(10_000_000_000_000_000u128) { "0.01-0.05" }
                                            else if cand.estimated_gross_profit >= alloy_primitives::U256::from(5_000_000_000_000_000u128) { "0.005-0.01" }
                                            else { "0.001-0.005" };
                                metrics.inc_hist_bucket(bucket);

                                let result = HistoricalReplayResult {
                                    case_id: format!("{}-{}", block_num, stats.would_trade),
                                    block_number: block_num,
                                    route_family: if cand.path.legs.len() > 2 { "multi".to_string() } else { "direct".to_string() },
                                    root_asset: root_asset.clone(),
                                    amount_in: cand.amount_in,
                                    predicted_amount_out: cand.estimated_amount_out,
                                    predicted_profit: cand.estimated_gross_profit,
                                    would_trade: cand.predicted_profit >= min_profit_wei,
                                    path: cand.path.clone(),
                                    recheck: None,
                                };
                                
                                // Stream to disk immediately
                                if let Some(ref mut writer) = target_export_file {
                                    let _ = serde_json::to_writer(writer as &mut dyn std::io::Write, &result);
                                    let _ = writer.write_all(b"\n");
                                    let _ = writer.flush();
                                }

                                all_replay_results.push(result.clone());
                                pending_rechecks.entry(block_num + (config.historical_recheck_blocks)).or_default().push(result);
                            }
                        }
                    }
                }
                
                // Phase 18 Metrics: Density & Clustering
                let density = (stats.would_trade as f64 / (block_num - config.historical_replay_start_block.unwrap_or(block_num) + 1) as f64) * 1000.0;
                metrics.set_hist_density(density as i64);
                
                if block_trades > 1 {
                    metrics.set_hist_clustering(1000); // simplified: 100% for this block
                } else {
                    metrics.set_hist_clustering(0);
                }

                info!("  Block {}: pools={}, nodes={}, edges={}, trades={}", block_num, snapshots.len(), graph.node_count(), graph.edge_count(), stats.would_trade);
            }
        }
        current_block_ptr = chunk_end + 1;
    }

    info!("Replay finished. Found {} results.", all_replay_results.len());

    let mut selected_results: Vec<HistoricalReplayResult> = Vec::new();
    let mut success_cases: Vec<_> = all_replay_results.iter()
        .filter(|r| r.recheck.as_ref().map_or(false, |rc| rc.drift_summary.is_still_profitable))
        .collect();
    
    success_cases.sort_by(|a, b| {
        let p_a = a.recheck.as_ref().map(|rc| rc.rechecked_profit).unwrap_or(AlloyU256::ZERO);
        let p_b = b.recheck.as_ref().map(|rc| rc.rechecked_profit).unwrap_or(AlloyU256::ZERO);
        p_b.cmp(&p_a)
    });
    for r in success_cases.iter().take(2) { selected_results.push((*r).clone()); }

    let mut invalidated_cases: Vec<_> = all_replay_results.iter()
        .filter(|r| r.recheck.as_ref().map_or(false, |rc| !rc.drift_summary.is_still_profitable))
        .collect();
    if let Some(r) = invalidated_cases.first() { selected_results.push((*r).clone()); }

    if selected_results.len() < 4 {
        for r in all_replay_results.iter() {
            if !selected_results.iter().any(|sr| sr.case_id == r.case_id) {
                selected_results.push(r.clone());
                if selected_results.len() >= 4 { break; }
            }
        }
    }

    let mut cases_to_export = Vec::new();
    for res in &selected_results {
        let mut pool_ids = Vec::new();
        let mut pool_kinds = Vec::new();
        let mut path_tokens = Vec::new();
        let mut leg_directions = Vec::new();
        path_tokens.push(res.route.root_asset.clone());
        for leg in &res.route.legs {
            pool_ids.push(leg.edge.pool_id.0.clone());
            pool_kinds.push(leg.edge.kind);
            path_tokens.push(leg.edge.token_out.clone());
            leg_directions.push(leg.edge.token_in.0 < leg.edge.token_out.0);
        }
        cases_to_export.push(arb_types::HistoricalCase {
            case_id: res.case_id.clone(),
            notes: format!("Phase 17 Spotlight: {}", res.route_family),
            fork_block_number: res.block_number,
            source_tx_hash: None,
            root_asset: res.root_asset.clone(),
            route_family: res.route_family.clone(),
            pool_ids,
            pool_kinds,
            path_tokens,
            leg_directions,
            amount_in: res.amount_in,
            expected_outcome: if res.recheck.as_ref().map_or(false, |rc| rc.drift_summary.is_still_profitable) { "success".into() } else { "revert".into() },
            guard_overrides: None,
            seed_data: None,
        });
    }

    if !selected_results.is_empty() {
        let json_data = serde_json::to_string_pretty(&cases_to_export)?;
        std::fs::write("fixtures/historical_cases_phase_17.json", json_data)?;
        info!("Exported {} cases. Invoking arb_battery...", selected_results.len());
        
        let status = std::process::Command::new("cargo")
            .args(["run", "--release", "--bin", "arb_battery"])
            .env("HISTORICAL_CASES_PATH", "fixtures/historical_cases_phase_17.json")
            .status()?;
        
        if status.success() {
            info!("arb_battery finished successfully.");
        }
    }

    let report_summary = HistoricalReplaySummary {
        start_block,
        end_block,
        total_blocks: end_block.saturating_sub(start_block) + 1,
        total_logs: stats.total_logs,
        candidates_considered: stats.candidates_considered,
        promoted_candidates: stats.would_trade, 
        would_trade_candidates: stats.would_trade,
        still_profitable_count: stats.still_profitable,
        invalidated_count: stats.rechecks_total.saturating_sub(stats.still_profitable),
        avg_profit_drift_wei: 0,
        fork_verifications: Vec::new(),
    };

    let summary_json = serde_json::to_string_pretty(&report_summary)?;
    std::fs::write("historical_replay_full_day_final.json", summary_json)?;
    info!("Phase 17 Summary saved.");

    // Phase 18: Export ALL promoted candidates to JSONL sidecar if requested
    if let Some(export_path) = std::env::var_os("EXPORT_CANDIDATES_PATH") {
        info!("Exporting all {} promoted candidates to {:?}...", all_replay_results.len(), export_path);
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(export_path)?;
        
        for res in &all_replay_results {
            let line = serde_json::to_string(res)?;
            use std::io::Write;
            let _ = writeln!(file, "{}", line);
        }
    }

    Ok(())
}
