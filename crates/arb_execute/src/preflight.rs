use alloy_provider::{Provider, ProviderBuilder};
use alloy_rpc_types_eth::TransactionRequest;
use arb_types::{PreflightResult, PreflightFailureReason};

pub struct PreflightChecker {
    rpc_url: String,
}

impl PreflightChecker {
    pub fn new(rpc_url: String) -> Self {
        Self { rpc_url }
    }

    pub async fn check(&self, tx: &TransactionRequest) -> PreflightResult {
        let url = match self.rpc_url.parse() {
            Ok(u) => u,
            Err(e) => return PreflightResult {
                success: false,
                failure_reason: Some(PreflightFailureReason::EthCallFailed(format!("Invalid RPC URL: {}", e))),
                gas_estimate: None,
            },
        };
        let provider = ProviderBuilder::new().on_http(url);

        // 1. eth_call
        if let Err(e) = provider.call(tx).await {
             return PreflightResult {
                success: false,
                failure_reason: Some(PreflightFailureReason::EthCallFailed(e.to_string())),
                gas_estimate: None,
            };
        }

        // 2. gas estimate
        match provider.estimate_gas(tx).await {
            Ok(gas) => PreflightResult {
                success: true,
                failure_reason: None,
                gas_estimate: Some(gas),
            },
            Err(e) => PreflightResult {
                success: false,
                failure_reason: Some(PreflightFailureReason::GasEstimateFailed(e.to_string())),
                gas_estimate: None,
            },
        }
    }
}
