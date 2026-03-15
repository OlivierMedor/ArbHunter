use serde::{Deserialize, Serialize};
use alloy_primitives::{U128, U256};

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

// ============================================================
// Phase 3: State Engine Types
// ============================================================

/// Opaque pool identifier (e.g. contract address string)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PoolId(pub String);

/// Supported pool models (adapter hints for the state engine)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PoolKind {
    /// Uniswap V2-style: reserve0 / reserve1
    ReserveBased,
    /// Uniswap V3-style: concentrated liquidity ticks
    ConcentratedLiquidity,
    /// Unknown / not yet classified
    Unknown,
}

/// Newtype wrapper for an ERC-20 token address
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TokenAddress(pub String);

/// Monotonic timestamp for ordering updates (block number + log index)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EventStamp {
    pub block_number: u64,
    pub log_index: u32,
}

/// Tracks when a pool was last updated and whether it is considered fresh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolFreshness {
    pub last_stamp: EventStamp,
    /// Milliseconds since the event was received (wall-clock age)
    pub age_ms: u64,
    pub is_stale: bool,
}

/// Reserve-based snapshot (Uniswap V2-style)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReserveSnapshot {
    pub reserve0: u128,
    pub reserve1: u128,
}

/// Concentrated liquidity snapshot (Uniswap V3-style)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CLSnapshot {
    pub sqrt_price_x96: U256,
    pub liquidity: U128,
    pub tick: i32,
}

/// Canonical pool state snapshot stored in the engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStateSnapshot {
    pub pool_id: PoolId,
    pub kind: PoolKind,
    pub token0: TokenAddress,
    pub token1: TokenAddress,
    pub reserves: Option<ReserveSnapshot>,
    pub cl_snapshot: Option<CLSnapshot>,
    pub freshness: PoolFreshness,
}

/// A single incoming state update derived from an ingest event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolUpdate {
    pub pool_id: PoolId,
    pub kind: PoolKind,
    pub token0: TokenAddress,
    pub token1: TokenAddress,
    pub reserves: Option<ReserveSnapshot>,
    pub cl_snapshot: Option<CLSnapshot>,
    pub stamp: EventStamp,
}
