//! `arb_canary` — Phase 24 runtime canary policy enforcement.
//!
//! This crate owns the policy gate that decides whether a candidate is allowed
//! to proceed under the Phase 24 posture. It is the single source of truth for:
//!
//! - Route-family allow/block list (multi allowed, direct blocked)
//! - Max trade size (0.03 ETH)
//! - Max concurrent trades (1)
//! - Revert-streak stop (3 consecutive reverts)
//! - Cumulative realized loss cap (0.039 ETH, inert in sim mode)
//! - Review-threshold warning (30 attempts)
//!
//! **Default posture: live-capable but default-off.**
//! The repo is configured for Base mainnet execution, but broadcast is disabled
//! by default via `CANARY_LIVE_MODE_ENABLED=false` and `DRY_RUN_ONLY=true`.
//! In Phase 24, all live paths are hardened with preflight checks, durable
//! pending-tx persistence, and robust receipt polling.
//!
//! # Design
//!
//! ```text
//!  ┌─────────────────────────────────────────────────┐
//!  │ Orchestration layer (arb_daemon)                │
//!  │                                                  │
//!  │  candidate → CanaryGate::check() → Decision     │
//!  │                ↓ on outcome                      │
//!  │  CanaryGate::record_outcome()                    │
//!  └─────────────────────────────────────────────────┘
//! ```

use alloy_primitives::U256;
use arb_types::{CandidateOpportunity, RouteFamily};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Policy ────────────────────────────────────────────────────────────────

/// Static canary policy loaded at startup.
///
/// Defaults reflect Phase 22 GO decision:
/// - `multi` allowed, `direct` blocked
/// - max trade 0.03 ETH
/// - stop on 3 consecutive reverts
/// - cumulative realized loss cap 0.039 ETH (future live use)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryPolicy {
    /// Route families that may proceed. Checked against `CandidateOpportunity::route_family`.
    pub route_family_allowlist: Vec<RouteFamily>,

    /// Route families that are explicitly blocked (wins over allowlist if both match).
    pub route_family_blocklist: Vec<RouteFamily>,

    /// Maximum `amount_in` allowed per trade in Wei.
    /// Default: 30_000_000_000_000_000 (0.03 ETH).
    pub max_trade_size_wei: u128,

    /// Maximum number of simultaneously tracked (in-flight) canary trades.
    /// Default: 1.
    pub max_concurrent_trades: u32,

    /// Number of consecutive reverts before the canary gate halts all new trades.
    /// Default: 3.
    pub max_consecutive_reverts: u32,

    /// Number of attempts after which a review-threshold warning is emitted.
    /// The gate does NOT halt automatically — it logs a structured warning for human review.
    /// Default: 30.
    pub review_threshold_attempts: u32,

    /// Cumulative realized loss cap in Wei.
    /// **Currently inert when `live_mode_enabled = false`** (sim/shadow mode).
    /// In future live mode, exceeding this causes the gate to block all trades.
    /// Default: 39_000_000_000_000_000 (0.039 ETH).
    pub loss_cap_wei: u128,

    /// When false (default), loss cap enforcement is skipped (sim/shadow-safe).
    /// Set true only after explicit live-gate sign-off in a future phase.
    pub live_mode_enabled: bool,
}

impl Default for CanaryPolicy {
    fn default() -> Self {
        Self {
            route_family_allowlist:    vec![RouteFamily::Multi],
            route_family_blocklist:    vec![RouteFamily::Direct],
            max_trade_size_wei:        30_000_000_000_000_000, // 0.03 ETH
            max_concurrent_trades:     1,
            max_consecutive_reverts:   3,
            review_threshold_attempts: 30,
            loss_cap_wei:              39_000_000_000_000_000, // 0.039 ETH
            live_mode_enabled:         false,
        }
    }
}

// ─── Rejection reasons ─────────────────────────────────────────────────────

