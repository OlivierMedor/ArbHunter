use arb_types::{HistoricalCase, GuardOverrides, TokenAddress, PoolKind};
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

    // 1. Success Case (V3) - USDC/WETH
    let token0_v3 = Address::from_str("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913")?;
    
    let v3_slot0 = "0x0000000000000000000000000000000000000000000000c99f92960c9df764210000000000000000000000000000000000000000000000000000000000fcf48f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
    let v3_liq = "0x00000000000000000000000000000000000000000000000013df38488e01bd8c";
    let v3_seed = format!("{}:{}", v3_slot0, v3_liq);

    let v2_seed = "0x000000000000000000000000000000000000000000000000000000736d5be95300000000000000000000000000000000000000178ea52e043681432faf0000000000000000000000000000000000000000000000000000000067dcb7d7";

    cases.push(HistoricalCase {
        case_id: "case_1_v3_success".into(),
        notes: "Uniswap V3 USDC/WETH success replay.".into(),
        fork_block_number: latest_block - 10,
        source_tx_hash: None,
        root_asset: TokenAddress(token0_v3.to_string()),
        route_family: "ConcentratedLiquidity_SingleLeg".into(),
        pool_ids: vec!["0xd0b53d9277642d899df5c87a3966a349a798f224".to_string()],
        pool_kinds: vec![PoolKind::ConcentratedLiquidity],
        path_tokens: vec![TokenAddress(token0_v3.to_string()), TokenAddress("0x4200000000000000000000000000000000000006".to_string())],
        leg_directions: vec![true],
        amount_in: U256::from(100_000_000u64),
        expected_outcome: "success".into(),
        guard_overrides: None,
        seed_data: Some(vec![serde_json::Value::String(v3_seed.clone())]),
    });

    cases.push(HistoricalCase {
        case_id: "case_2_v3_slippage_revert".into(),
        notes: "USDC/WETH revert due to impossible minOut.".into(),
        fork_block_number: latest_block - 10,
        source_tx_hash: None,
        root_asset: TokenAddress(token0_v3.to_string()),
        route_family: "ConcentratedLiquidity_SingleLeg".into(),
        pool_ids: vec!["0xd0b53d9277642d899df5c87a3966a349a798f224".to_string()],
        pool_kinds: vec![PoolKind::ConcentratedLiquidity],
        path_tokens: vec![TokenAddress(token0_v3.to_string()), TokenAddress("0x4200000000000000000000000000000000000006".to_string())],
        leg_directions: vec![true],
        amount_in: U256::from(100_000_000u64),
        expected_outcome: "slippage_revert".into(),
        guard_overrides: Some(GuardOverrides { min_amount_out: Some(U256::MAX), min_profit_wei: Some(U256::ZERO) }),
        seed_data: Some(vec![serde_json::Value::String(v3_seed.clone())]),
    });

    cases.push(HistoricalCase {
        case_id: "case_3_v3_no_profit_revert".into(),
        notes: "USDC/WETH revert due to high profit guard.".into(),
        fork_block_number: latest_block - 10,
        source_tx_hash: None,
        root_asset: TokenAddress(token0_v3.to_string()),
        route_family: "ConcentratedLiquidity_SingleLeg".into(),
        pool_ids: vec!["0xd0b53d9277642d899df5c87a3966a349a798f224".to_string()],
        pool_kinds: vec![PoolKind::ConcentratedLiquidity],
        path_tokens: vec![TokenAddress(token0_v3.to_string()), TokenAddress("0x4200000000000000000000000000000000000006".to_string())],
        leg_directions: vec![true],
        amount_in: U256::from(100_000_000u64),
        expected_outcome: "no_profit_revert".into(),
        guard_overrides: Some(GuardOverrides { min_amount_out: Some(U256::ZERO), min_profit_wei: Some(U256::from(100_000_000_000_000_000_000_u128)) }),
        seed_data: Some(vec![serde_json::Value::String(v3_seed)]),
    });

    cases.push(HistoricalCase {
        case_id: "case_4_v2_unsupported_revert".into(),
        notes: "Aerodrome USDC/DAI (V2)".into(),
        fork_block_number: latest_block - 10,
        source_tx_hash: None,
        root_asset: TokenAddress(token0_v3.to_string()),
        route_family: "ReserveBased_SingleLeg".into(),
        pool_ids: vec!["0x67b00b46fa4f4f24c03855c5c8013c0b938b3eec".to_string()],
        pool_kinds: vec![PoolKind::ReserveBased],
        path_tokens: vec![TokenAddress(token0_v3.to_string()), TokenAddress("0x50c5725949a6510A2929456A59912743D28b8821".to_string())],
        leg_directions: vec![true],
        amount_in: U256::from(100_000_000u64),
        expected_outcome: "unsupported_route_revert".into(),
        guard_overrides: None,
        seed_data: Some(vec![serde_json::Value::String(v2_seed.into())]),
    });

    fs::create_dir_all("fixtures").unwrap();
    let json = serde_json::to_string_pretty(&cases).unwrap();
    fs::write("fixtures/historical_cases.json", json).unwrap();
    println!("Successfully generated fixtures/historical_cases.json with 4 honest cases.");
    
    Ok(())
}
