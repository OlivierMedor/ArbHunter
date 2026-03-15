use alloy_primitives::keccak256;

fn main() {
    let sig = "Swap(address,address,int256,int256,uint160,uint128,int24)";
    let hash = keccak256(sig);
    println!("Hash: {:?}", hash);
}
