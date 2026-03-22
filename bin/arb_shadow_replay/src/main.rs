use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error};
use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;

use arb_config::Config;
use arb_metrics::MetricsRegistry;
use arb_types::{
    EventStamp, IngestEvent, PoolId, PoolKind, PoolUpdate, TokenAddress, QuoteSizeBucket,
    HistoricalReplayResult, HistoricalReplaySummary, HistoricalRecheckResult, HistoricalDriftSummary,
    ForkVerificationResult, PendingLogEvent, RoutePath,
};
use arb_ingest::DexDecoder;
use arb_state::StateEngine;
use arb_route::{RouteGraph, CandidateGenerator};
use arb_sim::LocalSimulator;
use alloy_primitives::{U256 as AlloyU256, Address as AlloyAddress, B256 as AlloyB256, b256};
use ethers::prelude::*;
use warp::Filter;

abigen!(
    IERC20Pool,
    r#"[
        function token0() external view returns (address)
        function token1() external view returns (address)
    ]"#
);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    let config = Config::load();
    if !config.enable_historical_shadow_replay {
        info!("Historical Shadow Replay is disabled. Set ENABLE_HISTORICAL_SHADOW_REPLAY=true to run.");
        return Ok(());
    }

    info!("Starting Historical Shadow Calibration Replay...");

    let metrics = Arc::new(MetricsRegistry::new());
    let metrics_port = config.historical_replay_metrics_port;
    
    // Start Metrics Server
    let metrics_state = metrics.clone();
    tokio::spawn(async move {
        let metrics_route = warp::path("metrics")
            .map(move || metrics_state.gather_metrics());
        
        info!("Serving Prometheus metrics at http://0.0.0.0:{}/metrics", metrics_port);
        warp::serve(metrics_route).run(([0, 0, 0, 0], metrics_port)).await;
    });

    // Provider for historical data
    let rpc_url = config.rpc_http_url.as_deref()
        .or(Some(&config.quicknode_http_url))
        .ok_or_else(|| anyhow::anyhow!("No RPC URL provided"))?;
    
    let provider = Arc::new(Provider::<Http>::try_from(rpc_url)?);
    let chain_id = provider.get_chainid().await?.as_u64();
    info!("Connected to Chain ID: {}", chain_id);

    // Calculate Block Range
    let latest_block = provider.get_block_number().await?.as_u64();
    let (start_block, end_block) = if let (Some(s), Some(e)) = (config.historical_replay_start_block, config.historical_replay_end_block) {
        (s, e)
    } else {
        let lookback_blocks = (config.historical_replay_lookback_hours as u64) * 3600 / 2; // ~2s per block on Base
        (latest_block.saturating_sub(lookback_blocks), latest_block)
    };

    info!("Replaying blocks {} to {} ({} blocks total)", start_block, end_block, end_block - start_block + 1);

    // Initialize Components
    let decoder = DexDecoder::new(metrics.clone());
    let state_engine = Arc::new(StateEngine::new(metrics.clone()));
    let generator = CandidateGenerator::new(state_engine.clone());
    let root_asset = TokenAddress(config.root_asset.clone());
    let buckets: Vec<QuoteSizeBucket> = config.quote_buckets
        .split(',')
        .filter_map(|s| s.trim().parse::<u128>().ok())
        .map(|a| QuoteSizeBucket::Custom(a))
        .collect();

    // Stats Tracker
    let mut stats = ReplayStats::new(start_block, end_block);
    let mut pending_rechecks: BTreeMap<u64, Vec<HistoricalReplayResult>> = BTreeMap::new();
    let mut results: Vec<HistoricalReplayResult> = Vec::new();
    let mut pool_tokens: HashMap<ethers::types::Address, (TokenAddress, TokenAddress)> = HashMap::new();

    info!("Lookback window: start_block={}, end_block={}, total={}", start_block, end_block, end_block - start_block + 1);

    // Topics we care about (Sync & Swap)
    let v2_sync_sig = b256!("1c91f030eb7c0a042c0211d40a5440311dec3b1285bc035ede49007f502574e4");
    let v3_swap_sig = b256!("c42079f94a6350d7e5735f2399b6d8de98a486f7af7d160cfd333044e7c75db1");
    let topics = vec![H256::from(v2_sync_sig.0), H256::from(v3_swap_sig.0)];
    
    for (i, t) in topics.iter().enumerate() {
        info!("Topic {}: {:?}", i, t);
    }

    // Fetch Logs in Chunks
    let chunk_size = 2000;
    let mut current_chunk_start = start_block;

    while current_chunk_start <= end_block {
        let current_chunk_end = (current_chunk_start + chunk_size - 1).min(end_block);
        info!("Fetching logs for block range {}..{}", current_chunk_start, current_chunk_end);
        
        let mut logs = Vec::new();
        for topic in &topics {
            let filter = ethers::types::Filter::new()
                .from_block(current_chunk_start)
                .to_block(current_chunk_end)
                .topic0(*topic);

            let topic_logs = provider.get_logs(&filter).await?;
            logs.extend(topic_logs);
        }
        
        info!("Fetched {} total logs for all topics in chunk", logs.len());

        // Group logs by block
        let mut logs_by_block: BTreeMap<u64, Vec<ethers::types::Log>> = BTreeMap::new();
        for log in logs {
            if let Some(bn) = log.block_number {
                logs_by_block.entry(bn.as_u64()).or_default().push(log);
            }
        }

        // Replay sequentially through current chunk
        for block_num in current_chunk_start..=current_chunk_end {
            // 1. Apply logs to update StateEngine
            if let Some(block_logs) = logs_by_block.get(&block_num) {
                stats.total_logs += block_logs.len() as u64;
                for log in block_logs {
                    let pl = PendingLogEvent {
                        address: format!("{:?}", log.address),
                        topics: log.topics.iter().map(|t| format!("{:?}", t)).collect(),
                        data: format!("0x{}", hex::encode(&log.data)),
                        transaction_hash: format!("{:?}", log.transaction_hash.unwrap_or_default()),
                        block_number: log.block_number.unwrap_or_default().as_u64(),
                        log_index: log.log_index.unwrap_or_default().as_u32(),
                    };
                    
                    if let Some(mut update) = decoder.decode_log(&pl) {
                        let pool_addr = log.address;
                        if !pool_tokens.contains_key(&pool_addr) {
                            info!("Fetching metadata for new pool: {:?}", pool_addr);
                            let contract = IERC20Pool::new(pool_addr, provider.clone());
                            let t0_res = contract.token_0().call().await;
                            let t1_res = contract.token_1().call().await;
                            
                            // Sleep to avoid rate limits
                            tokio::time::sleep(Duration::from_millis(50)).await;
                            
                            match (t0_res, t1_res) {
                                (Ok(t0), Ok(t1)) if !t0.is_zero() && !t1.is_zero() => {
                                    pool_tokens.insert(pool_addr, (TokenAddress(format!("{:?}", t0)), TokenAddress(format!("{:?}", t1))));
                                    info!("Metadata found: t0={:?}, t1={:?}", t0, t1);
                                }
                                _ => {
                                    warn!("Failed to fetch tokens for pool {:?}", pool_addr);
                                }
                            }
                        }
                        
                        if let Some((t0, t1)) = pool_tokens.get(&pool_addr) {
                            update.token0 = Some(t0.clone());
                            update.token1 = Some(t1.clone());
                        }
                        
                        state_engine.apply(update).await;
                    }
                }
            }

            // 2. Process pending rechecks for this block (N + delay)
            if let Some(to_recheck) = pending_rechecks.remove(&block_num) {
                for mut res in to_recheck {
                    metrics.inc_hist_rechecks();
                    
                    // Evaluate path at current state (which is block N + delay)
                    // We reuse the internal evaluate_path logic via a dedicated temporary quoter check
                    // Simplified: We assume the generator can re-evaluate a specific path.
                    // For now, we manually simulate or use the state engine directly.
                    
                    let mut current_amount = res.amount_in;
                    let mut possible = true;
                    for leg in &res.route.legs {
                        let edge = &leg.edge;
                        let zero_for_one = edge.token_in.0 < edge.token_out.0;
                        let next_opt = match edge.kind {
                            PoolKind::ReserveBased => state_engine.quote_v2(&edge.pool_id, current_amount, zero_for_one).await,
                            PoolKind::ConcentratedLiquidity => state_engine.quote_v3(&edge.pool_id, current_amount, zero_for_one).await,
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
                    
                    let profit_drift = if rechecked_profit >= res.predicted_profit {
                        (rechecked_profit - res.predicted_profit).to::<u128>() as i128
                    } else {
                        -((res.predicted_profit - rechecked_profit).to::<u128>() as i128)
                    };
                    
                    let output_drift = if rechecked_amount_out >= res.predicted_amount_out {
                        (rechecked_amount_out - res.predicted_amount_out).to::<u128>() as i128
                    } else {
                        -((res.predicted_amount_out - rechecked_amount_out).to::<u128>() as i128)
                    };

                    res.recheck = Some(HistoricalRecheckResult {
                        block_number: block_num,
                        rechecked_amount_out,
                        rechecked_profit,
                        drift_summary: HistoricalDriftSummary {
                            profit_drift_wei: profit_drift,
                            amount_out_drift_wei: output_drift,
                            is_still_profitable: rechecked_profit > AlloyU256::from(1000), // Tiny safety margin
                        },
                        invalidated_reason: if !possible { Some("LiquidityVanished".to_string()) } 
                                           else if rechecked_profit == AlloyU256::ZERO { Some("ProfitVanished".to_string()) }
                                           else { None },
                    });
                    
                    if res.recheck.as_ref().unwrap().drift_summary.is_still_profitable {
                        metrics.inc_hist_still_profitable();
                        stats.still_profitable_count += 1;
                    } else {
                        metrics.inc_hist_invalidated();
                        stats.invalidated_count += 1;
                    }
                    
                    metrics.add_hist_drift(profit_drift.abs() as u64, output_drift.abs() as u64);
                    stats.accumulated_profit_drift += profit_drift;
                    results.push(res);
                }
            }

            // 3. Plan candidates at THIS block
            let snapshots = state_engine.get_all_pools().await;
            let mut graph = RouteGraph::new();
            graph.build_from_snapshots(snapshots);
            
            if block_num % 500 == 0 {
                info!("Graph built: {} nodes, {} edges. Root asset: {}", graph.node_count(), graph.edge_count(), root_asset.0);
            }
            
            let candidates = generator.generate_candidates(&graph, &root_asset, &buckets).await;
            metrics.inc_hist_candidates();
            stats.candidates_considered += candidates.len() as u64;

            for cand in candidates {
                stats.promoted_candidates += 1;
                metrics.inc_hist_promoted();
                
                // Historical "Would Trade" assumes no competing bots for simplicity, 
                // but we record decay to measure that risk.
                if cand.estimated_gross_profit > AlloyU256::ZERO {
                    let family = if cand.path.legs.len() > 2 { "multi" } else { "direct" };
                    metrics.inc_hist_would_trade(family);
                    stats.would_trade_candidates += 1;
                    
                    let result = HistoricalReplayResult {
                        case_id: format!("{}-{}-{}", block_num, stats.would_trade_candidates, cand.path.legs.len()),
                        block_number: block_num,
                        route_family: family.to_string(),
                        root_asset: root_asset.clone(),
                        amount_in: cand.amount_in,
                        predicted_amount_out: cand.estimated_amount_out,
                        predicted_profit: cand.estimated_gross_profit,
                        would_trade: true,
                        route: cand.path.clone(),
                        recheck: None,
                    };
                    
                    let recheck_block = block_num + config.historical_recheck_blocks;
                    pending_rechecks.entry(recheck_block).or_default().push(result);
                }
            }
            
            if block_num % 100 == 0 {
                info!("Block {}: {} candidates, {} pending rechecks, state pools: {}", 
                    block_num, stats.candidates_considered, pending_rechecks.len(), state_engine.pool_count().await);
            }
        }

        current_chunk_start = current_chunk_end + 1;
    }

    info!("Historical Replay Complete!");

    // Save Summary
    let summary = HistoricalReplaySummary {
        start_block,
        end_block,
        total_blocks: end_block - start_block + 1,
        total_logs: stats.total_logs,
        candidates_considered: stats.candidates_considered,
        promoted_candidates: stats.promoted_candidates,
        would_trade_candidates: stats.would_trade_candidates,
        still_profitable_count: stats.still_profitable_count,
        invalidated_count: stats.invalidated_count,
        avg_profit_drift_wei: if stats.would_trade_candidates > 0 { stats.accumulated_profit_drift / (stats.would_trade_candidates as i128) } else { 0 },
        fork_verifications: Vec::new(),
    };
    
    let summary_json = serde_json::to_string_pretty(&summary)?;
    tokio::fs::write(&config.historical_replay_output_path, summary_json).await?;
    info!("Summary saved to {}", config.historical_replay_output_path);

    info!("Replay finished. Stats: Considered={}, WouldTrade={}, StillProfitable={}", 
        stats.candidates_considered, stats.would_trade_candidates, stats.still_profitable_count);
    
    // Keep server alive for Grafana
    info!("Keeping metrics endpoint active at http://0.0.0.0:{}/metrics", metrics_port);
    loop {
        sleep(Duration::from_secs(60)).await;
    }
}

struct ReplayStats {
    start_block: u64,
    end_block: u64,
    total_logs: u64,
    candidates_considered: u64,
    promoted_candidates: u64,
    would_trade_candidates: u64,
    still_profitable_count: u64,
    invalidated_count: u64,
    accumulated_profit_drift: i128,
}

impl ReplayStats {
    fn new(start: u64, end: u64) -> Self {
        Self {
            start_block: start,
            end_block: end,
            total_logs: 0,
            candidates_considered: 0,
            promoted_candidates: 0,
            would_trade_candidates: 0,
            still_profitable_count: 0,
            invalidated_count: 0,
            accumulated_profit_drift: 0,
        }
    }
}
