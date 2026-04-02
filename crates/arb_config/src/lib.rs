use dotenvy::dotenv;
use std::env;
use tracing;

#[derive(Debug, Clone)]
pub struct Config {
    pub quicknode_http_url: String,
    pub quicknode_wss_url: String,
    pub alchemy_wss_url: Option<String>,
    pub chain_id: u64,
    pub log_level: String,
    pub metrics_port: u16,
    pub enable_flashblocks: bool,
    pub enable_pending_logs: bool,
    pub enable_failover: bool,

    // Phase 6: Route Graph & Filter
    pub root_asset: String,
    pub min_gross_profit: String,
    pub min_gross_bps: u32,
    pub require_fresh: bool,
    pub quote_buckets: String,
    
    // Phase 9: Wallet & Submission
    pub signer_private_key: Option<String>,
    pub executor_contract_address: Option<String>,
    pub enable_broadcast: bool,
    pub dry_run_only: bool,

    // Phase 10: Preflight & Safe Broadcast
    pub rpc_http_url: Option<String>,
    pub require_preflight: bool,
    pub require_gas_estimate: bool,
    pub require_eth_call: bool,

    // Phase 12: Forked E2E Harness
    pub test_private_key: Option<String>,
    pub local_rpc_url: Option<String>,
    pub anvil_fork_url: Option<String>,

    // Phase 15: Shadow Mode
    pub enable_shadow_mode: bool,
    pub shadow_recheck_delay_ms: u64,
    pub shadow_min_profit_threshold: String,
    pub shadow_max_candidates_per_window: u32,
    pub shadow_write_journal: bool,
    pub shadow_journal_path: String,
    
    // Phase 16: Historical Shadow Replay
    pub enable_historical_shadow_replay: bool,
    pub historical_replay_lookback_hours: u32,
    pub historical_replay_start_block: Option<u64>,
    pub historical_replay_end_block: Option<u64>,
    pub historical_recheck_blocks: u64,
    pub historical_replay_output_path: String,
    pub historical_replay_metrics_port: u16,
    pub historical_max_cases_to_verify: u32,

    // Phase 23: Canary Policy Enforcement
    /// Comma-separated route families allowed through the canary gate. Default: "multi".
    pub canary_route_family_allowlist: String,
    /// Comma-separated route families blocked by the canary gate. Default: "direct".
    pub canary_route_family_blocklist: String,
    /// Max amount_in per canary trade in Wei. Default: 30_000_000_000_000_000 (0.03 ETH).
    pub canary_max_trade_size_wei: u128,
    /// Max simultaneous in-flight canary trades. Default: 1.
    pub canary_max_concurrent_trades: u32,
    /// Revert-streak threshold before gate halts. Default: 3.
    pub canary_max_consecutive_reverts: u32,
    /// Attempt count at which a review-threshold warning is emitted. Default: 30.
    pub canary_review_threshold_attempts: u32,
    /// Cumulative realized loss cap in Wei. Default: 50_000_000_000_000_000 (0.05 ETH).
    /// Inert when live_mode_enabled = false (sim/shadow mode).
    pub canary_loss_cap_wei: u128,
    /// Whether loss caps and active halting are enforced for live trading. Default: false.
    pub canary_live_mode_enabled: bool,

    // Phase 23: Tenderly Simulation Scaffold
    /// Optional Tenderly API key. When absent, Tenderly sim falls back to local-only.
    pub tenderly_api_key: Option<String>,
    /// Tenderly account slug (e.g. "my-org"). Required when tenderly_enabled = true.
    pub tenderly_account_slug: String,
    /// Tenderly project slug. Required when tenderly_enabled = true.
    pub tenderly_project_slug: String,
    /// Enable Tenderly pre-send simulation. Default: false (no credentials, no-op).
    pub tenderly_enabled: bool,

    // Phase 24: Live-Canary Hardening
    /// Path to the durable state file. Default: "canary_state.json".
    pub canary_state_path: String,
    /// Timeout for Tenderly simulations in milliseconds. Default: 10000.
    pub tenderly_timeout_ms: u64,
    /// Multiplier for preflight gas estimate in basis points. Default: 12000 (1.2x).
    pub gas_limit_multiplier_bps: u32,
    /// Minimum allowed gas limit. Default: 21000.
    pub gas_limit_min: u64,
    /// Maximum allowed gas limit. Default: 5000000.
    pub gas_limit_max: u64,
}

