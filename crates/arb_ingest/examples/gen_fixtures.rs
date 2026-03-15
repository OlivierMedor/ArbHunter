use alloy_sol_types::{SolValue};
use alloy_primitives::{address, U256};
use hex;

fn main() {
    // 1. Initialize(uint160, int24)
    // sqrtPriceX96 = 2^96 = 79228162514264337593543950336
    let sqrt_p = U256::from(79228162514264337593543950336u128);
    let tick = 0i32;
    let init_data = (sqrt_p, tick).abi_encode();
    println!("INIT_DATA: 0x{}", hex::encode(init_data));

    // 2. Mint topics
    // sig, owner, tickLower, tickUpper
    // owner: 0x000000000000000000000000000000000000dead
    // tickLower: -100
    // tickUpper: 100
    println!("MINT_TOPIC_OWNER: 0x000000000000000000000000000000000000000000000000000000000000dead");
    println!("MINT_TOPIC_LOWER: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff9c"); // -100
    println!("MINT_TOPIC_UPPER: 0x0000000000000000000000000000000000000000000000000000000000000064"); // 100

    // 2. Mint data (sender, amount, amount0, amount1)
    let sender = address!("000000000000000000000000000000000000beef");
    let amount = 1000000u128;
    let amount0 = U256::from(500);
    let amount1 = U256::from(500);
    let mint_data = (sender, amount, amount0, amount1).abi_encode();
    println!("MINT_DATA: 0x{}", hex::encode(mint_data));

    // 3. Burn data (amount, amount0, amount1)
    let amount_burn = 500000u128;
    let burn_data = (amount_burn, amount0, amount1).abi_encode();
    println!("BURN_DATA: 0x{}", hex::encode(burn_data));
}
