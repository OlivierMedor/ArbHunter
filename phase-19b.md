# Phase 19b Report: Targeted Gas Calibration

> Canonical branch: `phase-19b-targeted-gas-calibration`  
> Canonical artifact: `net_profitability_report.json`

---

## 1. Purpose

Phase 19b replaces the weak fallback Phase 19 result with a stronger, decision-grade gas calibration using a bounded 40-case targeted extraction.

This phase was designed to avoid:
- rescanning the 11.3 GB historical candidate dataset
- long-running freeze-prone file operations
- overclaiming precision that the available calibration data does not support

This phase remains:
- analytical only
- non-live
- non-broadcasting

Still deferred:
- live trading
- actual batched execution
- private orderflow / builder / relay integration

---

## 2. Calibration Method

Phase 19b uses a **global fallback calibration model** derived from a bounded 40-case sample.

### Source artifacts
- `execution_calibration_report.json`
- `gas_calibration_results.json`

### Global calibration assumptions
- Fork sample size: **40**
- Pass rate: **85%**
- Average gas on success: **185,000**
- Average gas on revert: **125,000**
- Fee scenarios:
  - Low = 25th percentile
  - Base = 50th percentile
  - High = 75th percentile

### Expected value formula
`Expected Net = pass_rate × (avg_gross − success_fee) − (1 − pass_rate) × revert_cost`

This model is intentionally conservative and is constrained so that:
- expected net never exceeds average gross profit
- zero-candidate buckets are treated as `INSUFFICIENT_EVIDENCE`

---

## 3. Key Results

### Global summary
- **Break-even minimum size:** ~`0.000655 ETH`
- **Safe production minimum size:** ~`0.010000 ETH`
- **Standalone method verdict:** `MARGINAL`
- **Batching research still justified:** `true`

### Size-bucket viability
- **0.01 ETH:** `MARGINAL`
- **0.03 ETH:** `MARGINAL`
- **0.05 ETH:** `MARGINAL`
- **0.04 ETH:** `INSUFFICIENT_EVIDENCE`

### Interpretation
The standalone one-opportunity-at-a-time method appears mathematically positive under the fallback model, but only with **thin margins**. The current expected net per trade is small enough that the method remains sensitive to:
- fee spikes
- execution variance
- revert costs
- market movement between opportunity detection and execution

---

## 4. What This Means

### Standalone strategy
The standalone strategy is **not unworkable**, but it is also **not robustly strong** under this calibration. It should be treated as **marginal**, not production-ready.

### Batchability
Batching research remains justified because:
- earlier phases showed high candidate density
- many opportunities co-occurred within the same block/window
- shared-cost execution may still be the best path to turn thin gross opportunities into stronger net profitability

---

## 5. Confidence and Limitations

This phase does **not** provide a true per-bucket measured gas model.

Instead, it uses:
- one bounded 40-case targeted calibration
- one globally applied fallback gas/pass-rate model
- bucket-level candidate counts from prior calibration artifacts

Because of that, these conclusions should be interpreted as:
- **decision-grade**
- **conservative**
- **approximate**

Not final:
- per-bucket exact gas truth
- live execution truth
- production profitability proof

---

## 6. Deferred Work

Still deferred after Phase 19b:
- stronger per-bucket gas calibration
- actual batched execution
- live canaries
- real-money execution
- private orderflow / builder / relay integration
- production rollout

---

## 7. Final Verdict

Phase 19b provides a truthful targeted gas calibration result.

The best current estimate is:

- **Break-even minimum size:** ~`0.000655 ETH`
- **Safe production minimum:** ~`0.01 ETH`
- **Standalone strategy:** `MARGINAL`
- **Future batching research:** `STILL JUSTIFIED`

The canonical source of truth for this phase is:

- `net_profitability_report.json`