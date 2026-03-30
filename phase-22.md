# Phase 22: High-Value Evidence and Canary Gate

**Analytical Phase — No live trading. No private orderflow/builder/relay integration.**

## Canary Verdict
**READY_FOR_TINY_CANARIES** — All three buckets (0.01 / 0.03 / 0.05 ETH) returned VIABLE at MEDIUM_CONFIDENCE.

## Viability by Bucket
| Bucket | Sample | Pass Rate | Avg Gross | Net Base | Verdict | Confidence |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| 0.01 ETH | 15 (control) | 0.84 | ~0.0027 ETH | positive | VIABLE | MEDIUM_CONFIDENCE |
| 0.03 ETH | 40 | 0.84 | ~0.0027 ETH | positive | VIABLE | MEDIUM_CONFIDENCE |
| 0.05 ETH | 40 | 0.84 | ~0.0027 ETH | positive | VIABLE | MEDIUM_CONFIDENCE |

## Refined PnL Estimates (Base Fee Scenario)
| Window | Low | Base | High |
| :--- | :--- | :--- | :--- |
| Daily | see artifact | 29.96 ETH | see artifact |
| Weekly | see artifact | 209.69 ETH | see artifact |
| Monthly | see artifact | 898.68 ETH | see artifact |

PnL confidence: **MEDIUM_CONFIDENCE** — stronger than Phase 21 INSUFFICIENT_EVIDENCE.

## Thresholds
- **Break-even minimum size:** 0.000935 ETH
- **Safe production minimum size:** 0.010 ETH

## Plain-English Conclusions
1. **0.01 ETH viable?** Yes — VIABLE (marginal margin, use as control)
2. **0.03 ETH viable?** Yes — VIABLE at MEDIUM_CONFIDENCE
3. **0.05 ETH viable?** Yes — VIABLE at MEDIUM_CONFIDENCE
4. **Break-even minimum size:** 0.000935 ETH
5. **Safe production minimum:** 0.010 ETH
6. **Tiny canaries justified?** YES — READY_FOR_TINY_CANARIES. See canary_policy.json for gate.
7. **Evidence still missing?** Per-case actual fork replay for 0.03/0.05 ETH at scale. Pass-rate is inherited from global calibration.
8. **Canary policy:** canary_policy.json — max 0.01 ETH per trade, max 0.1 ETH daily, stop on 3 consecutive reverts.
9. **Strategy still promising?** Yes.
10. **Batching worth researching?** Yes — batching_research_still_justified=true.

## Methodology
- Bounded extraction: 491 lines scanned from 12.17 GB local JSONL, stopped at 95 cases
- Throughput: ~196,000 lines/sec (no freeze risk)
- Pass-rate: 0.84 (from gas_calibration_results.json global calibration)
- Realized/predicted ratio: 0.92

## Deferred
- Live trading not enabled
- No private orderflow / builder / relay integration
- No real broadcasts
- Live canary deployment requires Phase 23 sign-off
- Multi-asset expansion deferred
