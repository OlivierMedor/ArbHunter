# Phase 19b Report: Targeted Gas Calibration

> **Canonical branch**: `phase-19b-targeted-gas-calibration`
> **Canonical artifact**: `net_profitability_report.json`

---

## 1. Targeted Gas Calibration
Phase 19b was designed to provide a stronger, decision-grade calibration using a bounded 40-case targeted extraction, explicitly skipping the full 11.3 GB file rescan. 

> **Context**: This is a bounded targeted fallback model. Bucket-specific gas/pass-rate calibration is approximated globally from the 40-case sample (85% simulated success). Conclusions should be interpreted as decision-grade but still conservative/approximate. Private orderflow / builder integration remains explicitly deferred. EV calculated strictly preventing Net > Gross profit.

### Net EV Formula
`Expected Net = pass_rate × (avg_gross − success_fee) − (1 − pass_rate) × revert_cost`

### Viability Summary
1. **0.01 ETH**: MARGINAL - Thin expected net margins (~ 0.000025 ETH)
2. **0.03 ETH**: MARGINAL - Thin expected net margins (~ 0.000025 ETH)
3. **0.05 ETH**: MARGINAL - Thin expected net margins (~ 0.000025 ETH)

*(0.04 ETH bucket explicitly flagged as INSUFFICIENT_EVIDENCE due to 0 candidate count).*

### Thresholds & Verdicts
- **Break-even minimum size**: ~ 0.000655 ETH
- **Safe production minimum size**: ~ 0.010000 ETH
- **Standalone Method Verdict**: MARGINAL
- **Batching Research**: STILL JUSTIFIED. Required to amortize the L1 baseline offset fees across multiple dense low-margin route setups.