use alloy_signer_local::PrivateKeySigner;
use alloy_primitives::Address;

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
            .map_err(|e| format!("Failed to parse SIGNER_PRIVATE_KEY: {}", e))?;
            
        Ok(Self { signer })
    }

    /// Returns the Ethereum address associated with this wallet.
    pub fn address(&self) -> Address {
        self.signer.address()
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
