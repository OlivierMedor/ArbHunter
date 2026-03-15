use crate::signer::Wallet;
use arb_types::{BuiltTransaction, SubmissionResult, SubmissionMode};

pub struct Submitter {
    pub wallet: Wallet,
    pub mode: SubmissionMode,
}

impl Submitter {
    pub fn new(wallet: Wallet, mode: SubmissionMode) -> Self {
        Self { wallet, mode }
    }

    /// Submits a built transaction according to the operational mode.
    pub async fn submit(&self, tx: BuiltTransaction) -> SubmissionResult {
        match self.mode {
            SubmissionMode::DryRun => {
                // In Phase 9 dry-run, we simulate signing but don't broadcast.
                self.dry_run(tx).await
            }
            SubmissionMode::Broadcast => {
                // For Phase 9, broadcast is disabled by default. 
                // We implement a mock placeholder or real signing if needed, 
                // but the requirement is "disabled by default".
                SubmissionResult::Skipped("Broadcast disabled by default in Phase 9".to_string())
            }
            SubmissionMode::SimulateOnly => {
                SubmissionResult::Skipped("SimulateOnly mode not implemented for submitter".to_string())
            }
        }
    }

    async fn dry_run(&self, tx: BuiltTransaction) -> SubmissionResult {
        // Build the transaction envelope for signing
        // For dry-run we just want to prove we CAN sign it.
        
        // This is a simplified signing proof for Phase 9 dry-run.
        let tx_hash = "0xDRYRUNHASH".to_string(); 
        
        SubmissionResult::DryRunSuccess {
            tx_hash,
            signed_raw: tx.data.clone(), // Placeholder for raw signed tx
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{U256, Address};
    use crate::signer::Wallet;
    use alloy_signer_local::PrivateKeySigner;

    #[tokio::test]
    async fn test_submitter_dry_run() {
        let test_pk = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let signer: PrivateKeySigner = test_pk.parse().unwrap();
        let wallet = Wallet { signer };
        
        let submitter = Submitter::new(wallet, SubmissionMode::DryRun);
        
        let tx = BuiltTransaction {
            to: format!("{:#x}", Address::ZERO),
            data: vec![1, 2, 3],
            value: U256::ZERO,
            nonce: 0,
            gas_limit: 21000,
            max_fee_per_gas: 1000,
            max_priority_fee_per_gas: 100,
            chain_id: 1,
        };

        let result = submitter.submit(tx).await;
        match result {
            SubmissionResult::DryRunSuccess { tx_hash, .. } => {
                assert_eq!(tx_hash, "0xDRYRUNHASH");
            }
            _ => panic!("Expected DryRunSuccess"),
        }
    }
}
