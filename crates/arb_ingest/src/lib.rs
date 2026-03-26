use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::broadcast;
use serde::Deserialize;
use arb_types::{
    FlashblockEvent, IngestEvent, PendingLogEvent, PoolUpdate, PoolId, 
    PoolKind, ReserveSnapshot, EventStamp, CLSnapshot,
    CLTickState, CLFullState
};
use arb_metrics::MetricsRegistry;
use std::collections::HashMap;

use std::str::FromStr;
use alloy_sol_types::{sol, SolEvent};
use alloy_primitives::{Address, Bytes, B256, U256, U128, LogData, b256};
use hex;

sol! {
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

        let log_data = LogData::new_unchecked(topics.clone(), data_bytes.clone());
        let address = Address::from_str(&log.address).ok()?;
        
        let alloy_log = alloy_primitives::Log {
            address,
            data: log_data,
        };

        let topic0 = topics[0];
        
        // Literal Signature Hashes
        let v2_sync_sig = b256!("1c91f030eb7c0a042c0211d40a5440311dec3b1285bc035ede49007f502574e4");
        let v3_swap_sig = b256!("c42079f94a6350d7e5735f2399b6d8de98a486f7af7d160cfd333044e7c75db1");
        let v3_init_sig = b256!("98636036cb66a7c1b51e5e34381ec12c96c442432650ee4a26e84cf92b8d0e51");
        let v3_mint_sig = b256!("7612745a114c40bc3a1709c991afbc55848d47155e7104b07fb8d1964f33fd20");
        let v3_burn_sig = b256!("0c396cd989a39f4459b53fa87f33f303dc2738e373a92cacfd67e028cd568da9");

        if topic0 == v2_sync_sig {
            if data_bytes.len() >= 64 {
                let r0 = U256::from_be_slice(&data_bytes[0..32]);
                let r1 = U256::from_be_slice(&data_bytes[32..64]);
                self.metrics.inc_dex_sync_events();
                return Some(PoolUpdate {
                    pool_id: PoolId(log.address.clone()),
                    kind: PoolKind::ReserveBased,
                    token0: None, 
                    token1: None,
                    fee_bps: Some(30),
                    reserves: Some(ReserveSnapshot {
                        reserve0: r0.saturating_to::<u128>(),
                        reserve1: r1.saturating_to::<u128>(),
                    }),
                    cl_snapshot: None,
                    cl_full_state: None,
                    stamp: EventStamp { block_number: log.block_number, log_index: log.log_index },
                });
            }
        } else if topic0 == v3_swap_sig {
            if let Ok(swap) = Swap::decode_log(&alloy_log, false) {
                self.metrics.inc_dex_cl_swap_events();
                return Some(PoolUpdate {
                    pool_id: PoolId(log.address.clone()),
                    kind: PoolKind::ConcentratedLiquidity,
                    token0: None,
                    token1: None,
                    fee_bps: Some(30),
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
                    stamp: EventStamp { block_number: log.block_number, log_index: log.log_index },
                });
            }
        } else if topic0 == v3_init_sig {
            if data_bytes.len() >= 64 {
                let sqrt_p = U256::from_be_slice(&data_bytes[0..32]);
                let tick = {
                    let mut b = [0u8; 4];
                    let src = &data_bytes[32..64];
                    let tick_bytes = &src[29..32];
                    if tick_bytes[0] & 0x80 != 0 { b[0] = 0xFF; }
                    b[1..4].copy_from_slice(tick_bytes);
                    i32::from_be_bytes(b)
                };
                return Some(PoolUpdate {
                    pool_id: PoolId(log.address.clone()),
                    kind: PoolKind::ConcentratedLiquidity,
                    token0: None,
                    token1: None,
                    fee_bps: Some(30),
                    reserves: None,
                    cl_snapshot: None,
                    cl_full_state: Some(CLFullState {
                        sqrt_price_x96: sqrt_p,
                        liquidity: U128::ZERO,
                        tick,
                        ticks: HashMap::new(),
                    }),
                    stamp: EventStamp { block_number: log.block_number, log_index: log.log_index },
                });
            }
        } else if topic0 == v3_mint_sig {
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
                    token0: None,
                    token1: None,
                    fee_bps: Some(30),
                    reserves: None,
                    cl_snapshot: None,
                    cl_full_state: Some(CLFullState {
                        sqrt_price_x96: U256::ZERO,
                        liquidity: U128::from(amount),
                        tick: 0,
                        ticks,
                    }),
                    stamp: EventStamp { block_number: log.block_number, log_index: log.log_index },
                });
            }
        } else if topic0 == v3_burn_sig {
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
                    token0: None,
                    token1: None,
                    fee_bps: Some(30),
                    reserves: None,
                    cl_snapshot: None,
                    cl_full_state: Some(CLFullState {
                        sqrt_price_x96: U256::ZERO,
                        liquidity: U128::from(amount),
                        tick: 0,
                        ticks,
                    }),
                    stamp: EventStamp { block_number: log.block_number, log_index: log.log_index },
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

    #[tokio::test]
    async fn test_decode_v2_sync() {
        let metrics = Arc::new(MetricsRegistry::new());
        let decoder = DexDecoder::new(metrics.clone());
        let log = PendingLogEvent {
            address: "0x0000000000000000000000000000000000000001".to_string(),
            topics: vec!["0x1c91f030eb7c0a042c0211d40a5440311dec3b1285bc035ede49007f502574e4".to_string()],
            data: "0x00000000000000000000000000000000000000000000000000000000000003e800000000000000000000000000000000000000000000000000000000000007d0".to_string(),
            transaction_hash: "0x...".to_string(),
            block_number: 100,
            log_index: 1,
        };

        let update = decoder.decode_log(&log).expect("Should decode Sync log");
        assert_eq!(update.kind, PoolKind::ReserveBased);
        assert_eq!(update.stamp.block_number, 100);
        assert_eq!(update.reserves.unwrap().reserve0, 1000);
    }

    #[tokio::test]
    async fn test_decode_v3_initialize() {
        let metrics = Arc::new(MetricsRegistry::new());
        let decoder = DexDecoder::new(metrics.clone());
        let log = PendingLogEvent {
            address: "0x0000000000000000000000000000000000000003".to_string(),
            topics: vec![
                "0x98636036cb66a7c1b51e5e34381ec12c96c442432650ee4a26e84cf92b8d0e51".to_string(),
            ],
            data: "0x00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000".to_string(),
            transaction_hash: "0x...".to_string(),
            block_number: 100,
            log_index: 2,
        };

        let update = decoder.decode_log(&log).expect("Should decode Initialize log");
        assert_eq!(update.kind, PoolKind::ConcentratedLiquidity);
        assert!(update.cl_full_state.is_some());
    }

    #[tokio::test]
    async fn test_replay_fixtures() {
        let metrics = Arc::new(MetricsRegistry::new());
        let pipeline = IngestPipeline::new(10, metrics.clone());
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let fixture_path = std::path::Path::new(&manifest_dir).join("../../fixtures/pending_logs.jsonl");
        let harness = ReplayHarness::new(fixture_path.to_str().unwrap().to_string());
        
        let mut rx = pipeline.subscribe();
        
        // This is a unit test, we can use a small count
        harness.run_replay(&pipeline).await.unwrap();
        
        let ev1 = rx.try_recv().unwrap();
        if let IngestEvent::PendingLog(pl) = ev1 {
            assert_eq!(pl.block_number, 12345678);
        } else { panic!("Expected PendingLog"); }

        let ev2 = rx.try_recv().unwrap();
        if let IngestEvent::PendingLog(pl) = ev2 {
            assert_eq!(pl.log_index, 2);
        } else { panic!("Expected PendingLog"); }
    }
}
