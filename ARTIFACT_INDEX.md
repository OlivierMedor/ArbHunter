# Artifact Index

This index tracks the canonical artifacts for the recent calibration and feasibility phases of the Base Arbitrage Engine.

| Phase | Canonical Branch | Canonical Artifact(s) | Supporting Artifact(s) | Status / Interpretation |
| :--- | :--- | :--- | :--- | :--- |
| **Phase 16** | `phase-16-historical-shadow-calibration-dashboard` | `historical_replay_calibration_final.json` | `phase-16.md`, `walkthrough.md` | Honest 1-hour calibration slice (Path B). Full 24h+ replay deferred. |
| **Phase 17** | `phase-17-full-day-calibration-and-fork-checks` | `historical_replay_full_day_final.json` | `fixtures/fork_verification_results.json`, `phase-17.md`, `walkthrough.md` | Full-day replay completed. Fork spot checks found execution realism issues. |
| **Phase 18** | `phase-18-final-calibration` | `execution_calibration_report.json` | `phase-18.md`, `walkthrough.md` | Size-bucket and execution calibration. Showed many 0.01 / 0.03 / 0.05 bucket opportunities and strong clustering. |
| **Phase 19b** | `phase-19b-targeted-gas-calibration` | `net_profitability_report.json` | `gas_calibration_results.json`, `fixtures/phase19b_calibration_fixture_full.json`, `phase-19b.md`, `walkthrough.md` | Targeted fallback gas calibration. Standalone strategy estimated as **MARGINAL** under a global fallback model. |
| **Phase 20** | `phase-20-package-feasibility-and-batch-simulation` | `package_batchability_report.json` | `phase-20.md`, `walkthrough.md`, `phase-20-results.md`, `scripts/analyzer.py` | Same-block package feasibility screen. No viable packages under strict “any pool overlap = conflict” rule. |

## Local-Only / Large Raw Files

The following classes of files are intentionally kept local and are not canonical repo artifacts:

- giant JSONL candidate exports
- giant checkpoint files
- temporary extraction outputs
- temporary debug logs

These local-only files may be used to generate compact canonical artifacts, but they are not the source of truth for repo review.

## Canonical Review Rule

For each phase, treat the “Canonical Artifact(s)” column above as the source of truth.
Supporting artifacts should explain and summarize the canonical artifact, not contradict it.
 - [Phase 20b Analysis (Complete)](file:///c:/Users/olivi/Documents/ArbHunger/phase-20b.md)
