# Phase 23 Walkthrough: Canary Safety & Tenderly Integration

## Status

**Verdict:** SIGN-OFF READY  
**Posture:** Simulation / Shadow-Only  
**Live Mode:** Disabled by default

This walkthrough supersedes the older Phase 22 canary walkthrough language. The old **0.12 ETH daily volume cap** belongs to the prior Phase 22 posture and is now historical only. The active Phase 23 posture is the safety-gated, simulation/shadow-only configuration described below.

## What Phase 23 Changed

Phase 23 turns the Phase 22 canary conclusions into enforceable runtime controls while keeping the system out of live-trading mode.

The main additions are:

- A dedicated runtime safety gate via `arb_canary`
- Typed route-family classification (`multi`, `direct`, `unknown`)
- Tenderly-based preflight simulation support
- Improved telemetry for canary attempts and shadow journaling
- Clearer separation between predicted gross profit and estimated execution cost

## Current Active Safety Policy

### Route-family policy

- **Allowlist:** `multi`
- **Blocklist:** `direct`, `unknown`

### Trade and execution limits

- **Max trade size:** `0.03 ETH`
- **Max concurrent trades:** `1`
- **Min predicted profit:** `0.001 ETH`
- **Stop on consecutive reverts:** `3`
- **Review threshold:** `30 attempts`

### Future live canary loss control

- **Cumulative realized loss cap:** `0.05 ETH`

Important: in **Phase 23**, this loss cap is wired for future live use but the system is still running in **simulation/shadow-only mode**. In that posture, the loss-cap path is present for telemetry and warning behavior, but it is not a live trading switch because live mode is disabled by default.

## Tenderly Integration

Tenderly is now integrated as an **additional preflight safety stage**.

Its purpose in this phase is to improve confidence before a future live send by simulating the transaction more realistically against current state. Tenderly status is part of preflight reporting alongside:

- `eth_call`
- `gas estimate`
- `Tenderly simulation`

This makes preflight failures easier to diagnose and easier to measure.

## Telemetry Improvements

Phase 23 improves telemetry so the project can support better route-family and canary analysis later.

Key telemetry improvements:

- Typed `RouteFamily` metadata instead of ad hoc placeholder strings
- Route-family-aware canary attempt tracking
- Revert streak tracking
- Review-threshold tracking
- Cumulative realized P&L tracking
- Cumulative realized loss tracking
- Shadow journaling with route-family metadata

This is important groundwork for a later adaptive canary or dynamic route/size learning system.

## Economics Model: What It Does and Does Not Mean

Phase 23 improves the **structure** of the economics model, but it does **not** yet make the runtime accounting fully realistic.

### What it does now

- Separates **predicted gross profit** from **estimated execution cost**
- Uses a simplified **5 Gwei L2 gas approximation** for runtime execution-cost tracking

### What it does not yet include fully

- **Base L1 data/security fees** are **not** currently included in the runtime approximation

So Phase 23 should be interpreted as:

- **better safety and better accounting structure**, not
- **fully production-accurate live P&L accounting**

## Verification Results Reported in Phase 23

The branch’s Phase 23 implementation report records the following verification results:

- `cargo check --workspace` — PASSED
- `cargo test -p arb_canary` — PASSED
- `cargo test -p arb_execute` — PASSED

## What Phase 23 Means Operationally

Phase 23 does **not** mean the bot is now live.

It means:

- the canary policy is now represented in code,
- preflight simulation is stronger,
- route-family telemetry is cleaner,
- the system is safer to evaluate,
- and the repo is positioned for a future, explicitly approved live-canary phase.

It does **not** yet mean:

- real trading is enabled,
- real broadcasts are enabled by default,
- economics are fully production-accurate,
- or live profitability has been proven.

## Current Constraints

- Default posture remains **Simulation/Shadow ONLY**
- `canary_live_mode_enabled` remains **false** by default
- Real broadcasts remain **disabled** by default
- No private orderflow / builder / relay integration is part of this phase
- Public mempool / Flashblock ingestion remains the active posture

## Historical Note

The following Phase 22 items are now historical context, not the active Phase 23 policy:

- Tiny-canary daily-volume framing
- `0.12 ETH` daily volume cap
- Phase 22 walkthrough wording

The active source-of-truth posture for this branch is the Phase 23 policy/reporting layer.

## Recommended Next Step

**Phase 24** should be the explicit live-canary decision phase.

That phase should decide whether to activate a tightly controlled live canary using the already-wired safeguards:

- `multi` only
- `0.03 ETH` max trade size
- `1` concurrent trade
- `3` consecutive reverts stop
- `30` attempt review threshold
- `0.05 ETH` cumulative realized loss cap

Before or during that phase, the most valuable remaining improvement is to tighten fee realism further, especially around **Base L1 data/security fees**.