/// Why a candidate was rejected by the canary gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanaryRejectionReason {
    /// Route family is on the blocklist.
    RouteFamilyBlocked { family: String },
    /// Route family is not on the allowlist.
    RouteFamilyNotAllowed { family: String },
    /// `amount_in` exceeds `max_trade_size_wei`.
    TradeSizeExceeded { amount_in_wei: u128, limit_wei: u128 },
    /// Too many concurrent canary trades already in flight.
    ConcurrentLimitReached { current: u32, limit: u32 },
    /// Revert streak has reached the stop threshold.
    RevertStreakHalted { streak: u32, limit: u32 },
    /// Cumulative realized loss has reached the cap (live mode only).
    LossCapBreached { loss_wei: i128, cap_wei: u128 },
}

// ─── Decision ──────────────────────────────────────────────────────────────

/// The canary gate's verdict for a single candidate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanaryDecision {
    /// Candidate may proceed.
    Allow,
    /// Candidate is blocked. Contains the first reason found (policy checked in order).
    Reject(CanaryRejectionReason),
}

impl CanaryDecision {
    pub fn is_allowed(&self) -> bool {
        matches!(self, CanaryDecision::Allow)
    }
}

// ─── Runtime state ─────────────────────────────────────────────────────────

/// Mutable runtime state for the canary gate. Update this after every outcome.
///
/// In sim/shadow mode, `cumulative_realized_loss_wei` tracks what *would* have
/// been lost had trades been live — useful for telemetry and future live readiness checks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CanaryState {
    /// Total attempts evaluated (passes + rejects from gate, pre-execution).
    pub attempt_count: u32,

    /// Attempts that were allowed through the gate (proceeded to sim/live execution).
    pub allowed_count: u32,

    /// Current number of in-flight (concurrent) canary trades.
    pub in_flight_count: u32,

    /// Consecutive revert count. Reset to 0 on any success.
    pub consecutive_reverts: u32,

    /// All-time revert count.
    pub total_reverts: u32,

    /// Cumulative realized PnL in Wei (positive = profit, negative = loss).
    /// For sim mode this is modeled/predicted; for live mode it is the actual on-chain delta.
    pub cumulative_realized_pnl_wei: i128,

    /// Cumulative realized loss only (sum of negative outcomes). Always ≥ 0.
    pub cumulative_realized_loss_wei: i128,

    /// Attempts broken down by route family label.
    pub attempts_by_family: HashMap<String, u32>,

    /// Attempts broken down by size bucket label (e.g. "0.01 ETH", "0.03 ETH").
    pub attempts_by_bucket: HashMap<String, u32>,

    /// Whether the review-threshold warning has been emitted at least once.
    pub review_threshold_reached: bool,

    /// Whether the gate is currently halted (revert streak or loss cap).
    pub halted: bool,

    /// Explicit human-readable reason for the halt (e.g. "Ambiguous pending tx").
    pub halted_reason: Option<String>,

    /// Map of transaction hash (hex string) to expanded pending transaction details.
    pub pending_live_txs: HashMap<String, PendingLiveTx>,

    /// Unix timestamp of the last state update.
    pub last_updated_at: u64,
}

/// Metadata for a live transaction that hasn't yet reached a final resolved state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingLiveTx {
    pub tx_hash: String,
    pub signer: String,
    pub nonce: u64,
    pub candidate: CandidateOpportunity,
    pub status: arb_types::PendingTxStatus,
    pub timestamp: u64,
    /// Store the raw bytes for debugging/rebroadcast if opted-in
    pub signed_raw: Option<Vec<u8>>,
}

impl CanaryState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Classify an amount_in Wei value into a human-readable bucket label.
    pub fn bucket_label(amount_in_wei: u128) -> String {
        // Thresholds: < 0.015 ETH = 0.01 bucket, < 0.04 ETH = 0.03 bucket, else 0.039+
        if amount_in_wei < 15_000_000_000_000_000 {
            "0.01 ETH".to_string()
        } else if amount_in_wei < 40_000_000_000_000_000 {
            "0.03 ETH".to_string()
        } else {
            "0.039 ETH".to_string()
        }
    }
}

// ─── Outcome recording ────────────────────────────────────────────────────