impl Config {
    pub fn load() -> Self {
        // Attempt to load .env file, ignore error if it doesn't exist
        let _ = dotenv();

        let parsed_config = Self {
            quicknode_http_url: env::var("QUICKNODE_HTTP_URL")
                .unwrap_or_else(|_| "http://localhost:8545".to_string()),
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
            
            // Phase 6
            root_asset: env::var("ROOT_ASSET")
                .unwrap_or_else(|_| "0x4200000000000000000000000000000000000006".to_string()), // WETH on Base
            min_gross_profit: env::var("MIN_GROSS_PROFIT")
                .unwrap_or_else(|_| "10000000000000000".to_string()), // 0.01 ETH
            min_gross_bps: env::var("MIN_GROSS_BPS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            require_fresh: env::var("REQUIRE_FRESH")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(true),
            quote_buckets: env::var("QUOTE_BUCKETS")
                .unwrap_or_else(|_| "100000000000000000,1000000000000000000,10000000000000000000".to_string()), // 0.1, 1, 10
            
            // Phase 9
            signer_private_key: env::var("SIGNER_PRIVATE_KEY").ok(),
            executor_contract_address: env::var("EXECUTOR_CONTRACT_ADDRESS").ok(),
            enable_broadcast: env::var("ENABLE_BROADCAST")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false),
            dry_run_only: env::var("DRY_RUN_ONLY")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(true),
            
            // Phase 10
            rpc_http_url: env::var("RPC_HTTP_URL").ok().or_else(|| env::var("QUICKNODE_HTTP_URL").ok()),
            require_preflight: env::var("REQUIRE_PREFLIGHT")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(true),
            require_gas_estimate: env::var("REQUIRE_GAS_ESTIMATE")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(true),
            require_eth_call: env::var("REQUIRE_ETH_CALL")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(true),

            // Phase 12
            test_private_key: env::var("TEST_PRIVATE_KEY").ok(),
            local_rpc_url: env::var("ANVIL_RPC_URL").ok(),
            anvil_fork_url: env::var("ANVIL_FORK_URL").ok(),

            // Phase 15
            enable_shadow_mode: env::var("ENABLE_SHADOW_MODE")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false),
            shadow_recheck_delay_ms: env::var("SHADOW_RECHECK_DELAY_MS")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .unwrap_or(5000),
            shadow_min_profit_threshold: env::var("SHADOW_MIN_PROFIT_THRESHOLD")
                .unwrap_or_else(|_| "0".to_string()),
            shadow_max_candidates_per_window: env::var("SHADOW_MAX_CANDIDATES_PER_WINDOW")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
            shadow_write_journal: env::var("SHADOW_WRITE_JOURNAL")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(true),
            shadow_journal_path: env::var("SHADOW_JOURNAL_PATH")
                .unwrap_or_else(|_| "shadow_journal.jsonl".to_string()),
            
            // Phase 16
            enable_historical_shadow_replay: env::var("ENABLE_HISTORICAL_SHADOW_REPLAY")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false),
            historical_replay_lookback_hours: env::var("HISTORICAL_REPLAY_LOOKBACK_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .unwrap_or(24),
            historical_replay_start_block: env::var("HISTORICAL_REPLAY_START_BLOCK").ok().and_then(|v| v.parse().ok()),
            historical_replay_end_block: env::var("HISTORICAL_REPLAY_END_BLOCK").ok().and_then(|v| v.parse().ok()),
            historical_recheck_blocks: env::var("HISTORICAL_RECHECK_BLOCKS")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .unwrap_or(1),
            historical_replay_output_path: env::var("HISTORICAL_REPLAY_OUTPUT_PATH")
                .unwrap_or_else(|_| "historical_replay_calibration_final.json".to_string()),
            historical_replay_metrics_port: env::var("HISTORICAL_REPLAY_METRICS_PORT")
                .unwrap_or_else(|_| "9091".to_string())
                .parse()
                .unwrap_or(9091),
            historical_max_cases_to_verify: env::var("HISTORICAL_MAX_CASES_TO_VERIFY")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),

