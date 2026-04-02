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
    pub block_number: u64,
    pub log_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IngestEvent {
    Flashblock(FlashblockEvent),
    PendingLog(PendingLogEvent),
}

// ============================================================
// Phase 3: State Engine Types
// ============================================================

use std::collections::HashMap;

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

/// Concentrated liquidity snapshot (Uniswap V3-style, top-level only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CLSnapshot {
    pub sqrt_price_x96: U256,
    pub liquidity: U128,
    pub tick: i32,
}

/// Per-tick state for concentrated liquidity pools
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CLTickState {
    pub liquidity_gross: u128,
    pub liquidity_net: i128,
}

/// Full depth model for a concentrated liquidity pool
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CLFullState {
    pub sqrt_price_x96: U256,
    pub liquidity: U128,
    pub tick: i32,
    pub ticks: HashMap<i32, CLTickState>,
}

/// Canonical pool state snapshot stored in the engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStateSnapshot {
    pub pool_id: PoolId,
    pub kind: PoolKind,
    pub token0: Option<TokenAddress>,
    pub token1: Option<TokenAddress>,
    pub fee_bps: u32,
    pub reserves: Option<ReserveSnapshot>,
    pub cl_snapshot: Option<CLSnapshot>,
    pub cl_full_state: Option<CLFullState>,
    pub freshness: PoolFreshness,
}

/// A single incoming state update derived from an ingest event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolUpdate {
    pub pool_id: PoolId,
    pub kind: PoolKind,
    pub token0: Option<TokenAddress>,
    pub token1: Option<TokenAddress>,
    pub fee_bps: Option<u32>,
    pub reserves: Option<ReserveSnapshot>,
    pub cl_snapshot: Option<CLSnapshot>,
    pub cl_full_state: Option<CLFullState>,
    pub stamp: EventStamp,
}
// ============================================================
// Phase 6: Route Graph & Filter Types
// ============================================================

/// Metadata for a directed edge in the route graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub pool_id: PoolId,
    pub kind: PoolKind,
    pub token_in: TokenAddress,
    pub token_out: TokenAddress,
    pub fee_bps: u32,
    pub is_stale: bool,
}

/// A single hop in a cyclic or linear route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteLeg {
    pub edge: GraphEdge,
}

/// A sequence of legs forming a path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutePath {
    pub legs: Vec<RouteLeg>,
    pub root_asset: TokenAddress,
}

/// Predefined notional sizes for local quoting (e.g. 0.1 ETH, 1 ETH, 10 ETH).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuoteSizeBucket {
    Small,
    Medium,
    Large,
    Custom(u128),
}

/// Typed route-family classification.
///
/// - `Direct`: single-hop or simple two-leg route through a single venue pair.
/// - `Multi`:  multi-hop route spanning multiple venue pairs or pool types.
/// - `Unknown`: not yet classified; treated conservatively (blocked by canary policy).
///
/// Phase 22 canary policy: `Multi` is allowed, `Direct` is blocked pending more evidence.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default, Copy)]
#[serde(rename_all = "lowercase")]
pub enum RouteFamily {
    Direct,
    Multi,
    #[default]
    Unknown,
}

impl RouteFamily {
    /// Returns the canonical string label used in telemetry / policy JSON.
    pub fn as_str(&self) -> &'static str {
        match self {
            RouteFamily::Direct  => "direct",
            RouteFamily::Multi   => "multi",
            RouteFamily::Unknown => "unknown",
        }
    }

    /// Parse from a string label (case-insensitive).
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "direct" => RouteFamily::Direct,
            "multi"  => RouteFamily::Multi,
            _        => RouteFamily::Unknown,
        }
    }

    /// Number of legs in a path that qualifies as Multi vs Direct.
    /// A path with >2 legs is always Multi; exactly 2 legs (round-trip) may be Direct.
    pub fn classify_by_leg_count(leg_count: usize) -> Self {
        match leg_count {
            0 | 1 => RouteFamily::Unknown,
            2     => RouteFamily::Direct,
            _     => RouteFamily::Multi,
        }
    }
}

impl std::fmt::Display for RouteFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for RouteFamily {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "direct" => Ok(RouteFamily::Direct),
            "multi" => Ok(RouteFamily::Multi),
            "unknown" => Ok(RouteFamily::Unknown),
            // Legacy mapping for battery generator
            "concentratedliquidity_cyclic" | "mixed_cyclic" => Ok(RouteFamily::Direct),
            _ => Ok(RouteFamily::Unknown),
        }
    }
}

/// A promoted candidate for refinement or execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateOpportunity {
    pub path: RoutePath,
    pub bucket: QuoteSizeBucket,
    pub amount_in: U256,
    pub estimated_amount_out: U256,
    pub estimated_gross_profit: U256,
    pub estimated_gross_bps: u32,
    pub is_fresh: bool,
    /// Typed route-family classification. Defaults to `Unknown` for backward compat.
    #[serde(default)]
    pub route_family: RouteFamily,
}

