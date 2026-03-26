use std::str::FromStr;
use std::sync::Arc;
use std::io::Write;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error as tracing_error};
use std::collections::{BTreeMap, HashMap, HashSet};

use arb_config::Config;
use arb_metrics::MetricsRegistry;
use arb_ingest::DexDecoder;
use arb_state::StateEngine;
use arb_route::{RouteGraph, CandidateGenerator};
use alloy_primitives::{U256 as AlloyU256, U128 as AlloyU128};
use arb_types::{
    EventStamp, IngestEvent, PoolId, PoolKind, PoolUpdate, TokenAddress, QuoteSizeBucket,
    HistoricalReplayResult, HistoricalReplaySummary, PendingLogEvent, RoutePath, CLSnapshot, ReserveSnapshot, RouteLeg,
    CandidateOpportunity
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
    would_trade: u64,
}

impl ReplayStats {
    fn new() -> Self {
        Self { total_logs: 0, would_trade: 0 }
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

fn get_route_id(path: &arb_types::RoutePath) -> String {
    path.legs.iter()
        .map(|leg| leg.edge.pool_id.0.clone())
        .collect::<Vec<_>>()
        .join("->")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let config = Config::load();
    if !config.enable_historical_shadow_replay {
        info!("Historical Shadow Replay is disabled.");
        return Ok(());
    }

    info!("Starting Phase 18 Final Calibration Replay (Structural Turbo Enabled)...");
    let metrics = Arc::new(MetricsRegistry::new());
    let rpc_url = config.quicknode_http_url.clone();
    let provider = Arc::new(Provider::<Http>::try_from(rpc_url)?);

    let metrics_state = metrics.clone();
    let metrics_port = config.historical_replay_metrics_port;
    tokio::spawn(async move {
        let metrics_route = warp::path("metrics").map(move || metrics_state.gather_metrics());
        warp::serve(metrics_route).run(([0, 0, 0, 0], metrics_port)).await;
    });

    let (s_raw, e_raw) = if let (Some(s), Some(e)) = (config.historical_replay_start_block, config.historical_replay_end_block) {
        (s, e)
    } else {
        let latest = provider.get_block_number().await?.as_u64();
        (latest.saturating_sub(1000u64), latest)
    };
    let start_block: u64 = s_raw;
    let end_block: u64 = e_raw;

    info!("Replaying {}..{} ({} blocks)", start_block, end_block, end_block - start_block + 1);

    let decoder = DexDecoder::new(metrics.clone());
    let state_engine = Arc::new(StateEngine::new(metrics.clone()));
    let generator = CandidateGenerator::new(state_engine.clone());
    
    let trimmed_root_o = config.root_asset.trim().to_string();
    let root_addr = ethers::types::Address::from_str(&trimmed_root_o)?;
    let root_asset = TokenAddress(normalize_addr(root_addr));
    
    let mut stats = ReplayStats::new();
    let mut pool_tokens: HashMap<ethers::types::Address, (TokenAddress, TokenAddress)> = HashMap::new();

    let v2_sync_sig = H256::from_slice(&hex::decode("1c91f030eb7c0a042c0211d40a5440311dec3b1285bc035ede49007f502574e4").expect("Valid v2_sync_sig hex"));
    let aero_swap_sig = H256::from_slice(&hex::decode("d78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822").expect("Valid aero_swap_sig hex"));
    let v3_swap_sig = H256::from_slice(&hex::decode("c42079f94a6350d7e5735f2399b6d8de98a486f7af7d160cfd333044e7c75db1").expect("Valid v3_swap_sig hex"));

    let chunk_size: u64 = 500;
    
    // Warm-up
    let lookback_blocks: u64 = (config.historical_replay_lookback_hours as u64) * 1800u64;
    let warmup_start: u64 = start_block.saturating_sub(lookback_blocks);
    info!("Running warmup discovery from {} to {}...", warmup_start, start_block);
    
    let mut warmup_ptr: u64 = warmup_start;
    while warmup_ptr < start_block {
        let warmup_end = (warmup_ptr + chunk_size).min(start_block).saturating_sub(1);
        for sig in [v2_sync_sig, aero_swap_sig, v3_swap_sig] {
            let filter = ethers::types::Filter::new().from_block(warmup_ptr).to_block(warmup_end).topic0(ValueOrArray::Value(sig));
            let logs_warmup_res = provider.get_logs(&filter).await;
            if let Ok(lw) = logs_warmup_res {
                for log in lw {
                    let pool_addr = log.address;
                    if !pool_tokens.contains_key(&pool_addr) {
                        let v2_pool = IERC20Pool::new(pool_addr, provider.clone());
                        let t0_call = v2_pool.token_0();
                        let t0_raw_res = t0_call.call().await;
                        let t1_call = v2_pool.token_1();
                        let t1_raw_res = t1_call.call().await;
                        if let (Ok(t0_raw), Ok(t1_raw)) = (t0_raw_res, t1_raw_res) {
                            let t0_ta = TokenAddress(normalize_addr(t0_raw));
                            let t1_ta = TokenAddress(normalize_addr(t1_raw));
                            pool_tokens.insert(pool_addr, (t0_ta.clone(), t1_ta.clone()));
                            
                            let mut update = PoolUpdate {
                                pool_id: PoolId(normalize_addr(pool_addr)),
                                kind: if log.topics[0] == v3_swap_sig { PoolKind::ConcentratedLiquidity } else { PoolKind::ReserveBased },
                                token0: Some(t0_ta),
                                token1: Some(t1_ta),
                                fee_bps: Some(30),
                                reserves: None,
                                cl_snapshot: None,
                                cl_full_state: None,
                                stamp: EventStamp { block_number: log.block_number.unwrap_or_else(|| U64::zero()).as_u64(), log_index: 0 },
                            };
                            if update.kind == PoolKind::ReserveBased {
                                let g_res_call = v2_pool.get_reserves();
                                if let Ok((r0, r1, _)) = g_res_call.call().await {
                                    update.reserves = Some(ReserveSnapshot { reserve0: r0, reserve1: r1 });
                                }
                            } else {
                                let v3_pool = IUniswapV3Pool::new(pool_addr, provider.clone());
                                let s0_call = v3_pool.slot_0();
                                let s0_res = s0_call.call().await;
                                let liq_call = v3_pool.liquidity();
                                let liq_res = liq_call.call().await;
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
    }
    
    let candidates_path = "historical_replay_full_day_candidates.jsonl";
    let mut writer = std::io::BufWriter::new(std::fs::OpenOptions::new().create(true).append(true).open(candidates_path)?);

    let mut current_block_ptr = start_block;
    let mut last_pool_count = 0;
    let mut cached_graph = RouteGraph::new();

    while current_block_ptr <= end_block {
        let chunk_end = (current_block_ptr + chunk_size).min(end_block + 1).saturating_sub(1);
        let mut logs = Vec::new();
        for sig in [v2_sync_sig, aero_swap_sig, v3_swap_sig] {
            let filter = ethers::types::Filter::new().from_block(current_block_ptr).to_block(chunk_end).topic0(ValueOrArray::Value(sig));
            let chunk_logs_res = provider.get_logs(&filter).await;
            if let Ok(l) = chunk_logs_res { logs.extend(l); }
        }
        
        let mut logs_by_block: BTreeMap<u64, Vec<ethers::types::Log>> = BTreeMap::new();
        for log in logs { if let Some(bn) = log.block_number { logs_by_block.entry(bn.as_u64()).or_default().push(log); } }

        for block_num in current_block_ptr..=chunk_end {
            if let Some(block_logs) = logs_by_block.get(&block_num) {
                stats.total_logs += block_logs.len() as u64;
                for log in block_logs {
                    let pool_addr = log.address;
                    if let Some((t0_a, t1_a)) = pool_tokens.get(&pool_addr) {
                        let topics_owned: Vec<String> = log.topics.iter().map(|t| format!("{:?}", t).to_lowercase()).collect();
                        let tx_h_o = format!("{:?}", log.transaction_hash.unwrap_or_default()).to_lowercase();
                        let d_o = format!("0x{}", hex::encode(&log.data));
                        let pl = PendingLogEvent {
                            address: normalize_addr(pool_addr),
                            topics: topics_owned,
                            data: d_o,
                            transaction_hash: tx_h_o,
                            block_number: block_num,
                            log_index: log.log_index.unwrap_or_default().as_u32(),
                        };
                        if let Some(mut update) = decoder.decode_log(&pl) {
                            update.token0 = Some(t0_a.clone()); update.token1 = Some(t1_a.clone());
                            state_engine.apply(update).await;
                        }
                    } else {
                        let v2_pool = IERC20Pool::new(pool_addr, provider.clone());
                        let t0_call = v2_pool.token_0();
                        let t0_r = t0_call.call().await;
                        let t1_call = v2_pool.token_1();
                        let t1_r = t1_call.call().await;
                        if let (Ok(t0_raw), Ok(t1_raw)) = (t0_r, t1_r) {
                            let t0_ta = TokenAddress(normalize_addr(t0_raw));
                            let t1_ta = TokenAddress(normalize_addr(t1_raw));
                            pool_tokens.insert(pool_addr, (t0_ta.clone(), t1_ta.clone()));
                        }
                    }
                }
            }

            let (snapshots, pool_map) = {
                let s = state_engine.get_all_pools_map().await;
                let v: Vec<_> = s.values().cloned().collect();
                (v, s)
            };
            if snapshots.len() > last_pool_count {
                cached_graph = RouteGraph::new();
                cached_graph.build_from_snapshots(snapshots);
                last_pool_count = pool_map.len();
            }

            let buckets = vec![
                QuoteSizeBucket::Custom(10_000_000_000_000_000), // 0.01 ETH
                QuoteSizeBucket::Custom(30_000_000_000_000_000), // 0.03 ETH
                QuoteSizeBucket::Custom(50_000_000_000_000_000), // 0.05 ETH
            ];

            let candidates = generator.generate_candidates(&cached_graph, &root_asset, &buckets, &pool_map);
            let mut block_unique_candidates = HashSet::new();
            for cand in candidates {
                let route_id = get_route_id(&cand.path);
                let family = if cand.path.legs.len() > 2 { "multi" } else { "direct" };
                let bucket_str = format!("{:?}", cand.bucket);
                let dedup_key = format!("{}-{}-{}-{}", block_num, family, route_id, bucket_str);

                if block_unique_candidates.insert(dedup_key) && cand.estimated_gross_profit > AlloyU256::ZERO {
                    stats.would_trade += 1;
                    let res = HistoricalReplayResult {
                        case_id: format!("{}-{}", block_num, stats.would_trade),
                        block_number: block_num,
                        route_family: family.to_string(),
                        root_asset: root_asset.clone(),
                        amount_in: cand.amount_in,
                        predicted_amount_out: cand.estimated_amount_out,
                        predicted_profit: cand.estimated_gross_profit,
                        bucket: bucket_str,
                        would_trade: true,
                        path: cand.path.clone(),
                        recheck: None,
                    };
                    let _ = serde_json::to_writer(&mut writer, &res);
                    let _ = writer.write_all(b"\n");
                }
            }
            if block_num % 50 == 0 { info!("  Block {}: trades={}, pools={}", block_num, stats.would_trade, last_pool_count); }
        }
        current_block_ptr = chunk_end + 1;
        let _ = writer.flush();
    }
    Ok(())
}
