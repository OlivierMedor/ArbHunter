Implement Phase 19 on a dedicated branch.

Suggested branch name:
phase-19-gas-net-profitability-and-artifact-cleanup

Before doing any code work:
1. Ensure work is being done on branch `phase-19-gas-net-profitability-and-artifact-cleanup`
   - if the branch does not exist, create it from current main
   - if it exists, switch to it
2. Do NOT work directly on main
3. Do NOT merge or create redundant Phase 18 branches

Goal:
Turn Phase 18’s gross-profit / size-bucket calibration into a truthful **net profitability** analysis by integrating gas and execution fee estimates, and add a small repo cleanup so canonical artifacts are easy to understand.

Important:
- No live trading
- No real broadcasts
- No private orderflow / builder / relay integration
- No new aggregator integration
- No mempool/PGA tactics
- No actual batched execution implementation in this phase

Private orderflow / builder / relay integration remains explicitly deferred.

==================================================
PHASE 19 OBJECTIVE
==================================================

By the end of this phase, the system should be able to answer, with data:

1. After gas/fees, what is the minimum viable **standalone** trade size?
2. Are 0.01 ETH trades net profitable?
3. Are 0.03 ETH trades net profitable?
4. Are 0.05 ETH trades net profitable?
5. Is the original one-opportunity-at-a-time method commercially viable in the tested window?
6. Does future batched execution still look justified after net fee modeling?

This phase should produce a truthful canonical artifact and dashboard that distinguish:
- gross simulated edge
- gas / fee burden
- estimated net profitability
- route-family / size-bucket viability

==================================================
REPO CLEANUP TASKS (ADD TO PHASE 19)
==================================================

Do a small repo artifact cleanup / organization step as part of this phase.

Required:
1. Create a simple artifact index file, for example:
   - `docs/ARTIFACT_INDEX.md`
   or
   - `ARTIFACT_INDEX.md`

2. In that file, list the canonical artifact for each recent phase:
   - Phase 16: historical_replay_calibration_final.json
   - Phase 17: historical_replay_full_day_final.json + fixtures/fork_verification_results.json
   - Phase 18: execution_calibration_report.json
   - Phase 19: net_profitability_report.json

3. If older non-canonical artifacts remain in the repo, do NOT blindly delete them unless they are clearly stale and unreferenced.
   At minimum, mark in the artifact index which files are canonical vs historical/reference.

4. Remove or correct any stale doc references that point to the wrong canonical artifact.

Do not do a huge repo refactor; keep this cleanup minimal and safe.

==================================================
PART 1 — CANONICAL PHASE 19 ARTIFACT
==================================================

Create one canonical final artifact for this phase:

- `net_profitability_report.json`

This should be the source of truth for Phase 19.

It must include, at minimum:

Global:
- replay_window_start_block
- replay_window_end_block
- total_blocks
- total_logs
- total_candidates
- total_weth_eligible_candidates
- source_artifacts_used

Per bucket:
- bucket_name
- candidate_count
- gross_profit_total
- gross_profit_avg
- estimated_avg_gas_used
- estimated_avg_total_fee_eth
- estimated_avg_net_profit_eth
- net_positive_count
- net_negative_count
- estimated_break_even_size_eth
- recommendation

Per route family:
- route_family
- candidate_count
- avg_gas_used
- avg_total_fee_eth
- avg_net_profit_eth
- estimated_min_viable_size_eth
- recommended_profit_haircut_bps

Top-level conclusions:
- standalone_min_viable_size_eth
- standalone_method_viable_in_test_window (true/false/qualified)
- batching_research_still_justified (true/false)
- notes

If exact fee modeling is not available for some component, label it explicitly as estimated/approximated.

==================================================
PART 2 — INPUTS / REUSE OF EXISTING DATA
==================================================

Prefer reusing existing Phase 18/17 outputs instead of rerunning the expensive 24h replay unless absolutely necessary.

Preferred inputs:
- execution_calibration_report.json
- historical_replay_full_day_final.json
- historical_replay_full_day_candidates.jsonl (local/generated sidecar if available)
- fixtures/fork_verification_results.json

If you need to regenerate a smaller derived sidecar for analysis, that is fine.
Do NOT rerun a full 24h replay unless there is no other truthful option.

==================================================
PART 3 — GAS / FEE MODEL
==================================================

Integrate gas and execution fee modeling honestly.

At minimum, model:
- execution gas used
- L2 execution fee
- any L1 data/security fee if available
- optional priority fee assumption (must be explicit and conservative)

Rules:
- use historical block/chain fee data if practical
- if exact historical fee reconstruction is not feasible, use a clearly labeled conservative estimate
- do NOT present estimated fees as exact if they are modeled

You may use stratified fork samples to estimate gas by:
- size bucket
- route family
- prediction strength tier

==================================================
PART 4 — STRATIFIED GAS / EXECUTION SAMPLE
==================================================

