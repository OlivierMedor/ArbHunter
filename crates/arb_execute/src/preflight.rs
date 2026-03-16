use alloy_provider::{Provider, ProviderBuilder};
use alloy_rpc_types_eth::TransactionRequest;
use arb_types::{PreflightResult, PreflightStatus};

pub struct PreflightChecker {
    rpc_url: String,
}

impl PreflightChecker {
    pub fn new(rpc_url: String) -> Self {
        Self { rpc_url }
    }

    pub async fn check(&self, tx: &TransactionRequest, require_eth_call: bool, require_gas_estimate: bool) -> PreflightResult {
        let mut eth_call_status = PreflightStatus::Skipped;
        let mut gas_estimate_status = PreflightStatus::Skipped;
        let mut gas_estimate = None;

        let url = match self.rpc_url.parse() {
            Ok(u) => u,
            Err(e) => return PreflightResult {
                overall_success: false,
                eth_call_status: PreflightStatus::Failed(format!("Invalid RPC URL: {}", e)),
                gas_estimate_status,
                gas_estimate: None,
            },
        };
        let provider = ProviderBuilder::new().on_http(url);

        // 1. eth_call
        if require_eth_call {
            match provider.call(tx).await {
                Ok(_) => {
                    eth_call_status = PreflightStatus::Passed;
                }
                Err(e) => {
                    eth_call_status = PreflightStatus::Failed(e.to_string());
                }
            }
        }

        // 2. gas estimate
        if require_gas_estimate {
            match provider.estimate_gas(tx).await {
                Ok(gas) => {
                    gas_estimate_status = PreflightStatus::Passed;
                    gas_estimate = Some(gas);
                }
                Err(e) => {
                    gas_estimate_status = PreflightStatus::Failed(e.to_string());
                }
            }
        }

        // Overall success: failed only if a required check failed
        let overall_success = !matches!(eth_call_status, PreflightStatus::Failed(_)) && 
                               !matches!(gas_estimate_status, PreflightStatus::Failed(_));

        PreflightResult {
            overall_success,
            eth_call_status,
            gas_estimate_status,
            gas_estimate,
        }
    }
}
