Implement Phase 13 on a dedicated branch.

Suggested branch name:
phase-13-historical-fork-battery

Important:
Assume the user is NOT a blockchain expert.
Do not require the user to manually choose historical cases.
You must select the initial historical cases automatically and explain them in plain English.

Goal:
Build a historical fork replay battery and post-trade attribution layer using a very small, beginner-friendly set of replay cases.

Scope:
- automatically choose the first historical cases
- fork the chain at those historical blocks
- run the existing end-to-end local harness
- compare predicted vs actual outcomes
- report results in simple language
- no live mainnet trading
- no new strategy logic
- no mempool/PGA tactics

==================================================
PHASE 13 OBJECTIVE
==================================================

By the end of this phase, the system should be able to:

1. Automatically select a small set of historical replay cases
2. Explain each case in plain English
3. Fork the chain at each case’s historical block
4. Run the full local path:
   candidate -> simulation -> execution plan -> signed tx -> local/fork submission -> receipt
5. Record:
   - predicted amount out
   - predicted profit
   - actual amount out
   - actual profit
   - gas used
   - success or revert
   - revert reason if failed
6. Produce a structured report and a simple human-readable summary

==================================================
CASE SELECTION RULES
==================================================

You must NOT ask the user to manually pick historical cases.

Instead:
1. scan a small bounded historical window
2. choose a small initial battery of 3 to 5 cases
3. make the set “small but meaningful”

The first battery should try to include:
- 1 likely success case
- 1 forced slippage revert case
- 1 forced no-profit revert case
- 1 concentrated-liquidity/V3 case if available
- optionally 1 edge case

Selection priorities:
- choose simple routes first
- prefer clearer, easier-to-debug cases
- prefer fewer hops over more hops
- prefer well-formed pool/state metadata
- explain why each case was selected

For forced failure cases:
- it is acceptable to derive them from a success candidate by tightening minOut or profit guard

==================================================
PLAIN-ENGLISH EXPLANATION REQUIREMENT
==================================================

For every selected case, provide:
- case id
- block number
- what kind of route it is
- why it was selected
- what outcome is expected

Explain this in plain English for a beginner.
Avoid jargon unless necessary.

==================================================
WORK AREAS
==================================================

Likely files/crates to work in:
- crates/arb_types
- crates/arb_config
- crates/arb_execute
- crates/arb_sim
- bin/arb_e2e
- fixtures/
- docs if needed

==================================================
PART 1 — HISTORICAL CASE FORMAT
==================================================

Add a simple case format, such as:
- case_id
- description
- fork_block_number
- route family
- root asset
- expected outcome
- notes

Store the selected cases in a clean, readable fixture file.

==================================================
PART 2 — BATTERY RUNNER
==================================================

Build a runner that:
- loads the selected cases
- forks the chain at the given block
- runs the full local harness
- captures a structured result for each case

Do not require live mainnet execution.

==================================================
PART 3 — ATTRIBUTION
==================================================

For each case, compute and store:
- predicted_amount_out
- predicted_profit
- actual_amount_out
- actual_profit
- gas_used
- success/revert
- revert_reason
- absolute_error
- relative_error

Also produce a plain-English per-case summary.

==================================================
PART 4 — REPORTING
==================================================

Produce:
- a machine-readable report
- a short human-readable summary
- aggregate stats:
  - success count
  - revert count
  - average gas used
  - average prediction error

==================================================
PART 5 — TESTING
==================================================

Run and report:
- cargo check --workspace
- cargo test --workspace
- docker compose config
- docker compose up -d anvil
- docker compose run --rm forge forge test
- one actual replay battery run with multiple cases

==================================================
PART 6 — DOCUMENTATION HONESTY
==================================================

Update docs/checklists/walkthrough so they clearly state:
- the agent selected the first historical cases automatically
- the user does not need blockchain expertise to choose cases
- the battery is intentionally small and educational at first
- live canaries are still deferred

==================================================
SOURCE-OF-TRUTH OUTPUTS REQUIRED
==================================================

At the end, include:

1. Git identity:
- git fetch origin
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-13-historical-fork-battery
- git status --short
- git log --oneline --decorate -5

2. Validation outputs:
- cargo check --workspace
- cargo test --workspace
- docker compose config
- docker compose up -d anvil
- docker compose run --rm forge forge test
- one actual replay battery run

3. Case list:
For each chosen case, print:
- case_id
- block_number
- expected outcome
- plain-English reason for selection

