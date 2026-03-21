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


   ---- update 2 ----

   Do a final Phase 13 execution-validation pass on the EXISTING branch `phase-13-historical-fork-battery`.

Do NOT create a new branch.
Do NOT add new features.
Do NOT add live trading logic, new strategy logic, or mempool tactics.
Do NOT expand scope beyond proving the historical fork battery actually runs end to end.

Goal:
Make Phase 13 merge-ready by actually running the replay battery against the local Dockerized Anvil harness and providing full source-of-truth outputs.

Important:
Do NOT claim success based only on compilation.
You must actually execute the battery and show the results.

==================================================
FIX 1 — USE DOCKERIZED ANVIL, NOT HOST PATH ANVIL
==================================================

Current problem:
The battery was not executed because `anvil` was not found on PATH.

Required fix:
- use the project’s Dockerized Anvil path instead
- start Anvil through docker compose
- do not require host `anvil` on PATH

Expected commands:
- docker compose config
- docker compose up -d anvil

Then run the battery against the local RPC URL.

==================================================
FIX 2 — ACTUALLY RUN THE BATTERY
==================================================

Run the real battery, not just the generator.

Required:
- execute `arb_battery_generator` if needed
- execute `arb_battery`
- produce results for multiple cases
- show per-case success/revert and attribution fields

The battery output must include at least:
- case id
- expected outcome
- actual outcome
- gas used
- success/revert
- revert reason if failed
- predicted vs actual comparison

==================================================
FIX 3 — SOURCE-OF-TRUTH OUTPUTS
==================================================

Provide these exact outputs:

1. Git identity:
- git fetch origin
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-13-historical-fork-battery
- git status --short
- git log --oneline --decorate -5

2. Proof commands:
- git show origin/phase-13-historical-fork-battery:fixtures/historical_cases.json
- git grep -n -E 'ReplayCase|ReplayCaseResult|ReplayFailureReason|AttributionSummary|fork_block_number' -- crates/ bin/ fixtures/ docs/
- git grep -n -E 'predicted_amount_out|predicted_profit|actual_amount_out|actual_profit|gas_used|revert_reason|absolute_error|relative_error' -- crates/ bin/

3. Validation commands:
- cargo check --workspace
- cargo test --workspace
- docker compose config
- docker compose up -d anvil
- docker compose run --rm forge forge test
- cargo run --bin arb_battery_generator
- cargo run --bin arb_battery

==================================================
REQUIRED OUTPUTS
==================================================

Provide:
1. Verdict
2. Changed-files summary
3. Checklist confirming:
   - historical cases file exists
   - generator works
   - battery actually runs
   - attribution fields are populated
   - multiple cases were executed
   - no live trading logic added
4. Exact outputs for all commands above
5. A short walkthrough describing:
   - how the battery was run
   - how Anvil was started
   - what happened in each case
   - what remains deferred


   ----- updates 3 -----

   Proceed with Phase 13, but tighten the implementation with these constraints:

1. Keep arb_battery_generator and arb_battery separate.
   - generator scans once and writes fixtures/historical_cases.json
   - battery only consumes that file and does not rescan history

2. Make historical_cases.json deterministic enough for replay.
   Each case should include at least:
   - case_id
   - fork_block_number
   - tx_hash or source event reference if available
   - root_asset
   - route_family
   - pool_ids
   - amount_in
   - expected_outcome
   - guard_overrides
   - notes
   If practical, also include token path / leg order and pool kind per leg.

3. During battery execution, use the local Anvil RPC as the execution-time source of truth.
   - external provider may be used for case generation
   - local Anvil RPC must be used during replay execution

4. Compute actual_amount_out primarily from final balance delta, not only Transfer logs.
   Transfer logs may be used as supporting evidence, but balance delta should be the primary attribution method.

5. For revert cases:
   - actual_amount_out should be null/omitted
   - actual_profit should be null/omitted
   - revert_reason must be populated
   - success_or_revert must be explicit

6. Keep the first battery to 4 cases:
   - 1 likely success
   - 1 forced slippage revert
   - 1 forced no-profit revert
   - 1 V3/CL case

7. Final output must include source-of-truth commands and the actual battery run output.

---- update 4 ----

Do a final Phase 13 merge-readiness pass on the EXISTING branch `phase-13-historical-fork-battery`.

