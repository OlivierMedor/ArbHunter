use prometheus::{Encoder, IntCounter, IntGauge, IntCounterVec, IntGaugeVec, Opts, Registry, TextEncoder};
use std::time::SystemTime;

#[derive(Clone)]
pub struct MetricsRegistry {
    pub registry: Registry,
    pub provider_connected_total: IntCounterVec,
    pub provider_disconnected_total: IntCounterVec,
    pub provider_connected: IntGaugeVec,
    pub provider_reconnect_attempts: IntCounter,
    pub provider_latency_ms: IntGauge,
    pub failover_switches: IntCounter,
    pub events_ingested_total: IntCounter,
    pub flashblocks_seen_total: IntCounter,
    pub pending_logs_seen_total: IntCounter,
    
    pub provider_frames_forwarded_total: IntCounter,
    pub malformed_payloads_total: IntCounter,
    pub daemon_startups_total: IntCounter,
    pub metrics_requests_total: IntCounter,
    pub active_provider: IntGaugeVec,
    pub daemon_start_time: SystemTime,
    pub daemon_uptime_seconds: IntGauge,

    // Phase 3: State engine metrics
    pub state_updates_total: IntCounter,
    pub pools_tracked_total: IntGauge,
    pub stale_pool_events_total: IntCounter,

    pub dex_sync_events_total: IntCounter,
    pub dex_cl_swap_events_total: IntCounter,
    pub unsupported_dex_logs_total: IntCounter,

    // Phase 5: Tickmap & Quoter metrics
    pub cl_ticks_tracked: IntGauge,
    pub cl_state_updates_total: IntCounter,
    pub local_quotes_total: IntCounter,
    pub local_quote_errors_total: IntCounter,
    
    // Phase 6: Route Graph & Candidate metrics
    pub route_nodes_total: IntGauge,
    pub route_edges_total: IntGauge,
    pub candidates_considered_total: IntCounter,
    pub candidates_promoted_total: IntCounter,
    pub quote_failures_total: IntCounter,
    pub stale_pool_skips_total: IntCounter,

    // Phase 7: Simulation metrics
    pub simulations_total: IntCounter,
    pub simulations_success_total: IntCounter,
    pub simulations_failed_total: IntCounter,
    pub candidates_validated_total: IntCounter,

    // Phase 9: Submission metrics
    pub submission_attempts_total: IntCounter,
    pub submission_signed_total: IntCounter,
    pub submission_broadcast_total: IntCounter,
    pub submission_failed_total: IntCounter,
    pub submission_dry_run_total: IntCounter,
    pub nonce_fetch_total: IntCounter,
    pub nonce_fetch_failures_total: IntCounter,
    pub tx_build_total: IntCounter,
    pub tx_build_failures_total: IntCounter,

    // Phase 10: Preflight metrics
    pub preflight_total: IntCounter,
    pub preflight_success_total: IntCounter,
    pub preflight_failed_total: IntCounter,
    pub preflight_eth_call_failed_total: IntCounter,
    pub preflight_gas_estimate_failed_total: IntCounter,

    // Phase 15: Shadow Mode Tracking
    pub shadow_candidates_total: IntCounter,
    pub shadow_promoted_total: IntCounter,
    pub shadow_would_trade_total: IntCounter,
    pub shadow_rechecks_total: IntCounter,
    pub shadow_still_profitable_total: IntCounter,
    pub shadow_invalidated_total: IntCounter,
    pub shadow_latest_profit_drift: IntGauge,
    pub shadow_latest_output_drift: IntGauge,

    // Phase 16: Historical Replay Stats
    pub hist_candidates_total: IntCounter,
    pub hist_promoted_total: IntCounter,
    pub hist_would_trade_total: IntCounter,
    pub hist_rechecks_total: IntCounter,
    pub hist_still_profitable_total: IntCounter,
    pub hist_invalidated_total: IntCounter,
    pub hist_profit_drift_total: IntCounter,
    pub hist_amount_out_drift_total: IntCounter,
    pub hist_route_family_total: IntCounterVec,
    pub hist_fork_verifications_total: IntCounter,
    pub hist_fork_verifications_success_total: IntCounter,
    pub hist_fork_verifications_failed_total: IntCounter,
    pub hist_fork_realized_profit_total: IntCounter,
    
    // Phase 18: Calibration Metrics
    pub hist_opportunity_density: IntGauge,
    pub hist_bucket_total: IntCounterVec,
    pub hist_clustering_freq: IntGauge,