            // Phase 23: Canary Policy
            canary_route_family_allowlist: env::var("CANARY_ROUTE_FAMILY_ALLOWLIST")
                .unwrap_or_else(|_| "multi".to_string()),
            canary_route_family_blocklist: env::var("CANARY_ROUTE_FAMILY_BLOCKLIST")
                .unwrap_or_else(|_| "direct".to_string()),
            canary_max_trade_size_wei: env::var("CANARY_MAX_TRADE_SIZE_WEI")
                .unwrap_or_else(|_| "30000000000000000".to_string())
                .parse()
                .unwrap_or(30_000_000_000_000_000),
            canary_max_concurrent_trades: env::var("CANARY_MAX_CONCURRENT_TRADES")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .unwrap_or(1),
            canary_max_consecutive_reverts: env::var("CANARY_MAX_CONSECUTIVE_REVERTS")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3),
            canary_review_threshold_attempts: env::var("CANARY_REVIEW_THRESHOLD_ATTEMPTS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            canary_loss_cap_wei: env::var("CANARY_LOSS_CAP_WEI")
                .unwrap_or_else(|_| "50000000000000000".to_string())
                .parse()
                .unwrap_or(50_000_000_000_000_000),
            canary_live_mode_enabled: env::var("CANARY_LIVE_MODE_ENABLED")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false),

            // Phase 23: Tenderly Scaffold
            tenderly_api_key: env::var("TENDERLY_API_KEY").ok(),
            tenderly_account_slug: env::var("TENDERLY_ACCOUNT_SLUG")
                .unwrap_or_else(|_| "".to_string()),
            tenderly_project_slug: env::var("TENDERLY_PROJECT_SLUG")
                .unwrap_or_else(|_| "".to_string()),
            tenderly_enabled: env::var("TENDERLY_ENABLED")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false),

            // Phase 24
            canary_state_path: env::var("CANARY_STATE_PATH")
                .unwrap_or_else(|_| "canary_state.json".to_string()),
            tenderly_timeout_ms: env::var("TENDERLY_TIMEOUT_MS")
                .unwrap_or_else(|_| "10000".to_string())
                .parse()
                .unwrap_or(10000),
            gas_limit_multiplier_bps: env::var("GAS_LIMIT_MULTIPLIER_BPS")
                .unwrap_or_else(|_| "12000".to_string())
                .parse()
                .unwrap_or(12000),
            gas_limit_min: env::var("GAS_LIMIT_MIN")
                .unwrap_or_else(|_| "21000".to_string())
                .parse()
                .unwrap_or(21000),
            gas_limit_max: env::var("GAS_LIMIT_MAX")
                .unwrap_or_else(|_| "5000000".to_string())
                .parse()
                .unwrap_or(5000000),
        };

        // Safety Gate: Ambiguous States
        if parsed_config.canary_live_mode_enabled && parsed_config.dry_run_only {
            panic!("FATAL SECURITY GATE: CANARY_LIVE_MODE_ENABLED=true while DRY_RUN_ONLY=true. Ambiguous half-live state is prohibited.");
        }

        if parsed_config.enable_shadow_mode && parsed_config.enable_broadcast {
            panic!("FATAL SECURITY GATE: ENABLE_SHADOW_MODE and ENABLE_BROADCAST cannot both be true. Shadow mode must never have live broadcast capability.");
        }

        parsed_config.validate_live_canary_config();

        parsed_config
    }

    /// Validates that all required configuration is present if live-canary mode is enabled.
    /// Panics if requirements are not met to ensure a fail-fast startup.
    pub fn validate_live_canary_config(&self) {
        if !self.canary_live_mode_enabled {
            return;
        }

        tracing::info!("Validating live-canary configuration...");

        if self.signer_private_key.is_none() {
            panic!("FATAL: CANARY_LIVE_MODE_ENABLED requires SIGNER_PRIVATE_KEY.");
        }
        if self.executor_contract_address.is_none() {
            panic!("FATAL: CANARY_LIVE_MODE_ENABLED requires EXECUTOR_CONTRACT_ADDRESS.");
        }
        if !self.tenderly_enabled {
            panic!("FATAL: CANARY_LIVE_MODE_ENABLED requires TENDERLY_ENABLED=true as a safety gate.");
        }
        if self.tenderly_api_key.is_none() {
            panic!("FATAL: CANARY_LIVE_MODE_ENABLED requires TENDERLY_API_KEY.");
        }
        if self.rpc_http_url.is_none() {
            panic!("FATAL: CANARY_LIVE_MODE_ENABLED requires RPC_HTTP_URL for preflight/Tenderly.");
        }
        if self.dry_run_only {
            panic!("FATAL: CANARY_LIVE_MODE_ENABLED requires DRY_RUN_ONLY=false.");
        }

        tracing::info!("Live-canary configuration is VALID.");
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
            std::env::set_var("ROOT_ASSET", "0xTEST");
            std::env::set_var("MIN_GROSS_PROFIT", "123");
            std::env::set_var("MIN_GROSS_BPS", "50");
            std::env::set_var("REQUIRE_FRESH", "false");
            std::env::set_var("QUOTE_BUCKETS", "1,2,3");
            std::env::set_var("SIGNER_PRIVATE_KEY", "0xPK");
            std::env::set_var("EXECUTOR_CONTRACT_ADDRESS", "0xCONTRACT");
            std::env::set_var("ENABLE_BROADCAST", "true");
            std::env::set_var("DRY_RUN_ONLY", "false");
            std::env::set_var("RPC_HTTP_URL", "http://rpc");
            std::env::set_var("REQUIRE_PREFLIGHT", "false");
            std::env::set_var("REQUIRE_GAS_ESTIMATE", "true");
            std::env::set_var("REQUIRE_ETH_CALL", "1");
            std::env::set_var("TEST_PRIVATE_KEY", "0xTESTPK");
            std::env::set_var("ANVIL_RPC_URL", "http://local");
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
        assert_eq!(config.root_asset, "0xTEST");
        assert_eq!(config.min_gross_profit, "123");
        assert_eq!(config.min_gross_bps, 50);
        assert!(!config.require_fresh);
        assert_eq!(config.quote_buckets, "1,2,3");
        assert_eq!(config.signer_private_key, Some("0xPK".to_string()));
        assert_eq!(config.executor_contract_address, Some("0xCONTRACT".to_string()));
        assert!(config.enable_broadcast);
        assert!(!config.dry_run_only);
        assert_eq!(config.rpc_http_url, Some("http://rpc".to_string()));
        assert!(!config.require_preflight);
        assert!(config.require_gas_estimate);
        assert_eq!(config.require_eth_call, true);
        assert_eq!(config.test_private_key, Some("0xTESTPK".to_string()));
        assert_eq!(config.local_rpc_url, Some("http://local".to_string()));
        assert_eq!(config.enable_shadow_mode, false);
    }
}
