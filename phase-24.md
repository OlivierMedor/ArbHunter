# Phase 24: Live-Canary Hardening — Implementation Report

This report summarizes the completion of Phase 24, which focused on hardening the `ArbHunter` live-trading lane with robust, receipt-based attribution and crash-resilient transaction persistence.

## Executive Summary
Phase 24 has successfully transformed the live-trading lane from a "best-effort" simulation-based accounting system into a "safe-by-default," receipt-governed engine. The system now enforces strict on-chain validation for every live trade, persists pending state to prevent loss during crashes, and automatically halts on any ambiguous outcome or incomplete attribution.

**Current Posture:** Live-capable but **DEFAULT-OFF**. Activation requires explicit operator configuration and satisfies multiple safety gates.

## Key Accomplishments

### 1. Durable Pending Transaction Persistence
- **Pre-Broadcast Recording**: Transactions are now persisted to `canary_state.json` *before* the broadcast attempt. This ensures that even a crash during the `send_raw_transaction` RPC call does not lose track of a potentially flying transaction.
- **Expanded Metadata**: Each pending record includes the transaction hash, signer address, nonce, candidate metadata, and a precise status (e.g., `Submitted`, `SendFailedUnconfirmed`).
- **Persistence First**: The `CanaryGate` now owns the persistence logic, ensuring `in_flight_count` is reliably recovered on restart.

### 2. Startup Reconciliation & Recovery
- **Automatic Multi-Stage Resolution**: On startup, the daemon now scans for any pending transactions in `canary_state.json`.
- **Multi-Stage Resolution**: Attempts to resolve via `eth_getTransactionReceipt` -> `eth_getTransactionByHash` -> `sender_nonce`.
- **Halt on Ambiguity**: If a transaction is missing but the nonce hasn't passed, the system **HALTS** the live lane and requires manual operator review, preventing the "double-spend" or "overlapping nonce" risk.

### 3. Strict Receipt-Based Attribution
- **ExecutionSuccess Event**: The `ArbExecutor` contract now emits an `ExecutionSuccess` event. The daemon parses this event directly from the transaction receipt logs.
- **Source of Truth**: Realized PnL is now calculated using the actual `amountOut` and `amountIn` reported by the contract, minus the actual `effective_gas_price` and `gas_used` from the receipt.
- **Halt on Incomplete Attribution**: If a transaction succeeds on-chain but the `ExecutionSuccess` event is missing or unparseable, the system marks the situation as `INCOMPLETE_ATTRIBUTION` and **HALTS**, preventing optimistic but unverified PnL updates.

### 5. Robust Receipt Polling & Timeouts
- **Configurable Polling**: Implemented a `tokio::time::sleep` polling loop in `Submitter::wait_for_receipt` with configurable `receipt_poll_interval_ms` and `receipt_timeout_ms`.
- **Timeout Safety**: Introduced `SubmissionResult::Timeout`. If a timeout is reached, the transaction is kept pending in the `CanaryGate`, ensuring the live lane remains blocked until resolution.
- **Detailed Logging**: Added structured logs for polling start, each retry, success, and timeout.

### 6. Outcome Classification & Revert Protection
- **CanaryOutcomeReason**: Introduced an explicit classification enum (`ConfirmedSuccess`, `ConfirmedRevert`, `DroppedOrReplaced`, `TimeoutStillPending`, `IncompleteAttribution`).
- **Revert Counter Protection**: Safety counters (`consecutive_reverts`) are **only** incremented on `ConfirmedRevert`. Ambiguous states like `DroppedOrReplaced` or `Timeout` are recorded for history but do not trigger false-positive halts.
- **Halt on Ambiguity**: `IncompleteAttribution` triggers a mandatory gate halt to prevent unverified PnL updates.

### 7. Safety Gates & Activation
- **Fail-Fast Startup**: The daemon now panics on startup if `CANARY_LIVE_MODE_ENABLED=true` but `DRY_RUN_ONLY=true`, preventing ambiguous "half-live" states.
- **Policy Enforcement**: Preserved all Phase 23 limits (0.03 ETH trade size, 1 concurrent trade, 3-revert halt, 0.039 ETH loss cap).

## Files Changed

### `crates/arb_types`
- [lib.rs](file:///c:/Users/olivi/Documents/ArbHunger/crates/arb_types/src/lib.rs): Renamed `logs` to `receipt_logs` in `SubmissionResult::Success`. Added `Reverted` variant for granular failure tracking.

### `crates/arb_execute`
- [builder.rs](file:///c:/Users/olivi/Documents/ArbHunger/crates/arb_execute/src/builder.rs): Added `ExecutionSuccess` event to `sol!` macro. Consolidated definitions.
- [submitter.rs](file:///c:/Users/olivi/Documents/ArbHunger/crates/arb_execute/src/submitter.rs): Refactored for two-stage sign/broadcast. Implemented `apply_preflight_and_overrides` and `get_transaction` (tx-by-hash) for strict reconciliation.
- [lib.rs](file:///c:/Users/olivi/Documents/ArbHunger/crates/arb_execute/src/lib.rs): Exported new event types and helpers.

### `crates/arb_canary`
- [lib.rs](file:///c:/Users/olivi/Documents/ArbHunger/crates/arb_canary/src/lib.rs): Overhauled `CanaryState`. Implemented `CanaryOutcomeReason` for strict safety counter protection. Updated crate-docs to "live-capable, default-off".

### `bin/arb_daemon`
- [main.rs](file:///c:/Users/olivi/Documents/ArbHunger/bin/arb_daemon/src/main.rs): Implemented multi-stage reconciliation with `DroppedOrReplaced` accounting. Refactored submission loop to handle polling timeouts.

## Verification Results
- **Compilation**: `cargo check --workspace` passes (confirmed after fixing `Alloy` 0.8 type mismatches).
- **Persistence Test**: Verified that `canary_state.json` correctly stores pending transactions and recovers them on restart.
- **Reconciliation Logic**: Unit tests confirm that nonce-exceeded cases resolve to "dropped" while unreached nonces trigger a halt.
- **Event Parsing**: Verified that `ExecutionSuccess::decode_log` correctly extracts profit data from mock receipt logs.

## Next Steps / Remaining Work
- **Contract Deployment**: The `ArbExecutor` contract must be redeployed with the new `ExecutionSuccess` event to enable full attribution.
- **Manual Verification**: Perform a single-trade manual smoke test on Base (mainnet-fork first) before full live activation.

## Final Status
**The repo is LIVE-CAPABLE but DEFAULT-OFF.**
It is **Operator-Ready** for the first canary deployment on Base.