# Phase 24 Walkthrough: Controlled Live-Canary Hardening

## Status

**Verdict:** LIVE-READY (Operator Activated)  
**Posture:** LIVE-CAPABLE / DEFAULT-OFF  
**Live Mode:** Requires explicit operator configuration (`ENABLE_BROADCAST=true`)

This walkthrough describes the hardened, live-trading canary lane implemented in Phase 24. It focus on "safe-by-default" behavior and strict attribution.

## What Phase 24 Changed

Phase 24 transformed the Phase 23 "shadow-only" system into a production-hardened live trading engine with robust fail-safes. 

The main additions are:

### 1. Durable Pending Transaction Persistence
- **Pre-Broadcast Recording**: Transactions are recorded to `canary_state.json` *before* the broadcast attempt.
- **Signed Raw Persistence**: Signed raw transaction bytes are optionally cached, ensuring a crash during `send_raw_transaction` does not lose track of the in-flight hash.
- **Status Tracking**: Adds `SendFailedUnconfirmed` and `awaiting_receipt` states for granular recovery.

### 2. Startup Reconciliation & Recovery
- **Automatic Recovery**: On daemon restart, the system identifies all pending transactions and resolves them against the chain.
- **Multi-Stage Reconciliation**: Uses a hierarchy of `eth_getTransactionReceipt` -> `eth_getTransactionByHash` -> `sender_nonce`.
- **Halt-on-Ambiguity**: If a transaction's final status cannot be determined (e.g. pending but not in mempool, and nonce not yet reached), the live lane **HALTS** to prevent overlapping nonce errors.

### 3. Strict Receipt-Based Attribution
- **ExecutionSuccess Event**: The `ArbExecutor` contract now emits an `ExecutionSuccess` event containing the actual `amountOut` and net profit.
- **On-Chain Source of Truth**: Realized PnL is no longer estimated; it is parsed directly from receipt logs.
- **Mandatory Attribution**: The daemon **HALTS** if a successful receipt is missing the `ExecutionSuccess` event or if logs are unparseable.
- **Real Fee Calculation**: Captures actual `gas_used` and `effective_gas_price` (including Base L1 data fees) for all trade accounting.

### 4. Contract Hardening
- **Callback Security**: `uniswapV3SwapCallback` now strictly validates `msg.sender` against the expected Uniswap V3 Pool context for the active route.

### 5. Fail-Fast Safety Gates
- **Configuration Parity**: The daemon panics at startup if `CANARY_LIVE_MODE_ENABLED=true` while `DRY_RUN_ONLY=true`, preventing ambiguous "half-live" states.
- **Tenderly Enforcement**: Live mode requires valid Tenderly API keys and slugs for preflight simulation.

## Active Safety Policy (Phase 24)

### Route-family policy
- **Allowlist:** `multi`
- **Blocklist:** `direct`, `unknown`

### Trade and execution limits
- **Max trade size:** `0.03 ETH`
- **Max concurrent trades:** `1`
- **Min predicted profit:** `0.001 ETH`
- **Stop on consecutive reverts:** `3`
- **Review threshold:** `30 attempts`
- **Cumulative realized loss cap:** `0.05 ETH`

## Verification Results
- `cargo check --workspace` — PASSED (Alloy 0.8 compatibility verified)
- Durable persistence integration tests — PASSED
- Startup reconciliation logic — VERIFIED
- Safety gate panic tests — PASSED

## Operational Notes
1. **Default State**: The repo is live-capable but **broadcasts are disabled by default**.
2. **Operator Activation**: Transitioning to live mode requires setting `CANARY_LIVE_MODE_ENABLED=true`, `ENABLE_BROADCAST=true`, and `DRY_RUN_ONLY=false`.
3. **Recovery**: If the daemon crashes, simply restart. The reconciliation logic will resolve any orphaned transactions before accepting new ones.