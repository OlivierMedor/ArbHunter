use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderKind {
    QuickNode,
    Alchemy,
    Other,
}

impl ProviderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderKind::QuickNode => "quicknode",
            ProviderKind::Alchemy => "alchemy",
            ProviderKind::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderHealth {
    Healthy,
    Degraded,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderLatencySample {
    pub provider: ProviderKind,
    pub latency_ms: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatusSnapshot {
    pub provider: ProviderKind,
    pub health: ProviderHealth,
    pub recent_latency_ms: u64,
    pub reconnect_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashblockEvent {
    pub base_fee_per_gas: u64,
    pub block_number: u64,
    pub transaction_count: usize,
    // Minimal for now
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingLogEvent {
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
    pub transaction_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IngestEvent {
    Flashblock(FlashblockEvent),
    PendingLog(PendingLogEvent),
}
