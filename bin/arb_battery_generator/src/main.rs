use arb_types::{HistoricalCase, GuardOverrides, TokenAddress, PoolKind};
use alloy_primitives::{U256, Address, B256};
use alloy_provider::{ProviderBuilder, Provider};
use arb_config::Config;
use std::fs;
use hex;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("DEBUG: Generator main started.");
    let config = Config::load();
    println!("DEBUG: Config loaded.");
    
    let rpc_url = config.rpc_http_url.as_ref()
        .filter(|s| !s.is_empty())
        .cloned()
        .or(config.alchemy_wss_url.as_ref().map(|s| s.replace("wss://", "https://")))
        .expect("No valid RPC URL found in .env");
        
    println!("Using RPC URL: {}", rpc_url);
    let provider = ProviderBuilder::new().on_http(rpc_url.parse()?);

    println!("DEBUG: Calling get_block_number...");
    let latest_block = provider.get_block_number().await.expect("FAILED get_block_number");
    println!("DEBUG: Latest block retrieved: {}", latest_block);

    let mut cases = Vec::new();

    // 4 ACTIVE POOLS ON BASE MAINNET (Verified via Browser)
    let pools = vec![
        ("0x6C561b446416e1a00e8e93e221854d6EA4171372", PoolKind::ConcentratedLiquidity, "WETH/USDC V3 0.3%"),
        ("0xd0b53D9277642d899df5C87A3966a349A798f224", PoolKind::ConcentratedLiquidity, "USDC/WETH V3 0.05%"),
        ("0xb2cc224c1C9fEE385f8ad6A55b4D94E92359dc59", PoolKind::ConcentratedLiquidity, "WETH/USDC Aero Slipstream"),
        ("0x67b00b46fA4F4F24c03855c5c8013C0b938b3Eec", PoolKind::ReserveBased, "DAI/USDC Aerodrome Stable"),
    ];

    for (p_addr, kind, label) in pools {
        let pool_address = Address::from_str(p_addr)?;
        let block_number = latest_block - 50; // Use a very recent block to ensure it's "live"
        let log_block_hex = format!("0x{:x}", block_number);
        
        let t0_val: serde_json::Value = provider.raw_request("eth_call".into(), (serde_json::json!({"to": pool_address, "data": "0x0dfe1681"}), &log_block_hex)).await.unwrap_or(serde_json::Value::Null);
        let t1_val: serde_json::Value = provider.raw_request("eth_call".into(), (serde_json::json!({"to": pool_address, "data": "0xd21220a7"}), &log_block_hex)).await.unwrap_or(serde_json::Value::Null);

        let t0_str = t0_val.as_str().unwrap_or("0x");
        let t1_str = t1_val.as_str().unwrap_or("0x");

        if t0_str == "0x" || t1_str == "0x" || t0_str.len() < 66 || t1_str.len() < 66 {
            println!("DEBUG: Failed to fetch tokens for {} ({}) - response was {} / {}", label, pool_address, t0_str, t1_str);
            continue;
        }

        let token0 = Address::from_str(&format!("0x{}", &t0_str[t0_str.len()-40..]))?;
        let token1 = Address::from_str(&format!("0x{}", &t1_str[t1_str.len()-40..]))?;

        let case_id = format!("case_{}_{:?}_success", cases.len() + 1, kind);
        cases.push(HistoricalCase {
            case_id: case_id.clone(),
            notes: format!("Historical {} pool at block {}.", label, block_number),
            fork_block_number: block_number,
            root_asset: TokenAddress(token0.to_string()),
            route_family: format!("{:?}_SingleLeg", kind),
            pool_ids: vec![pool_address.to_string()],
            pool_kinds: vec![kind],
            path_tokens: vec![TokenAddress(token0.to_string()), TokenAddress(token1.to_string())],
            leg_directions: vec![true],
            amount_in: U256::from(100_000_000_000_000_000u128), // 0.1 tokens nominal
            expected_outcome: "success".to_string(),
            guard_overrides: None,
            seed_data: None,
        });
        
        // Add one failure case derivative for logic testing
        if cases.len() == 1 {
            cases.push(HistoricalCase {
                case_id: "case_2_slippage_revert".to_string(),
                notes: "Forced slippage revert derived from Case 1.".to_string(),
                fork_block_number: block_number,
                root_asset: TokenAddress(token0.to_string()),
                route_family: format!("{:?}_SingleLeg", kind),
                pool_ids: vec![pool_address.to_string()],
                pool_kinds: vec![kind],
                path_tokens: vec![TokenAddress(token0.to_string()), TokenAddress(token1.to_string())],
                leg_directions: vec![true],
                amount_in: U256::from(100_000_000_000_000_000u128),
                expected_outcome: "slippage_revert".to_string(),
                guard_overrides: Some(GuardOverrides {
                    min_amount_out: Some(U256::MAX),
                    min_profit_wei: None,
                }),
                seed_data: None,
            });
        }
    }

    if cases.is_empty() {
        return Err("No historical cases generated.".into());
    }

    fs::create_dir_all("fixtures").unwrap();
    let json = serde_json::to_string_pretty(&cases).unwrap();
    fs::write("fixtures/historical_cases.json", json).unwrap();
    println!("Successfully generated fixtures/historical_cases.json with {} honest cases.", cases.len());
    
    Ok(())
}