    // Phase 23: Canary Telemetry
    /// Canary attempts by (route_family, bucket) labels.
    pub canary_attempts_total: IntCounterVec,
    /// Canary reverts by (route_family, bucket) labels.
    pub canary_reverts_total: IntCounterVec,
    /// Current consecutive revert streak.
    pub canary_consecutive_reverts: IntGauge,
    /// Cumulative realized PnL in Wei (positive or negative, as i64).
    pub canary_realized_pnl_wei: IntGauge,
    /// Cumulative realized loss in Wei (>= 0).
    pub canary_cumulative_loss_wei: IntGauge,
    /// Policy-block events, labeled by rejection reason.
    pub canary_policy_blocks_total: IntCounterVec,
    /// Incremented once when the review threshold is first reached.
    pub canary_review_threshold_reached_total: IntCounter,
    /// Total allowed-through-gate count.
    pub canary_allowed_total: IntCounter,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        let registry = Registry::new();

        let provider_connected_total = IntCounterVec::new(Opts::new("arb_provider_connected_total", "Total provider connections established"), &["provider"]).unwrap();
        let provider_disconnected_total = IntCounterVec::new(Opts::new("arb_provider_disconnected_total", "Total provider disconnections"), &["provider"]).unwrap();
        let provider_connected = IntGaugeVec::new(Opts::new("arb_provider_connected", "Provider currently connected (1=yes, 0=no)"), &["provider"]).unwrap();
        let provider_reconnect_attempts = IntCounter::new("arb_provider_reconnect_attempts_total", "Total provider reconnect attempts").unwrap();
        let provider_latency_ms = IntGauge::new("arb_provider_latency_ms", "Current provider latency in ms (stubbed)").unwrap();
        let failover_switches = IntCounter::new("arb_provider_failover_switches_total", "Total failover switches").unwrap();
        
        let events_ingested_total = IntCounter::new("arb_events_ingested_total", "Total events successfully ingested").unwrap();
        let flashblocks_seen_total = IntCounter::new("arb_flashblocks_seen_total", "Total flashblocks seen").unwrap();
        let pending_logs_seen_total = IntCounter::new("arb_pending_logs_seen_total", "Total pending logs seen").unwrap();
        let provider_frames_forwarded_total = IntCounter::new("arb_provider_frames_forwarded_total", "Total raw websocket frames forwarded").unwrap();
        let malformed_payloads_total = IntCounter::new("arb_malformed_payloads_total", "Total malformed payloads discarded").unwrap();
        let daemon_startups_total = IntCounter::new("arb_daemon_startups_total", "Total daemon startups via process boot").unwrap();
        let metrics_requests_total = IntCounter::new("arb_metrics_requests_total", "Total scrapes of metrics endpoint").unwrap();
        let active_provider = IntGaugeVec::new(Opts::new("arb_active_provider", "Currently active provider indicator"), &["provider"]).unwrap();
        let daemon_uptime_seconds = IntGauge::new("arb_daemon_uptime_seconds", "Seconds elapsed since daemon startup").unwrap();

        // Phase 3: State engine metrics
        let state_updates_total = IntCounter::new("arb_state_updates_total", "Total pool state updates applied").unwrap();
        let pools_tracked_total = IntGauge::new("arb_pools_tracked_total", "Current number of pools tracked in state engine").unwrap();
        let stale_pool_events_total = IntCounter::new("arb_stale_pool_events_total", "Pool updates rejected as stale (out-of-order)").unwrap();

        let dex_sync_events_total = IntCounter::new("arb_dex_sync_events_total", "Total Uniswap V2 Sync events decoded").unwrap();
        let dex_cl_swap_events_total = IntCounter::new("arb_dex_cl_swap_events_total", "Total Uniswap V3 Swap events decoded").unwrap();
        let unsupported_dex_logs_total = IntCounter::new("arb_unsupported_dex_logs_total", "Total DEX logs seen but not supported for state").unwrap();

        // Phase 5: Tickmap & Quoter metrics
        let cl_ticks_tracked = IntGauge::new("arb_cl_ticks_tracked_total", "Current number of CL ticks tracked").unwrap();
        let cl_state_updates_total = IntCounter::new("arb_cl_state_updates_total", "Total CL-specific state updates (Mint/Burn/Init)").unwrap();
        let local_quotes_total = IntCounter::new("arb_local_quotes_total", "Total local quote requests").unwrap();
        let local_quote_errors_total = IntCounter::new("arb_local_quote_errors_total", "Total local quote errors").unwrap();

