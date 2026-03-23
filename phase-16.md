Implement Phase 16 on a dedicated branch.

Suggested branch name:
phase-16-historical-shadow-calibration-dashboard

Before doing any code work:
1. Ensure work is being done on branch `phase-16-historical-shadow-calibration-dashboard`
   - if the branch does not exist, create it from current main
   - if it exists, switch to it
2. Do NOT work directly on main

Goal:
Build a truthful historical shadow-calibration system that replays confirmed historical Base Mainnet data through the real pipeline, measures candidate frequency and decay, and exposes the results in a Grafana dashboard.

Note:
For this branch, the final merge-ready proof is **Path B: an honest 1-hour calibration slice** using blocks `43638000` to `43639800`. A full 24h+ replay and fork verification remain deferred.

Important:
- Prefer reusing the existing Prometheus/Grafana stack already in the repo
- Prefer reusing existing provider/env values from earlier phases
- Do NOT require new wallets or new secrets unless absolutely necessary
- No live trading
- No real broadcasts
- No new strategy logic
- No aggregator integration
- No mempool/PGA tactics

This phase is NOT about:
- live canary trades
- private relays
- EV learning policy automation
- multi-wallet fleet logic
- production rollout

==================================================
PHASE 16 OBJECTIVE
==================================================

By the end of this phase, the system should be able to:

1. Run a bounded historical calibration replay over confirmed Base Mainnet data
2. Replay that window through the real pipeline:
   ingest -> state -> route -> simulation -> would_trade decision
3. Recheck each candidate after a configurable historical delay (prefer block-based, not wall-clock based)
4. Record:
   - candidates considered
   - promoted candidates
   - would_trade candidates
   - still_profitable after delay
   - invalidated candidates
   - profit drift
   - amount_out drift
   - route family / venue family breakdown
5. Produce a machine-readable summary and a human-readable report
6. Expose those replay statistics in Grafana so they can be viewed in the browser

For final merge-ready proof on this branch:
- use **Path B: a high-signal 1-hour calibration slice**
- full 24h+ replay remains deferred

==================================================
HIGH-LEVEL DESIGN
==================================================

Implement this as a separate historical replay/calibration path, not by overloading live shadow mode.

Preferred architecture:
- A dedicated replay binary such as `arb_shadow_replay`
- It runs a bounded historical calibration window
- It emits:
  - a summary JSON artifact
  - optional JSONL/per-case output
  - a Prometheus metrics endpoint that remains alive long enough for Grafana to scrape
- Full fork verification for every candidate remains deferred

==================================================
PART 1 — CONFIG
==================================================

Add minimal historical replay config as needed, for example:
- ENABLE_HISTORICAL_SHADOW_REPLAY
- HISTORICAL_REPLAY_LOOKBACK_HOURS
- HISTORICAL_REPLAY_START_BLOCK
- HISTORICAL_REPLAY_END_BLOCK
- HISTORICAL_RECHECK_BLOCKS
- HISTORICAL_REPLAY_OUTPUT_PATH
- HISTORICAL_REPLAY_METRICS_PORT
- HISTORICAL_MAX_CASES_TO_VERIFY
- HISTORICAL_ROUTE_FAMILY_FILTER (optional)
- HISTORICAL_ROOT_ASSET_FILTER (optional)

Rules:
- reuse existing RPC_HTTP_URL / ANVIL_FORK_URL / ANVIL_RPC_URL where possible
- do not require new secrets if existing env values are sufficient
- use safe defaults
- no live broadcast

For merge-ready proof on this branch:
- use Path B blocks `43638000` to `43639800`
- default output path should be:
  `historical_replay_calibration_final.json`

Update `.env.example` accordingly with placeholders only.

==================================================
PART 2 — TYPE SYSTEM
==================================================

Add minimal shared types as needed, such as:
- HistoricalReplayCase
- HistoricalReplayResult
- HistoricalReplaySummary
- HistoricalRecheckResult
- HistoricalDriftSummary
- HistoricalRouteFamilyStats
- ForkVerificationCase
- ForkVerificationResult

Each replay result should capture at minimum:
- case_id
- block_number
- route_family
- root_asset
- amount_in
- predicted_amount_out
- predicted_profit
- would_trade
- rechecked_amount_out
- rechecked_profit
- is_still_profitable
- profit_drift
- amount_out_drift
- invalidated_reason if any

Keep them minimal and serializable.

Note:
- ForkVerification* types may remain present in the type system
- actual fork verification for the final proof is deferred on this branch

==================================================
PART 3 — HISTORICAL REPLAY RUNNER
==================================================

Build a dedicated historical replay runner.

Requirements:
- use a bounded historical confirmed-data window
- process historical logs in order
- feed them through the real pipeline as much as possible
- record candidate frequency and would_trade decisions
- perform historical delayed recheck using block-based delay (preferred) or another honest deterministic method
- produce per-case output and aggregate summary

Important:
- this is about calibration and opportunity capture statistics
- do NOT fake opportunities
- do NOT use live shadow-mode mock injection here
- if no candidates appear in the chosen window, explain that honestly and choose a better historical window rather than faking results

