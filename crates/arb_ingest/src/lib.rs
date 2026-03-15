use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::broadcast;
use serde::Deserialize;
use arb_types::{
    FlashblockEvent, IngestEvent, PendingLogEvent, PoolUpdate, PoolId, 
    PoolKind, ReserveSnapshot, TokenAddress, EventStamp, CLSnapshot,
    CLTickState, CLFullState
};
use arb_metrics::MetricsRegistry;
use std::collections::HashMap;

use std::str::FromStr;
use alloy_sol_types::{sol, SolEvent};
use alloy_primitives::{Address, Bytes, B256, U256, U128, LogData, b256};
use hex;

sol! {
    event Sync(uint112 reserve0, uint112 reserve1);
    event Swap(
        address indexed sender,
        address indexed recipient,
        int256 amount0,
        int256 amount1,
        uint160 sqrtPriceX96,
        uint128 liquidity,
        int24 tick
    );
    event Initialize(uint160 sqrtPriceX96, int24 tick);
    event Mint(
        address sender,
        address indexed owner,
        int24 indexed tickLower,
        int24 indexed tickUpper,
        uint128 amount,
        uint256 amount0,
        uint256 amount1
    );
    event Burn(
        address indexed owner,
        int24 indexed tickLower,
        int24 indexed tickUpper,
        uint128 amount,
        uint256 amount0,
        uint256 amount1
    );
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
enum RawPayload {
    Flashblock(FlashblockEvent),
    PendingLog(PendingLogEvent),
}

pub struct DexDecoder {
    metrics: Arc<MetricsRegistry>,
}

impl DexDecoder {
    pub fn new(metrics: Arc<MetricsRegistry>) -> Self {
        Self { metrics }
    }

    pub fn decode_log(&self, log: &PendingLogEvent) -> Option<PoolUpdate> {
        let topics: Vec<B256> = log.topics.iter()
            .filter_map(|t| B256::from_str(t).ok())
            .collect();
        
        if topics.is_empty() { return None; }

        let data_bytes = match hex::decode(log.data.trim_start_matches("0x")) {
            Ok(d) => Bytes::from(d),
            Err(_) => return None,
        };

        let log_data = LogData::new_unchecked(topics.clone(), data_bytes);
        let address = Address::from_str(&log.address).ok()?;
        
        let alloy_log = alloy_primitives::Log {
            address,
            data: log_data,
        };

        let topic0 = topics[0];
        
        if topic0 == Sync::SIGNATURE_HASH {
            if let Ok(sync) = Sync::decode_log(&alloy_log, false) {
                self.metrics.inc_dex_sync_events();
                return Some(PoolUpdate {
                    pool_id: PoolId(log.address.clone()),
                    kind: PoolKind::ReserveBased,
                    token0: TokenAddress("".to_string()), 
                    token1: TokenAddress("".to_string()),
                    reserves: Some(ReserveSnapshot {
                        reserve0: {
                            let b: [u8; 14] = sync.reserve0.to_be_bytes();
                            let mut buf = [0u8; 16];
                            buf[2..16].copy_from_slice(&b);
                            u128::from_be_bytes(buf)
                        },
                        reserve1: {
                            let b: [u8; 14] = sync.reserve1.to_be_bytes();
                            let mut buf = [0u8; 16];
                            buf[2..16].copy_from_slice(&b);
                            u128::from_be_bytes(buf)
                        },
                    }),
                    cl_snapshot: None,
                    cl_full_state: None,
                    stamp: EventStamp { block_number: 0, log_index: 0 },
                });
            }
        } else if topic0 == Swap::SIGNATURE_HASH {
            if let Ok(swap) = Swap::decode_log(&alloy_log, false) {
                self.metrics.inc_dex_cl_swap_events();
                return Some(PoolUpdate {
                    pool_id: PoolId(log.address.clone()),
                    kind: PoolKind::ConcentratedLiquidity,
                    token0: TokenAddress("".to_string()),
                    token1: TokenAddress("".to_string()),
                    reserves: None,
                    cl_snapshot: Some(CLSnapshot {
                        sqrt_price_x96: {
                            let b: [u8; 20] = swap.sqrtPriceX96.to_be_bytes();
                            let mut buf = [0u8; 32];
                            buf[12..32].copy_from_slice(&b);
                            U256::from_be_bytes(buf)
                        },
                        liquidity: {
                            let b: [u8; 16] = swap.liquidity.to_be_bytes();
                            U128::from_be_bytes(b)
                        },
                        tick: {
                            let mut b = [0u8; 4];
                            let src: [u8; 3] = swap.tick.to_be_bytes();
                            if src[0] & 0x80 != 0 { b[0] = 0xFF; }
                            b[1..4].copy_from_slice(&src);
                            i32::from_be_bytes(b)
                        },
                    }),
                    cl_full_state: None,
                    stamp: EventStamp { block_number: 0, log_index: 0 },
                });
            }
        } else if topic0 == Initialize::SIGNATURE_HASH {
            if let Ok(init) = Initialize::decode_log(&alloy_log, false) {
                return Some(PoolUpdate {
                    pool_id: PoolId(log.address.clone()),
                    kind: PoolKind::ConcentratedLiquidity,
                    token0: TokenAddress("".to_string()),
                    token1: TokenAddress("".to_string()),
                    reserves: None,
                    cl_snapshot: None,
                    cl_full_state: Some(CLFullState {
                        sqrt_price_x96: {
                            let b: [u8; 20] = init.sqrtPriceX96.to_be_bytes();
                            let mut buf = [0u8; 32];
                            buf[12..32].copy_from_slice(&b);
                            U256::from_be_bytes(buf)
                        },
                        liquidity: U128::ZERO,
                        tick: {
                            let mut b = [0u8; 4];
                            let src: [u8; 3] = init.tick.to_be_bytes();
                            if src[0] & 0x80 != 0 { b[0] = 0xFF; }
                            b[1..4].copy_from_slice(&src);
                            i32::from_be_bytes(b)
                        },
                        ticks: HashMap::new(),
                    }),
                    stamp: EventStamp { block_number: 0, log_index: 0 },
                });
            }
        } else if topic0 == Mint::SIGNATURE_HASH {
            if let Ok(mint) = Mint::decode_log(&alloy_log, false) {
                let amount = {
                    let b: [u8; 16] = mint.amount.to_be_bytes();
                    u128::from_be_bytes(b)
                };
                let tick_lower = {
                    let mut b = [0u8; 4];
                    let src: [u8; 3] = mint.tickLower.to_be_bytes();
                    if src[0] & 0x80 != 0 { b[0] = 0xFF; }
                    b[1..4].copy_from_slice(&src);
                    i32::from_be_bytes(b)
                };
                let tick_upper = {
                    let mut b = [0u8; 4];
                    let src: [u8; 3] = mint.tickUpper.to_be_bytes();
                    if src[0] & 0x80 != 0 { b[0] = 0xFF; }
                    b[1..4].copy_from_slice(&src);
                    i32::from_be_bytes(b)
                } ;
                
                let mut ticks = HashMap::new();
                ticks.insert(tick_lower, CLTickState { liquidity_gross: amount, liquidity_net: amount as i128 });
                ticks.insert(tick_upper, CLTickState { liquidity_gross: amount, liquidity_net: -(amount as i128) });

                return Some(PoolUpdate {
                    pool_id: PoolId(log.address.clone()),
                    kind: PoolKind::ConcentratedLiquidity,
                    token0: TokenAddress("".to_string()),
                    token1: TokenAddress("".to_string()),
                    reserves: None,
                    cl_snapshot: None,
                    cl_full_state: Some(CLFullState {
                        sqrt_price_x96: U256::ZERO,
                        liquidity: U128::from(amount),
                        tick: 0,
                        ticks,
                    }),
                    stamp: EventStamp { block_number: 0, log_index: 0 },
                });
            }
        } else if topic0 == Burn::SIGNATURE_HASH {
            if let Ok(burn) = Burn::decode_log(&alloy_log, false) {
                let amount = {
                    let b: [u8; 16] = burn.amount.to_be_bytes();
                    u128::from_be_bytes(b)
                };
                let tick_lower = {
                    let mut b = [0u8; 4];
                    let src: [u8; 3] = burn.tickLower.to_be_bytes();
                    if src[0] & 0x80 != 0 { b[0] = 0xFF; }
                    b[1..4].copy_from_slice(&src);
                    i32::from_be_bytes(b)
                };
                let tick_upper = {
                    let mut b = [0u8; 4];
                    let src: [u8; 3] = burn.tickUpper.to_be_bytes();
                    if src[0] & 0x80 != 0 { b[0] = 0xFF; }
                    b[1..4].copy_from_slice(&src);
                    i32::from_be_bytes(b)
                };

                let mut ticks = HashMap::new();
                ticks.insert(tick_lower, CLTickState { liquidity_gross: amount, liquidity_net: -(amount as i128) });
                ticks.insert(tick_upper, CLTickState { liquidity_gross: amount, liquidity_net: amount as i128 });

                return Some(PoolUpdate {
                    pool_id: PoolId(log.address.clone()),
                    kind: PoolKind::ConcentratedLiquidity,
                    token0: TokenAddress("".to_string()),
                    token1: TokenAddress("".to_string()),
                    reserves: None,
                    cl_snapshot: None,
                    cl_full_state: Some(CLFullState {
                        sqrt_price_x96: U256::ZERO,
                        liquidity: U128::from(amount),
                        tick: 0,
                        ticks,
                    }),
                    stamp: EventStamp { block_number: 0, log_index: 0 },
                });
            }
        }
        
        self.metrics.inc_unsupported_dex_logs();
        None
    }
}

pub struct IngestPipeline {
    tx: broadcast::Sender<IngestEvent>,
    metrics: Arc<MetricsRegistry>,
    pub decoder: DexDecoder,
}

impl IngestPipeline {
    pub fn new(capacity: usize, metrics: Arc<MetricsRegistry>) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        let decoder = DexDecoder::new(metrics.clone());
        Self { tx, metrics, decoder }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<IngestEvent> {
        self.tx.subscribe()
    }

    pub fn broadcast_event(&self, event: IngestEvent) -> Result<usize, broadcast::error::SendError<IngestEvent>> {
        self.tx.send(event)
    }

    pub fn handle_raw_payload(&self, payload: &str) {
        match serde_json::from_str::<RawPayload>(payload) {
            Ok(RawPayload::Flashblock(fb)) => {
                self.metrics.inc_events_ingested();
                self.metrics.inc_flashblocks_seen();
                let _ = self.broadcast_event(IngestEvent::Flashblock(fb));
            }
            Ok(RawPayload::PendingLog(pl)) => {
                self.metrics.inc_events_ingested();
                self.metrics.inc_pending_logs_seen();
                
                // Attempt decoding for state-relevant DEX logs
                if let Some(_pool_update) = self.decoder.decode_log(&pl) {
                    // In Phase 4, we broadcast the raw log. 
                    // The bridge in arb_daemon will handle the conversion to PoolUpdate.
                    // This keeps the pipeline generic for now.
                }
                
                let _ = self.broadcast_event(IngestEvent::PendingLog(pl));
            }
            Err(e) => {
                self.metrics.inc_malformed_payloads();
                let _ = e;
            }
        }
    }
}


pub struct ReplayHarness {
    fixture_path: String,
}

impl ReplayHarness {
    pub fn new(fixture_path: String) -> Self {
        Self { fixture_path }
    }

    pub async fn run_replay(&self, pipeline: &IngestPipeline) -> Result<(), String> {
        let path = Path::new(&self.fixture_path);
        if !path.exists() {
            return Err(format!("Fixture file does not exist: {}", self.fixture_path));
        }

        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        for line in content.lines() {
            if !line.trim().is_empty() {
                pipeline.handle_raw_payload(line);
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Unit tests for DEX decoding deferred to Phase 6 due to ABI padding complexities in mock environments.
}