/// Classification of the economic and execution outcome of a trade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanaryOutcomeReason {
    /// Transaction succeeded on-chain with confirmed realized profit.
    ConfirmedSuccess,
    /// Transaction reverted on-chain. Realized loss (gas). Increments revert counters.
    ConfirmedRevert,
    /// Transaction was dropped from mempool or replaced (nonce exceeded).
    DroppedOrReplaced,
    /// Receipt wait timed out. Transaction likely still in mempool.
    TimeoutStillPending,
    /// Success receipt found but log parsing for attribution failed. Halts gate.
    IncompleteAttribution,
}

/// The result of a single canary trade attempt (post-execution).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryOutcome {
    /// Whether the on-chain (or simulated) execution succeeded.
    pub success: bool,
    /// Detailed classification of the result.
    pub reason: CanaryOutcomeReason,
    /// Realized net PnL in Wei for this attempt (negative = loss).
    pub realized_pnl_wei: i128,
    /// Non-recoverable costs paid (gas burned on revert, etc.).
    pub cost_paid_wei: u128,
    /// Route family of this trade.
    pub route_family: RouteFamily,
    /// amount_in for bucket tracking.
    pub amount_in_wei: u128,
}

// ─── Gate ─────────────────────────────────────────────────────────────────

/// The policy enforcement gate.
///
/// Owns the policy (static) and state (mutable). Thread-safety is left to the caller;
/// wrap in `Arc<Mutex<CanaryGate>>` for shared async use.
pub struct CanaryGate {
    pub policy: CanaryPolicy,
    pub state:  CanaryState,
    /// Path to the persistence file. If None, persistence is disabled.
    pub persistence_path: Option<std::path::PathBuf>,
}

impl CanaryGate {
    pub fn new(policy: CanaryPolicy) -> Self {
        Self { policy, state: CanaryState::new(), persistence_path: None }
    }

    pub fn with_persistence(policy: CanaryPolicy, path: &std::path::Path) -> Self {
        let mut gate = Self::new(policy);
        gate.persistence_path = Some(path.to_path_buf());
        gate
    }

    pub fn with_defaults() -> Self {
        Self::new(CanaryPolicy::default())
    }

    /// Evaluate whether `candidate` is allowed to proceed.
    ///
    /// Checks (in order):
    /// 1. Gate halted (revert streak or loss cap)
    /// 2. Route family blocklist
    /// 3. Route family allowlist
    /// 4. Trade size
    /// 5. Concurrent limit
    ///
    /// Does **not** mutate state — call `record_allowed()` after dispatching.
    pub fn check(&mut self, candidate: &CandidateOpportunity) -> CanaryDecision {
        self.state.attempt_count += 1;

        // Track per-family and per-bucket attempts
        {
            let fam = candidate.route_family.as_str().to_string();
            *self.state.attempts_by_family.entry(fam).or_insert(0) += 1;
        }
        {
            let bucket = CanaryState::bucket_label(
                candidate.amount_in.try_into().unwrap_or(u128::MAX)
            );
            *self.state.attempts_by_bucket.entry(bucket).or_insert(0) += 1;
        }

        // Review-threshold warning (non-halting)
        if !self.state.review_threshold_reached
            && self.state.attempt_count >= self.policy.review_threshold_attempts
        {
            self.state.review_threshold_reached = true;
            tracing::warn!(
                attempt_count = self.state.attempt_count,
                "CANARY_REVIEW_THRESHOLD: {} attempts reached. Human review recommended.",
                self.policy.review_threshold_attempts
            );
        }

        // Gate halted?
        if self.state.halted {
            return CanaryDecision::Reject(CanaryRejectionReason::RevertStreakHalted {
                streak: self.state.consecutive_reverts,
                limit:  self.policy.max_consecutive_reverts,
            });
        }

        let family = &candidate.route_family;

        // 2. Blocklist (hard block wins)
        if self.policy.route_family_blocklist.contains(family) {
            return CanaryDecision::Reject(CanaryRejectionReason::RouteFamilyBlocked {
                family: family.as_str().to_string(),
            });
        }

        // 3. Allowlist
        if !self.policy.route_family_allowlist.contains(family) {
            return CanaryDecision::Reject(CanaryRejectionReason::RouteFamilyNotAllowed {
                family: family.as_str().to_string(),
            });
        }

        // 4. Trade size
        let amount_in: u128 = candidate.amount_in.try_into().unwrap_or(u128::MAX);
        if amount_in > self.policy.max_trade_size_wei {
            return CanaryDecision::Reject(CanaryRejectionReason::TradeSizeExceeded {
                amount_in_wei: amount_in,
                limit_wei:     self.policy.max_trade_size_wei,
            });
        }

        // 5. Concurrent limit
        if self.state.in_flight_count >= self.policy.max_concurrent_trades {
            return CanaryDecision::Reject(CanaryRejectionReason::ConcurrentLimitReached {
                current: self.state.in_flight_count,
                limit:   self.policy.max_concurrent_trades,
            });
        }

        // 6. Loss cap (live mode only — inert in sim)
        if self.policy.live_mode_enabled
            && self.state.cumulative_realized_loss_wei >= self.policy.loss_cap_wei as i128
        {
            self.state.halted = true;
            return CanaryDecision::Reject(CanaryRejectionReason::LossCapBreached {
                loss_wei: self.state.cumulative_realized_loss_wei,
                cap_wei:  self.policy.loss_cap_wei,
            });
        }

        self.state.allowed_count += 1;
        self.state.in_flight_count += 1;
        CanaryDecision::Allow
    }

