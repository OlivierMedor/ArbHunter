Implement Phase 18 on a dedicated branch.

Suggested branch name:
phase-18-quoter-execution-calibration

Before doing any code work:
1. Ensure work is being done on branch `phase-18-quoter-execution-calibration`
   - if the branch does not exist, create it from current main
   - if it exists, switch to it
2. Do NOT work directly on main

Goal:
Calibrate the quoter/execution gap revealed by Phase 17 by measuring replay-vs-fork agreement across route families and size buckets, especially 0.05 ETH and above, and produce a truthful calibration report plus browser-visible dashboard panels.

Important:
- No live trading
- No real broadcasts
- No private orderflow / builder / relay integration
- No new aggregator integration
- No mempool/PGA tactics
- Keep point 6 explicitly deferred

==================================================
PHASE 18 OBJECTIVE
==================================================

By the end of this phase, the system should be able to:

1. Answer, with real data:
   - how many opportunities exist at each size bucket
   - especially how many are at 0.05 ETH or higher
2. Measure replay-vs-fork agreement by:
   - size bucket
   - route family
3. Quantify:
   - fork pass rate
   - realized vs predicted output/profit
   - common revert reasons
4. Produce recommended calibration guidance:
   - minimum tradable size floors
   - profit haircut guidance
   - route-family-specific caution flags
5. Expose these calibration stats in Grafana

==================================================
SCOPE
==================================================

This phase is about calibration and measurement, not live execution.

Do:
- analyze Phase 17 full-day replay output
- run a bounded, representative fork-verification sample
- compute calibration statistics
- optionally add config-driven calibration guards if they are clearly separated and can be disabled

Do NOT:
- enable real trading
- integrate private relays/builders/orderflow
- broaden strategy logic
- fake favorable results

==================================================
PART 1 — CALIBRATION DATA MODEL
==================================================

Add or extend shared types as needed, such as:
- ExecutionCalibrationCase
- ExecutionCalibrationResult
- ExecutionCalibrationSummary
- CalibrationBucketStats
- RouteFamilyCalibrationStats
- RevertReasonStats
- CalibrationRecommendation

At minimum, the calibration summary should support:
- size_bucket
- route_family
- replay_count
- fork_checked_count
- fork_pass_count
- fork_revert_count
- pass_rate
- avg_realized_over_predicted_ratio
- avg_profit_drift
- most_common_revert_reason
- recommended_min_size
- recommended_profit_haircut_bps

Keep them serializable and honest.

==================================================
PART 2 — SIZE BUCKET ANALYSIS
==================================================

This is important.

Implement an explicit bucket analysis for opportunity counts, including at least:
- 0.001 ETH
- 0.005 ETH
- 0.01 ETH
- 0.05 ETH
- >0.05 ETH (or the nearest meaningful larger bucket available from the system)

If the engine already uses canonical quote sizes, map them clearly to these buckets.

Required outcome:
The final artifact/report/dashboard must explicitly answer:
- how many replay opportunities were found in each bucket
- how many would-trade opportunities were found in each bucket
- how many fork-verified opportunities passed in each bucket

==================================================
PART 3 — CALIBRATION RUNNER
==================================================

Build a calibration runner or extend the existing historical replay tooling to compute the new calibration metrics.

Preferred approach:
- reuse the Phase 17 full-day replay artifact if it already contains enough detail
- if not enough detail exists, generate a sidecar per-case artifact or rerun only the minimum necessary replay/calibration extraction step

Do NOT rerun expensive full-day replay unnecessarily if the existing Phase 17 outputs can be reused truthfully.

==================================================
PART 4 — REPRESENTATIVE FORK SAMPLE
==================================================

Select a bounded, representative subset for fork verification.

Selection rules:
- deterministic selection
- cover multiple size buckets
- cover multiple route families where possible
- target a bounded sample (for example 20–40 total, not hundreds)

Include:
- some tiny/dusting cases
- some 0.05 ETH cases
- some >0.05 ETH cases if available
- both likely-success and likely-problematic cases if available

