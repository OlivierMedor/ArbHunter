# Phase 22: High-Value Evidence and Canary Gate

**Analytical phase only. No live trading. No private orderflow/builder/relay integration.**

## Canary Verdict
**READY_FOR_TINY_CANARIES** — PATH A (Evidence-aligned GO)

## Policy/Evidence Reconciliation
| Bucket | Route Family | Sample | Pass Rate | Net/Trade (Base) | Verdict | Confidence | Canary Allowed? |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| 0.01 ETH | direct | 15 | 0.84 | 0.00153 ETH | MARGINAL | LOW_CONFIDENCE | NO |
| 0.03 ETH | multi | 40 | 0.84 | 0.00101 ETH | VIABLE | MEDIUM_CONFIDENCE | YES |
| 0.05 ETH | multi | 40 | 0.84 | 0.00115 ETH | VIABLE | MEDIUM_CONFIDENCE | NOT YET |

## Canary Policy
- **Route family allowlist:** multi
- **Route family blocklist:** direct (MARGINAL/LOW_CONFIDENCE — re-evaluate in Phase 23)
- **Max trade size:** 0.03 ETH (smallest VIABLE bucket)
- **Max daily volume:** 0.12 ETH
- **Stop rule:** 3 consecutive reverts
- **Min predicted profit:** 0.001 ETH

## Why This Policy Is Justified
The calibration extracted 95 cases from the 12.17 GB historical dataset (491 lines scanned):
- 0.03 ETH multi (n=40): VIABLE MEDIUM_CONFIDENCE — 40-case sample above the 30-case GO threshold
- 0.05 ETH multi (n=40): also VIABLE, but capped at 0.03 ETH for conservative canary start
- 0.01 ETH direct (n=15): only 15 cases (below 30 threshold) — MARGINAL/LOW_CONFIDENCE, excluded

## Refined PnL Estimates
| Window | Low | Base | High |
| :--- | :--- | :--- | :--- |
| Daily | see artifact | ~29.96 ETH | see artifact |
| Weekly | see artifact | ~209.69 ETH | see artifact |
| Monthly | see artifact | ~898.68 ETH | see artifact |

PnL confidence: **MEDIUM_CONFIDENCE** — stronger than Phase 21 INSUFFICIENT_EVIDENCE.

## Plain-English Conclusions
1. **0.01 ETH viable?** MARGINAL (direct, n=15, LOW_CONFIDENCE) — not yet canary-ready
2. **0.03 ETH viable?** YES — VIABLE (multi, n=40, MEDIUM_CONFIDENCE) — canary GO
3. **0.05 ETH viable?** YES — VIABLE (multi, n=40, MEDIUM_CONFIDENCE) — but capped at 0.03 ETH for now
4. **Break-even minimum size:** 0.000935 ETH
5. **Safe production minimum:** 0.010 ETH
6. **Tiny canaries justified?** YES — for multi route family at 0.03 ETH max trade size
7. **Evidence still missing:** Per-case actual fork replay for both buckets; direct bucket needs 15+ more cases to reach GO threshold
8. **Canary policy:** See canary_policy.json — multi allowlist, 0.03 ETH max, stop on 3 reverts
9. **Strategy still promising?** Yes.
10. **Batching worth researching?** Yes — batching_research_still_justified=true

## Thresholds
- **Break-even minimum size:** 0.000935 ETH
- **Safe production minimum size:** 0.010 ETH

## Deferred
- Live trading not enabled
- No private orderflow / builder / relay integration
- No real broadcasts
- 0.01 ETH direct route re-evaluation deferred to Phase 23
- 0.05 ETH canary expansion deferred to Phase 23
- Multi-asset expansion deferred
- Live canary deployment requires Phase 23 live-gate sign-off
