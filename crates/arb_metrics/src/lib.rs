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
}
