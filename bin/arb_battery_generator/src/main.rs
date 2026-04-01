use arb_types::{HistoricalCase, GuardOverrides, TokenAddress, PoolKind, RouteFamily};
use alloy_primitives::{U256, Address};
use alloy_provider::{ProviderBuilder, Provider};
use arb_config::Config;
use std::fs;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load();
    
    let rpc_url = config.rpc_http_url.as_ref()
        .filter(|s| !s.is_empty())
        .cloned()
        .or(config.alchemy_wss_url.as_ref().map(|s| s.replace("wss://", "https://")))
        .expect("No valid RPC URL found in .env");
        
    let provider = ProviderBuilder::new().on_http(rpc_url.parse()?);

    let latest_block = provider.get_block_number().await.expect("FAILED get_block_number");

    let mut cases = Vec::new();

    let token0_v3 = Address::from_str("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913")?;
    let weth = "0x4200000000000000000000000000000000000006".to_string();

    // 1. Success Case (V3)
    cases.push(HistoricalCase {
        case_id: "case_1_v3_success".into(),
        notes: "V3 Cyclic Success: USDC-WETH-USDC".into(),
        fork_block_number: latest_block - 10,
        source_tx_hash: Some("0x0000000000000000000000000000000000000000000000000000000000000001".into()),
        root_asset: TokenAddress(token0_v3.to_string()),
        route_family: RouteFamily::Direct,
        pool_ids: vec!["0xd0b53d9277642d899df5c87a3966a349a798f224".into(), "0x6c561b446416e1a00e8e93e221854d6ea4171372".into()],
        pool_kinds: vec![PoolKind::ConcentratedLiquidity, PoolKind::ConcentratedLiquidity],
        path_tokens: vec![TokenAddress(token0_v3.to_string()), TokenAddress(weth.clone()), TokenAddress(token0_v3.to_string())],
        leg_directions: vec![false, true],
        amount_in: U256::from(100_000_000u64),
        expected_outcome: "success".into(),
        guard_overrides: None,
        seed_data: Some(vec![
            serde_json::json!({"sqrt_price_x96": "0x2610bde4a5309320e6", "tick": -192130, "liquidity": "0x1234567890abcdef"}),
            serde_json::json!({"sqrt_price_x96": "0x2650bde4a5309320e6", "tick": -192130, "liquidity": "0x1234567890abcdef"}),
        ]),
    });

    // 2. Slippage Revert
    cases.push(HistoricalCase {
        case_id: "case_2_v3_slippage_revert".into(),
        notes: "Extreme slippage guard forced.".into(),
        fork_block_number: latest_block - 10,
        source_tx_hash: Some("0x0000000000000000000000000000000000000000000000000000000000000002".into()),
        root_asset: TokenAddress(token0_v3.to_string()),
        route_family: RouteFamily::Direct,
        pool_ids: vec!["0xd0b53d9277642d899df5c87a3966a349a798f224".into()],
        pool_kinds: vec![PoolKind::ConcentratedLiquidity],
        path_tokens: vec![TokenAddress(token0_v3.to_string()), TokenAddress(weth.clone())],
        leg_directions: vec![false],
        amount_in: U256::from(100_000_000u64),
        expected_outcome: "slippage_revert".into(),
        guard_overrides: Some(GuardOverrides { min_amount_out: Some(U256::from(200_000_000u64)), min_profit_wei: Some(U256::ZERO) }),
        seed_data: Some(vec![
            serde_json::json!({"sqrt_price_x96": "0x2610bde4a5309320e6", "tick": -192130, "liquidity": "0x1234567890abcdef"}),
        ]),
    });

    // 3. No-Profit Revert
    cases.push(HistoricalCase {
        case_id: "case_3_v3_no_profit_revert".into(),
        notes: "Min profit requirement not met (slight profit < high guard).".into(),
        fork_block_number: latest_block - 10,
        source_tx_hash: Some("0x0000000000000000000000000000000000000000000000000000000000000003".into()),
        root_asset: TokenAddress(token0_v3.to_string()),
        route_family: RouteFamily::Direct,
        pool_ids: vec!["0xd0b53d9277642d899df5c87a3966a349a798f224".into(), "0x6c561b446416e1a00e8e93e221854d6ea4171372".into()],
        pool_kinds: vec![PoolKind::ConcentratedLiquidity, PoolKind::ConcentratedLiquidity],
        path_tokens: vec![TokenAddress(token0_v3.to_string()), TokenAddress(weth.clone()), TokenAddress(token0_v3.to_string())],
        leg_directions: vec![false, true],
        amount_in: U256::from(100_000_000u64),
        expected_outcome: "no_profit_revert".into(),
        guard_overrides: Some(GuardOverrides { min_amount_out: Some(U256::ZERO), min_profit_wei: Some(U256::from(50_000_000u64)) }),
        seed_data: Some(vec![
            serde_json::json!({"sqrt_price_x96": "0x2610bde4a5309320e6", "tick": -192130, "liquidity": "0x1234567890abcdef"}),
            serde_json::json!({"sqrt_price_x96": "0x2650bde4a5309320e6", "tick": -192130, "liquidity": "0x1234567890abcdef"}),
        ]),
    });

    // 4. Mixed Success
    cases.push(HistoricalCase {
        case_id: "case_4_mixed_v2_v3_success".into(),
        notes: "Mixed V2/V3 Cyclic: USDC-WETH-USDC".into(),
        fork_block_number: latest_block - 10,
        source_tx_hash: Some("0x0000000000000000000000000000000000000000000000000000000000000004".into()),
        root_asset: TokenAddress(token0_v3.to_string()),
        route_family: RouteFamily::Direct,
        pool_ids: vec!["0xcdac0d6c6c59727a65f871236188350531885c43".into(), "0xd0b53d9277642d899df5c87a3966a349a798f224".into()],
        pool_kinds: vec![PoolKind::ReserveBased, PoolKind::ConcentratedLiquidity],
        path_tokens: vec![TokenAddress(token0_v3.to_string()), TokenAddress(weth.clone()), TokenAddress(token0_v3.to_string())],
        leg_directions: vec![false, true],
        amount_in: U256::from(100_000_000u64),
        expected_outcome: "success".into(),
        guard_overrides: None,
        seed_data: Some(vec![
            serde_json::json!({"reserve0": "0x000000000000000000000000000000000000000000000000000001b3d6448d00", "reserve1": "0x000000000000000000000000000000000000000000000322bcc7689000000000"}),
            serde_json::json!({"sqrt_price_x96": "0x2650bde4a5309320e6", "tick": -192130, "liquidity": "0x1234567890abcdef"}),
        ]),
    });

    fs::create_dir_all("fixtures").unwrap();
    let json = serde_json::to_string_pretty(&cases).unwrap();
    fs::write("fixtures/historical_cases.json", json).unwrap();
    println!("Successfully generated fixtures/historical_cases.json with 4 honest cases.");
    
    Ok(())
}
