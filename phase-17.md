Implement Phase 17 on a dedicated branch.

Suggested branch name:
phase-17-full-day-calibration-and-fork-checks

Before doing any code work:
1. Ensure work is being done on branch `phase-17-full-day-calibration-and-fork-checks`
   - if the branch does not exist, create it from current main
   - if it exists, switch to it
2. Do NOT work directly on main

Goal:
Run a truthful full-day historical calibration replay for March 22, 2026 if provider coverage supports it (or the nearest recent 24h window that works reliably), then automatically select a small representative subset of discovered trades for fork-based verification.

No live trading.
No broadcasts.
No new strategy logic.
No aggregator integration.

==================================================
PHASE 17 OBJECTIVE
==================================================

By the end of this phase, the system should be able to:

1. Run a truthful 24h historical replay over a real full-day window
2. Produce a canonical full-day replay artifact
3. Expose the full-day stats in Grafana
4. Automatically choose a small representative subset of trades for deeper fork validation
5. Verify those selected trades on a local fork using the existing harness
6. Compare replay prediction vs fork execution result

==================================================
FULL-DAY REPLAY REQUIREMENTS
==================================================

Use a 24h window if provider limits allow it.

Preferred target:
- March 22, 2026 on Base Mainnet

If that exact 24h window is not feasible due to provider/history limits:
- choose the nearest recent 24h window that is reliable
- document the exact start/end blocks used
- do not fake the date/range

The full-day replay must truthfully report:
- start_block
- end_block
- total_blocks
- total_logs
- candidates_considered
- promoted_candidates
- would_trade_candidates
- still_profitable_count
- invalidated_count
- avg_profit_drift_wei
- route-family breakdown

Canonical artifact should be something like:
- `historical_replay_full_day_final.json`

==================================================
SELECTED FORK VERIFICATION
==================================================

Automatically pick a small subset from the replay results.

Target sample:
- 2 likely successful would-trade candidates
- 1 invalidated candidate
- 1 additional candidate from another route family if useful

For each selected case:
- record case id
- block number
- route family
- predicted amount out / profit
- predicted recheck result

Then fork the chain at the relevant historical block and run local verification using the existing harness.

For each selected case, record:
- fork_success true/false
- actual_amount_out
- actual_profit
- gas_used
- revert_reason if any
- prediction_error if computable

Do not verify hundreds of cases. Keep it small and honest.

==================================================
DASHBOARD REQUIREMENTS
==================================================

Update/add a Grafana dashboard so I can view:

- full-day candidates considered
- full-day would-trade count
- full-day still-profitable count
- invalidated count
- average drift
- route family breakdown
- selected fork verification summary
- selected fork verification pass/fail table if practical

Keep using the existing Grafana/Prometheus stack.

==================================================
OUTPUTS REQUIRED
==================================================

Provide:

1. Verdict
2. Changed-files summary
3. Checklist confirming:
   - truthful full-day replay completed
   - canonical full-day artifact produced
   - dashboard updated and browser-validated
   - selected fork verification completed
   - no live trading logic added

4. Exact source-of-truth outputs:
- git fetch origin
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-17-full-day-calibration-and-fork-checks
- git status --short
- git log --oneline --decorate -5

5. Validation outputs:
- cargo check --workspace
- cargo test --workspace
- docker compose config
- docker compose up -d prometheus grafana
- docker compose run --rm forge forge test
- exact replay command used
- exact fork verification command(s) used

6. Artifact proof:
- show the canonical full-day replay artifact
- show the selected fork verification artifact/report

7. Browser proof:
- dashboard name
- panel names checked
- actual values shown
- short browser validation summary

Do not go beyond this scope.