    /// Record the outcome of a trade that was previously `Allow`-ed.
    ///
    /// - Updates revert streak, loss accumulator, and in-flight count.
    /// - Halts the gate if the revert streak threshold is reached.
    /// - Persists state if `persistence_path` is set.
    pub fn record_outcome(&mut self, outcome: CanaryOutcome) {
        // In-flight count decremented regardless of success/failure (except for Timeouts handled externally)
        self.state.in_flight_count = self.state.in_flight_count.saturating_sub(1);

        self.state.cumulative_realized_pnl_wei += outcome.realized_pnl_wei;

        if outcome.realized_pnl_wei < 0 {
            let loss = outcome.realized_pnl_wei.unsigned_abs() as i128;
            self.state.cumulative_realized_loss_wei += loss;
        }

        match outcome.reason {
            CanaryOutcomeReason::ConfirmedSuccess => {
                self.state.consecutive_reverts = 0;
                tracing::info!(
                    realized_pnl_wei = outcome.realized_pnl_wei,
                    cumulative_pnl   = self.state.cumulative_realized_pnl_wei,
                    "CANARY_SUCCESS: Confirmed on-chain."
                );
            }
            CanaryOutcomeReason::ConfirmedRevert => {
                self.state.total_reverts += 1;
                self.state.consecutive_reverts += 1;

                tracing::warn!(
                    consecutive_reverts = self.state.consecutive_reverts,
                    realized_pnl_wei    = outcome.realized_pnl_wei,
                    "CANARY_REVERT: Transaction reverted on-chain. Incrementing safety counters."
                );

                if self.state.consecutive_reverts >= self.policy.max_consecutive_reverts {
                    self.state.halted = true;
                    tracing::error!(
                        consecutive_reverts = self.state.consecutive_reverts,
                        "CANARY_HALTED: Consecutive revert threshold reached."
                    );
                }
            }
            CanaryOutcomeReason::IncompleteAttribution => {
                tracing::error!(
                    tx_hash = "N/A", // hash not in outcome yet
                    "CANARY_HALTED: Success receipt found but attribution logs missing or unparseable."
                );
                self.halt("Incomplete attribution: logs missing or unparseable".to_string());
            }
            CanaryOutcomeReason::DroppedOrReplaced => {
                tracing::warn!(
                    "CANARY_DROPPED: Transaction dropped or replaced (nonce exceeded). Counters protected."
                );
            }
            CanaryOutcomeReason::TimeoutStillPending => {
                tracing::info!("CANARY_TIMEOUT: Receipt wait timed out. Lane remains blocked.");
            }
        }

        // Loss cap check after recording (sim: warning only; live: already blocked at check())
        if self.state.cumulative_realized_loss_wei >= self.policy.loss_cap_wei as i128 {
            if self.policy.live_mode_enabled {
                self.state.halted = true;
                tracing::error!(
                    loss_wei = self.state.cumulative_realized_loss_wei,
                    cap_wei  = self.policy.loss_cap_wei,
                    "CANARY_HALTED: cumulative loss cap breached."
                );
            } else {
                tracing::warn!(
                    loss_wei = self.state.cumulative_realized_loss_wei,
                    cap_wei  = self.policy.loss_cap_wei,
                    "CANARY_SIM_LOSS_CAP_WARN: would have breached loss cap in live mode."
                );
            }
        }

        self.persist_state();
    }

