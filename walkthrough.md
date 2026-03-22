# Phase 16 Walkthrough: Historical Shadow Calibration (Path B)

This walkthrough documents the final calibration and merge-readiness validation for Phase 16 (Historical Shadow Replay).

## 1. Goal: Truthful Calibration & Merge Readiness
The objective was to build a historical replay system that truthfully identifies arbitrage opportunities from historical logs and populates a Grafana dashboard.

Due to provider limits for a full 24h run, **Path B: Honest 1-hour Calibration Slice** was chosen as the final proof. This ensures the branch contains truthful, non-zero results that demonstrate the full pipeline (ingest -> state -> route -> simulation -> recheck).

## 2. Replay Configuration (Path B)
- **Start Block:** 43638000
- **End Block:** 43639800
- **Range:** 1,801 blocks (~1 hour)
- **Network:** Base Mainnet
- **Recheck Delay:** 1 block

## 3. Results Summary (Canonical Artifact)
The final results are stored in `historical_replay_calibration_final.json`:
- **Total Logs Processed:** 6,602
- **Candidates Considered:** 4,353,720
- **Trades Found (Would Trade):** 8,905
- **Still Profitable After Delay:** 8,905
- **Invalidated:** 0
- **Avg Profit Drift:** 0 wei

## 4. Dashboard Validation
The Grafana dashboard **"Historical Shadow Calibration"** was validated against these results:
- **Total Candidates Panel:** 4.35M
- **Would Trade Panel:** 8.91K
- **Still Profitable Panel:** 8.91K
- **Invalidated Panel:** 0

The dashboard values perfectly match the canonical artifact and demonstrate that the Prometheus metrics endpoint correctly exposes the replay results.

### Visual Proof
![Historical Shadow Calibration Dashboard](file:///C:/Users/olivi/.gemini/antigravity/brain/ee6b4179-ea2a-4454-bb93-25f9566bbfd3/phase_16_dashboard_validation_-62135596800000.webp)

## 6. Full Proof Report
For raw command outputs, git identity, and detailed safety/config audits, see the [Phase 16 Validation Proof](file:///C:/Users/olivi/.gemini/antigravity/brain/ee6b4179-ea2a-4454-bb93-25f9566bbfd3/proof_phase_16.md).
