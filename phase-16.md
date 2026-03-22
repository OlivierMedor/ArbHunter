Implement Phase 16 on a dedicated branch.

Suggested branch name:
phase-16-historical-shadow-calibration-dashboard

Before doing any code work:
1. Ensure work is being done on branch `phase-16-historical-shadow-calibration-dashboard`
   - if the branch does not exist, create it from current main
   - if it exists, switch to it
2. Do NOT work directly on main

Goal:
Build a 24h+ historical shadow-calibration system that replays historical confirmed chain data through the real pipeline, measures candidate frequency and decay, verifies a small selected subset on forked execution, and exposes the results in a Grafana dashboard that can be viewed in the browser.

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

1. Run a 24h+ historical replay over a bounded Base Mainnet window using confirmed historical data
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
7. Also run a small selected fork-verification subset (for example 1 success + 1 revert) using the existing fork harness, so the dashboard/report can include “historical replay stats” plus “selected fork verification stats”

==================================================
HIGH-LEVEL DESIGN
==================================================

Please implement this as a separate historical replay/calibration path, not by modifying live shadow mode into something confusing.

Preferred architecture:
- A dedicated replay binary (for example `arb_shadow_replay` or similarly clear name)
- It runs a 24h+ historical window
- It emits:
  - summary JSON artifact
  - JSONL per-case output if useful
  - Prometheus metrics endpoint that stays up after the run so Grafana can scrape and display the results
- A separate small selected fork-verification step reusing the Phase 12/13/14 execution harness

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
- if 24h exact is unreliable due to provider limits, choose the nearest recent 24h+ bounded window that works reliably and document the exact block range used
- use safe defaults
- no live broadcast

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

==================================================
PART 3 — 24H+ HISTORICAL REPLAY RUNNER
==================================================

Build a dedicated historical replay runner.

Requirements:
- use a bounded historical window of at least ~24h if possible
- process historical confirmed data in order
- feed it through the real pipeline as much as possible
- record candidate frequency and would_trade decisions
- perform historical delayed recheck using block-based delay (preferred) or another honest deterministic method
- produce per-case output and aggregate summary

Important:
- this is about calibration and opportunity capture statistics
- do NOT fake opportunities
- do NOT use live shadow-mode mock injection here
- if no candidates appear in the chosen window, explain that honestly and choose a better historical window rather than faking results

==================================================
PART 4 — SELECTED FORK VERIFICATION
==================================================

From the replay results, automatically choose a very small representative subset for deeper verification, such as:
- 1 likely success
- 1 likely revert or invalidated case

Then reuse the existing local/fork execution harness to verify those specific cases on a fork.

Record:
- actual tx success/revert
- gas used
- actual realized output/profit if applicable
- revert reason if applicable

This is a small “spot check” layer, not a full replay of every candidate on a fork.

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

==================================================
PART 6 — DASHBOARD
==================================================

This is important: I want to view the results in the browser.

Please add or update Grafana dashboards so the historical replay statistics can be viewed visually.

Preferred approach:
- add a dedicated Grafana dashboard called something like:
  "Historical Shadow Calibration"
or
  "Historical Replay Calibration"

Panels should include at least:
- candidates considered
- promoted candidates
- would_trade candidates
- still-profitable after recheck
- invalidated candidates
- average profit drift
- average amount_out drift
- route family breakdown
- selected fork verification pass/fail
- a summary stat for the historical window used

If helpful, also include:
- a table or stat panel for the selected verified cases
- a simple panel for replay block range / lookback hours

Use the existing Grafana/Prometheus stack if possible.

==================================================
PART 7 — BROWSER VALIDATION
==================================================

After implementing:
- start/ensure Prometheus and Grafana are running
- run the historical replay/calibration job so the metrics endpoint is populated
- load the Grafana dashboard in the browser
- verify that the dashboard actually shows non-zero or meaningful data
- if the dashboard is blank, fix the scrape/config/dashboard wiring until it works

I explicitly want this phase to end with browser-visible stats if possible.

==================================================
PART 8 — DOCUMENTATION HONESTY
==================================================

Update docs/checklists/walkthrough so they clearly state:

Real after Phase 16:
- 24h+ historical replay/calibration exists
- candidate frequency and decay are measured
- delayed historical rechecks are measured
- selected fork verification exists
- dashboard shows historical replay stats in the browser

Still deferred:
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
- git grep -n -E 'arb_hist_|historical replay|Historical Shadow Calibration|Historical Replay Calibration' -- crates/ docker-compose.yml infra/grafana docs/

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
- selected fork verification output

5. Browser/dashboard validation:
- confirm the dashboard was opened in the browser
- provide a short browser validation summary
- report which dashboard panels displayed meaningful data

==================================================
REQUIRED OUTPUTS
==================================================

Provide:

1. A verdict:
- fully working
- working with known limitations
- blocked (and why)

2. A checklist confirming:
- 24h+ historical replay/calibration added
- candidate/recheck/drift stats added
- selected fork verification added
- Grafana dashboard added/updated
- browser validation completed
- no live trading logic added

3. Changed-files summary

4. A walkthrough artifact describing:
- how the historical replay window is chosen
- how delayed historical recheck works
- how selected fork verification works
- what the dashboard shows
- what remains deferred to the next phase

5. The source-of-truth outputs listed above

Do not go beyond Phase 16.