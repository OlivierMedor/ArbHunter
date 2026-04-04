use serde::{Deserialize, Serialize};
use alloy_rpc_types_eth::TransactionRequest;
use reqwest::Client;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenderlySimConfig {
    pub api_key: String,
    pub account_slug: String,
    pub project_slug: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenderlySimResponse {
    pub transaction: TenderlyTransaction,
    pub simulation: TenderlySimulation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenderlyTransaction {
    pub status: bool,
    pub gas_used: u64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenderlySimulation {
    pub id: String,
    pub status: bool,
}

pub struct TenderlySimulator {
    config: TenderlySimConfig,
    client: Client,
}

impl TenderlySimulator {
    pub fn new(config: TenderlySimConfig) -> Self {
        let timeout = Duration::from_millis(config.timeout_ms);
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { config, client }
    }

    pub async fn simulate(&self, tx: &TransactionRequest) -> Result<TenderlySimResponse, String> {
        let url = format!(
            "https://api.tenderly.co/api/v1/account/{}/project/{}/simulate",
            self.config.account_slug, self.config.project_slug
        );

        let to_addr = tx.to.and_then(|kind| match kind {
            alloy_primitives::TxKind::Call(addr) => Some(addr.to_string()),
            _ => None,
        });

        let input_hex = tx.input.input().map(|b| b.to_string()).unwrap_or_else(|| "0x".to_string());

        let payload = serde_json::json!({
            "network_id": "8453", // Base Mainnet
            "from": tx.from.map(|f| f.to_string()),
            "to": to_addr,
            "input": input_hex,
            "gas": tx.gas,
            "gas_price": tx.gas_price,
            "value": tx.value.unwrap_or_default().to_string(),
            "save": true,
            "save_if_fails": true,
            "simulation_type": "full",
        });

        let response = self.client.post(&url)
            .header("X-Access-Key", &self.config.api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Tenderly request failed: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Tenderly API error ({}): {}", status, body));
        }

        let res: TenderlySimResponse = response.json()
            .await
            .map_err(|e| format!("Failed to parse Tenderly response: {}", e))?;

        Ok(res)
    }
}