// ============================================================
// Phase 7: Pending-State Simulation Types
// ============================================================

/// Represents a distinct reason why a candidate simulation failed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SimulationFailureReason {
    RouteNotFound,
    InsufficientLiquidity,
    SlippageExceeded,
    StaleState,
    QuoteFailed,
}

/// The status of a candidate simulation attempt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SimOutcomeStatus {
    Success,
    Failed(SimulationFailureReason),
    Skipped,
}

/// Represents a request to simulate a promoted candidate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationRequest {
    pub candidate: CandidateOpportunity,
}

/// The structured result of performing a simulation/dry-run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub request: SimulationRequest,
    pub status: SimOutcomeStatus,
    pub expected_amount_out: Option<U256>,
    pub expected_profit: Option<U256>,
    pub expected_gas_used: Option<u64>,
    pub leg_amounts_out: Vec<U256>,
}

/// The high-level validation result to be logged or passed to execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateValidationResult {
    pub sim_result: SimulationResult,
    pub is_valid: bool,
}


// ============================================================
// Phase 8: Execution Plan Types
// ============================================================

/// Specifies how to validate minimum output to protect against slippage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinOutConstraint {
    pub min_amount_out: U256,
}

/// Generic slippage guard container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlippageGuard {
    pub min_out: MinOutConstraint,
    pub min_profit_wei: U256,
}

// FlashLoanSpec moved to Phase 11 section

/// Reasons why a candidate could not be converted into an execution plan
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlanBuildFailureReason {
    UnsupportedPoolKind,
    UnsupportedRouteStructure,
    InsufficientProfit,
}

/// A single execution step across a pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLeg {
    pub pool_id: PoolId,
    pub pool_kind: PoolKind,
    pub token_in: TokenAddress,
    pub token_out: TokenAddress,
    pub zero_for_one: bool, // Helps with generic route encoding
    pub amount_out: U256,
}

/// A deterministic, sequential path to execute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPath {
    pub legs: Vec<ExecutionLeg>,
}

/// Expected state transition numbers for the arbitrage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedOutcome {
    pub amount_in: U256,
    pub expected_amount_out: U256,
    pub expected_profit: U256,
}

/// The deterministic plan representing the arbitrage transaction actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub target_token: TokenAddress,
    pub path: ExecutionPath,
    pub outcome: ExpectedOutcome,
    pub guard: SlippageGuard,
    pub flash_loan: Option<FlashLoanSpec>,
}

// ============================================================
// Phase 9: Wallet, Signing, and Submission Types
// ============================================================

/// Represents a built transaction request ready for signing or simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltTransaction {
    pub to: String,
    pub data: Vec<u8>,
    pub value: U256,
    pub nonce: u64,
    pub gas_limit: u64,
    pub max_fee_per_gas: u128,
    pub max_priority_fee_per_gas: u128,
    pub gas_price: Option<u128>,
    pub chain_id: u64,
}

/// Reasons why a transaction submission might fail
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SubmissionFailureReason {
    SignerMissing,
    NonceMismatch,
    ReplacementUnderpriced,
    InsufficientFunds,
    ExecutionReverted(String),
    NetworkError(String),
    DroppedFromMempool,
    PreflightFailed(String),
    UnknownOverride,
}

/// The state of a submission attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubmissionResult {
    /// Transaction broadcast successfully
    Success {
        tx_hash: String,
        gas_used: u128,
        effective_gas_price: u128,
        l1_fee_wei: Option<u128>,
    },
    /// Dry-run successful (no broadcast)
    DryRunSuccess { tx_hash: String, signed_raw: Vec<u8> },
    /// Submission failed with a specific reason
    Failed(SubmissionFailureReason),
    /// Submission skipped (e.g. gas too high)
    Skipped(String),
}

/// Simple model for tracking the current nonce state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonceState {
    pub address: String,
    pub next_nonce: u64,
    pub pending_count: u32,
}

/// Policy for fee selection (EIP-1559)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeePolicy {
    pub max_fee_per_gas: u128,
    pub max_priority_fee_per_gas: u128,
    pub base_fee_multiplier: f64,
}

/// Configuration for the signing wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub address: String,
    pub chain_id: u64,
}

/// Operational mode for the submission pipeline
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubmissionMode {
    Broadcast,
    DryRun,
    SimulateOnly,
}

