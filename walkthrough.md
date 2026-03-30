# Phase 22 Walkthrough: High-Value Evidence and Canary Gate

**Analytical phase only. No live trading. No private orderflow/builder/relay.**

## Policy/Evidence Reconciliation (PATH A)
- 0.01 ETH direct: MARGINAL/LOW_CONFIDENCE (n=15) — excluded from canary
- 0.03 ETH multi: VIABLE/MEDIUM_CONFIDENCE (n=40) — canary GO at 0.03 ETH max
- 0.05 ETH multi: VIABLE/MEDIUM_CONFIDENCE (n=40) — deferred, capped at 0.03 ETH for now

## Canary Policy
- Route family allowlist: multi
- Route family blocklist: direct
- Max trade size: 0.03 ETH
- Max daily volume: 0.12 ETH
- Stop on 3 consecutive reverts

## Key Results
- Canary verdict: READY_FOR_TINY_CANARIES (multi, 0.03 ETH)
- Break-even: 0.000935 ETH
- Safe production min: 0.010 ETH
- Daily PnL base: ~29.96 ETH | Weekly: ~209.69 ETH | Monthly: ~898.68 ETH
- PnL confidence: MEDIUM_CONFIDENCE (vs Phase 21 INSUFFICIENT_EVIDENCE)

## Artifacts
- standalone_canary_go_no_go_report.json — canonical Phase 22 artifact
- high_value_calibration_results.json — supporting calibration (95 cases)
- canary_policy.json — explicit GO gate (multi allowlist, max 0.03 ETH, stop 3 reverts)
- fixtures/phase22_high_value_fixture.json — compact 95-case stratified fixture
