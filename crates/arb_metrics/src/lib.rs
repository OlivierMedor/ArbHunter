use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct MetricsRegistry {
    pub provider_connected: Arc<AtomicU64>,
    pub provider_disconnected: Arc<AtomicU64>,
    pub provider_reconnect_attempts: Arc<AtomicU64>,
    pub provider_latency_ms: Arc<AtomicU64>,
    pub failover_switches: Arc<AtomicU64>,
    pub events_ingested_total: Arc<AtomicU64>,
    pub flashblocks_seen_total: Arc<AtomicU64>,
    pub pending_logs_seen_total: Arc<AtomicU64>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn inc_provider_connected(&self) {
        self.provider_connected.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_provider_disconnected(&self) {
        self.provider_disconnected.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_reconnect_attempts(&self) {
        self.provider_reconnect_attempts.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_provider_latency_ms(&self, latency: u64) {
        self.provider_latency_ms.store(latency, Ordering::Relaxed);
    }

    pub fn inc_failover_switches(&self) {
        self.failover_switches.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_events_ingested(&self) {
        self.events_ingested_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_flashblocks_seen(&self) {
        self.flashblocks_seen_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_pending_logs_seen(&self) {
        self.pending_logs_seen_total.fetch_add(1, Ordering::Relaxed);
    }
}