Final proof for this branch:
- use a truthful 1-hour calibration slice
- canonical artifact:
  `historical_replay_calibration_final.json`

==================================================
PART 4 — FORK VERIFICATION
==================================================

Fork verification for the final Path B proof is deferred.

What is deferred:
- automatic selection of replay candidates for fork spot-checks
- replaying selected candidates on a local fork as part of the final Phase 16 proof

If fork verification is added in a later phase, it should be documented separately and reflected in the final artifact.

==================================================
PART 5 — METRICS
==================================================

Expose historical replay metrics honestly, for example:
- arb_hist_candidates_total
- arb_hist_promoted_total
- arb_hist_would_trade_total
- arb_hist_rechecks_total
- arb_hist_still_profitable_total
- arb_hist_invalidated_total
- arb_hist_profit_drift_wei_gauge
- arb_hist_amount_out_drift_wei_gauge
- arb_hist_avg_profit_drift
- arb_hist_avg_output_drift
- arb_hist_route_family_count{family=...}
- arb_hist_fork_verifications_total
- arb_hist_fork_verifications_success_total
- arb_hist_fork_verifications_failed_total

The replay runner should keep serving these metrics after the run completes so Grafana can display them.

Do not fake metrics.

For the final Path B proof:
- metrics must reflect the 1-hour calibration slice truthfully
- if fork verification counters are zero, that is acceptable as long as docs/artifacts say fork verification is deferred

==================================================
PART 6 — DASHBOARD
==================================================

This is important: I want to view the results in the browser.

Add or update a dedicated Grafana dashboard called:

Historical Shadow Calibration

Panels should include at least:
- candidates considered
- promoted candidates
- would_trade candidates
- still-profitable after recheck
- invalidated candidates
- average profit drift
- average amount_out drift
- route family breakdown
- a summary stat for the replay window used

Optional:
- table/stat panel for fork verification (may be empty/deferred in final Path B proof)
- block range / lookback hours panel

Use the existing Grafana/Prometheus stack if possible.

==================================================
PART 7 — BROWSER VALIDATION
==================================================

After implementing:
- start/ensure Prometheus and Grafana are running
- run the historical replay/calibration job so the metrics endpoint is populated
- load the Grafana dashboard in the browser
- verify that the dashboard shows non-zero or otherwise meaningful data
- if the dashboard is blank, fix the scrape/config/dashboard wiring until it works

For final proof:
- the dashboard must show values matching the canonical artifact
- those values must correspond to the 1-hour calibration slice

==================================================
PART 8 — DOCUMENTATION HONESTY
==================================================

Update docs/checklists/walkthrough so they clearly state:

Real after Phase 16:
- candidate frequency and decay are measured
- delayed historical rechecks are measured
- dashboard shows historical replay stats in the browser
- final proof uses a truthful 1-hour calibration slice (Path B)

Still deferred:
- full 24h+ historical replay as final proof
- fork verification as part of the final proof
- live canaries
- real-money execution
- private relays
- EV learning policy automation
- production fleet scaling

Do not oversell beyond what is implemented.

==================================================
VALIDATION REQUIRED
==================================================

Run and report:

1. Git identity:
- git fetch origin
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-16-historical-shadow-calibration-dashboard
- git status --short
- git log --oneline --decorate -5

2. Grep proofs:
- git grep -n -E 'ENABLE_HISTORICAL_SHADOW_REPLAY|HISTORICAL_REPLAY|HISTORICAL_RECHECK_BLOCKS|HISTORICAL_REPLAY_METRICS_PORT' -- crates/arb_config .env.example bin/
- git grep -n -E 'HistoricalReplayResult|HistoricalReplaySummary|HistoricalDriftSummary|ForkVerificationResult' -- crates/ bin/
- git grep -n -E 'arb_hist_|historical replay|Historical Shadow Calibration|historical_calibration.json' -- crates/ docker-compose.yml infra/grafana docs/

3. Workspace / tool validation:
- cargo check --workspace
- cargo test --workspace
- docker compose config
- docker compose up -d prometheus grafana
- docker compose run --rm forge forge test

4. Historical replay run:
- exact command used
- exact block range / lookback used
- replay summary output
- canonical artifact contents

5. Browser/dashboard validation:
- confirm the dashboard was opened in the browser
- report which panels displayed meaningful data
- confirm they match the canonical artifact

==================================================
REQUIRED OUTPUTS
==================================================

Provide:

1. A verdict:
- fully working
- working with known limitations
- blocked (and why)

2. A checklist confirming:
- 1-hour historical calibration slice (Path B) added
- candidate/recheck/drift stats added
- fork verification deferred
- Grafana dashboard added/updated
- browser validation completed
- no live trading logic added

3. Changed-files summary

4. A walkthrough artifact describing:
- how the historical replay window is chosen
- how delayed historical recheck works
- what the dashboard shows
- what remains deferred to the next phase

5. The source-of-truth outputs listed above

Do not go beyond Phase 16.