        // Phase 6: Route Graph & Candidate metrics
        let route_nodes_total = IntGauge::new("arb_route_nodes_total", "Total nodes in the local route graph").unwrap();
        let route_edges_total = IntGauge::new("arb_route_edges_total", "Total edges in the local route graph").unwrap();
        let candidates_considered_total = IntCounter::new("arb_candidates_considered_total", "Total candidates considered").unwrap();
        let candidates_promoted_total = IntCounter::new("arb_candidates_promoted_total", "Total candidates promoted above threshold").unwrap();
        let quote_failures_total = IntCounter::new("arb_quote_failures_total", "Total local quote failures during search").unwrap();
        let stale_pool_skips_total = IntCounter::new("arb_stale_pool_skips_total", "Total stale pools skipped in routing").unwrap();

        // Phase 7: Simulation metrics
        let simulations_total = IntCounter::new("arb_simulations_total", "Total simulations run").unwrap();
        let simulations_success_total = IntCounter::new("arb_simulations_success_total", "Total simulations that succeeded").unwrap();
        let simulations_failed_total = IntCounter::new("arb_simulations_failed_total", "Total simulations that failed").unwrap();
        let candidates_validated_total = IntCounter::new("arb_candidates_validated_total", "Total candidates validated and accepted for execution/logging").unwrap();

        // Phase 9: Submission metrics
        let submission_attempts_total = IntCounter::new("arb_submission_attempts_total", "Total transaction submission attempts").unwrap();
        let submission_signed_total = IntCounter::new("arb_submission_signed_total", "Total transactions successfully signed").unwrap();
        let submission_broadcast_total = IntCounter::new("arb_submission_broadcast_total", "Total transactions successfully broadcast").unwrap();
        let submission_failed_total = IntCounter::new("arb_submission_failed_total", "Total transaction submissions that failed").unwrap();
        let submission_dry_run_total = IntCounter::new("arb_submission_dry_run_total", "Total transaction dry-runs").unwrap();
        let nonce_fetch_total = IntCounter::new("arb_nonce_fetch_total", "Total nonce fetch attempts").unwrap();
        let nonce_fetch_failures_total = IntCounter::new("arb_nonce_fetch_failures_total", "Total nonce fetch failures").unwrap();
        let tx_build_total = IntCounter::new("arb_tx_build_total", "Total transaction build attempts").unwrap();
        let tx_build_failures_total = IntCounter::new("arb_tx_build_failures_total", "Total transaction build failures").unwrap();

        // Phase 10
        let preflight_total = IntCounter::new("arb_preflight_total", "Total preflight validation attempts").unwrap();
        let preflight_success_total = IntCounter::new("arb_preflight_success_total", "Total preflight successes").unwrap();
        let preflight_failed_total = IntCounter::new("arb_preflight_failed_total", "Total preflight failures").unwrap();
        let preflight_eth_call_failed_total = IntCounter::new("arb_preflight_eth_call_failed_total", "Total preflight eth_call failures").unwrap();
        let preflight_gas_estimate_failed_total = IntCounter::new("arb_preflight_gas_estimate_failed_total", "Total preflight gas estimate failures").unwrap();

        // Phase 15
        let shadow_candidates_total = IntCounter::new("arb_shadow_candidates_total", "Total candidates considered in shadow mode").unwrap();
        let shadow_promoted_total = IntCounter::new("arb_shadow_promoted_total", "Total candidates promoted to shadow plans").unwrap();
        let shadow_would_trade_total = IntCounter::new("arb_shadow_would_trade_total", "Total shadow combinations that would have traded").unwrap();
        let shadow_rechecks_total = IntCounter::new("arb_shadow_rechecks_total", "Total shadow delayed rechecks performed").unwrap();
        let shadow_still_profitable_total = IntCounter::new("arb_shadow_still_profitable_total", "Total rechecked items that were still profitable").unwrap();
        let shadow_invalidated_total = IntCounter::new("arb_shadow_invalidated_total", "Total rechecked items that lost profitability").unwrap();
        let shadow_latest_profit_drift = IntGauge::new("arb_shadow_latest_profit_drift", "Latest observed shadow profit drift in wei (can be negative)").unwrap();
        let shadow_latest_output_drift = IntGauge::new("arb_shadow_latest_output_drift", "Latest observed shadow output drift in wei").unwrap();

