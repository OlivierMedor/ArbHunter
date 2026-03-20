use arb_types::{HistoricalCase, GuardOverrides, TokenAddress, PoolKind};
use alloy_primitives::{U256, Address, B256};
use alloy_provider::{ProviderBuilder, Provider};
use arb_config::Config;
use std::fs;
use hex;
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

    // 1. Success Case (V2) - Aerodrome Stable DAI/USDC
    let p_dai_usdc = Address::from_str("0x67b00b46fA4F4F24c03855c5c8013C0b938b3Eec")?;
    let block = latest_block - 2000;
    let log_block_hex = format!("0x{:x}", block);
    
    let t0_val: serde_json::Value = provider.raw_request("eth_call".into(), (serde_json::json!({"to": p_dai_usdc, "data": "0x0dfe1681"}), &log_block_hex)).await?;
    let t1_val: serde_json::Value = provider.raw_request("eth_call".into(), (serde_json::json!({"to": p_dai_usdc, "data": "0xd21220a7"}), &log_block_hex)).await?;
    let token0 = Address::from_str(&format!("0x{}", &t0_val.as_str().unwrap()[26..]))?;
    let token1 = Address::from_str(&format!("0x{}", &t1_val.as_str().unwrap()[26..]))?;

    cases.push(HistoricalCase {
        case_id: "case_1_v2_success".into(),
        notes: "Aerodrome Stable DAI/USDC success replay.".into(),
        fork_block_number: block,
        source_tx_hash: None,
        root_asset: TokenAddress(token0.to_string()),
        route_family: "ReserveBased_SingleLeg".into(),
        pool_ids: vec![p_dai_usdc.to_string()],
        pool_kinds: vec![PoolKind::ReserveBased],
        path_tokens: vec![TokenAddress(token0.to_string()), TokenAddress(token1.to_string())],
        leg_directions: vec![true],
        amount_in: U256::from(100_000_000_000_000_000u128), // 0.1 tokens
        expected_outcome: "success".into(),
        guard_overrides: None,
        seed_data: None,
    });

    // 2. Forced Slippage Revert (Derived from Case 1)
    cases.push(HistoricalCase {
        case_id: "case_2_v2_slippage_revert".into(),
        notes: "Forced slippage revert (min_out = MAX).".into(),
        fork_block_number: block,
        source_tx_hash: None,
        root_asset: TokenAddress(token0.to_string()),
        route_family: "ReserveBased_SingleLeg".into(),
        pool_ids: vec![p_dai_usdc.to_string()],
        pool_kinds: vec![PoolKind::ReserveBased],
        path_tokens: vec![TokenAddress(token0.to_string()), TokenAddress(token1.to_string())],
        leg_directions: vec![true],
        amount_in: U256::from(100_000_000_000_000_000u128),
        expected_outcome: "slippage_revert".into(),
        guard_overrides: Some(GuardOverrides {
            min_amount_out: Some(U256::MAX),
            min_profit_wei: None,
        }),
        seed_data: None,
    });

    // 3. Forced No-Profit Revert (Derived from Case 1)
    cases.push(HistoricalCase {
        case_id: "case_3_v2_no_profit_revert".into(),
        notes: "Forced no-profit revert (min_profit = MAX).".into(),
        fork_block_number: block,
        source_tx_hash: None,
        root_asset: TokenAddress(token0.to_string()),
        route_family: "ReserveBased_SingleLeg".into(),
        pool_ids: vec![p_dai_usdc.to_string()],
        pool_kinds: vec![PoolKind::ReserveBased],
        path_tokens: vec![TokenAddress(token0.to_string()), TokenAddress(token1.to_string())],
        leg_directions: vec![true],
        amount_in: U256::from(100_000_000_000_000_000u128),
        expected_outcome: "no_profit_revert".into(),
        guard_overrides: Some(GuardOverrides {
            min_amount_out: None,
            min_profit_wei: Some(U256::MAX),
        }),
        seed_data: None,
    });

    // 4. V3 / CL Case - Uniswap V3 USDC/WETH 0.05%
    let p_v3_usdc_weth = Address::from_str("0xd0b53D9277642d899df5C87A3966a349A798f224")?;
    let t0_v3_val: serde_json::Value = provider.raw_request("eth_call".into(), (serde_json::json!({"to": p_v3_usdc_weth, "data": "0x0dfe1681"}), &log_block_hex)).await?;
    let t1_v3_val: serde_json::Value = provider.raw_request("eth_call".into(), (serde_json::json!({"to": p_v3_usdc_weth, "data": "0xd21220a7"}), &log_block_hex)).await?;
    let token0_v3 = Address::from_str(&format!("0x{}", &t0_v3_val.as_str().unwrap()[26..]))?;
    let token1_v3 = Address::from_str(&format!("0x{}", &t1_v3_val.as_str().unwrap()[26..]))?;

    cases.push(HistoricalCase {
        case_id: "case_4_v3_success".into(),
        notes: "Uniswap V3 USDC/WETH success replay.".into(),
        fork_block_number: block,
        source_tx_hash: None,
        root_asset: TokenAddress(token0_v3.to_string()),
        route_family: "ConcentratedLiquidity_SingleLeg".into(),
        pool_ids: vec![p_v3_usdc_weth.to_string()],
        pool_kinds: vec![PoolKind::ConcentratedLiquidity],
        path_tokens: vec![TokenAddress(token0_v3.to_string()), TokenAddress(token1_v3.to_string())],
        leg_directions: vec![true],
        amount_in: U256::from(100_000_000u64), // 0.1 USDC (6 decimals)
        expected_outcome: "success".into(),
        guard_overrides: None,
        seed_data: None,
    });

    fs::create_dir_all("fixtures").unwrap();
    let json = serde_json::to_string_pretty(&cases).unwrap();
    fs::write("fixtures/historical_cases.json", json).unwrap();
    println!("Successfully generated fixtures/historical_cases.json with 4 honest cases.");
    
    Ok(())
}
