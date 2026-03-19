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