        // Phase 16
        let hist_candidates_total = IntCounter::new("arb_hist_candidates_total", "Total candidates considered in historical replay").unwrap();
        let hist_promoted_total = IntCounter::new("arb_hist_promoted_total", "Total candidates promoted in historical replay").unwrap();
        let hist_would_trade_total = IntCounter::new("arb_hist_would_trade_total", "Total candidates that would have traded in historical replay").unwrap();
        let hist_rechecks_total = IntCounter::new("arb_hist_rechecks_total", "Total historical rechecks performed").unwrap();
        let hist_still_profitable_total = IntCounter::new("arb_hist_still_profitable_total", "Total historical rechecks still profitable").unwrap();
        let hist_invalidated_total = IntCounter::new("arb_hist_invalidated_total", "Total historical rechecks invalidated").unwrap();
        let hist_profit_drift_total = IntCounter::new("arb_hist_profit_drift_total", "Cumulative profit drift in wei (absolute sum of drift)").unwrap();
        let hist_amount_out_drift_total = IntCounter::new("arb_hist_amount_out_drift_total", "Cumulative amount_out drift in wei").unwrap();
        let hist_route_family_total = IntCounterVec::new(Opts::new("arb_hist_route_family_total", "Historical candidates by route family"), &["family"]).unwrap();
        let hist_fork_verifications_total = IntCounter::new("arb_hist_fork_verifications_total", "Total fork verifications").unwrap();
        let hist_fork_verifications_success_total = IntCounter::new("arb_hist_fork_verifications_success_total", "Total successful fork verifications").unwrap();
        let hist_fork_verifications_failed_total = IntCounter::new("arb_hist_fork_verifications_failed_total", "Total failed fork verifications").unwrap();
        let hist_fork_realized_profit_total = IntCounter::new("arb_hist_fork_realized_profit_total", "Total realized profit from successful fork verifications").unwrap();
        
        // Phase 18
        let hist_opportunity_density = IntGauge::new("arb_hist_opportunity_density", "Average trade opportunities per block (x1000)").unwrap();
        let hist_bucket_total = IntCounterVec::new(Opts::new("arb_hist_bucket_total", "Historical candidates by size bucket"), &["bucket"]).unwrap();
        let hist_clustering_freq = IntGauge::new("arb_hist_clustering_freq", "Frequency of multiple candidates per block (x1000)").unwrap();

        // Phase 23: Canary Telemetry
        let canary_attempts_total = IntCounterVec::new(Opts::new("arb_canary_attempts_total", "Canary gate attempts by route family and bucket"), &["route_family", "bucket"]).unwrap();
        let canary_reverts_total = IntCounterVec::new(Opts::new("arb_canary_reverts_total", "Canary execution reverts by route family and bucket"), &["route_family", "bucket"]).unwrap();
        let canary_consecutive_reverts = IntGauge::new("arb_canary_consecutive_reverts", "Current consecutive canary revert streak").unwrap();
        let canary_realized_pnl_wei = IntGauge::new("arb_canary_realized_pnl_wei", "Cumulative realized canary PnL in Wei (can be negative)").unwrap();
        let canary_cumulative_loss_wei = IntGauge::new("arb_canary_cumulative_loss_wei", "Cumulative realized canary loss in Wei (non-negative)").unwrap();
        let canary_policy_blocks_total = IntCounterVec::new(Opts::new("arb_canary_policy_blocks_total", "Canary gate policy rejections by reason"), &["reason"]).unwrap();
        let canary_review_threshold_reached_total = IntCounter::new("arb_canary_review_threshold_reached_total", "Times the canary review threshold was reached").unwrap();
        let canary_allowed_total = IntCounter::new("arb_canary_allowed_total", "Total candidates allowed through the canary gate").unwrap();

