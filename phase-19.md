# Phase 19: Gas-Aware Net Profitability Calibration

**Status**: Fallback Reset
**Result**: INSUFFICIENT_EVIDENCE / WEAK

## Objective
Replace the overcomplicated/invalid approach with the fastest truthful method that produces a decision-grade net profitability report.

## Fallback Methodology
The methodology relies strictly on:
1. The canonical Phase 18 artifact (`execution_calibration_report.json`)
2. The small stratified fork-calibration sample (`fixtures/fork_verification_results.json`)
3. Historical base fee percentile estimates (Low=25th, Base=50th, High=75th)

## Key Findings

### Viability Questions & Answers
1. **Is 0.01 ETH standalone execution viable after fees?**
   INCONCLUSIVE. Fork sample yields 0% pass rate. Assumed INSUFFICIENT_EVIDENCE.
2. **Is 0.03 ETH standalone execution viable after fees?**
   INCONCLUSIVE. See above.
3. **Is 0.05 ETH standalone execution viable after fees?**
   INCONCLUSIVE. See above.
4. **What is the break-even minimum size?**
   ~0.0008 ETH (based on High fee + Revert cost).
5. **What is the safe production minimum size?**
   ~0.01 ETH (Assuming a solved pass-rate).
6. **Is the standalone method promising, marginal, or weak?**
   WEAK. The 100% revert rate in available evidence cannot justify a Promising result.
7. **Is future batching research still justified?**
   YES. Required to offset high L1 tracking costs and improve aggregate pass rate.

### Expected Value Formula
`Expected Net = pass_rate × (avg_gross_profit_per_trade − success_fee) − (1 − pass_rate) × revert_cost`

Because the tested fork pass rate was 0 (0 successes across 4 sample evaluations), the expected net profit evaluates negatively to the baseline revert cost.

## Conclusion
The prior positive profitability metric was mathematically invalid as it erroneously allowed net expected profit to exceed the actual bounded maximum gross profit derived in Phase 18. This fallback confirms that without valid execution success proofs, the standalone method remains inconclusive/weak.