// ============================================================
// Phase 10: Preflight & Broadcast Types
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PreflightStatus {
    Passed,
    Failed(String),
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightResult {
    pub overall_success: bool,
    pub eth_call_status: PreflightStatus,
    pub gas_estimate_status: PreflightStatus,
    pub tenderly_status: PreflightStatus,
    pub gas_estimate: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastResult {
    pub success: bool,
    pub tx_hash: Option<String>,
    pub error: Option<String>,
}

// ============================================================
// Phase 11: Atomic & Flash Loan Execution
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FlashLoanProviderKind {
    Mock,
    BalancerV2,
    UniswapV2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashLoanSpec {
    pub provider: FlashLoanProviderKind,
    pub asset: String,
    pub amount: alloy_primitives::U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepaymentGuard {
    pub asset: String,
    pub amount: alloy_primitives::U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfitGuard {
    pub min_profit_wei: alloy_primitives::U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicExecutionPlan {
    pub flash_loan: Option<FlashLoanSpec>,
    pub legs: Vec<ExecutionLeg>,
    pub min_amount_out: alloy_primitives::U256,
    pub repayment: Option<RepaymentGuard>,
    pub profit_guard: ProfitGuard,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AtomicExecutionFailureReason {
    InsufficientRepayment,
    SlippageExceeded,
    NoProfit,
    FlashLoanFailed(String),
    ContractReverted(String),
}

// ============================================================
// Phase 13: Historical Replay Battery & Attribution
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardOverrides {
    pub min_profit_wei: Option<U256>,
    pub min_amount_out: Option<U256>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalCase {
    pub case_id: String,
    pub notes: String,
    pub fork_block_number: u64,
    pub source_tx_hash: Option<String>,
    pub root_asset: TokenAddress,
    pub route_family: RouteFamily,
    pub pool_ids: Vec<String>,
    pub pool_kinds: Vec<PoolKind>,
    pub path_tokens: Vec<TokenAddress>,
    pub leg_directions: Vec<bool>, // zero_for_one mapping
    pub amount_in: U256,
    pub expected_outcome: String, // "success", "slippage_revert", "no_profit_revert"
    pub guard_overrides: Option<GuardOverrides>,
    pub seed_data: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionResult {
    pub case_id: String,
    pub actual_status: String,
    pub predicted_amount_out: U256,
    pub predicted_profit: U256,
    pub actual_amount_out: Option<U256>,
    pub actual_profit: Option<U256>,
    pub gas_used: u64,
    pub success_or_revert: bool,
    pub revert_reason: Option<String>,
    pub absolute_error: U256,
    pub relative_error: f64,
}

// ============================================================
// Phase 15: Live Shadow Mode
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftSummary {
    pub profit_drift_wei: i128,
    pub amount_out_drift_wei: i128,
    pub is_still_profitable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowRecheckResult {
    pub timestamp_ms: u64,
    pub rechecked_amount_out: U256,
    pub rechecked_profit: U256,
    pub drift_summary: DriftSummary,
    pub invalidated_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowJournalEntry {
    pub timestamp_ms: u64,
    pub candidate_id: String,
    pub route_family: RouteFamily,
    pub root_asset: TokenAddress,
    pub amount_in: U256,
    pub predicted_amount_out: U256,
    pub predicted_profit: U256,
    pub predicted_gas: Option<u64>,
    pub would_trade: bool,
    pub reason: String,
    // Populated during the delayed recheck block
    pub recheck: Option<ShadowRecheckResult>,
}

// ============================================================
// Phase 16: Historical Shadow Calibration
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalDriftSummary {
    pub profit_drift_wei: i128,
    pub amount_out_drift_wei: i128,
    pub is_still_profitable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalRecheckResult {
    pub block_number: u64,
    pub rechecked_amount_out: U256,
    pub rechecked_profit: U256,
    pub drift_summary: HistoricalDriftSummary,
    pub invalidated_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalReplayResult {
    pub case_id: String,
    pub block_number: u64,
    pub route_family: RouteFamily,
    pub root_asset: TokenAddress,
    pub amount_in: U256,
    pub predicted_amount_out: U256,
    pub predicted_profit: U256,
    pub bucket: String,
    pub would_trade: bool,
    #[serde(alias = "route")]
    pub path: RoutePath,
    pub recheck: Option<HistoricalRecheckResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalReplaySummary {
    pub start_block: u64,
    pub end_block: u64,
    pub total_blocks: u64,
    pub total_logs: u64,
    pub candidates_considered: u64,
    pub promoted_candidates: u64,
    pub would_trade_candidates: u64,
    pub still_profitable_count: u64,
    pub invalidated_count: u64,
    pub avg_profit_drift_wei: i128,
    pub fork_verifications: Vec<ForkVerificationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkVerificationResult {
    pub case_id: String,
    pub success: bool,
    pub realized_profit: Option<U256>,
    pub gas_used: u64,
    pub revert_reason: Option<String>,
}