Do NOT create a new branch.
Do NOT add new features.
Do NOT add live trading logic or new strategy logic.
Do NOT expand scope beyond cleanup + proof.

Goal:
Make Phase 13 merge-ready by removing debug/secret-leaking output and providing the actual source-of-truth + battery execution outputs.

==================================================
FIX 1 — REMOVE DEBUG / SECRET-LEAKING PRINTS
==================================================

Current problem:
The generator appears to print debug messages and may print the RPC URL directly.

Required fix:
- remove all DEBUG prints from arb_battery_generator and arb_battery
- do NOT print RPC URLs, endpoint tokens, private keys, or other secrets
- keep logs concise and safe

Acceptance criteria:
- no `println!` or logging of RPC URLs / secrets in generator or battery runner
- no `DEBUG:` strings remain in these binaries

==================================================
FIX 2 — PROVIDE ACTUAL SOURCE-OF-TRUTH OUTPUTS
==================================================

Do NOT summarize.
Run and paste the exact outputs of:

1. Git identity:
- git fetch origin
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-13-historical-fork-battery
- git status --short
- git log --oneline --decorate -5

2. Proof commands:
- git grep -n -E 'DEBUG:|Using RPC URL' -- bin/arb_battery_generator bin/arb_battery
- git grep -n -E 'HistoricalCase|AttributionResult|ReplayCaseResult|ReplayFailureReason|AttributionSummary|fork_block_number' -- crates/ bin/ fixtures/ docs/
- git grep -n -E 'predicted_amount_out|predicted_profit|actual_amount_out|actual_profit|gas_used|revert_reason|absolute_error|relative_error' -- crates/ bin/

3. Validation:
- cargo check --workspace
- cargo test --workspace
- docker compose config
- docker compose up -d anvil
- docker compose run --rm forge forge test
- cargo run --bin arb_battery_generator
- cargo run --bin arb_battery

==================================================
FIX 3 — SHOW THE REAL BATTERY RUN
==================================================

The final output must include the actual terminal output of:
- arb_battery_generator
- arb_battery

The arb_battery output must clearly show:
- multiple cases executed
- success/revert status per case
- gas used
- predicted vs actual values
- revert reasons where applicable

==================================================
REQUIRED OUTPUTS
==================================================

Provide:
1. Verdict
2. Changed-files summary
3. Checklist confirming:
   - debug/secret-leaking prints removed
   - source-of-truth outputs included
   - battery actually executed
   - multiple cases were run
   - no live trading logic added
4. Exact outputs for all commands above
5. A short walkthrough describing:
   - how the battery was run
   - what happened in each case
   - what remains deferred

Do not go beyond this scope.


---- update 5 ----

Do a final Phase 13 correctness pass on the EXISTING branch `phase-13-historical-fork-battery`.

Do NOT create a new branch.
Do NOT add new features.
Do NOT add live trading logic or new strategy logic.
Do NOT expand scope beyond making the historical replay battery actually execute and report honest attribution.

Goal:
Make Phase 13 merge-ready by:
1. getting the battery to run successfully,
2. replacing placeholder attribution with real values,
3. leaving the branch clean and fully tested.

==================================================
FIX 1 — MAKE THE BATTERY ACTUALLY RUN
==================================================

Current problem:
`cargo run --bin arb_battery` fails with:
- eth_call error

Required fix:
- diagnose and fix the battery execution path so the replay battery actually runs against the local Dockerized Anvil fork
- if the selected historical blocks are too old / unsupported by the provider, choose cases that are still historically real but reliably executable through the available fork provider
- during replay execution, use the local Anvil RPC as the execution-time source of truth
- do not claim success unless the battery actually runs to completion across multiple cases

==================================================
FIX 2 — MAKE ATTRIBUTION REAL
==================================================

Current problem:
The code still appears to use placeholder-style attribution:
- actual_amount_out = case.amount_in + profit
- actual_profit = profit
- absolute_error = 0
- relative_error = 0.0

Required fix:
- derive actual_amount_out from real post-trade balance delta or honest local execution outcome
- derive actual_profit from the actual execution result
- compute absolute_error honestly
- compute relative_error honestly
- for revert cases, actual_amount_out / actual_profit may be null/omitted, but revert_reason must be populated

