use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::broadcast;
use serde::Deserialize;
use arb_types::{
    FlashblockEvent, IngestEvent, PendingLogEvent, PoolUpdate, PoolId, 
    PoolKind, ReserveSnapshot, TokenAddress, EventStamp, CLSnapshot
};
use arb_metrics::MetricsRegistry;

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
        eprintln!("DEBUG: ENTERING decode_log");
        if log.topics.is_empty() {
            eprintln!("DEBUG: No topics");
            return None;
        }

        let topic0 = match B256::from_str(&log.topics[0]) {
            Ok(t) => t,
            Err(_) => return None,
        };

        let data_bytes = match hex::decode(log.data.trim_start_matches("0x")) {
            Ok(d) => Bytes::from(d),
            Err(_) => return None,
        };

        // Match topics using literal hashes for robustness
        let sync_sig = b256!("1c91f030eb7c0a042c0211d40a5440311dec3b1285bc035ede49007f502574e4");
        let swap_sig = b256!("c42079f94a6350d7e5735f2399b6d8de98a486f7af7d160cfd333044e7c75db1");

        if topic0 == sync_sig {
            use alloy_sol_types::{SolType, sol_data::*};
            type SyncData = (Uint<112>, Uint<112>);
            if let Ok(sync_data) = SyncData::abi_decode(&data_bytes, true) {
                self.metrics.inc_dex_sync_events();
                return Some(PoolUpdate {
                    pool_id: PoolId(log.address.clone()),
                    kind: PoolKind::ReserveBased,
                    token0: TokenAddress("".to_string()), 
                    token1: TokenAddress("".to_string()),
                    reserves: Some(ReserveSnapshot {
                        reserve0: {
                            let b: [u8; 14] = sync_data.0.to_be_bytes();
                            let mut buf = [0u8; 16];
                            buf[2..16].copy_from_slice(&b);
                            u128::from_be_bytes(buf)
                        },
                        reserve1: {
                            let b: [u8; 14] = sync_data.1.to_be_bytes();
                            let mut buf = [0u8; 16];
                            buf[2..16].copy_from_slice(&b);
                            u128::from_be_bytes(buf)
                        },
                    }),
                    cl_snapshot: None,
                    stamp: EventStamp {
                        block_number: 0,
                        log_index: 0,
                    },
                });
            }
        } else if topic0 == swap_sig {
            use alloy_sol_types::{SolType, sol_data::*};
            // amount0, amount1, sqrtPriceX96, liquidity, tick
            type SwapData = (Int<256>, Int<256>, Uint<160>, Uint<128>, Int<24>);
            if let Ok(swap_data) = SwapData::abi_decode(&data_bytes, true) {
                self.metrics.inc_dex_cl_swap_events();
                
                return Some(PoolUpdate {
                    pool_id: PoolId(log.address.clone()),
                    kind: PoolKind::ConcentratedLiquidity,
                    token0: TokenAddress("".to_string()),
                    token1: TokenAddress("".to_string()),
                    reserves: None,
                    cl_snapshot: Some(CLSnapshot {
                        sqrt_price_x96: U256::from_be_bytes({
                            let mut b = [0u8; 32];
                            let src: [u8; 20] = swap_data.2.to_be_bytes();
                            b[12..32].copy_from_slice(&src);
                            b
                        }),
                        liquidity: U128::from(u128::from_be_bytes(swap_data.3.to_be_bytes())),
                        tick: {
                            let tick_bytes: [u8; 3] = swap_data.4.to_be_bytes();
                            let mut b = [0u8; 4];
                            if tick_bytes[0] & 0x80 != 0 {
                                b[0] = 0xFF;
                            }
                            b[1..4].copy_from_slice(&tick_bytes);
                            i32::from_be_bytes(b)
                        },
                    }),
                    stamp: EventStamp {
                        block_number: 0,
                        log_index: 0,
                    },
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
        use std::fs;
        use std::path::Path;
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

    #[tokio::test]
    async fn test_decode_v2_sync() {
        let metrics = Arc::new(MetricsRegistry::new());
        let decoder = DexDecoder::new(metrics.clone());
        let log = PendingLogEvent {
            address: "0x1111111111111111111111111111111111111111".to_string(),
            topics: vec![
                "0x1c91f030eb7c0a042c0211d40a5440311dec3b1285bc035ede49007f502574e4".to_string(),
            ],
            data: "0x000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000c8".to_string(),
            transaction_hash: "0x...".to_string(),
        };

        let update = decoder.decode_log(&log).expect("Should decode Sync log");
        assert_eq!(update.kind, arb_types::PoolKind::ReserveBased);
        let reserves = update.reserves.unwrap();
        assert_eq!(reserves.reserve0, 100);
        assert_eq!(reserves.reserve1, 200);
    }

    #[tokio::test]
    async fn test_decode_v3_swap() {
        let metrics = Arc::new(MetricsRegistry::new());
        let decoder = DexDecoder::new(metrics.clone());
        let log = PendingLogEvent {
            address: "0x2222222222222222222222222222222222222222".to_string(),
            topics: vec![
                "0xc42079f94a6350d7e5735f2399b6d8de98a486f7af7d160cfd333044e7c75db1".to_string(),
                "0x000000000000000000000000000000000000000000000000000000000000dead".to_string(),
                "0x000000000000000000000000000000000000000000000000000000000000beef".to_string(),
            ],
            // 5 slots: amount0, amount1, sqrtPriceX96, liquidity, tick
            data: "0x0000000000000000000000000000000000000000000000000000000000000064ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff9c000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000f42400000000000000000000000000000000000000000000000000000000000000000".to_string(),
            transaction_hash: "0x...".to_string(),
        };

        let update = decoder.decode_log(&log).expect("Should decode Swap log");
        assert_eq!(update.kind, arb_types::PoolKind::ConcentratedLiquidity);
        let cl = update.cl_snapshot.unwrap();
        assert_eq!(cl.tick, 0);
        assert_eq!(cl.liquidity, alloy_primitives::U128::from(1000000u128));
    }
}
