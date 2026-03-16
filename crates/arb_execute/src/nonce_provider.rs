use alloy_provider::{Provider, ProviderBuilder};
use alloy_primitives::Address;
use alloy_transport_http::Http;
use reqwest::Client;

pub struct NonceProvider {
    rpc_url: String,
}

impl NonceProvider {
    pub fn new(rpc_url: String) -> Self {
        Self { rpc_url }
    }

    pub async fn get_nonce(&self, address: Address) -> Result<u64, String> {
        let url = self.rpc_url.parse().map_err(|e| format!("Invalid RPC URL: {}", e))?;
        let provider = ProviderBuilder::new().on_http(url);
        
        provider.get_transaction_count(address)
            .await
            .map_err(|e| format!("Failed to fetch nonce for {}: {}", address, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Mocking provider is complex with alloy, usually done via a mock transport.
    // For Phase 10 plumbing, we'll focus on the interface and usage in submitter.
}
