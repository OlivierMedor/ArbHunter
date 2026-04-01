use arb_config::Config;
use arb_types::{
    ExecutionPlan, ExpectedOutcome, ExecutionPath, ExecutionLeg, SlippageGuard, MinOutConstraint, TokenAddress, PoolId, SubmissionMode, SubmissionResult, PoolKind
};
use arb_execute::builder::TxBuilder;
use arb_execute::submitter::Submitter;
use arb_execute::signer::Wallet;
use arb_metrics::MetricsRegistry;
use alloy_signer_local::PrivateKeySigner;
use std::sync::Arc;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;
use alloy_provider::{ProviderBuilder, Provider};
use alloy_primitives::{Address, B256};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Phase 12 E2E local execution harness...");

    // 1. Load config
    let config = Config::load();
    let rpc_url = config.local_rpc_url.clone().expect("ANVIL_RPC_URL must be specified in .env");
    let test_pk = config.test_private_key.clone().expect("TEST_PRIVATE_KEY must be specified in .env");

    let executor_address = "0x5FbDB2315678afecb367f032d93F642f64180aa3"
        .parse::<Address>()
        .expect("Invalid default executor address");

    // 2. Setup Submitter
    let signer: PrivateKeySigner = test_pk.parse()?;
    let wallet = Wallet { signer };
    let metrics = Arc::new(MetricsRegistry::new());
    
    let submitter = Submitter::new(
        wallet,
        SubmissionMode::Broadcast, // We are intentionally broadcasting locally
        metrics,
        Some(rpc_url.clone()),
        false, // skip preflight purely for this simple harness test
        false, 
        false,
        None,
    );

    // --- VALID TRANSACTION ---
    info!("--- SUBMITTING VALID TRANSACTION ---");
    // 3. Create a pseudo ExecutionPlan to submit.
    // Instead of forcing the Planner to pick a real pool, we bypass that and just manually assemble an
    // ExecutionPlan that TxBuilder can encode.
    let plan = ExecutionPlan {
        target_token: TokenAddress("0x4200000000000000000000000000000000000006".to_string()),
        path: ExecutionPath {
            legs: vec![ExecutionLeg {
                pool_id: PoolId("0x0000000000000000000000000000000000022222".to_string()),
                pool_kind: PoolKind::ConcentratedLiquidity,
                token_in: TokenAddress("0x4200000000000000000000000000000000000006".to_string()),
                token_out: TokenAddress("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string()),
                zero_for_one: true,
                amount_out: alloy_primitives::U256::from(99),
            }],
        },
        outcome: ExpectedOutcome {
            amount_in: alloy_primitives::U256::from(100),
            expected_amount_out: alloy_primitives::U256::from(99),
            expected_profit: alloy_primitives::U256::from(1),
        },
        guard: SlippageGuard {
            min_out: MinOutConstraint {
                min_amount_out: alloy_primitives::U256::from(0), // Will pass
            },
            min_profit_wei: alloy_primitives::U256::ZERO,
        },
        flash_loan: None,
    };

    let builder = TxBuilder::new(executor_address, 31337);
    // 4. Build the actual transaction data.
    let built_tx = builder.build_tx(&plan, 1, 10_000_000_000, 100_000_000, 2100000).expect("failed to build Tx");

    // 5. Submit!
    info!("Submitting transaction directly to Anvil at {}", rpc_url);
    let result = submitter.submit(built_tx.clone()).await;

    // 6. Output processing and receipt parsing
    let provider = ProviderBuilder::new().on_http(rpc_url.parse()?);
    if let SubmissionResult::Success { tx_hash } = result {
        info!("Valid Tx Hash: {}", tx_hash);
        info!("Waiting for valid receipt...");
        loop {
            let receipt = provider.get_transaction_receipt(B256::from_str(&tx_hash)?).await?;
            if let Some(r) = receipt {
                info!("===== VALID RECEIPT =====");
                info!("Status: {:?}", r.status());
                info!("Gas Used: {}", r.gas_used);
                info!("Block Number: {:?}", r.block_number);
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    } else {
        error!("Valid transaction submission failed/skipped: {:?}", result);
    }

    // --- REVERTING TRANSACTION ---
    info!("--- SUBMITTING FAILING TRANSACTION ---");
    // Will fail because min_out is higher than actual output (0)
    let plan_fail = ExecutionPlan {
        target_token: TokenAddress("0x4200000000000000000000000000000000000006".to_string()),
        path: ExecutionPath { legs: vec![] },
        outcome: ExpectedOutcome {
            amount_in: alloy_primitives::U256::from(100),
            expected_amount_out: alloy_primitives::U256::from(99),
            expected_profit: alloy_primitives::U256::from(1),
        },
        guard: SlippageGuard {
            min_out: MinOutConstraint {
                min_amount_out: alloy_primitives::U256::from(99999), // Will revert slippage
            },
            min_profit_wei: alloy_primitives::U256::ZERO,
        },
        flash_loan: None,
    };

    let built_fail_tx = builder.build_tx(&plan_fail, 2, 10_000_000_000, 100_000_000, 2100000).expect("failed to build Tx");
    info!("Submitting failing transaction directly to Anvil at {}", rpc_url);
    let result_fail = submitter.submit(built_fail_tx.clone()).await;

    if let SubmissionResult::Success { tx_hash } = result_fail {
        info!("Invalid Tx Hash: {}", tx_hash);
        info!("Waiting for revert receipt...");
        loop {
            let receipt = provider.get_transaction_receipt(B256::from_str(&tx_hash)?).await?;
            if let Some(r) = receipt {
                info!("===== REVERT RECEIPT =====");
                info!("Status: {:?}", r.status());
                info!("Gas Used: {}", r.gas_used);
                info!("Block Number: {:?}", r.block_number);
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    } else {
        error!("Failing transaction submission failed/skipped: {:?}", result_fail);
    }

    Ok(())
}
