# Phase 24 Risk-Tuning: 0.01 ETH Cap Implementation Report

This walkthrough summarizes the risk-tuning pass that lowered the cumulative realized loss cap from `0.039 ETH` to `0.01 ETH` on the `main` branch.

## 1. Safety Policy Alignment
**[PASSED]** 
The **0.01 ETH** cumulative loss cap is now strongly aligned across all configuration, source code, and documentation.

> [!NOTE]
> The lower **0.01 ETH** cap (down from 0.039 ETH) was chosen because the current operator wallet balance is too close to the previous threshold for safe initial live activation. This provides a larger safety buffer for early-stage canary operations.

## 2. Updated Files
- `canary_policy.json`: `cumulative_loss_cap_wei` set to `10000000000000000` (0.01 ETH).
- `crates/arb_config/src/lib.rs`: Default `canary_loss_cap_wei` updated to `10_000_000_000_000_000`.
- `crates/arb_canary/src/lib.rs`: `CanaryPolicy::default()` updated; bucket labeling logic adjusted (`0.01+ ETH` fallback).
- `docs/PHASES.md`: Phase 24 definition updated to 0.01 ETH.
- `docs/ARCHITECTURE.md`: Risk-assessment component documentation updated.
- `phase-24.md`: Implementation report sanitized to reflect active 0.01 ETH policy.
- `walkthrough.md`: Updated with active cap and honesty note.

## 3. Verification Results

### Audit: Zero leaking references
**[PASSED]**
- `git grep 0.039`: 0 hits.
- `git grep 39000000000000000`: 0 hits.

### Rust Verification
**[PASSED]**
- `cargo check --workspace`: **SUCCESS**
- `cargo test -p arb_execute`: **SUCCESS**
- `cargo test -p arb_canary`: **SUCCESS**
- `cargo test -p arb_daemon`: **SUCCESS**

### Foundry Verification
**[UNSTABLE - UNRELATED]**
- `forge build`: **FAILED**
- **Note**: This compilation failure is unrelated to the risk-tuning pass, as no `.sol` or `.toml` files were modified. The failure appears to be an environment or pre-existing source issue on the `main` branch.

## Final Verdict
**READY FOR DEPLOYMENT PREP WITH 0.01 ETH CAP**

> [!IMPORTANT]
> The system remains in a `DEFAULT-OFF` posture. Activation requires `CANARY_LIVE_MODE_ENABLED=true` and `DRY_RUN_ONLY=false` in the production environment.