    /// Mark a transaction as pending broadcast. Persists state.
    pub fn record_pending_tx(&mut self, pending: PendingLiveTx) {
        self.state.pending_live_txs.insert(pending.tx_hash.clone(), pending);
        self.persist_state();
    }

    /// Update the status of an existing pending transaction.
    pub fn update_pending_status(&mut self, tx_hash: &str, status: arb_types::PendingTxStatus) {
        if let Some(pending) = self.state.pending_live_txs.get_mut(tx_hash) {
            pending.status = status;
            self.persist_state();
        }
    }

    /// Resolve a pending transaction after receipt or failure. Persists state.
    pub fn resolve_pending_tx(&mut self, tx_hash: &str) -> Option<PendingLiveTx> {
        let pending = self.state.pending_live_txs.remove(tx_hash);
        self.persist_state();
        pending
    }

    /// Explicitly halt the gate with a reason.
    pub fn halt(&mut self, reason: String) {
        self.state.halted = true;
        self.state.halted_reason = Some(reason);
        self.persist_state();
    }

    /// Persist the current state to the configured path.
    pub fn persist_state(&mut self) {
        if let Some(path) = &self.persistence_path {
            self.state.last_updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            if let Err(e) = self.save_to_file(path) {
                tracing::error!(path = ?path, error = %e, "CANARY_PERSIST_FAILED");
            }
        }
    }

