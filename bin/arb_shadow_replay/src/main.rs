use std::str::FromStr;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error};
use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;

use arb_config::Config;
use arb_metrics::MetricsRegistry;
use arb_ingest::DexDecoder;
use arb_state::StateEngine;
use arb_route::{RouteGraph, CandidateGenerator};
use alloy_primitives::{U256 as AlloyU256};
use arb_types::{
    EventStamp, IngestEvent, PoolId, PoolKind, PoolUpdate, TokenAddress, QuoteSizeBucket,
    HistoricalReplayResult, HistoricalReplaySummary, HistoricalRecheckResult, HistoricalDriftSummary,
    ForkVerificationResult, PendingLogEvent, RoutePath, ShadowRecheckResult
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    let config = Config::load();
    if !config.enable_historical_shadow_replay {
        info!("Historical Shadow Replay is disabled.");
        return Ok(());
    }

    info!("Starting Phase 17 Full-Day Calibration Replay...");

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
        (latest - 1000, latest)
    };

    info!("Replaying {}..{} ({} blocks)", start_block, end_block, end_block - start_block + 1);

    let decoder = DexDecoder::new(metrics.clone());
    let state_engine = Arc::new(StateEngine::new(metrics.clone()));
    let generator = CandidateGenerator::new(state_engine.clone());
    let root_asset = TokenAddress(config.root_asset.clone());
    
    let mut stats = ReplayStats::new();
    let mut pending_rechecks: BTreeMap<u64, Vec<HistoricalReplayResult>> = BTreeMap::new();
    let mut all_replay_results: Vec<HistoricalReplayResult> = Vec::new();
    let mut pool_tokens: HashMap<ethers::types::Address, (TokenAddress, TokenAddress)> = HashMap::new();

    let v2_sync_sig = H256::from_slice(&hex::decode("1c91f030eb7c0a042c0211d40a5440311dec3b1285bc035ede49007f502574e4").unwrap());
    let aero_swap_sig = H256::from_slice(&hex::decode("d78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822").unwrap());
    let v3_swap_sig = H256::from_slice(&hex::decode("c42079f94a6350d7e5735f2399b6d8de98a486f7af7d160cfd333044e7c75db1").unwrap());

    let chunk_size = 500;
    let mut current_block_ptr = start_block;

    while current_block_ptr <= end_block {
        let chunk_end = (current_block_ptr + chunk_size - 1).min(end_block);
        let mut chunk_logs = Vec::new();
        for sig in [v2_sync_sig, aero_swap_sig, v3_swap_sig] {
            let filter = ethers::types::Filter::new()
                .from_block(current_block_ptr)
                .to_block(chunk_end)
                .topic0(ValueOrArray::Value(sig));
            if let Ok(l) = provider.get_logs(&filter).await {
                chunk_logs.extend(l);
            }
        }
        
        let mut logs_by_block: BTreeMap<u64, Vec<ethers::types::Log>> = BTreeMap::new();
        for log in chunk_logs {
            if let Some(bn) = log.block_number {
                logs_by_block.entry(bn.as_u64()).or_default().push(log);
            }
        }

        for block_num in current_block_ptr..=chunk_end {
            if let Some(block_logs) = logs_by_block.get(&block_num) {
                stats.total_logs += block_logs.len() as u64;
                for log in block_logs {
                    let pool_addr = log.address;
                    let topic0 = log.topics.get(0).cloned();
                    
                    if let Some(t0) = topic0 {
                        let is_sync = t0 == v2_sync_sig;
                        let is_v3_swap = t0 == v3_swap_sig;

                        if is_sync || is_v3_swap {
                            if !pool_tokens.contains_key(&pool_addr) {
                                let v2_pool = IERC20Pool::new(pool_addr, provider.clone());
                                if let (Ok(t0_addr), Ok(t1_addr)) = (v2_pool.token_0().call().await, v2_pool.token_1().call().await) {
                                    if !t0_addr.is_zero() && !t1_addr.is_zero() {
                                        let t0_ta = TokenAddress(format!("{:?}", t0_addr));
                                        let t1_ta = TokenAddress(format!("{:?}", t1_addr));
                                        pool_tokens.insert(pool_addr, (t0_ta.clone(), t1_ta.clone()));
                                        if is_sync {
                                            if let Ok((r0, r1, _)) = v2_pool.get_reserves().call().await {
                                                state_engine.apply(PoolUpdate {
                                                    pool_id: PoolId(format!("{:?}", pool_addr)),
                                                    kind: PoolKind::ReserveBased,
                                                    token0: Some(t0_ta),
                                                    token1: Some(t1_ta),
                                                    fee_bps: Some(30),
                                                    reserves: Some(arb_types::ReserveSnapshot { reserve0: r0, reserve1: r1 }),
                                                    cl_snapshot: None,
                                                    cl_full_state: None,
                                                    stamp: EventStamp { block_number: block_num - 1, log_index: 0 },
                                                }).await;
                                            }
                                        } else {
                                            let v3_pool = IUniswapV3Pool::new(pool_addr, provider.clone());
                                            if let (Ok(slot0), Ok(liq), Ok(fee)) = (v3_pool.slot_0().call().await, v3_pool.liquidity().call().await, v3_pool.fee().call().await) {
                                                state_engine.apply(PoolUpdate {
                                                    pool_id: PoolId(format!("{:?}", pool_addr)),
                                                    kind: PoolKind::ConcentratedLiquidity,
                                                    token0: Some(t0_ta),
                                                    token1: Some(t1_ta),
                                                    fee_bps: Some(fee as u32),
                                                    reserves: None,
                                                    cl_snapshot: Some(arb_types::CLSnapshot {
                                                        sqrt_price_x96: AlloyU256::from_str(&U256::from(slot0.0).to_string()).unwrap(),
                                                        liquidity: alloy_primitives::U128::from_str(&liq.to_string()).unwrap(),
                                                        tick: slot0.1,
                                                    }),
                                                    cl_full_state: None,
                                                    stamp: EventStamp { block_number: block_num - 1, log_index: 0 },
                                                }).await;
                                            }
                                        }
                                    }
                                }
                            }

                            if let Some((t0_a, t1_a)) = pool_tokens.get(&pool_addr) {
                                let pl = PendingLogEvent {
                                    address: format!("{:?}", pool_addr),
                                    topics: log.topics.iter().map(|t| format!("{:?}", t)).collect(),
                                    data: format!("0x{}", hex::encode(&log.data)),
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

            let snapshots = state_engine.get_all_pools().await;
            if !snapshots.is_empty() {
                let mut graph = RouteGraph::new();
                graph.build_from_snapshots(snapshots.clone());
                let candidates = generator.generate_candidates(&graph, &root_asset, &vec![
                    QuoteSizeBucket::Custom(1_000_000_000_000_000), 
                ]).await;
                
                stats.candidates_considered += candidates.len() as u64;
                for _ in 0..candidates.len() { metrics.inc_hist_candidates(); }

                for cand in candidates {
                    if cand.estimated_gross_profit > AlloyU256::ZERO {
                        stats.would_trade += 1;
                        metrics.inc_hist_would_trade(if cand.path.legs.len() > 2 { "multi" } else { "direct" });

                        let result = HistoricalReplayResult {
                            case_id: format!("{}-{}", block_num, stats.would_trade),
                            block_number: block_num,
                            route_family: if cand.path.legs.len() > 2 { "multi".to_string() } else { "direct".to_string() },
                            root_asset: root_asset.clone(),
                            amount_in: cand.amount_in,
                            predicted_amount_out: cand.estimated_amount_out,
                            predicted_profit: cand.estimated_gross_profit,
                            would_trade: true,
                            route: cand.path,
                            recheck: None,
                        };
                        pending_rechecks.entry(block_num + (config.historical_recheck_blocks)).or_default().push(result);
                    }
                }
                
                if block_num % 100 == 0 {
                    info!("  Block {}: pools={}, nodes={}, edges={}, trades={}", block_num, snapshots.len(), graph.node_count(), graph.edge_count(), stats.would_trade);
                }
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
        let json = serde_json::to_string_pretty(&cases_to_export)?;
        std::fs::write("fixtures/historical_cases_phase_17.json", json)?;
        info!("Exported {} cases. Invoking arb_battery...", selected_results.len());
        
        let status = std::process::Command::new("cargo")
            .args(["run", "--release", "--bin", "arb_battery"])
            .env("HISTORICAL_CASES_PATH", "fixtures/historical_cases_phase_17.json")
            .status()?;
        
        if status.success() {
            info!("arb_battery finished successfully.");
        }
    }

    let summary = HistoricalReplaySummary {
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

    let summary_json = serde_json::to_string_pretty(&summary)?;
    std::fs::write("historical_replay_full_day_final.json", summary_json)?;
    info!("Phase 17 Summary saved.");

    Ok(())
}
