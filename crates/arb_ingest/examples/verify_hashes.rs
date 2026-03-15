use alloy_sol_types::{sol, SolEvent};

sol! {
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

fn main() {
    println!("Initialize: {:?}", <Initialize as SolEvent>::SIGNATURE_HASH);
    println!("Mint: {:?}", <Mint as SolEvent>::SIGNATURE_HASH);
    println!("Burn: {:?}", <Burn as SolEvent>::SIGNATURE_HASH);
}
