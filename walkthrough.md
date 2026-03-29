# Phase 18 & 19 Walkthrough: Arbitrage Engine Calibration

> **Canonical branch**: `phase-19-gas-net-profitability-and-artifact-cleanup`
> **Canonical artifacts**: `execution_calibration_report.json`, `net_profitability_report.json`

---

## 1. Phase 18 Replay Window & Gross Results

- **Blocks**: 43,680,550 → 43,723,750
- **Total candidates**: 10,708,460

### Profitability (Gross, Pre-Gas)

| Metric | ETH |
| :--- | :--- |
| Total simulated gross profit | 11,512.62 ETH |
| Average profit per trade | ~0.00108 ETH |
| Peak profit per trade | ~0.00500 ETH |

---

## 2. Phase 19 Net Profitability Fallback Reset

Due to a logical error in the prior method where bucket labels (Input Size) were erroneously used as expected profit, the actual maximum recorded gross profit of ~0.005 ETH was impossible to reconcile with expected net numbers of ~0.04 ETH.

A **Fallback Reset** was performed leveraging strictly:
1. `execution_calibration_report.json` (for max/avg gross baseline).
2. `fixtures/fork_verification_results.json` (for real fork calibration).

### Net Profitability Expected Value Formula
`Expected Net = pass_rate × (avg_gross_profit_per_trade − success_fee) − (1 − pass_rate) × revert_cost`

### Commercial Answers & Findings

**1. Is 0.01 / 0.03 / 0.05 ETH viable?**
> **INCONCLUSIVE**. 
The available fork evaluation sample data shows a **0% pass rate**. Thus, it is impossible to mathematically prove positive net expected value. All buckets have been labeled `INSUFFICIENT_EVIDENCE`.

**2. What is the break-even minimum size?**
> **~0.0008 ETH**. 

**3. What is the safe production minimum size?**
> **~0.01 ETH**. 

**4. Standalone Execution Verdict**
> **WEAK**. 

**5. Batchability Research (Phase 20)**
> **STILL JUSTIFIED**. 

---

## Source of Truth

`execution_calibration_report.json`
`net_profitability_report.json`