        registry.register(Box::new(provider_connected_total.clone())).unwrap();
        registry.register(Box::new(provider_disconnected_total.clone())).unwrap();
        registry.register(Box::new(provider_connected.clone())).unwrap();
        registry.register(Box::new(provider_reconnect_attempts.clone())).unwrap();
        registry.register(Box::new(provider_latency_ms.clone())).unwrap();
        registry.register(Box::new(failover_switches.clone())).unwrap();
        registry.register(Box::new(events_ingested_total.clone())).unwrap();
        registry.register(Box::new(flashblocks_seen_total.clone())).unwrap();
        registry.register(Box::new(pending_logs_seen_total.clone())).unwrap();
        registry.register(Box::new(provider_frames_forwarded_total.clone())).unwrap();
        registry.register(Box::new(malformed_payloads_total.clone())).unwrap();
        registry.register(Box::new(daemon_startups_total.clone())).unwrap();
        registry.register(Box::new(metrics_requests_total.clone())).unwrap();
        registry.register(Box::new(active_provider.clone())).unwrap();
        registry.register(Box::new(daemon_uptime_seconds.clone())).unwrap();
        registry.register(Box::new(state_updates_total.clone())).unwrap();
        registry.register(Box::new(pools_tracked_total.clone())).unwrap();
        registry.register(Box::new(stale_pool_events_total.clone())).unwrap();
        registry.register(Box::new(dex_sync_events_total.clone())).unwrap();
        registry.register(Box::new(dex_cl_swap_events_total.clone())).unwrap();
        registry.register(Box::new(unsupported_dex_logs_total.clone())).unwrap();
        registry.register(Box::new(cl_ticks_tracked.clone())).unwrap();
        registry.register(Box::new(cl_state_updates_total.clone())).unwrap();
        registry.register(Box::new(local_quotes_total.clone())).unwrap();
        registry.register(Box::new(local_quote_errors_total.clone())).unwrap();
        registry.register(Box::new(route_nodes_total.clone())).unwrap();
        registry.register(Box::new(route_edges_total.clone())).unwrap();
        registry.register(Box::new(candidates_considered_total.clone())).unwrap();
        registry.register(Box::new(candidates_promoted_total.clone())).unwrap();
        registry.register(Box::new(quote_failures_total.clone())).unwrap();
        registry.register(Box::new(stale_pool_skips_total.clone())).unwrap();

        registry.register(Box::new(simulations_total.clone())).unwrap();
        registry.register(Box::new(simulations_success_total.clone())).unwrap();
        registry.register(Box::new(simulations_failed_total.clone())).unwrap();
        registry.register(Box::new(candidates_validated_total.clone())).unwrap();

        registry.register(Box::new(submission_attempts_total.clone())).unwrap();
        registry.register(Box::new(submission_signed_total.clone())).unwrap();
        registry.register(Box::new(submission_broadcast_total.clone())).unwrap();
        registry.register(Box::new(submission_failed_total.clone())).unwrap();
        registry.register(Box::new(submission_dry_run_total.clone())).unwrap();
        registry.register(Box::new(nonce_fetch_total.clone())).unwrap();
        registry.register(Box::new(nonce_fetch_failures_total.clone())).unwrap();
        registry.register(Box::new(tx_build_total.clone())).unwrap();
        registry.register(Box::new(tx_build_failures_total.clone())).unwrap();

        registry.register(Box::new(preflight_total.clone())).unwrap();
        registry.register(Box::new(preflight_success_total.clone())).unwrap();
        registry.register(Box::new(preflight_failed_total.clone())).unwrap();
        registry.register(Box::new(preflight_eth_call_failed_total.clone())).unwrap();
        registry.register(Box::new(preflight_gas_estimate_failed_total.clone())).unwrap();
        registry.register(Box::new(hist_fork_realized_profit_total.clone())).unwrap();

        registry.register(Box::new(shadow_candidates_total.clone())).unwrap();
        registry.register(Box::new(shadow_promoted_total.clone())).unwrap();
        registry.register(Box::new(shadow_would_trade_total.clone())).unwrap();
        registry.register(Box::new(shadow_rechecks_total.clone())).unwrap();
        registry.register(Box::new(shadow_still_profitable_total.clone())).unwrap();
        registry.register(Box::new(shadow_invalidated_total.clone())).unwrap();
        registry.register(Box::new(shadow_latest_profit_drift.clone())).unwrap();
        registry.register(Box::new(shadow_latest_output_drift.clone())).unwrap();

        registry.register(Box::new(hist_candidates_total.clone())).unwrap();
        registry.register(Box::new(hist_promoted_total.clone())).unwrap();
        registry.register(Box::new(hist_would_trade_total.clone())).unwrap();
        registry.register(Box::new(hist_rechecks_total.clone())).unwrap();
        registry.register(Box::new(hist_still_profitable_total.clone())).unwrap();
        registry.register(Box::new(hist_invalidated_total.clone())).unwrap();
        registry.register(Box::new(hist_profit_drift_total.clone())).unwrap();
        registry.register(Box::new(hist_amount_out_drift_total.clone())).unwrap();
        registry.register(Box::new(hist_route_family_total.clone())).unwrap();
        registry.register(Box::new(hist_fork_verifications_total.clone())).unwrap();
        registry.register(Box::new(hist_fork_verifications_success_total.clone())).unwrap();
        registry.register(Box::new(hist_fork_verifications_failed_total.clone())).unwrap();
        registry.register(Box::new(hist_opportunity_density.clone())).unwrap();
        registry.register(Box::new(hist_bucket_total.clone())).unwrap();
        registry.register(Box::new(hist_clustering_freq.clone())).unwrap();

