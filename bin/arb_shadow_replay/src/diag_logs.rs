use ethers::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let rpc_url = "https://ultra-weathered-panorama.base-mainnet.quiknode.pro/2201752fbbf22452c52ed752559b6ddf9f5d91ea/";
    let provider = Provider::<Http>::try_from(rpc_url)?;
    
    let latest = provider.get_block_number().await?;
    println!("Latest block: {}", latest);
    
    let v3_swap_sig = H256::from_slice(&hex::decode("c42079f94a6350d7e5735f2399b6d8de98a486f7af7d160cfd333044e7c75db1").unwrap());
    
    let filter = Filter::new()
        .from_block(latest - 10)
        .to_block(latest)
        .topic0(v3_swap_sig);
        
    let logs = provider.get_logs(&filter).await?;
    println!("Found {} logs in last 10 blocks", logs.len());
    
    for (i, log) in logs.iter().take(10).enumerate() {
        println!("Log {}: addr={:?}, topics={:?}", i, log.address, log.topics);
    }
    
    Ok(())
}