==================================================
REQUIRED OUTPUTS
==================================================

Provide:

1. A verdict:
- fully working
- working with known limitations
- blocked (and why)

2. A checklist confirming:
- historical cases were chosen automatically
- plain-English explanations were provided
- replay battery runner added
- attribution added
- multiple cases were executed
- no live trading logic added

3. Changed-files summary

4. A walkthrough artifact describing:
- how the cases were chosen
- how the battery works
- how attribution is computed
- what remains deferred to the next phase

5. The source-of-truth outputs listed above

Do not go beyond Phase 13.


---- update ----

Do a final Phase 13 merge-readiness pass on the EXISTING branch `phase-13-historical-fork-battery`.

Do NOT create a new branch.
Do NOT add live trading logic.
Do NOT add new strategy logic.
Do NOT expand scope beyond making Phase 13 honestly historical and attribution-driven.

Goal:
Make Phase 13 merge-ready by replacing the remaining canned/mock battery behavior with a truthful historical replay battery and real attribution.

==================================================
FIX 1 — GENERATOR MUST BE A REAL BOUNDED SCAN
==================================================

Current problem:
`arb_battery_generator` appears to hardcode one block number and a few static pools/cases.

Required fix:
- replace the hardcoded generator behavior with a real bounded scan OR a real selection from previously promoted candidates
- the first battery may still produce only 3–5 cases, but they must be selected from real historical data, not just written as canned cases
- keep the chosen cases small/simple, but honest

At minimum, the output cases should be selected from:
- a bounded historical window
OR
- previously promoted candidates from the pipeline

==================================================
FIX 2 — BATTERY MUST REUSE THE REAL PIPELINE HONESTLY
==================================================

Current problem:
`arb_battery` appears to construct dummy state and dummy candidate estimates instead of reusing the real historical case data honestly.

Required fix:
- stop fabricating giant reserve snapshots and dummy estimated outputs/profits for replay cases
- the replay battery must use:
  candidate -> simulation -> execution plan -> signed tx -> local submit -> receipt
- if a simplification remains, document it explicitly and keep it minimal

The important rule:
Do not call it “historical replay battery” if it still mostly runs on invented state.

==================================================
FIX 3 — ACTUAL OUTCOME / ATTRIBUTION MUST BE REAL
==================================================

Current problem:
`actual_amount_out`, `actual_profit`, and `relative_error` appear to be placeholder/naive values.

Required fix:
- derive actual_amount_out and actual_profit from the real local execution result
- compute absolute_error honestly
- compute relative_error honestly
- if some field cannot yet be made real, document it clearly and do not pretend otherwise

==================================================
FIX 4 — CASE FILE SHOULD REMAIN SMALL BUT HONEST
==================================================

Keep the first battery to 3–5 cases, but ensure:
- at least 1 likely success
- at least 1 slippage revert
- at least 1 no-profit revert
- at least 1 V3/CL case

Plain-English notes are good; keep them.

==================================================
VALIDATION REQUIRED
==================================================

Run and report:

1. Source of truth:
- git fetch origin
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-13-historical-fork-battery
- git status --short
- git log --oneline --decorate -5

2. Proof commands:
- git grep -n -E '22000000|v2_pool|v3_pool|U256::MAX' -- bin/arb_battery_generator fixtures/historical_cases.json
- git grep -n -E 'actual_out = U256::ZERO|actual_profit = U256::ZERO|relative_error = if sim_profit > U256::ZERO' -- bin/arb_battery
- git grep -n -E 'predicted_amount_out|predicted_profit|actual_amount_out|actual_profit|gas_used|revert_reason|absolute_error|relative_error' -- crates/ bin/

3. Validation:
- cargo check --workspace
- cargo test --workspace
- docker compose config
- docker compose up -d anvil
- docker compose run --rm forge forge test
- one actual replay battery run with multiple cases

==================================================
REQUIRED OUTPUTS
==================================================

Provide:
1. Verdict
2. Changed-files summary
3. Checklist confirming:
   - generator now selects cases from real historical data or prior promoted candidates
   - battery reuses the real pipeline honestly
   - actual outcomes and attribution are real
   - no live trading logic added
4. Exact outputs for all source-of-truth and proof commands above
5. A short walkthrough describing:
   - how cases are selected
   - how the battery reuses the real pipeline
   - how attribution is computed
   - what remains deferred