use std::sync::Arc;
use tracing::{info, warn};
use arb_metrics::MetricsRegistry;
use crate::signer::Wallet;
use crate::preflight::PreflightChecker;
use crate::tenderly::TenderlySimConfig;
use arb_types::{BuiltTransaction, SubmissionResult, SubmissionMode, SubmissionFailureReason, PreflightStatus};
use alloy_provider::{Provider, ProviderBuilder};
use alloy_rpc_types_eth::TransactionRequest;
use alloy_primitives::Address;

pub struct Submitter {
    pub wallet: Wallet,
    pub mode: SubmissionMode,
    pub metrics: Arc<MetricsRegistry>,
    pub rpc_url: Option<String>,
    pub require_preflight: bool,
    pub require_eth_call: bool,
    pub require_gas_estimate: bool,
    pub tenderly_config: Option<TenderlySimConfig>,
    pub canary_live_mode_enabled: bool,
    pub gas_limit_multiplier_bps: u32,
    pub gas_limit_min: u64,
    pub gas_limit_max: u64,
}

impl Submitter {
    pub fn new(
        wallet: Wallet,
        mode: SubmissionMode,
        metrics: Arc<MetricsRegistry>,
        rpc_url: Option<String>,
        require_preflight: bool,
        require_eth_call: bool,
        require_gas_estimate: bool,
        tenderly_config: Option<TenderlySimConfig>,
        canary_live_mode_enabled: bool,
        gas_limit_multiplier_bps: u32,
        gas_limit_min: u64,
        gas_limit_max: u64,
    ) -> Self {
        Self {
            wallet,
            mode,
            metrics,
            rpc_url,
            require_preflight,
            require_eth_call,
            require_gas_estimate,
            tenderly_config,
            canary_live_mode_enabled,
            gas_limit_multiplier_bps,
            gas_limit_min,
            gas_limit_max,
        }
    }

    /// Submits a built transaction according to the operational mode.
    pub async fn submit(&self, tx: BuiltTransaction) -> SubmissionResult {
        self.metrics.inc_submission_attempts();

        // 1. Preflight check if required
        let mut tx = tx; // Make mutable for gas override
        if self.require_preflight {
            if let Some(url) = &self.rpc_url {
                self.metrics.inc_preflight();
                let checker = PreflightChecker::new(url.clone(), self.tenderly_config.clone());
                let tx_req = self.build_request(&tx);
                let preflight = checker.check(&tx_req, self.require_eth_call, self.require_gas_estimate).await;
                
                // Log honest statuses
                info!(
                    "Preflight result: overall_success={}, eth_call={:?}, gas_estimate={:?}, tenderly={:?}",
                    preflight.overall_success, preflight.eth_call_status, preflight.gas_estimate_status, preflight.tenderly_status
                );

                if !preflight.overall_success {
                    self.metrics.inc_preflight_failed();
                    
                    // Specific failure metrics
                    if let arb_types::PreflightStatus::Failed(_) = preflight.eth_call_status {
                        self.metrics.inc_preflight_eth_call_failed();
                    }
                    if let arb_types::PreflightStatus::Failed(_) = preflight.gas_estimate_status {
                        self.metrics.inc_preflight_gas_estimate_failed();
                    }

                    self.metrics.inc_submission_failed();
                    let msg = format!(
                        "Preflight failed: eth_call={:?}, gas={:?}, tenderly={:?}",
                        preflight.eth_call_status, preflight.gas_estimate_status, preflight.tenderly_status
                    );
                    return SubmissionResult::Failed(SubmissionFailureReason::PreflightFailed(msg));
                }

                // SUBMITTER-LEVEL GAS OVERRIDE: Apply multiplier and clamp
                if let Some(est_gas) = preflight.gas_estimate {
                    let mut new_limit = (est_gas as u128 * self.gas_limit_multiplier_bps as u128 / 10000) as u64;
                    // Clamp
                    if new_limit < self.gas_limit_min { new_limit = self.gas_limit_min; }
                    if new_limit > self.gas_limit_max { new_limit = self.gas_limit_max; }
                    
                    info!(original = tx.gas_limit, estimate = est_gas, overridden = new_limit, "CANARY_GAS_OVERRIDE");
                    tx.gas_limit = new_limit;
                }

                self.metrics.inc_preflight_success();
            }
        }

        match self.mode {
            SubmissionMode::DryRun => {
                // In dry-run, we simulate signing but don't broadcast.
                self.dry_run(tx).await
            }
            SubmissionMode::Broadcast => {
                // Part 5: Safe Broadcast Path
                if let Some(url) = &self.rpc_url {
                    self.broadcast(tx, url).await
                } else {
                    self.metrics.inc_submission_failed();
                    SubmissionResult::Failed(SubmissionFailureReason::NetworkError("RPC URL missing for broadcast".to_string()))
                }
            }
            SubmissionMode::SimulateOnly => {
                SubmissionResult::Skipped("SimulateOnly mode not implemented for submitter".to_string())
            }
        }
    }

