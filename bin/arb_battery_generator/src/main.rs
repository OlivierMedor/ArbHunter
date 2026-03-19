use arb_types::{HistoricalCase, GuardOverrides, TokenAddress, PoolKind};
use alloy_primitives::U256;
use std::fs;

fn main() {
    let block_number = 22000000;
    let weth = TokenAddress("0x4200000000000000000000000000000000000006".to_string());
    let usdc = TokenAddress("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string());

    // Aerodrome Base V2 Pool WETH/USDC
    let v2_pool = "0x4a3636608d7bcdd730ce8670a41b53e0fde6ef28".to_string();
    
    // UniswapV3 Base WETH/USDC 0.05%
    let v3_pool = "0xd0b53D927E91100f074D2338bd4845D884C0F7a8".to_string();

    let amount_in = U256::from(100_000_000_000_000_000u128); // 0.1 WETH

    let mut cases = Vec::new();

    // Case 1: Likely Success (V2)
    cases.push(HistoricalCase {
        case_id: "case_01_v2_success".to_string(),
        notes: "Standard WETH -> USDC V2 swap designed to succeed naturally.".to_string(),
        fork_block_number: block_number,
        root_asset: weth.clone(),
        route_family: "V2_SingleLeg".to_string(),
        pool_ids: vec![v2_pool.clone()],
        pool_kinds: vec![PoolKind::ReserveBased],
        path_tokens: vec![weth.clone(), usdc.clone()],
        leg_directions: vec![true], // token0 -> token1
        amount_in,
        expected_outcome: "success".to_string(),
        guard_overrides: None,
        seed_data: None,
    });

    // Case 2: Forced Slippage Revert
    cases.push(HistoricalCase {
        case_id: "case_02_v2_slippage_revert".to_string(),
        notes: "Same V2 route, but forced to fail simulation due to an impossibly high minAmountOut.".to_string(),
        fork_block_number: block_number,
        root_asset: weth.clone(),
        route_family: "V2_SingleLeg".to_string(),
        pool_ids: vec![v2_pool.clone()],
        pool_kinds: vec![PoolKind::ReserveBased],
        path_tokens: vec![weth.clone(), usdc.clone()],
        leg_directions: vec![true],
        amount_in,
        expected_outcome: "slippage_revert".to_string(),
        guard_overrides: Some(GuardOverrides {
            min_amount_out: Some(U256::MAX),
            min_profit_wei: None,
        }),
        seed_data: None,
    });

    // Case 3: Forced No-profit Revert (High min_profit_wei)
    cases.push(HistoricalCase {
        case_id: "case_03_v2_noprofit_revert".to_string(),
        notes: "Same V2 route, but forced to fail due to high minProfit requirement.".to_string(),
        fork_block_number: block_number,
        root_asset: weth.clone(),
        route_family: "V2_SingleLeg".to_string(),
        pool_ids: vec![v2_pool.clone()],
        pool_kinds: vec![PoolKind::ReserveBased],
        path_tokens: vec![weth.clone(), usdc.clone()],
        leg_directions: vec![true],
        amount_in,
        expected_outcome: "no_profit_revert".to_string(),
        guard_overrides: Some(GuardOverrides {
            min_amount_out: None,
            min_profit_wei: Some(U256::MAX),
        }),
        seed_data: None,
    });

    // Case 4: V3 Route
    cases.push(HistoricalCase {
        case_id: "case_04_v3_success".to_string(),
        notes: "Standard WETH -> USDC V3 swap designed to succeed naturally (Concentrated Liquidity test).".to_string(),
        fork_block_number: block_number,
        root_asset: weth.clone(),
        route_family: "V3_SingleLeg".to_string(),
        pool_ids: vec![v3_pool],
        pool_kinds: vec![PoolKind::ConcentratedLiquidity],
        path_tokens: vec![weth.clone(), usdc.clone()],
        leg_directions: vec![true],
        amount_in,
        expected_outcome: "success".to_string(),
        guard_overrides: None,
        seed_data: None,
    });

    fs::create_dir_all("fixtures").unwrap();
    let json = serde_json::to_string_pretty(&cases).unwrap();
    fs::write("fixtures/historical_cases.json", json).unwrap();
    println!("Successfully generated fixtures/historical_cases.json with 4 deterministic cases.");
}