Do NOT leave placeholder attribution in merge-ready code.

==================================================
FIX 3 — CLEAN BRANCH STATE
==================================================

Current problem:
The branch is not clean.

Required fix:
- commit or revert all intended changes
- final `git status --short` must be clean

==================================================
FIX 4 — FULL VALIDATION
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
- git grep -n -E 'actual_amount_out|actual_profit|absolute_error|relative_error' -- bin/arb_battery crates/
- git grep -n -E 'case.amount_in \+ profit|U256::ZERO|0.0' -- bin/arb_battery crates/
- git ls-files .env

3. Validation:
- cargo check --workspace
- cargo test --workspace
- docker compose config
- docker compose up -d anvil
- docker compose run --rm forge forge test
- cargo run --bin arb_battery_generator
- cargo run --bin arb_battery

==================================================
REQUIRED OUTPUTS
==================================================

Provide:
1. Verdict
2. Changed-files summary
3. Checklist confirming:
   - battery actually runs successfully
   - attribution is real
   - branch is clean
   - cargo check passes
   - cargo test passes
   - .env is not tracked
   - no live trading logic added
4. Exact outputs for all commands above
5. A short walkthrough describing:
   - how the battery was fixed
   - how attribution is now computed
   - what remains deferred to the next phase

Do not go beyond this scope.

---- update 6 ----
Do a final Phase 13 success-path completion pass on the EXISTING branch `phase-13-historical-fork-battery`.

Do NOT create a new branch.
Do NOT add live trading logic.
Do NOT add new strategy logic.
Do NOT expand scope beyond making the historical battery demonstrate at least one real success case on the fork.

Goal:
Make Phase 13 truly merge-ready by achieving at least one actual successful historical replay execution, while preserving honest revert attribution for failure cases.

==================================================
FIX 1 — ADD ONE REAL FORK-EXECUTABLE SWAP PATH
==================================================

Current problem:
The battery runs, but all cases revert because the current executor path is still mock-only for real fork swaps.

Required fix:
- implement the minimum real swap-call path needed for at least one supported venue on the fork
- this can be one simple venue/path only
- keep it narrow and deterministic
- do NOT broaden into full multi-venue production logic

Acceptable target:
- one reserve-based path OR one CL/V3 path
- enough to make at least one historical battery case succeed on the fork

==================================================
FIX 2 — BATTERY MUST PRODUCE BOTH:
==================================================

At least:
- 1 successful case
- 1 revert case

The final battery output must show both.

==================================================
FIX 3 — KEEP ATTRIBUTION HONEST
==================================================

For successful cases:
- actual_amount_out must be real
- actual_profit must be real
- gas_used must be real
- absolute_error and relative_error must be real

For revert cases:
- actual_amount_out / actual_profit may be null
- revert_reason must be populated
- relative_error may be 1.0 if that is the honest interpretation

==================================================
FIX 4 — CASE COUNT
==================================================

Restore the intended small-but-meaningful battery:
- 1 likely success
- 1 forced slippage revert
- 1 forced no-profit revert
- 1 V3/CL case if practical

At least 4 cases total if feasible.

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

2. Validation:
- cargo check --workspace
- cargo test --workspace
- docker compose config
- docker compose up -d anvil
- docker compose run --rm forge forge test
- cargo run --bin arb_battery_generator
- cargo run --bin arb_battery

3. Proof commands:
- git grep -n -E 'actual_amount_out|actual_profit|absolute_error|relative_error|revert_reason' -- bin/arb_battery crates/
- git grep -n -E 'swap|exactInput|exactInputSingle|sync|executeAtomicPlan' -- contracts/ crates/ bin/

==================================================
REQUIRED OUTPUTS
==================================================

Provide:
1. Verdict
2. Changed-files summary
3. Checklist confirming:
   - at least one historical success case works
   - at least one revert case works
   - attribution is real
   - battery includes multiple cases
   - no live trading logic added
4. Exact outputs for all commands above
5. A short walkthrough describing:
   - what success path was added
   - why it is enough for this phase
   - what remains deferred


   ---- update 7 ----

   Do a final Phase 13 success-proof pass on the EXISTING branch `phase-13-historical-fork-battery`.

Do NOT create a new branch.
Do NOT add live trading logic.
Do NOT add new strategy logic.
Do NOT expand scope beyond making the historical replay battery produce a genuinely meaningful result set.