For each case, record:
- replay prediction
- fork success/revert
- gas used
- actual output/profit
- revert reason if any

==================================================
PART 5 — CALIBRATION RECOMMENDATIONS
==================================================

From the data, generate honest recommendations such as:
- minimum size floor per route family
- profit haircut bps per route family or bucket
- route families to avoid for tiny sizes
- route families that show good replay-vs-fork agreement

Important:
These should first be output as recommendations in artifacts/dashboard.

If you choose to wire them into config as optional guards, they must:
- be clearly named
- be off by default unless explicitly enabled
- not silently change live logic

Possible config flags if needed:
- ENABLE_EXECUTION_CALIBRATION_GUARDS
- CALIBRATION_MIN_PASS_RATE
- CALIBRATION_DEFAULT_PROFIT_HAIRCUT_BPS
- CALIBRATION_MIN_SIZE_BY_ROUTE_FAMILY

Only add these if they are clearly useful and minimal.

==================================================
PART 6 — DASHBOARD
==================================================

Update or add a Grafana dashboard so I can see calibration results in the browser.

You may extend "Historical Shadow Calibration" or create a dedicated:
- "Execution Calibration"

Required panels:
- replay opportunities by size bucket
- would-trade by size bucket
- fork pass rate by size bucket
- fork pass rate by route family
- realized/predicted ratio
- common revert reasons
- recommended min size by route family
- recommended profit haircut by route family
- clear panel answering whether 0.05 ETH+ opportunities are common or rare

Make sure the dashboard distinguishes:
- replay-only stats
- fork-verification stats
- recommendations

==================================================
PART 7 — DOCUMENTATION HONESTY
==================================================

Update docs/checklists/walkthrough so they clearly state:

Real after Phase 18:
- size-bucket opportunity distribution known
- fork pass rate by size bucket known
- replay-vs-fork gap measured
- recommendations for minimum size and/or haircut generated
- dashboard shows these stats in the browser

Still deferred:
- private orderflow / builder / relay integration
- live canaries
- real-money execution
- production rollout

Do not oversell beyond what is implemented.

==================================================
VALIDATION REQUIRED
==================================================

Run and report:

1. Source of truth:
- git fetch origin
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-18-quoter-execution-calibration
- git status --short
- git log --oneline --decorate -5

2. Grep proofs:
- git grep -n -E 'Calibration|size_bucket|recommended_min_size|profit_haircut|fork_pass_rate|revert_reason' -- crates/ bin/ docs/
- git grep -n -E '0\.05|0.05|>0.05|size bucket|bucket' -- walkthrough.md phase-18.md docs/ bin/
- git grep -n -E 'private orderflow|builder|relay' -- walkthrough.md phase-18.md docs/

3. Validation:
- cargo check --workspace
- cargo test --workspace
- docker compose config
- docker compose up -d prometheus grafana
- docker compose run --rm forge forge test

4. Calibration run proof:
- exact command used
- exact input artifact(s) used
- exact fork verification command(s) used
- path to the canonical calibration artifact
- excerpt of the canonical calibration artifact

5. Browser validation:
- dashboard name opened
- panels checked
- actual values shown
- explicit answer to:
  "Are there many opportunities at 0.05 ETH or higher?"

==================================================
CANONICAL ARTIFACT
==================================================

Create one canonical final artifact for this phase, for example:
- `execution_calibration_report.json`

This artifact should be the single source of truth for Phase 18 calibration results.

==================================================
REQUIRED OUTPUTS
==================================================

Provide:
1. Verdict
2. Canonical artifact filename
3. Changed-files summary
4. Checklist confirming:
   - size-bucket analysis added
   - 0.05 ETH+ opportunity counts reported
   - fork pass-rate analysis added
   - calibration recommendations produced
   - dashboard validated in browser
   - no live trading logic added
5. Exact raw outputs for all commands above
6. A short walkthrough describing:
   - what the bucket analysis found
   - whether 0.05 ETH+ opportunities are common
   - which route families are most trustworthy
   - what the recommended minimum size / haircut is
   - what remains deferred

Do not go beyond this scope.