    fn build_request(&self, tx: &BuiltTransaction) -> TransactionRequest {
        let to = tx.to.parse::<Address>().ok().map(alloy_primitives::TxKind::Call);
        let mut req = TransactionRequest {
            to,
            input: alloy_rpc_types_eth::TransactionInput::new(tx.data.clone().into()),
            value: Some(tx.value),
            nonce: Some(tx.nonce),
            gas: Some(tx.gas_limit),
            chain_id: Some(tx.chain_id),
            ..Default::default()
        };

        if let Some(gas_price) = tx.gas_price {
            req.gas_price = Some(gas_price);
        } else {
            req.max_fee_per_gas = Some(tx.max_fee_per_gas);
            req.max_priority_fee_per_gas = Some(tx.max_priority_fee_per_gas);
        }
        
        req
    }

    async fn broadcast(&self, tx: BuiltTransaction, rpc_url: &str) -> SubmissionResult {
        // Sign the transaction
        let (signed_raw, _) = match self.wallet.sign_tx(tx).await {
            Ok(res) => {
                self.metrics.inc_submission_signed();
                res
            }
            Err(e) => {
                self.metrics.inc_submission_failed();
                return SubmissionResult::Failed(SubmissionFailureReason::NetworkError(format!("Signing failure: {}", e)));
            }
        };

        // Broadcast to network
        let url = match rpc_url.parse() {
            Ok(u) => u,
            Err(e) => {
                self.metrics.inc_submission_failed();
                return SubmissionResult::Failed(SubmissionFailureReason::NetworkError(format!("Invalid RPC URL: {}", e)));
            }
        };
        let provider = ProviderBuilder::new().on_http(url);

        match provider.send_raw_transaction(&signed_raw).await {
            Ok(pending) => {
                self.metrics.inc_submission_broadcast();
                let tx_hash = format!("{:#x}", *pending.tx_hash());

                // In live canary mode, we MUST wait for the receipt to perform attribution.
                if self.canary_live_mode_enabled {
                    info!(tx_hash = %tx_hash, "CANARY_LIVE_BROADCAST: Waiting for receipt...");
                    match pending.get_receipt().await {
                        Ok(receipt) => {
                            if !receipt.status() {
                                self.metrics.inc_submission_failed();
                                return SubmissionResult::Failed(SubmissionFailureReason::ExecutionReverted("Transaction reverted on-chain".to_string()));
                            }

                            // Extract L1 Fee if available (Base/OP Stack specific)
                            let mut l1_fee_wei = None;
                            
                            // Using serde_json as a robust fallback for extension fields in Alloy 0.8
                            if let Ok(json) = serde_json::to_value(&receipt) {
                                if let Some(l1_fee) = json.get("l1Fee") {
                                    if let Some(l1_fee_str) = l1_fee.as_str() {
                                        l1_fee_wei = u128::from_str_radix(l1_fee_str.trim_start_matches("0x"), 16).ok();
                                    } else if let Some(l1_fee_u64) = l1_fee.as_u64() {
                                        l1_fee_wei = Some(l1_fee_u64 as u128);
                                    }
                                }
                            }

                            info!(
                                tx_hash = %tx_hash,
                                gas_used = receipt.gas_used,
                                l1_fee = ?l1_fee_wei,
                                "CANARY_RECEIPT_CONFIRMED"
                            );

                            SubmissionResult::Success {
                                tx_hash,
                                gas_used: receipt.gas_used,
                                effective_gas_price: receipt.effective_gas_price,
                                l1_fee_wei,
                            }
                        }
                        Err(e) => {
                            self.metrics.inc_submission_failed();
                            SubmissionResult::Failed(SubmissionFailureReason::NetworkError(format!("Failed to fetch receipt: {}", e)))
                        }
                    }
                } else {
                    // Non-live-broadcast: return immediately with hash
                    SubmissionResult::Success {
                        tx_hash,
                        gas_used: 0,
                        effective_gas_price: 0,
                        l1_fee_wei: None,
                    }
                }
            }
            Err(e) => {
                self.metrics.inc_submission_failed();
                SubmissionResult::Failed(SubmissionFailureReason::NetworkError(e.to_string()))
            }
        }
    }