Goal:
Make Phase 13 merge-ready by:
1. producing at least one real success case and one real revert case,
2. making the no-profit case genuinely distinct from the slippage case,
3. removing remaining debug-style output,
4. providing exact source-of-truth outputs and real battery-run evidence.

==================================================
FIX 1 — BATTERY MUST PRODUCE A REAL SUCCESS CASE
==================================================

Current problem:
The earlier battery output did not prove a real successful case.

Required fix:
- adjust the case generator and/or battery execution path so the battery produces at least:
  - 1 actual successful case
  - 1 actual revert case
- success must come from a real local/fork execution, not a fabricated status
- the success case should have:
  - success=true
  - actual_amount_out populated
  - actual_profit populated
  - gas_used populated
  - absolute_error / relative_error computed honestly

You may keep the battery small, but it must demonstrate both a success and a revert.

==================================================
FIX 2 — MAKE THE NO-PROFIT CASE GENUINELY DISTINCT
==================================================

Current problem:
The no-profit case risks being just another slippage-style revert.

Required fix:
- ensure the no-profit case fails because of the profit guard, not because of minOut/slippage
- ensure the slippage case fails because of slippage/minOut, not because of profit guard
- document clearly which guard is responsible in each case
- battery output should make the distinction obvious

==================================================
FIX 3 — CASE SET SHOULD BE SMALL BUT MEANINGFUL
==================================================

Required case mix:
- 1 likely success
- 1 slippage revert
- 1 no-profit revert
- 1 V3/CL case

If 5th case remains, it must add real value.
Do not pad the battery with redundant cases.

Each case in fixtures/historical_cases.json should include:
- case_id
- fork_block_number
- root_asset
- route_family
- pool_ids
- amount_in
- expected_outcome
- guard_overrides
- notes
- source_tx_hash if available

Keep the file deterministic and readable.

==================================================
FIX 4 — REMOVE DEBUG-STYLE OUTPUT
==================================================

Current problem:
arb_battery still emits internal/debug-style prints.

Required fix:
- remove noisy debug-style prints such as raw balance debug lines or internal submission dumps
- keep output concise and useful:
  - case id
  - expected outcome
  - actual outcome
  - gas used
  - actual profit
  - revert reason if failed
  - aggregate summary

Do NOT print secrets, RPC URLs, or noisy internal-only details.

==================================================
FIX 5 — ATTRIBUTION MUST STAY HONEST
==================================================

For successful cases:
- actual_amount_out must come from real balance delta or equivalent honest local execution outcome
- actual_profit must be real
- gas_used must be real
- absolute_error must be real
- relative_error must be real

For revert cases:
- actual_amount_out may be null
- actual_profit may be null
- revert_reason must be populated
- absolute_error / relative_error must still be computed honestly where applicable

Do NOT reintroduce placeholder attribution.

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
- git grep -n -E 'actual_amount_out|actual_profit|absolute_error|relative_error|revert_reason' -- bin/arb_battery crates/
- git grep -n -E 'DEBUG:|bal_before|bal_after|submission_result|Using RPC URL' -- bin/arb_battery bin/arb_battery_generator
- git show origin/phase-13-historical-fork-battery:fixtures/historical_cases.json

3. Validation:
- cargo check --workspace
- cargo test --workspace
- docker compose config
- docker compose up -d anvil
- docker compose run --rm forge forge test
- cargo run --bin arb_battery_generator
- cargo run --bin arb_battery

==================================================
SUCCESS CRITERIA
==================================================

The final arb_battery output must show:
- at least 1 case with success=true
- at least 1 case with success=false
- a distinct slippage revert case
- a distinct no-profit revert case
- at least 1 V3/CL case
- aggregate summary

==================================================
REQUIRED OUTPUTS
==================================================

Provide:
1. Verdict
2. Changed-files summary
3. Checklist confirming:
   - at least one successful case exists
   - at least one revert case exists
   - slippage and no-profit cases are distinct
   - debug-style output removed
   - attribution remains honest
   - no live trading logic added
4. Exact outputs for all commands above
5. A short walkthrough describing:
   - how the successful case was made to work
   - how the revert cases differ
   - how attribution is computed
   - what remains deferred to the next phase

Do not go beyond this scope.