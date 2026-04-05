use arb_execute::tenderly::{TenderlySimConfig, TenderlySimulator};
use alloy_rpc_types_eth::TransactionRequest;
use alloy_primitives::{Address, hex, TxKind};
use std::env;

#[tokio::test]
async fn test_tenderly_harness_live_path() {
    let api_key = match env::var("TENDERLY_API_KEY") {
        Ok(v) => v,
        Err(_) => {
            println!("Skipping tenderly_harness: TENDERLY_API_KEY not set");
            return;
        }
    };
    let account_slug = match env::var("TENDERLY_ACCOUNT_SLUG") {
        Ok(v) => v,
        Err(_) => {
            println!("Skipping tenderly_harness: TENDERLY_ACCOUNT_SLUG not set");
            return;
        }
    };
    let project_slug = match env::var("TENDERLY_PROJECT_SLUG") {
        Ok(v) => v,
        Err(_) => {
            println!("Skipping tenderly_harness: TENDERLY_PROJECT_SLUG not set");
            return;
        }
    };

    let config = TenderlySimConfig {
        api_key,
        account_slug,
        project_slug,
        timeout_ms: 10000,
    };

    let simulator = TenderlySimulator::new(config);

    // Construct a dummy transaction targeting the public Base WETH contract to ensure Tenderly can simulate it.
    // WETH on Base: 0x4200000000000000000000000000000000000006
    let weth_address: Address = "0x4200000000000000000000000000000000000006".parse().unwrap();
    let from_address: Address = "0xFF77F9edFA4936A70Cc380B3F907f53Ef5ECB0d9".parse().unwrap(); // operator

    let mut tx = TransactionRequest::default();
    tx.from = Some(from_address);
    tx.to = Some(TxKind::Call(weth_address));
    tx.input = alloy_primitives::Bytes::from(hex!("095ea7b30000000000000000000000000000000000000000000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")).into(); // approve(address,uint256)
    tx.gas = Some(100_000u64);
    tx.gas_price = Some(1_000_000_000u128);

    let result = simulator.simulate(&tx).await;
    
    // We expect Tenderly API to succeed and return a valid JSON response parsing into TenderlySimResponse
    assert!(result.is_ok(), "Tenderly simulation failed: {:?}", result);
    
    let res = result.unwrap();
    println!("Tenderly Tx Status: {}", res.transaction.status);
    println!("Tenderly Sim ID: {}", res.simulation.id);
    
    // The status should be successful given it's a simple approve on WETH
    assert!(res.transaction.status, "Transaction simulated as reverted!");
}
