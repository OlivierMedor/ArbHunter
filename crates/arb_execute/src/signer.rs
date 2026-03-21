use alloy_signer_local::PrivateKeySigner;
use alloy_signer::Signer;
use alloy_consensus::{TxEip1559, TxLegacy, TxEnvelope, SignableTransaction};
use alloy_network::TxSigner;
use alloy_primitives::{Address, Bytes};
use arb_types::BuiltTransaction;

#[derive(Debug)]
pub struct Wallet {
    pub signer: PrivateKeySigner,
}

impl Wallet {
    /// Loads a wallet from the SIGNER_PRIVATE_KEY environment variable.
    /// Returns an error if the variable is missing or the key is invalid.
    pub fn from_env() -> Result<Self, String> {
        let pk = std::env::var("SIGNER_PRIVATE_KEY")
            .map_err(|_| "SIGNER_PRIVATE_KEY environment variable is missing".to_string())?;
        
        if pk.is_empty() {
            return Err("SIGNER_PRIVATE_KEY is empty".to_string());
        }

        let signer: PrivateKeySigner = pk.parse()
            .map_err(|_| "Failed to parse SIGNER_PRIVATE_KEY".to_string())?;
            
        Ok(Self { signer })
    }

    /// Returns the Ethereum address associated with this wallet.
    pub fn address(&self) -> Address {
        self.signer.address()
    }

    /// Signs a BuiltTransaction and returns the RLP encoded signed transaction and the transaction hash.
    pub async fn sign_tx(&self, tx: BuiltTransaction) -> Result<(Vec<u8>, String), String> {
        let to_addr = tx.to.parse::<Address>()
            .map_err(|e| format!("Invalid 'to' address in BuiltTransaction: {}", e))?;

        let signed_raw = if let Some(gas_price) = tx.gas_price {
            let mut tx_inner = TxLegacy {
                chain_id: Some(tx.chain_id),
                nonce: tx.nonce,
                gas_limit: tx.gas_limit,
                gas_price,
                to: alloy_primitives::TxKind::Call(to_addr),
                value: tx.value,
                input: Bytes::from(tx.data),
            };
            let signature = self.signer.sign_transaction(&mut tx_inner).await
                .map_err(|e| format!("Failed to sign legacy transaction: {}", e))?;
            let envelope = TxEnvelope::Legacy(tx_inner.into_signed(signature));
            alloy_rlp::encode(&envelope)
        } else {
            let mut tx_inner = TxEip1559 {
                chain_id: tx.chain_id,
                nonce: tx.nonce,
                gas_limit: tx.gas_limit,
                max_fee_per_gas: tx.max_fee_per_gas,
                max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
                to: alloy_primitives::TxKind::Call(to_addr),
                value: tx.value,
                input: Bytes::from(tx.data),
                access_list: Default::default(),
            };
            let signature = self.signer.sign_transaction(&mut tx_inner).await
                .map_err(|e| format!("Failed to sign EIP-1559 transaction: {}", e))?;
            let envelope = TxEnvelope::Eip1559(tx_inner.into_signed(signature));
            alloy_rlp::encode(&envelope)
        };

        let hash = format!("0x{:x}", alloy_primitives::keccak256(&signed_raw));

        Ok((signed_raw, hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_from_env_missing() {
        unsafe { std::env::remove_var("SIGNER_PRIVATE_KEY"); }
        let result = Wallet::from_env();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "SIGNER_PRIVATE_KEY environment variable is missing");
    }

    #[test]
    fn test_wallet_from_env_invalid() {
        unsafe { std::env::set_var("SIGNER_PRIVATE_KEY", "invalid_key"); }
        let result = Wallet::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse SIGNER_PRIVATE_KEY"));
    }

    #[test]
    fn test_wallet_from_valid_env() {
        // Sample private key (do not use in production)
        let test_pk = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        unsafe { std::env::set_var("SIGNER_PRIVATE_KEY", test_pk); }
        
        let wallet = Wallet::from_env().expect("Should load valid wallet");
        // Address for this PK is 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
        assert_eq!(
            format!("{:?}", wallet.address()),
            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
        );
    }
}