Use a bounded stratified sample to calibrate gas and execution viability.

Target:
- 20–40 cases total

Stratify by:
- size bucket
- route family
- prediction strength

For each selected case, record:
- predicted gross profit
- gas used
- estimated total fee
- estimated net profit
- actual fork success/revert if already available
- revert reason if applicable

Do not expand into a huge battery.
This is calibration, not production verification.

==================================================
PART 5 — MINIMUM SIZE THRESHOLD LOGIC
==================================================

This is the heart of the phase.

Use the data to estimate the minimum standalone trade value threshold.

Provide:
- break-even minimum size by route family
- recommended safe minimum size by route family
- default global minimum size recommendation

Be explicit about the distinction between:
- break-even minimum
- safe production minimum

And answer in plain English:
- Is 0.01 ETH too small for standalone execution?
- Is 0.03 ETH viable?
- Is 0.05 ETH viable?
- What is the best current estimate of the minimum standalone trade size?

==================================================
PART 6 — DASHBOARD
==================================================

Add or update a dashboard so I can view the Phase 19 results in the browser.

Recommended dashboard name:
- `Net Profitability Calibration`
or
- extend `Execution Calibration` clearly if that is cleaner

Required panels:
- gross profit by size bucket
- estimated total fee by size bucket
- estimated net profit by size bucket
- net-positive count by size bucket
- minimum viable size by route family
- route-family gas/fee comparison
- top-level recommendation: standalone viable or not
- top-level recommendation: batching research justified or not

The dashboard must clearly distinguish:
- gross
- estimated fee
- estimated net

==================================================
PART 7 — DOCUMENTATION HONESTY
==================================================

Update:
- `walkthrough.md`
- `phase-19.md`
- artifact index file
- any related summary/checklist docs if you use them

The docs must clearly state:

Real after Phase 19:
- gas/net modeling added
- minimum standalone size estimate produced
- 0.01 / 0.03 / 0.05 ETH viability assessed
- dashboard shows net profitability stats

Still deferred:
- private orderflow / builder / relay integration
- live canaries
- real-money execution
- actual batched execution
- production rollout

Do not oversell beyond what was implemented.

==================================================
PART 8 — PLAIN-ENGLISH CONCLUSIONS REQUIRED
==================================================

The final report must explicitly answer:

1. What is the estimated minimum standalone profitable trade size?
2. Are there enough trades above that size in a typical day to matter?
3. Is the original standalone strategy commercially promising, marginal, or weak in this tested window?
4. Does the data still justify future research into batched execution of small compatible opportunities?

==================================================
VALIDATION REQUIRED
==================================================

Run and report:

1. Source of truth:
- git fetch origin
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-19-gas-net-profitability-and-artifact-cleanup
- git status --short
- git log --oneline --decorate -5

2. Grep proofs:
- git grep -n -E 'net_profitability_report|minimum viable size|safe minimum|gross profit|net profit|estimated fee' -- walkthrough.md phase-19.md docs/ crates/ bin/
- git grep -n -E '0.01|0.03|0.05|break-even|safe minimum|batching research justified' -- walkthrough.md phase-19.md net_profitability_report.json docs/
- git grep -n -E 'private orderflow|builder|relay' -- walkthrough.md phase-19.md docs/

3. Validation:
- cargo check --workspace
- cargo test --workspace
- docker compose config
- docker compose up -d prometheus grafana
- docker compose run --rm forge forge test

4. Calibration/net-fee run proof:
- exact command used
- exact input artifacts used
- path to `net_profitability_report.json`
- excerpt of the artifact

5. Browser validation:
- dashboard name opened
- panels checked
- actual values shown
- explicit answers to:
  - Is 0.01 ETH viable?
  - Is 0.03 ETH viable?
  - Is 0.05 ETH viable?
  - What is the current minimum standalone size estimate?

==================================================
SUCCESS CRITERIA
==================================================

Phase 19 is merge-ready only if:
- the branch is clean
- HEAD matches origin
- the canonical artifact exists and is truthful
- the artifact index exists and identifies canonical artifacts by phase
- docs and artifact tell the same story
- the dashboard shows meaningful net-profitability values
- the phase explicitly answers the minimum-size question
- private orderflow / builder / relay integration remains deferred

==================================================
REQUIRED OUTPUTS
==================================================

Provide:
1. Verdict
2. Canonical artifact filename
3. Changed-files summary
4. Checklist confirming:
   - gas/net modeling added
   - artifact index added
   - minimum size estimate produced
   - 0.01 / 0.03 / 0.05 viability answered
   - dashboard validated in browser
   - no live trading logic added
5. Exact raw outputs for all commands above
6. A short walkthrough describing:
   - what inputs were used
   - what minimum trade threshold was estimated
   - whether the standalone method looks viable
   - whether future batching research still looks justified
   - what remains deferred

Do not go beyond this scope.