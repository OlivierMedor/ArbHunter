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
    pool_registry: HashMap<Address, (Address, Address, u32)>,
}

impl DexDecoder {
    pub fn new(metrics: Arc<MetricsRegistry>) -> Self {
        let mut pool_registry = HashMap::new();
        
        // Base WETH & USDC
        let weth = Address::from_str("0x4200000000000000000000000000000000000006").unwrap();
        let usdc = Address::from_str("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913").unwrap();
        let dai = Address::from_str("0x50c5725949A6F0c72E6C4d6412B4714b9c17d74D").unwrap();
        let cbeth = Address::from_str("0x2Ae3F1Ec7F1F5012CEEab0185B7d5038c642eE4F").unwrap();
        let eurc = Address::from_str("0x60a3E35Cc302bFA44Cb288Bc5a4F316Fdb1adb42").unwrap();

        // 1. Uniswap V3 WETH/USDC 0.05%
        pool_registry.insert(
            Address::from_str("0xd0b53D9277af2a1232B10DA23019458451D59Ab8").unwrap(),
            (usdc, weth, 5)
        );

        // 2. Uniswap V3 WETH/USDC 0.3%
        pool_registry.insert(
            Address::from_str("0x4C36388bE6F5448377Ec0abc25d97aa2c2D2833F").unwrap(),
            (usdc, weth, 30)
        );

        // 3. Aerodrome WETH/USDC (vAMM)
        pool_registry.insert(
            Address::from_str("0xcAD71900fBBc18eB3f412499d638914c6210f06B").unwrap(),
            (usdc, weth, 30)
        );

        // 4. More active WETH/USDC pools from logs
        pool_registry.insert(
            Address::from_str("0x088b26d97270cec5922176654ec484a1b307ba90").unwrap(),
            (usdc, weth, 30)
        );
        pool_registry.insert(
            Address::from_str("0x7f6052f7cb4f473721ab4d12a6b3b43e7e2e5781").unwrap(),
            (usdc, weth, 30)
        );

        // --- NEW TOKENS (DAI, cbETH, EURC) ---

        // 5. Aerodrome sAMM DAI/USDC (Stable)
        pool_registry.insert(
            Address::from_str("0xcd462f495b450625488737c15eac874591461a34").unwrap(),
            (dai, usdc, 1)
        );

        // 6. Aerodrome vAMM cbETH/WETH (Volatile)
        pool_registry.insert(
            Address::from_str("0x10F1D49581Aca85949e295368aEF87B114316912").unwrap(),
            (cbeth, weth, 30)
        );

        // 7. Aerodrome sAMM EURC/USDC (Stable)
        pool_registry.insert(
            Address::from_str("0x5825227AC345152864C6551b9E361D4C30D4e277").unwrap(),
            (eurc, usdc, 1)
        );

        // 8. Uniswap V3 cbETH/WETH (0.05%)
        pool_registry.insert(
            Address::from_str("0x10648ba41b8565907cfa1496765fa4d95390aa0d").unwrap(),
            (cbeth, weth, 5)
        );

        // 9. Uniswap V3 EURC/USDC (0.01%)
        pool_registry.insert(
            Address::from_str("0x7279c08a36333e12c3fc81747963264c100d66fb").unwrap(),
            (eurc, usdc, 1)
        );

        Self { metrics, pool_registry }
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
        let v2_sync_sig = b256!("1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1");
        let v3_swap_sig = b256!("c42079f94a6350d7e5735f2399b6d8de98a486f7af7d160cfd333044e7c75db1");
        let v3_init_sig = b256!("98636036cb66a7c1b51e5e34381ec12c96c442432650ee4a26e84cf92b8d0e51");
        let v3_mint_sig = b256!("7612745a114c40bc3a1709c991afbc55848d47155e7104b07fb8d1964f33fd20");
        let v3_burn_sig = b256!("0c396cd989a39f4459b53fa87f33f303dc2738e373a92cacfd67e028cd568da9");

        if topic0 == v2_sync_sig {
            if data_bytes.len() >= 64 {
                let r0 = U256::from_be_slice(&data_bytes[0..32]);
                let r1 = U256::from_be_slice(&data_bytes[32..64]);
                self.metrics.inc_dex_sync_events();
                let (t0, t1, fee) = match self.pool_registry.get(&address) {
                    Some((a, b, f)) => (Some(arb_types::TokenAddress(format!("{:#x}", a))), Some(arb_types::TokenAddress(format!("{:#x}", b))), Some(*f)),
                    None => {
                        tracing::debug!(?address, "DEX log from unknown pool address. Skipping.");
                        (None, None, None)
                    }
                };

                if t0.is_none() { return None; }

                return Some(PoolUpdate {
                    pool_id: PoolId(log.address.clone()),
                    kind: PoolKind::ReserveBased,
                    token0: t0, 
                    token1: t1,
                    fee_bps: fee,
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
                let (t0, t1, fee) = match self.pool_registry.get(&address) {
                    Some((a, b, f)) => (Some(arb_types::TokenAddress(format!("{:#x}", a))), Some(arb_types::TokenAddress(format!("{:#x}", b))), Some(*f)),
                    None => {
                        tracing::debug!(?address, "DEX log from unknown pool address. Skipping.");
                        (None, None, None)
                    }
                };

                if t0.is_none() { return None; }

                return Some(PoolUpdate {
                    pool_id: PoolId(log.address.clone()),
                    kind: PoolKind::ConcentratedLiquidity,
                    token0: t0,
                    token1: t1,
                    fee_bps: fee,
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
                let (t0, t1, fee) = match self.pool_registry.get(&address) {
                    Some((a, b, f)) => (Some(arb_types::TokenAddress(format!("{:#x}", a))), Some(arb_types::TokenAddress(format!("{:#x}", b))), Some(*f)),
                    None => {
                        tracing::debug!(?address, "DEX log from unknown pool address. Skipping.");
                        (None, None, None)
                    }
                };

                if t0.is_none() { return None; }

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
                    token0: t0,
                    token1: t1,
                    fee_bps: fee,
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
                let (t0, t1, fee) = match self.pool_registry.get(&address) {
                    Some((a, b, f)) => (Some(arb_types::TokenAddress(format!("{:#x}", a))), Some(arb_types::TokenAddress(format!("{:#x}", b))), Some(*f)),
                    None => {
                        tracing::debug!(?address, "DEX log from unknown pool address. Skipping.");
                        (None, None, None)
                    }
                };

                if t0.is_none() { return None; }

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
                    token0: t0,
                    token1: t1,
                    fee_bps: fee,
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
                let (t0, t1, fee) = match self.pool_registry.get(&address) {
                    Some((a, b, f)) => (Some(arb_types::TokenAddress(format!("{:#x}", a))), Some(arb_types::TokenAddress(format!("{:#x}", b))), Some(*f)),
                    None => {
                        tracing::debug!(?address, "DEX log from unknown pool address. Skipping.");
                        (None, None, None)
                    }
                };

                if t0.is_none() { return None; }

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
                    token0: t0,
                    token1: t1,
                    fee_bps: fee,
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

#[derive(Deserialize, Debug)]
struct EthSubscriptionLog {
    address: String,
    topics: Vec<String>,
    data: String,
    #[serde(rename = "transactionHash")]
    transaction_hash: String,
    #[serde(rename = "blockNumber")]
    block_number: String,
    #[serde(rename = "logIndex")]
    log_index: String,
}

#[derive(Deserialize, Debug)]
struct EthSubscriptionParams {
    result: EthSubscriptionLog,
}

#[derive(Deserialize, Debug)]
struct EthSubscriptionFrame {
    method: String,
    params: EthSubscriptionParams,
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
        // Attempt 1: Legacy format
        if let Ok(raw) = serde_json::from_str::<RawPayload>(payload) {
            match raw {
                RawPayload::Flashblock(fb) => {
                    self.metrics.inc_events_ingested();
                    self.metrics.inc_flashblocks_seen();
                    let _ = self.broadcast_event(IngestEvent::Flashblock(fb));
                }
                RawPayload::PendingLog(pl) => {
                    self.metrics.inc_events_ingested();
                    self.metrics.inc_pending_logs_seen();
                    let _ = self.broadcast_event(IngestEvent::PendingLog(pl));
                }
            }
            return;
        }

        // Attempt 2: Standard JSON-RPC format
        if let Ok(frame) = serde_json::from_str::<EthSubscriptionFrame>(payload) {
            if frame.method == "eth_subscription" {
                let log = frame.params.result;
                let parse_hex_u64 = |s: &str| u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap_or_default();
                let parse_hex_u32 = |s: &str| u32::from_str_radix(s.trim_start_matches("0x"), 16).unwrap_or_default();

                let pl = PendingLogEvent {
                    address: log.address,
                    topics: log.topics,
                    data: log.data,
                    transaction_hash: log.transaction_hash,
                    block_number: parse_hex_u64(&log.block_number),
                    log_index: parse_hex_u32(&log.log_index),
                };

                tracing::info!(address = %pl.address, "Pending log received via Alchemy/JSON-RPC");

                self.metrics.inc_events_ingested();
                self.metrics.inc_pending_logs_seen();
                let _ = self.broadcast_event(IngestEvent::PendingLog(pl));
                return;
            }
        }

        self.metrics.inc_malformed_payloads();
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

    #[tokio::test]
    async fn test_handle_standard_rpc_frame() {
        let metrics = Arc::new(MetricsRegistry::new());
        let pipeline = IngestPipeline::new(10, metrics.clone());
        let mut rx = pipeline.subscribe();

        let raw_json = r#"{"jsonrpc":"2.0","method":"eth_subscription","params":{"subscription":"0xc27dd02ab8e69866641007d2918374f2","result":{"address":"0x088b26d97270cec5922176654ec484a1b307ba90","topics":["0x1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1"],"data":"0x0000000000000000000000000000000000000000008743111df5e908e914153a0000000000000000000000000000000000000000000000070b7b933a29eeaba6","blockHash":"0x5778c0d4dda7daf3d6d174d1e3148adc16c6616f9666fb4eb455018e4d794953","blockNumber":"0x2a4c9f7","blockTimestamp":"0x69d3f0d1","transactionHash":"0xb9323baed2594239e466996267792375faff237190a6ef90db948fadac70d1d8","transactionIndex":"0x47","logIndex":"0x203","removed":false}}}"#;

        pipeline.handle_raw_payload(raw_json);

        let event = rx.try_recv().expect("Should receive an event");
        if let IngestEvent::PendingLog(pl) = event {
            assert_eq!(pl.address, "0x088b26d97270cec5922176654ec484a1b307ba90");
            assert_eq!(pl.block_number, 0x2a4c9f7);
            assert_eq!(pl.log_index, 0x203);
            assert_eq!(pl.transaction_hash, "0xb9323baed2594239e466996267792375faff237190a6ef90db948fadac70d1d8");
        } else {
            panic!("Expected PendingLog event");
        }
    }
}
