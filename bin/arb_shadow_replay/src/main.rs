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
        function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)
    ]"#
);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    let config = Config::load();
    if !config.enable_historical_shadow_replay {
        info!("Historical Shadow Replay is disabled.");
        return Ok(());
    }

    info!("Starting Phase 16 Calibration Replay...");

    let metrics = Arc::new(MetricsRegistry::new());
    let metrics_port = config.historical_replay_metrics_port;
    
    let rpc_url = &config.quicknode_http_url;
    // Mask token for log compliance
    let masked_url = if let Some(pos) = rpc_url.rfind('/') {
        if pos < rpc_url.len() - 1 {
            format!("{}/***/", &rpc_url[..pos])
        } else {
            rpc_url.clone()
        }
    } else {
        rpc_url.clone()
    };
    info!("Initializing provider with masked URL: {}", masked_url);
    let provider = Arc::new(Provider::<Http>::try_from(rpc_url)?);

    let metrics_state = metrics.clone();
    tokio::spawn(async move {
        let metrics_route = warp::path("metrics")
            .map(move || metrics_state.gather_metrics());
        
        info!("Serving metrics at http://0.0.0.0:{}", metrics_port);
        warp::serve(metrics_route).run(([0, 0, 0, 0], metrics_port)).await;
    });

    let latest_block = provider.get_block_number().await?.as_u64();
    let (start_block, end_block) = if let (Some(s), Some(e)) = (config.historical_replay_start_block, config.historical_replay_end_block) {
        (s, e)
    } else {
        let lookback = (config.historical_replay_lookback_hours as u64) * 3600 / 2;
        (latest_block.saturating_sub(lookback), latest_block)
    };

    info!("Replaying {}..{} ({} blocks)", start_block, end_block, end_block - start_block + 1);

    let decoder = DexDecoder::new(metrics.clone());
    let state_engine = Arc::new(StateEngine::new(metrics.clone()));
    let generator = CandidateGenerator::new(state_engine.clone());
    let root_asset = TokenAddress(config.root_asset.clone());
    
    let mut stats = ReplayStats::new(start_block, end_block);
    let mut pending_rechecks: BTreeMap<u64, Vec<HistoricalReplayResult>> = BTreeMap::new();
    let mut results: Vec<HistoricalReplayResult> = Vec::new();
    let mut pool_tokens: HashMap<ethers::types::Address, (TokenAddress, TokenAddress)> = HashMap::new();

    let v2_sync_sig = H256::from_slice(&hex::decode("1c91f030eb7c0a042c0211d40a5440311dec3b1285bc035ede49007f502574e4").unwrap());
    let aero_swap_sig = H256::from_slice(&hex::decode("d78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822").unwrap());
    let v3_swap_sig = H256::from_slice(&hex::decode("c42079f94a6350d7e5735f2399b6d8de98a486f7af7d160cfd333044e7c75db1").unwrap());

    let chunk_size = 500;
    let mut current_block = start_block;

    while current_block <= end_block {
        let chunk_end = (current_block + chunk_size - 1).min(end_block);
        info!("Chunk {}..{}", current_block, chunk_end);
        
        let mut chunk_logs = Vec::new();
        for sig in [v2_sync_sig, aero_swap_sig, v3_swap_sig] {
            let filter = ethers::types::Filter::new()
                .from_block(current_block)
                .to_block(chunk_end)
                .topic0(ValueOrArray::Value(sig));
            
            match provider.get_logs(&filter).await {
                Ok(l) => { chunk_logs.extend(l); }
                Err(e) => { warn!("Query failed for sig {:?}: {}", sig, e); }
            }
        }
        
        info!("  Fetched {} logs", chunk_logs.len());

        let mut logs_by_block: BTreeMap<u64, Vec<ethers::types::Log>> = BTreeMap::new();
        for log in chunk_logs {
            if let Some(bn) = log.block_number {
                logs_by_block.entry(bn.as_u64()).or_default().push(log);
            }
        }

        for block_num in current_block..=chunk_end {
            if let Some(block_logs) = logs_by_block.get(&block_num) {
                stats.total_logs += block_logs.len() as u64;
                for log in block_logs {
                    let pool_addr = log.address;
                    let topic0 = log.topics.get(0).cloned();
                    
                    if let Some(t0) = topic0 {
                        let is_sync = t0 == v2_sync_sig;
                        let is_aero_swap = t0 == aero_swap_sig;
                        let is_v3_swap = t0 == v3_swap_sig;

                        if is_sync || is_aero_swap || is_v3_swap {
                            if !pool_tokens.contains_key(&pool_addr) {
                                let contract = IERC20Pool::new(pool_addr, provider.clone());
                                match (contract.token_0().call().await, contract.token_1().call().await) {
                                    (Ok(t0_addr), Ok(t1_addr)) if !t0_addr.is_zero() && !t1_addr.is_zero() => {
                                        pool_tokens.insert(pool_addr, (TokenAddress(format!("{:?}", t0_addr)), TokenAddress(format!("{:?}", t1_addr))));
                                        
                                        // Initial reserve fetch
                                        if let Ok((r0, r1, _)) = contract.get_reserves().call().await {
                                            state_engine.apply(PoolUpdate {
                                                pool_id: PoolId(format!("{:?}", pool_addr)),
                                                kind: PoolKind::ReserveBased,
                                                token0: Some(TokenAddress(format!("{:?}", t0_addr))),
                                                token1: Some(TokenAddress(format!("{:?}", t1_addr))),
                                                fee_bps: Some(30),
                                                reserves: Some(arb_types::ReserveSnapshot { reserve0: r0, reserve1: r1 }),
                                                cl_snapshot: None,
                                                cl_full_state: None,
                                                stamp: EventStamp { block_number: block_num - 1, log_index: 0 },
                                            }).await;
                                        }
                                    }
                                    _ => {
                                        continue;
                                    }
                                }
                                tokio::time::sleep(Duration::from_millis(10)).await;
                            }

                            if let Some((t0_addr, t1_addr)) = pool_tokens.get(&pool_addr) {
                                let pl = PendingLogEvent {
                                    address: format!("{:?}", pool_addr),
                                    topics: log.topics.iter().map(|t| format!("{:?}", t)).collect(),
                                    data: format!("0x{}", hex::encode(&log.data)),
                                    transaction_hash: format!("{:?}", log.transaction_hash.unwrap_or_default()),
                                    block_number: log.block_number.unwrap_or_default().as_u64(),
                                    log_index: log.log_index.unwrap_or_default().as_u32(),
                                };

                                if is_sync {
                                    if log.data.len() >= 64 {
                                        let r0 = AlloyU256::from_be_slice(&log.data[0..32]);
                                        let r1 = AlloyU256::from_be_slice(&log.data[32..64]);
                                        state_engine.apply(PoolUpdate {
                                            pool_id: PoolId(format!("{:?}", pool_addr)),
                                            kind: PoolKind::ReserveBased,
                                            token0: Some(t0_addr.clone()),
                                            token1: Some(t1_addr.clone()),
                                            fee_bps: Some(30),
                                            reserves: Some(arb_types::ReserveSnapshot { reserve0: r0.to::<u128>(), reserve1: r1.to::<u128>() }),
                                            cl_snapshot: None,
                                            cl_full_state: None,
                                            stamp: EventStamp { block_number: block_num, log_index: pl.log_index },
                                        }).await;
                                    }
                                } else if is_v3_swap {
                                    if let Some(mut update) = decoder.decode_log(&pl) {
                                        update.token0 = Some(t0_addr.clone());
                                        update.token1 = Some(t1_addr.clone());
                                        state_engine.apply(update).await;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Rechecks
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
                    } else {
                        stats.invalidated += 1;
                        metrics.inc_hist_invalidated();
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
                    results.push(res);
                }
            }

            // Candidates
            let snapshots = state_engine.get_all_pools().await;
            if !snapshots.is_empty() {
                let mut graph = RouteGraph::new();
                graph.build_from_snapshots(snapshots.clone());
                let candidates = generator.generate_candidates(&graph, &root_asset, &vec![
                    QuoteSizeBucket::Custom(10_000_000_000_000_000), // 0.01 ETH
                    QuoteSizeBucket::Custom(100_000_000_000_000_000), // 0.1 ETH
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
                            route: cand.path.clone(),
                            recheck: None,
                        };
                        pending_rechecks.entry(block_num + 2).or_default().push(result);
                    }
                }
            }
            
            if block_num % 100 == 0 {
                info!("  Block {}: pools={}, trades={}", block_num, snapshots.len(), stats.would_trade);
            }
        }
        current_block = chunk_end + 1;
    }

    info!("Replay Complete. Saving summary...");
    let summary = HistoricalReplaySummary {
        start_block,
        end_block,
        total_blocks: end_block - start_block + 1,
        total_logs: stats.total_logs,
        candidates_considered: stats.candidates_considered,
        promoted_candidates: stats.would_trade, // In this replayer, would_trade is the promotion criteria
        would_trade_candidates: stats.would_trade,
        still_profitable_count: stats.still_profitable,
        invalidated_count: stats.invalidated,
        avg_profit_drift_wei: 0,
        fork_verifications: Vec::new(),
    };
    
    let summary_json = serde_json::to_string_pretty(&summary)?;
    tokio::fs::write(&config.historical_replay_output_path, summary_json).await?;
    
    info!("Phase 16 Calibration Finished. Metrics available at http://0.0.0.0:{}/metrics", metrics_port);
    info!("Keeping process alive for Grafana scraping (Ctrl+C to stop)...");
    loop { sleep(Duration::from_secs(60)).await; }
}

struct ReplayStats {
    total_logs: u64,
    candidates_considered: u64,
    would_trade: u64,
    rechecks_total: u64,
    still_profitable: u64,
    invalidated: u64,
}

impl ReplayStats {
    fn new(_: u64, _: u64) -> Self {
        Self {
            total_logs: 0,
            candidates_considered: 0,
            would_trade: 0,
            rechecks_total: 0,
            still_profitable: 0,
            invalidated: 0,
        }
    }
}
