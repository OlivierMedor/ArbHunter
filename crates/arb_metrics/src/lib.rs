use prometheus::{Encoder, IntCounter, IntGauge, Registry, TextEncoder};

#[derive(Clone)]
pub struct MetricsRegistry {
    pub registry: Registry,
    pub provider_connected: IntCounter,
    pub provider_disconnected: IntCounter,
    pub provider_reconnect_attempts: IntCounter,
    pub provider_latency_ms: IntGauge,
    pub failover_switches: IntCounter,
    pub events_ingested_total: IntCounter,
    pub flashblocks_seen_total: IntCounter,
    pub pending_logs_seen_total: IntCounter,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        let registry = Registry::new();

        let provider_connected = IntCounter::new("arb_provider_connected_total", "Total provider connections established").unwrap();
        let provider_disconnected = IntCounter::new("arb_provider_disconnected_total", "Total provider disconnections").unwrap();
        let provider_reconnect_attempts = IntCounter::new("arb_provider_reconnect_attempts_total", "Total provider reconnect attempts").unwrap();
        let provider_latency_ms = IntGauge::new("arb_provider_latency_ms", "Current provider latency in ms").unwrap();
        let failover_switches = IntCounter::new("arb_provider_failover_switches_total", "Total failover switches").unwrap();
        let events_ingested_total = IntCounter::new("arb_events_ingested_total", "Total events successfully ingested").unwrap();
        let flashblocks_seen_total = IntCounter::new("arb_flashblocks_seen_total", "Total flashblocks seen").unwrap();
        let pending_logs_seen_total = IntCounter::new("arb_pending_logs_seen_total", "Total pending logs seen").unwrap();

        registry.register(Box::new(provider_connected.clone())).unwrap();
        registry.register(Box::new(provider_disconnected.clone())).unwrap();
        registry.register(Box::new(provider_reconnect_attempts.clone())).unwrap();
        registry.register(Box::new(provider_latency_ms.clone())).unwrap();
        registry.register(Box::new(failover_switches.clone())).unwrap();
        registry.register(Box::new(events_ingested_total.clone())).unwrap();
        registry.register(Box::new(flashblocks_seen_total.clone())).unwrap();
        registry.register(Box::new(pending_logs_seen_total.clone())).unwrap();

        Self {
            registry,
            provider_connected,
            provider_disconnected,
            provider_reconnect_attempts,
            provider_latency_ms,
            failover_switches,
            events_ingested_total,
            flashblocks_seen_total,
            pending_logs_seen_total,
        }
    }

    pub fn inc_provider_connected(&self) {
        self.provider_connected.inc();
    }

    pub fn inc_provider_disconnected(&self) {
        self.provider_disconnected.inc();
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
    
    pub fn gather_metrics(&self) -> String {
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
}