    async fn dry_run(&self, tx: BuiltTransaction) -> SubmissionResult {
        // Build the transaction envelope for signing
        // For dry-run we produce a real signature and hash but don't broadcast.
        match self.wallet.sign_tx(tx).await {
            Ok((signed_raw, tx_hash)) => {
                self.metrics.inc_submission_signed();
                self.metrics.inc_submission_dry_run();
                SubmissionResult::DryRunSuccess {
                    tx_hash,
                    signed_raw,
                }
            }
            Err(e) => {
                self.metrics.inc_submission_failed();
                SubmissionResult::Failed(arb_types::SubmissionFailureReason::NetworkError(format!("Local signing failure: {}", e)))
            }
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
        
        let metrics = Arc::new(MetricsRegistry::new());
        let submitter = Submitter::new(wallet, SubmissionMode::DryRun, metrics, None, false, false, false, None, false, 12000, 21000, 5000000);
        
        let tx = BuiltTransaction {
            to: format!("{:#x}", Address::ZERO),
            data: vec![1, 2, 3],
            value: U256::ZERO,
            nonce: 0,
            gas_limit: 21000,
            max_fee_per_gas: 1000,
            max_priority_fee_per_gas: 100,
            gas_price: None,
            chain_id: 1,
        };

        let result = submitter.submit(tx).await;
        match result {
            SubmissionResult::DryRunSuccess { tx_hash, signed_raw } => {
                assert!(tx_hash.starts_with("0x"));
                assert!(tx_hash.len() > 10);
                assert!(!signed_raw.is_empty());
            }
            _ => panic!("Expected DryRunSuccess, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_submitter_preflight_disabled() {
        let test_pk = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let signer: PrivateKeySigner = test_pk.parse().unwrap();
        let wallet = Wallet { signer };
        let metrics = Arc::new(MetricsRegistry::new());
        // require_preflight = false
        let submitter = Submitter::new(wallet, SubmissionMode::DryRun, metrics, None, false, true, true, None, false, 12000, 21000, 5000000);
        
        let tx = BuiltTransaction {
            to: format!("{:#x}", Address::ZERO),
            data: vec![],
            value: U256::ZERO,
            nonce: 0,
            gas_limit: 21000,
            max_fee_per_gas: 1000,
            max_priority_fee_per_gas: 100,
            gas_price: None,
            chain_id: 1,
        };

        let result = submitter.submit(tx).await;
        // Should succeed in DryRun because preflight is skipped
        assert!(matches!(result, SubmissionResult::DryRunSuccess { .. }));
    }

    #[tokio::test]
    async fn test_submitter_preflight_required_but_no_url() {
        let test_pk = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let signer: PrivateKeySigner = test_pk.parse().unwrap();
        let wallet = Wallet { signer };
        let metrics = Arc::new(MetricsRegistry::new());
        // require_preflight = true, but rpc_url = None
        let submitter = Submitter::new(wallet, SubmissionMode::DryRun, metrics, None, true, true, true, None, false, 12000, 21000, 5000000);
        
        let tx = BuiltTransaction {
            to: format!("{:#x}", Address::ZERO),
            data: vec![],
            value: U256::ZERO,
            nonce: 0,
            gas_limit: 21000,
            max_fee_per_gas: 1000,
            max_priority_fee_per_gas: 100,
            gas_price: None,
            chain_id: 1,
        };

        let result = submitter.submit(tx).await;
        // Should continue to DryRun because url is None
        assert!(matches!(result, SubmissionResult::DryRunSuccess { .. }));
    }
}
