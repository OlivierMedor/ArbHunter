use std::sync::Arc;
use arb_ingest::DexDecoder;
use arb_metrics::MetricsRegistry;
use arb_types::PendingLogEvent;

#[tokio::main]
async fn main() {
    let metrics = Arc::new(MetricsRegistry::new());
    let decoder = DexDecoder::new(metrics.clone());
    
    let sync_log = PendingLogEvent {
        address: "0x1111111111111111111111111111111111111111".to_string(),
        topics: vec![
            "0x1c91f030eb7c0a042c0211d40a5440311dec3b1285bc035ede49007f502574e4".to_string(),
        ],
        data: "0x000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000c8".to_string(),
        transaction_hash: "0x...".to_string(),
        block_number: 100,
        log_index: 0,
    };

    println!("Attempting to decode Sync log...");
    match decoder.decode_log(&sync_log) {
        Some(update) => println!("Sync Success: {:?}", update),
        None => println!("Sync Failed"),
    }

    let swap_log = PendingLogEvent {
        address: "0x2222222222222222222222222222222222222222".to_string(),
        topics: vec![
            "0xc42079f94a6350d7e5735f2399b6d8de98a486f7af7d160cfd333044e7c75db1".to_string(),
            "0x000000000000000000000000000000000000000000000000000000000000dead".to_string(),
            "0x000000000000000000000000000000000000000000000000000000000000beef".to_string(),
        ],
        data: "0x0000000000000000000000000000000000000000000000000000000000000064\
                    ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff9c\
                    0000000000000000000000000000000000000000000000010000000000000000\
                    00000000000000000000000000000000000000000000000000000000000f4240\
                    0000000000000000000000000000000000000000000000000000000000000000".replace(|c: char| c.is_whitespace() || c == '\\', ""),
        transaction_hash: "0x...".to_string(),
        block_number: 100,
        log_index: 1,
    };

    println!("Attempting to decode Swap log...");
    match decoder.decode_log(&swap_log) {
        Some(update) => println!("Swap Success: {:?}", update),
        None => println!("Swap Failed"),
    }
}
