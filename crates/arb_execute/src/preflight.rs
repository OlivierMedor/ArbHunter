use crate::tenderly::{TenderlySimulator, TenderlySimConfig};
use alloy_rpc_types_eth::TransactionRequest;
use alloy_provider::{Provider, ProviderBuilder};
use arb_types::{PreflightResult, PreflightStatus};

pub struct PreflightChecker {
    rpc_url: String,
    tenderly_sim: Option<TenderlySimulator>,
}

impl PreflightChecker {
    pub fn new(rpc_url: String, tenderly_config: Option<TenderlySimConfig>) -> Self {
        Self { 
            rpc_url,
            tenderly_sim: tenderly_config.map(TenderlySimulator::new),
        }
    }

    pub async fn check(&self, tx: &TransactionRequest, require_eth_call: bool, require_gas_estimate: bool) -> PreflightResult {
        let mut eth_call_status = PreflightStatus::Skipped;
        let mut gas_estimate_status = PreflightStatus::Skipped;
        let mut tenderly_status = PreflightStatus::Skipped;
        let mut gas_estimate = None;

        let url = match self.rpc_url.parse() {
            Ok(u) => u,
            Err(e) => return PreflightResult {
                overall_success: false,
                eth_call_status: PreflightStatus::Failed(format!("Invalid RPC URL: {}", e)),
                gas_estimate_status,
                tenderly_status,
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

        // 3. Tenderly simulation if enabled and simulator is present
        if let Some(tenderly) = &self.tenderly_sim {
            match tenderly.simulate(tx).await {
                Ok(res) => {
                    if res.simulation.status {
                        tenderly_status = PreflightStatus::Passed;
                        // Gas estimate from Tenderly if not already set or more reliable
                        if gas_estimate.is_none() {
                            gas_estimate = Some(res.transaction.gas_used);
                        }
                    } else {
                        tenderly_status = PreflightStatus::Failed(res.transaction.error_message.unwrap_or_else(|| "Tenderly simulation failed".to_string()));
                    }
                }
                Err(e) => {
                    tenderly_status = PreflightStatus::Failed(e);
                }
            }
        }

        // Overall success: failed only if a required check failed
        let overall_success = !matches!(eth_call_status, PreflightStatus::Failed(_)) && 
                               !matches!(gas_estimate_status, PreflightStatus::Failed(_)) &&
                               !matches!(tenderly_status, PreflightStatus::Failed(_));

        PreflightResult {
            overall_success,
            eth_call_status,
            gas_estimate_status,
            tenderly_status,
            gas_estimate,
        }
    }
}