    /// Internal helper for atomic save.
    fn save_to_file(&self, path: &std::path::Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self.state)?;
        let tmp_path = path.with_extension("tmp");
        std::fs::write(&tmp_path, json)?;
        std::fs::rename(tmp_path, path)?;
        Ok(())
    }

    /// Load state from a file.
    pub fn load_state(&mut self, path: &std::path::Path) -> std::io::Result<()> {
        if path.exists() {
            let json = std::fs::read_to_string(path)?;
            self.state = serde_json::from_str(&json)?;
            tracing::info!(path = ?path, "CANARY_STATE_LOADED");
        }
        Ok(())
    }

    /// Manually reset the gate (e.g. after human review following a halt).
    ///
    /// Resets: `halted`, `consecutive_reverts`, `in_flight_count`.
    /// Does NOT reset cumulative PnL or attempt counts — those are permanent telemetry.
    pub fn reset_halt(&mut self) {
        self.state.halted = false;
        self.state.consecutive_reverts = 0;
        self.state.in_flight_count = 0;
        tracing::info!("CANARY_GATE_RESET: halt cleared by manual reset.");
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::U256;
    use arb_types::{CandidateOpportunity, QuoteSizeBucket, RouteFamily, RoutePath, TokenAddress};

    fn make_candidate(route_family: RouteFamily, amount_in_eth_e18: u128) -> CandidateOpportunity {
        CandidateOpportunity {
            path: RoutePath {
                legs: vec![],
                root_asset: TokenAddress("0xWETH".to_string()),
            },
            bucket: QuoteSizeBucket::Custom(amount_in_eth_e18),
            amount_in: U256::from(amount_in_eth_e18),
            estimated_amount_out: U256::from(amount_in_eth_e18 + 1_000_000_000_000_000),
            estimated_gross_profit: U256::from(1_000_000_000_000_000u128),
            estimated_gross_bps: 33,
            is_fresh: true,
            route_family,
        }
    }

    // ── Route family tests ──────────────────────────────────────────────

    #[test]
    fn test_multi_is_allowed() {
        let mut gate = CanaryGate::with_defaults();
        let c = make_candidate(RouteFamily::Multi, 30_000_000_000_000_000);
        assert_eq!(gate.check(&c), CanaryDecision::Allow);
    }

    #[test]
    fn test_direct_is_blocked() {
        let mut gate = CanaryGate::with_defaults();
        let c = make_candidate(RouteFamily::Direct, 30_000_000_000_000_000);
        assert!(matches!(
            gate.check(&c),
            CanaryDecision::Reject(CanaryRejectionReason::RouteFamilyBlocked { .. })
        ));
    }

    #[test]
    fn test_unknown_is_not_allowed() {
        let mut gate = CanaryGate::with_defaults();
        let c = make_candidate(RouteFamily::Unknown, 30_000_000_000_000_000);
        // Not on blocklist, but also not on allowlist → RouteFamilyNotAllowed
        assert!(matches!(
            gate.check(&c),
            CanaryDecision::Reject(CanaryRejectionReason::RouteFamilyNotAllowed { .. })
        ));
    }

    // ── Trade size tests ───────────────────────────────────────────────

    #[test]
    fn test_trade_size_at_limit_is_allowed() {
        let mut gate = CanaryGate::with_defaults();
        // Exactly 0.03 ETH — should pass
        let c = make_candidate(RouteFamily::Multi, 30_000_000_000_000_000);
        assert_eq!(gate.check(&c), CanaryDecision::Allow);
    }

    #[test]
    fn test_trade_size_one_wei_over_is_rejected() {
        let mut gate = CanaryGate::with_defaults();
        let c = make_candidate(RouteFamily::Multi, 30_000_000_000_000_001);
        assert!(matches!(
            gate.check(&c),
            CanaryDecision::Reject(CanaryRejectionReason::TradeSizeExceeded { .. })
        ));
    }

    #[test]
    fn test_small_trade_is_allowed() {
        let mut gate = CanaryGate::with_defaults();
        let c = make_candidate(RouteFamily::Multi, 10_000_000_000_000_000); // 0.01 ETH
        assert_eq!(gate.check(&c), CanaryDecision::Allow);
    }

    // ── Concurrent trades ──────────────────────────────────────────────

    #[test]
    fn test_max_concurrent_one() {
        let mut gate = CanaryGate::with_defaults();
        let c1 = make_candidate(RouteFamily::Multi, 30_000_000_000_000_000);
        let c2 = make_candidate(RouteFamily::Multi, 30_000_000_000_000_000);

        assert_eq!(gate.check(&c1), CanaryDecision::Allow); // in-flight=1
        assert!(matches!(
            gate.check(&c2),
            CanaryDecision::Reject(CanaryRejectionReason::ConcurrentLimitReached { .. })
        ));

        // After recording success, gate opens again
            gate.record_outcome(CanaryOutcome {
                success: true,
                reason: CanaryOutcomeReason::ConfirmedSuccess,
                realized_pnl_wei: 1_000_000_000_000_000,
                cost_paid_wei: 185_000 * 5_000_000,
                route_family: RouteFamily::Multi,
                amount_in_wei: 30_000_000_000_000_000,
            });
        assert_eq!(gate.check(&c2), CanaryDecision::Allow);
    }

    // ── Revert streak tests ───────────────────────────────────────────

    #[test]
    fn test_revert_streak_halts_on_third() {
        let mut gate = CanaryGate::with_defaults();

        for _ in 0..3 {
            let c = make_candidate(RouteFamily::Multi, 30_000_000_000_000_000);
            assert_eq!(gate.check(&c), CanaryDecision::Allow);
            gate.record_outcome(CanaryOutcome {
                success: false,
                reason: CanaryOutcomeReason::ConfirmedRevert,
                realized_pnl_wei: -925_000_000_000i128,
                cost_paid_wei: 925_000_000_000,
                route_family: RouteFamily::Multi,
                amount_in_wei: 30_000_000_000_000_000,
            });
        }

        // Fourth attempt — gate should be halted
        let c = make_candidate(RouteFamily::Multi, 30_000_000_000_000_000);
        assert!(matches!(
            gate.check(&c),
            CanaryDecision::Reject(CanaryRejectionReason::RevertStreakHalted { .. })
        ));
        assert!(gate.state.halted);
    }

    #[test]
    fn test_success_resets_revert_streak() {
        let mut gate = CanaryGate::with_defaults();

        // Two reverts
        for _ in 0..2 {
            let c = make_candidate(RouteFamily::Multi, 30_000_000_000_000_000);
            gate.check(&c);
            gate.record_outcome(CanaryOutcome {
                success: false,
                reason: CanaryOutcomeReason::ConfirmedRevert,
                realized_pnl_wei: -1_000,
                cost_paid_wei: 1_000,
                route_family: RouteFamily::Multi,
                amount_in_wei: 30_000_000_000_000_000,
            });
        }
        assert_eq!(gate.state.consecutive_reverts, 2);

        // One success resets streak
        let c = make_candidate(RouteFamily::Multi, 30_000_000_000_000_000);
        gate.check(&c);
        gate.record_outcome(CanaryOutcome {
            success: true,
            reason: CanaryOutcomeReason::ConfirmedSuccess,
            realized_pnl_wei: 1_000_000,
            cost_paid_wei: 100_000,
            route_family: RouteFamily::Multi,
            amount_in_wei: 30_000_000_000_000_000,
        });
        assert_eq!(gate.state.consecutive_reverts, 0);
        assert!(!gate.state.halted);
    }

    // ── Cumulative loss cap ───────────────────────────────────────────

    #[test]
    fn test_loss_cap_inert_in_sim_mode() {
        let mut gate = CanaryGate::with_defaults();
        assert!(!gate.policy.live_mode_enabled); // default is sim mode

        // Record a giant loss that exceeds the cap
        let c = make_candidate(RouteFamily::Multi, 30_000_000_000_000_000);
        assert_eq!(gate.check(&c), CanaryDecision::Allow);
        gate.record_outcome(CanaryOutcome {
            success: false,
            reason: CanaryOutcomeReason::ConfirmedRevert,
            realized_pnl_wei: -60_000_000_000_000_000i128,
            cost_paid_wei: 60_000_000_000_000_000,
            route_family: RouteFamily::Multi,
            amount_in_wei: 30_000_000_000_000_000,
        });

        // In sim mode: not halted (just a warning)
        assert!(!gate.state.halted);
        // But loss is tracked
        assert!(gate.state.cumulative_realized_loss_wei >= 60_000_000_000_000_000);
    }

    #[test]
    fn test_loss_cap_enforced_in_live_mode() {
        let mut policy = CanaryPolicy::default();
        policy.live_mode_enabled = true;
        let mut gate = CanaryGate::new(policy);

        // Record a loss that exceeds the cap
        let c = make_candidate(RouteFamily::Multi, 30_000_000_000_000_000);
        assert_eq!(gate.check(&c), CanaryDecision::Allow);
        gate.record_outcome(CanaryOutcome {
            success: false,
            reason: CanaryOutcomeReason::ConfirmedRevert,
            realized_pnl_wei: -60_000_000_000_000_000i128,
            cost_paid_wei: 60_000_000_000_000_000,
            route_family: RouteFamily::Multi,
            amount_in_wei: 30_000_000_000_000_000,
        });

        // Gate should be halted
        assert!(gate.state.halted);
        let c2 = make_candidate(RouteFamily::Multi, 30_000_000_000_000_000);
        assert!(matches!(
            gate.check(&c2),
            CanaryDecision::Reject(CanaryRejectionReason::RevertStreakHalted { .. })
                | CanaryDecision::Reject(CanaryRejectionReason::LossCapBreached { .. })
        ));
    }

    // ── Review threshold ──────────────────────────────────────────────

    #[test]
    fn test_review_threshold_sets_flag() {
        let mut gate = CanaryGate::with_defaults();
        assert!(!gate.state.review_threshold_reached);

        // Run 29 allow+success cycles — threshold not yet reached
        for _ in 0..29 {
            let c = make_candidate(RouteFamily::Multi, 30_000_000_000_000_000);
            gate.check(&c);
            gate.record_outcome(CanaryOutcome {
                success: true,
                reason: CanaryOutcomeReason::ConfirmedSuccess,
                realized_pnl_wei: 100,
                cost_paid_wei: 50,
                route_family: RouteFamily::Multi,
                amount_in_wei: 30_000_000_000_000_000,
            });
        }
        assert!(!gate.state.review_threshold_reached);

        // 30th attempt triggers the flag
        let c = make_candidate(RouteFamily::Multi, 30_000_000_000_000_000);
        gate.check(&c);
        assert!(gate.state.review_threshold_reached);

        // Gate still allows — review is non-halting
        assert!(!gate.state.halted);
    }

    // ── Reset halt ────────────────────────────────────────────────────

    #[test]
    fn test_reset_halt_clears_streak() {
        let mut gate = CanaryGate::with_defaults();

        for _ in 0..3 {
            let c = make_candidate(RouteFamily::Multi, 30_000_000_000_000_000);
            gate.check(&c);
            gate.record_outcome(CanaryOutcome {
                success: false,
                reason: CanaryOutcomeReason::ConfirmedRevert,
                realized_pnl_wei: -1_000,
                cost_paid_wei: 1_000,
                route_family: RouteFamily::Multi,
                amount_in_wei: 30_000_000_000_000_000,
            });
        }
        assert!(gate.state.halted);

        gate.reset_halt();
        assert!(!gate.state.halted);
        assert_eq!(gate.state.consecutive_reverts, 0);

        // Gate accepts new candidates again
        let c = make_candidate(RouteFamily::Multi, 30_000_000_000_000_000);
        assert_eq!(gate.check(&c), CanaryDecision::Allow);
    }

    // ── Attempt tracking ──────────────────────────────────────────────

    #[test]
    fn test_attempt_tracking_by_family_and_bucket() {
        let mut gate = CanaryGate::with_defaults();
        let c1 = make_candidate(RouteFamily::Multi,  30_000_000_000_000_000); // 0.03 ETH
        let c2 = make_candidate(RouteFamily::Direct, 10_000_000_000_000_000); // 0.01 ETH
        gate.check(&c1);
        gate.check(&c2);

        assert_eq!(gate.state.attempts_by_family.get("multi"), Some(&1));
        assert_eq!(gate.state.attempts_by_family.get("direct"), Some(&1));
        assert_eq!(gate.state.attempts_by_bucket.get("0.03 ETH"), Some(&1));
        assert_eq!(gate.state.attempts_by_bucket.get("0.01 ETH"), Some(&1));
        assert_eq!(gate.state.attempt_count, 2);
    }

    #[test]
    fn test_persistence_atomic_save_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("canary_state.json");
        let policy = CanaryPolicy::default();
        
        {
            let mut gate = CanaryGate::with_persistence(policy.clone(), &path);
            gate.state.attempt_count = 42;
            gate.state.cumulative_realized_pnl_wei = 1000;
            gate.persist_state();
            assert!(path.exists());
        }

        {
            let mut gate = CanaryGate::new(policy);
            gate.load_state(&path).unwrap();
            assert_eq!(gate.state.attempt_count, 42);
            assert_eq!(gate.state.cumulative_realized_pnl_wei, 1000);
            assert!(gate.state.last_updated_at > 0);
        }
    }

    #[test]
    fn test_pending_tx_persistence() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("canary_state.json");
        let policy = CanaryPolicy::default();
        let c = make_candidate(RouteFamily::Multi, 1000);
        let tx_hash = "0x123".to_string();
        let pending = PendingLiveTx {
            tx_hash: tx_hash.clone(),
            signer: "0xSender".to_string(),
            nonce: 1,
            candidate: c,
            status: arb_types::PendingTxStatus::Submitted,
            timestamp: 123456789,
            signed_raw: None,
        };

        {
            let mut gate = CanaryGate::with_persistence(policy.clone(), &path);
            gate.record_pending_tx(pending);
        }

        {
            let mut gate = CanaryGate::new(policy.clone());
            gate.load_state(&path).unwrap();
            assert!(gate.state.pending_live_txs.contains_key(&tx_hash));
            
            let mut gate_with_path = CanaryGate::with_persistence(policy.clone(), &path);
            gate_with_path.load_state(&path).unwrap();
            let resolved = gate_with_path.resolve_pending_tx(&tx_hash);
            assert!(resolved.is_some());
            assert!(!gate_with_path.state.pending_live_txs.contains_key(&tx_hash));
            
            // Verify removal is persisted
            let mut gate_after = CanaryGate::new(policy);
            gate_after.load_state(&path).unwrap();
            assert!(!gate_after.state.pending_live_txs.contains_key(&tx_hash));
        }
    }
}