        // Phase 23
        registry.register(Box::new(canary_attempts_total.clone())).unwrap();
        registry.register(Box::new(canary_reverts_total.clone())).unwrap();
        registry.register(Box::new(canary_consecutive_reverts.clone())).unwrap();
        registry.register(Box::new(canary_realized_pnl_wei.clone())).unwrap();
        registry.register(Box::new(canary_cumulative_loss_wei.clone())).unwrap();
        registry.register(Box::new(canary_policy_blocks_total.clone())).unwrap();
        registry.register(Box::new(canary_review_threshold_reached_total.clone())).unwrap();
        registry.register(Box::new(canary_allowed_total.clone())).unwrap();

        daemon_startups_total.inc();
        active_provider.with_label_values(&["quicknode"]).set(0);
        active_provider.with_label_values(&["alchemy"]).set(0);

        Self {
            registry,
            provider_connected_total,
            provider_disconnected_total,
            provider_connected,
            provider_reconnect_attempts,
            provider_latency_ms,
            failover_switches,
            events_ingested_total,
            flashblocks_seen_total,
            pending_logs_seen_total,
            provider_frames_forwarded_total,
            malformed_payloads_total,
            daemon_startups_total,
            metrics_requests_total,
            active_provider,
            daemon_start_time: SystemTime::now(),
            daemon_uptime_seconds,
            state_updates_total,
            pools_tracked_total,
            stale_pool_events_total,
            dex_sync_events_total,
            dex_cl_swap_events_total,
            unsupported_dex_logs_total,
            cl_ticks_tracked,
            cl_state_updates_total,
            local_quotes_total,
            local_quote_errors_total,
            route_nodes_total,
            route_edges_total,
            candidates_considered_total,
            candidates_promoted_total,
            quote_failures_total,
            stale_pool_skips_total,
            simulations_total,
            simulations_success_total,
            simulations_failed_total,
            candidates_validated_total,
            submission_attempts_total,
            submission_signed_total,
            submission_broadcast_total,
            submission_failed_total,
            submission_dry_run_total,
            nonce_fetch_total,
            nonce_fetch_failures_total,
            tx_build_total,
            tx_build_failures_total,
            preflight_total,
            preflight_success_total,
            preflight_failed_total,
            preflight_eth_call_failed_total,
            preflight_gas_estimate_failed_total,
            shadow_candidates_total,
            shadow_promoted_total,
            shadow_would_trade_total,
            shadow_rechecks_total,
            shadow_still_profitable_total,
            shadow_invalidated_total,
            shadow_latest_profit_drift,
            shadow_latest_output_drift,
            hist_candidates_total,
            hist_promoted_total,
            hist_would_trade_total,
            hist_rechecks_total,
            hist_still_profitable_total,
            hist_invalidated_total,
            hist_profit_drift_total,
            hist_amount_out_drift_total,
            hist_route_family_total,
            hist_fork_verifications_total,
            hist_fork_verifications_success_total,
            hist_fork_verifications_failed_total,
            hist_fork_realized_profit_total,
            hist_opportunity_density,
            hist_bucket_total,
            hist_clustering_freq,
            canary_attempts_total,
            canary_reverts_total,
            canary_consecutive_reverts,
            canary_realized_pnl_wei,
            canary_cumulative_loss_wei,
            canary_policy_blocks_total,
            canary_review_threshold_reached_total,
            canary_allowed_total,
        }
    }

    pub fn inc_state_updates(&self) {
        self.state_updates_total.inc();
    }

    pub fn set_pools_tracked(&self, count: i64) {
        self.pools_tracked_total.set(count);
    }

    pub fn inc_stale_pool_events(&self) {
        self.stale_pool_events_total.inc();
    }

    pub fn inc_dex_sync_events(&self) {
        self.dex_sync_events_total.inc();
    }

    pub fn inc_dex_cl_swap_events(&self) {
        self.dex_cl_swap_events_total.inc();
    }

    pub fn inc_unsupported_dex_logs(&self) {
        self.unsupported_dex_logs_total.inc();
    }

    pub fn set_cl_ticks_tracked(&self, count: i64) {
        self.cl_ticks_tracked.set(count);
    }

    pub fn inc_cl_state_updates(&self) {
        self.cl_state_updates_total.inc();
    }

    pub fn inc_local_quotes(&self) {
        self.local_quotes_total.inc();
    }

    pub fn inc_local_quote_errors(&self) {
        self.local_quote_errors_total.inc();
    }

    pub fn set_route_nodes(&self, count: i64) {
        self.route_nodes_total.set(count);
    }

    pub fn set_route_edges(&self, count: i64) {
        self.route_edges_total.set(count);
    }

    pub fn inc_candidates_considered(&self) {
        self.candidates_considered_total.inc();
    }

    pub fn inc_candidates_promoted(&self) {
        self.candidates_promoted_total.inc();
    }

    pub fn inc_quote_failures(&self) {
        self.quote_failures_total.inc();
    }

    pub fn inc_stale_pool_skips(&self) {
        self.stale_pool_skips_total.inc();
    }

    // Phase 7
    pub fn inc_simulations(&self) {
        self.simulations_total.inc();
    }
    
    pub fn inc_simulations_success(&self) {
        self.simulations_success_total.inc();
    }

    pub fn inc_simulations_failed(&self) {
        self.simulations_failed_total.inc();
    }

    pub fn inc_candidates_validated(&self) {
        self.candidates_validated_total.inc();
    }

    // Phase 9
    pub fn inc_submission_attempts(&self) {
        self.submission_attempts_total.inc();
    }

    pub fn inc_submission_signed(&self) {
        self.submission_signed_total.inc();
    }

    pub fn inc_submission_broadcast(&self) {
        self.submission_broadcast_total.inc();
    }

    pub fn inc_submission_failed(&self) {
        self.submission_failed_total.inc();
    }

    pub fn inc_submission_dry_run(&self) {
        self.submission_dry_run_total.inc();
    }

    pub fn inc_nonce_fetch(&self) {
        self.nonce_fetch_total.inc();
    }

    pub fn inc_nonce_fetch_failures(&self) {
        self.nonce_fetch_failures_total.inc();
    }

    pub fn inc_tx_build(&self) {
        self.tx_build_total.inc();
    }

    pub fn inc_tx_build_failures(&self) {
        self.tx_build_failures_total.inc();
    }

    // Phase 10
    pub fn inc_preflight(&self) {
        self.preflight_total.inc();
    }

    pub fn inc_preflight_success(&self) {
        self.preflight_success_total.inc();
    }

    pub fn inc_preflight_failed(&self) {
        self.preflight_failed_total.inc();
    }

    pub fn inc_preflight_eth_call_failed(&self) {
        self.preflight_eth_call_failed_total.inc();
    }

    pub fn inc_preflight_gas_estimate_failed(&self) {
        self.preflight_gas_estimate_failed_total.inc();
    }

    pub fn inc_provider_connected(&self, provider: &str) {
        self.provider_connected_total.with_label_values(&[provider]).inc();
        self.provider_connected.with_label_values(&[provider]).set(1);
    }

    pub fn inc_provider_disconnected(&self, provider: &str) {
        self.provider_disconnected_total.with_label_values(&[provider]).inc();
        self.provider_connected.with_label_values(&[provider]).set(0);
    }

    pub fn inc_reconnect_attempts(&self) {
        self.provider_reconnect_attempts.inc();
    }

    pub fn set_provider_latency_ms(&self, latency: u64) {
        self.provider_latency_ms.set(latency as i64);
    }

    pub fn inc_failover_switches(&self) {
        self.failover_switches.inc();
    }

    pub fn inc_events_ingested(&self) {
        self.events_ingested_total.inc();
    }

    pub fn inc_flashblocks_seen(&self) {
        self.flashblocks_seen_total.inc();
    }

    pub fn inc_pending_logs_seen(&self) {
        self.pending_logs_seen_total.inc();
    }

    pub fn inc_provider_frames_forwarded(&self) {
        self.provider_frames_forwarded_total.inc();
    }

    pub fn inc_malformed_payloads(&self) {
        self.malformed_payloads_total.inc();
    }

    pub fn set_active_provider(&self, provider: &str) {
        self.active_provider.with_label_values(&["quicknode"]).set(0);
        self.active_provider.with_label_values(&["alchemy"]).set(0);
        if provider == "quicknode" || provider == "alchemy" {
            self.active_provider.with_label_values(&[provider]).set(1);
        }
    }

    pub fn gather_metrics(&self) -> String {
        self.metrics_requests_total.inc();
        if let Ok(duration) = self.daemon_start_time.elapsed() {
            self.daemon_uptime_seconds.set(duration.as_secs() as i64);
        }

        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }

    // Phase 15
    pub fn inc_shadow_candidates(&self) {
        self.shadow_candidates_total.inc();
    }
    
    pub fn inc_shadow_promoted(&self) {
        self.shadow_promoted_total.inc();
    }
    
    pub fn inc_shadow_would_trade(&self) {
        self.shadow_would_trade_total.inc();
    }
    
    pub fn inc_shadow_rechecks(&self) {
        self.shadow_rechecks_total.inc();
    }
    
    pub fn inc_shadow_still_profitable(&self) {
        self.shadow_still_profitable_total.inc();
    }
    
    pub fn inc_shadow_invalidated(&self) {
        self.shadow_invalidated_total.inc();
    }
    
    pub fn update_shadow_drift(&self, profit_drift: i128, output_drift: i128) {
        // Simple gauge mapping for the simplest metric exposure
        self.shadow_latest_profit_drift.set(profit_drift as i64);
        self.shadow_latest_output_drift.set(output_drift as i64);
    }

    // Phase 16
    pub fn inc_hist_candidates(&self) { self.hist_candidates_total.inc(); }
    pub fn inc_hist_promoted(&self) { self.hist_promoted_total.inc(); }
    pub fn inc_hist_would_trade(&self, family: &str) { 
        self.hist_would_trade_total.inc(); 
        self.hist_route_family_total.with_label_values(&[family]).inc();
    }
    pub fn inc_hist_rechecks(&self) { self.hist_rechecks_total.inc(); }
    pub fn inc_hist_still_profitable(&self) { self.hist_still_profitable_total.inc(); }
    pub fn inc_hist_invalidated(&self) { self.hist_invalidated_total.inc(); }
    pub fn add_hist_drift(&self, profit_drift: u64, amount_out_drift: u64) {
        self.hist_profit_drift_total.inc_by(profit_drift);
        self.hist_amount_out_drift_total.inc_by(amount_out_drift);
    }
    pub fn inc_hist_fork_verification(&self, success: bool, profit_wei: u64) {
        self.hist_fork_verifications_total.inc();
        if success {
            self.hist_fork_verifications_success_total.inc();
            self.hist_fork_realized_profit_total.inc_by(profit_wei);
        } else {
            self.hist_fork_verifications_failed_total.inc();
        }
    }

    // Phase 18
    pub fn set_hist_density(&self, density_x1000: i64) {
        self.hist_opportunity_density.set(density_x1000);
    }
    pub fn inc_hist_bucket(&self, bucket: &str) {
        self.hist_bucket_total.with_label_values(&[bucket]).inc();
    }
    pub fn set_hist_clustering(&self, freq_x1000: i64) {
        self.hist_clustering_freq.set(freq_x1000);
    }

    // Phase 23: Canary Telemetry helpers
    pub fn inc_canary_attempt(&self, route_family: &str, bucket: &str) {
        self.canary_attempts_total.with_label_values(&[route_family, bucket]).inc();
    }
    pub fn inc_canary_revert(&self, route_family: &str, bucket: &str) {
        self.canary_reverts_total.with_label_values(&[route_family, bucket]).inc();
    }
    pub fn set_canary_consecutive_reverts(&self, n: u32) {
        self.canary_consecutive_reverts.set(n as i64);
    }
    pub fn set_canary_realized_pnl_wei(&self, wei: i128) {
        self.canary_realized_pnl_wei.set(wei.clamp(i64::MIN as i128, i64::MAX as i128) as i64);
    }
    pub fn set_canary_cumulative_loss_wei(&self, wei: i128) {
        self.canary_cumulative_loss_wei.set(wei.clamp(0, i64::MAX as i128) as i64);
    }
    pub fn inc_canary_policy_block(&self, reason: &str) {
        self.canary_policy_blocks_total.with_label_values(&[reason]).inc();
    }
    pub fn inc_canary_review_threshold_reached(&self) {
        self.canary_review_threshold_reached_total.inc();
    }
    pub fn inc_canary_allowed(&self) {
        self.canary_allowed_total.inc();
    }
}
