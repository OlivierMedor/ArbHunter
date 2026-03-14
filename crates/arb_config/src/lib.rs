use dotenvy::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub quicknode_wss_url: String,
    pub alchemy_wss_url: Option<String>,
    pub chain_id: u64,
    pub log_level: String,
    pub metrics_port: u16,
    pub enable_flashblocks: bool,
    pub enable_pending_logs: bool,
    pub enable_failover: bool,
}

impl Config {
    pub fn load() -> Self {
        // Attempt to load .env file, ignore error if it doesn't exist
        let _ = dotenv();

        Self {
            quicknode_wss_url: env::var("QUICKNODE_WSS_URL")
                .expect("FATAL: QUICKNODE_WSS_URL missing! A real endpoint is strictly required for Phase 2 live provider mode."),
            alchemy_wss_url: env::var("ALCHEMY_WSS_URL").ok(),
            chain_id: env::var("CHAIN_ID")
                .unwrap_or_else(|_| "8453".to_string())
                .parse()
                .expect("CHAIN_ID must be a valid u64"),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            metrics_port: env::var("METRICS_PORT")
                .unwrap_or_else(|_| "9090".to_string())
                .parse()
                .expect("METRICS_PORT must be a valid u16"),
            enable_flashblocks: env::var("ENABLE_FLASHBLOCKS")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false),
            enable_pending_logs: env::var("ENABLE_PENDING_LOGS")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false),
            enable_failover: env::var("ENABLE_FAILOVER")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        // Setup mock environment
        unsafe {
            std::env::set_var("QUICKNODE_WSS_URL", "wss://qnode");
            std::env::set_var("ALCHEMY_WSS_URL", "wss://alchemy");
            std::env::set_var("CHAIN_ID", "8453");
            std::env::set_var("LOG_LEVEL", "debug");
            std::env::set_var("METRICS_PORT", "9091");
            std::env::set_var("ENABLE_FLASHBLOCKS", "true");
            std::env::set_var("ENABLE_PENDING_LOGS", "1");
            std::env::set_var("ENABLE_FAILOVER", "false");
        }

        let config = Config::load();
        
        assert_eq!(config.quicknode_wss_url, "wss://qnode");
        assert_eq!(config.alchemy_wss_url, Some("wss://alchemy".to_string()));
        assert_eq!(config.chain_id, 8453);
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.metrics_port, 9091);
        assert!(config.enable_flashblocks);
        assert!(config.enable_pending_logs);
        assert!(!config.enable_failover);
    }
}
