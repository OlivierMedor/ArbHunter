use ethers::prelude::*;
use std::sync::Arc;
use hex;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let start_block = args.get(1).map(|s| s.parse::<u64>().unwrap()).unwrap_or(43638000);
    let end_block = start_block + 10;
    
    println!("Diagnosing blocks {} to {} with manual filter", start_block, end_block);

    let rpc_url = "https://ultra-weathered-panorama.base-mainnet.quiknode.pro/2201752fbbf22452c52ed752559b6ddf9f5d91ea/";
    let provider = Provider::<Http>::try_from(rpc_url)?;
    
    let v2_sync_sig = H256::from_slice(&hex::decode("1c91f030eb7c0a042c0211d40a5440311dec3b1285bc035ede49007f502574e4").unwrap());
    
    // Test with alternative filter
    let filter = ethers::types::Filter::new()
        .from_block(start_block)
        .to_block(end_block)
        .topic0(ValueOrArray::Value(v2_sync_sig));
        
    let logs = provider.get_logs(&filter).await?;
    println!("Filtered logs: Found {}", logs.len());
    
    Ok(())
}
