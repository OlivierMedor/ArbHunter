Implement Phase 14 on a dedicated branch.

Suggested branch name:
phase-14-v2-venue-execution

Before doing any code work:
1. Ensure work is being done on branch `phase-14-v2-venue-execution`
   - if the branch does not exist, create it from current main
   - if it exists, switch to it
2. Do NOT work directly on main

Goal:
Expand the local/fork execution harness so it can execute a real reserve-based venue path, not just the current V3 path.

Why this phase exists:
Phase 13 proved that:
- V3 historical replay can succeed
- slippage/no-profit revert cases are working
- V2/Aerodrome-style case is still labeled as unsupported and reverts

This phase should close that gap.

Scope:
- add one real reserve-based execution path (Aerodrome V2 / Uniswap V2-style)
- extend Rust execution planning / builder / contract alignment for that path
- update historical battery so at least one V2-style case can succeed honestly
- preserve all existing guards and attribution
- no live mainnet trading
- no aggregator integration
- no new strategy logic
- no mempool/PGA tactics

This phase is NOT about:
- live canaries
- private relays
- EV learning
- multi-wallet fleet logic
- new route discovery logic

==================================================
PHASE 14 OBJECTIVE
==================================================

By the end of this phase, the system should be able to:

1. Represent and execute one real reserve-based venue path on the local/fork harness
2. Keep the existing V3 path working
3. Support at least one V2/Aerodrome-style success case in the historical replay battery
4. Preserve slippage / profit / repayment safety checks
5. Prove the expanded venue coverage with Rust tests, Foundry tests, and replay battery output

==================================================
WORK AREAS
==================================================

Likely files/crates to work in:
- crates/arb_types
- crates/arb_execute
- contracts/
- bin/arb_battery
- bin/arb_battery_generator
- fixtures/
- docs if needed

Keep the scope narrow and execution-focused.

==================================================
PART 1 — TYPE / PLAN SUPPORT
==================================================

Extend the existing execution-plan model only as much as needed to support a reserve-based real venue path.

Possible additions:
- venue / pool family discriminator on execution legs
- reserve-based swap call parameters
- any minimal route metadata needed for V2/Aerodrome execution

Rules:
- do not add broad platform abstraction if one narrow extension is enough
- keep types honest and minimal
- preserve compatibility with existing V3 path

==================================================
PART 2 — RUST BUILDER / PLANNER SUPPORT
==================================================

In `crates/arb_execute`:
- extend the builder/planner so a validated reserve-based candidate can become a real executable plan for the supported V2-style venue
- keep existing V3 execution plan support intact
- ensure invalid / unsupported routes still return structured errors

Important:
- no new broadcast strategy logic
- no new wallet logic
- no aggregator usage

==================================================
PART 3 — SOLIDITY EXECUTION SUPPORT
==================================================

In `contracts/`:
Add the minimum contract-side support required to execute one reserve-based venue path on the fork.

Expected direction:
- add a real V2/Aerodrome-style execution path to the executor contract
- preserve existing slippage guard
- preserve existing profit guard
- preserve authorization checks
- preserve repayment logic for the atomic path if relevant

Requirements:
- the contract must actually call the supported venue path, not mock it
- do not add unrelated venue support
- do not bloat the contract with strategy logic

==================================================
PART 4 — FOUNDRY TESTS
==================================================

Add/expand Foundry tests to prove:
1. V3 path still succeeds
2. V2/Aerodrome-style path can now succeed
3. slippage revert still works
4. no-profit revert still works
5. unauthorized caller still reverts
6. unsupported route still reverts if still applicable

Keep the tests deterministic and local/fork-safe.

==================================================
PART 5 — HISTORICAL BATTERY UPGRADE
==================================================

Update the Phase 13 battery so it reflects the new coverage honestly.

Requirements:
- keep the small deterministic case set
- ensure at least:
  - 1 V3 success
  - 1 V2/Aerodrome-style success
  - 1 slippage revert
  - 1 no-profit revert
- if any route family is still unsupported, label it honestly

Attribution must remain real:
- actual_amount_out from real balance delta or equivalent honest local result
- actual_profit real
- absolute_error real
- relative_error real
- revert_reason populated for revert cases

==================================================
PART 6 — DOCUMENTATION HONESTY
==================================================

Update docs/checklists/walkthrough so they clearly state:

Real after Phase 14:
- V3 execution path works
- one reserve-based venue execution path works
- historical replay battery covers both route families
- attribution remains real

Still deferred:
- live canaries
- aggregator integration
- private relays
- EV learning
- production execution policy

Do not oversell beyond what is implemented.

==================================================
VALIDATION REQUIRED
==================================================

Run and report:

1. Rust validation:
- cargo check --workspace
- cargo test --workspace

2. Solidity validation:
- docker compose run --rm forge forge build
- docker compose run --rm forge forge test

3. Harness validation:
- docker compose config
- docker compose up -d anvil
- cargo run --bin arb_battery_generator
- cargo run --bin arb_battery

==================================================
SOURCE-OF-TRUTH OUTPUTS REQUIRED
==================================================

At the end, include these exact commands and outputs:

1. Git identity:
- git fetch origin
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-14-v2-venue-execution
- git status --short
- git log --oneline --decorate -5

2. Grep proofs:
- git grep -n -E 'Aerodrome|UniswapV2|swapExactTokens|swap|executeAtomicPlan|executePlan' -- contracts/ crates/
- git grep -n -E 'unsupported_route_revert|slippage_revert|no_profit_revert|v2|v3' -- fixtures/ bin/
- git grep -n -E 'actual_amount_out|actual_profit|absolute_error|relative_error|revert_reason' -- bin/arb_battery crates/

3. Validation outputs:
- cargo check --workspace
- cargo test --workspace
- docker compose config
- docker compose up -d anvil
- docker compose run --rm forge forge build
- docker compose run --rm forge forge test
- cargo run --bin arb_battery_generator
- cargo run --bin arb_battery

==================================================
REQUIRED OUTPUTS
==================================================

Provide:

1. A verdict:
- fully working
- working with known limitations
- blocked (and why)

2. A checklist confirming:
- reserve-based execution path added
- V3 path still works
- V2/Aerodrome-style success case exists
- slippage and no-profit revert cases still work
- historical battery updated honestly
- no live trading logic added

3. Changed-files summary

4. A walkthrough artifact describing:
- what reserve-based venue path was added
- how the contract executes it
- how Rust planning/building was extended
- how the battery now covers both V2-style and V3-style execution
- what remains deferred to the next phase

5. The source-of-truth outputs listed above

A good Phase 14 outcome is simple: the historical battery should end with one real V3 success, one real V2/Aerodrome-style success, one slippage revert, and one no-profit revert.

Do not go beyond Phase 14.

---- update 1 ----

Proceed with Phase 14, but tighten the implementation with these constraints:

1. Do NOT relabel Case 4 to success until the battery output proves it actually succeeds.
2. If the current Aerodrome USDC/DAI case requires stable-pool math that is too complex for this pass, switch the reserve-based success target to a simpler true V2-style volatile pool first.
3. Keep the final milestone strict:
   - 1 real V3 success
   - 1 real reserve-based success
   - 1 slippage revert
   - 1 no-profit revert
4. The contract and Rust builder must explicitly distinguish venue type and correctly handle token0/token1 and amount0Out/amount1Out for reserve-based execution.
5. Final output must include the actual source-of-truth commands and the real battery output proving the reserve-based success case.


---- update 1 ----

Do a final Phase 14 correctness pass on the EXISTING branch `phase-14-v2-venue-execution`.

Do NOT create a new branch.
Do NOT add new features.
Do NOT add live trading logic, aggregator logic, or new strategy logic.
Do NOT expand scope beyond fixing the remaining Phase 14 blockers.

Goal:
Make Phase 14 merge-ready by:
1. getting one real reserve-based success in the historical battery,
2. fixing the very large attribution mismatch on the V3 success case,
3. leaving the branch clean and fully validated.

==================================================
FIX 1 — CLEAN BRANCH STATE
==================================================

Current problem:
The branch is not clean.

Required fix:
- commit or revert all intended changes
- final `git status --short` must be clean

==================================================
FIX 2 — GET A REAL V2 / RESERVE-BASED SUCCESS CASE
==================================================

Current problem:
The battery still shows:
- case_4_v2_unsupported_revert | FALSE

Required fix:
- adjust the reserve-based execution path, case selection, or fixture so that at least one reserve-based historical case succeeds in the battery
- do NOT relabel failure as success
- keep the battery honest

Success criteria for the battery:
- 1 real V3 success
- 1 real reserve-based/V2 success
- 1 slippage revert
- 1 no-profit revert

==================================================
FIX 3 — FIX ATTRIBUTION ERROR
==================================================

Current problem:
The V3 success case shows ~99.97% error, which is too large to treat as valid attribution.

Required fix:
- diagnose and correct the mismatch between predicted and actual attribution
- check:
  - token unit normalization
  - balance delta accounting
  - gas accounting
  - route leg accounting
  - profit denominator / error calculation
- keep attribution honest
- do NOT just suppress the error metric

Acceptance criteria:
- the successful case(s) should have a sane prediction error
- actual_amount_out and actual_profit must still come from real local execution outcome / balance delta

==================================================
FIX 4 — FINAL VALIDATION
==================================================

Run and report:

1. Source of truth:
- git fetch origin
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-14-v2-venue-execution
- git status --short
- git log --oneline --decorate -5

2. Proof commands:
- git grep -n -E 'case_4_v2|unsupported_revert|v2_success|v3_success|slippage_revert|no_profit_revert' -- fixtures/ bin/
- git grep -n -E 'actual_amount_out|actual_profit|absolute_error|relative_error|revert_reason' -- bin/arb_battery crates/
- git grep -n -E 'swap|amount0Out|amount1Out|ReserveBased|Aerodrome|UniswapV2' -- contracts/ crates/

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
   - branch is clean
   - one V3 success exists
   - one reserve-based/V2 success exists
   - one slippage revert exists
   - one no-profit revert exists
   - attribution is now sane and honest
   - no live trading logic added
4. Exact outputs for all commands above
5. A short walkthrough describing:
   - how the reserve-based success case was made to work
   - what caused the attribution mismatch and how it was fixed
   - what remains deferred to the next phase

Do not go beyond this scope.


---- update 2 ----

Do a final Phase 14 correctness pass on the EXISTING branch `phase-14-v2-venue-execution`.

Do NOT create a new branch.
Do NOT add new features.
Do NOT add live trading logic, aggregator logic, or new strategy logic.
Do NOT expand scope beyond fixing the remaining battery-behavior blockers.

Goal:
Make Phase 14 merge-ready by getting the battery to demonstrate the intended four-case outcome set honestly.

==================================================
FIX 1 — MAKE CASE 4 ACTUALLY SUCCEED
==================================================

Current problem:
The battery still shows:
- case_4_mixed_v2_v3_success -> FALSE / On-chain Revert

Required fix:
- diagnose why the mixed V2/V3 path still reverts
- fix the route execution path, fixture, or executor logic so that this case actually succeeds
- do NOT relabel it as failure; this phase needs one real mixed/reserve-based success

==================================================
FIX 2 — MAKE THE NO-PROFIT CASE TRULY A NO-PROFIT CASE
==================================================

Current problem:
case_3_no_profit_revert is currently showing:
- Match FALSE
and the runner log shows:
- Simulation Outcome: Failed(SlippageExceeded)

Required fix:
- ensure the no-profit case fails because of the profit guard, not because of slippage
- separate the no-profit and slippage failure modes cleanly
- battery output must make this distinction obvious

==================================================
FIX 3 — ALIGN GENERATOR AND FIXTURE NAMING
==================================================

Current problem:
The generator code and committed fixture file use inconsistent case ids/naming.

Required fix:
- make generator output and fixtures/historical_cases.json use the same case ids and outcome labels
- keep naming stable and honest

==================================================
FIX 4 — KEEP ATTRIBUTION HONEST
==================================================

Do not regress:
- actual_amount_out from real balance delta or equivalent honest execution outcome
- actual_profit real
- absolute_error real
- relative_error real
- revert_reason populated for failures

==================================================
VALIDATION REQUIRED
==================================================

Run and report:

1. Source of truth:
- git fetch origin
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-14-v2-venue-execution
- git status --short
- git log --oneline --decorate -5

2. Proof commands:
- git grep -n -E 'case_1_v3_success|case_2_slippage_revert|case_3_no_profit_revert|case_4_mixed_v2_v3_success' -- bin/arb_battery_generator fixtures/historical_cases.json bin/arb_battery
- git grep -n -E 'actual_amount_out|actual_profit|absolute_error|relative_error|revert_reason' -- bin/arb_battery crates/
- git grep -n -E 'swap|ReserveBased|V3|Aerodrome|UniswapV2' -- contracts/ crates/

3. Validation:
- cargo check --workspace
- cargo test --workspace
- docker compose run --rm forge forge test
- cargo run --bin arb_battery_generator
- cargo run --bin arb_battery

==================================================
SUCCESS CRITERIA
==================================================

The final battery output must show:
- case_1_v3_success -> TRUE
- case_2_slippage_revert -> TRUE
- case_3_no_profit_revert -> TRUE
- case_4_mixed_v2_v3_success -> TRUE

And the battery summary should clearly reflect that all 4 cases behaved as expected.

==================================================
REQUIRED OUTPUTS
==================================================

Provide:
1. Verdict
2. Changed-files summary
3. Checklist confirming:
   - mixed V2/V3 case now succeeds
   - no-profit case now fails for the correct reason
   - case naming is aligned
   - attribution remains honest
   - no live trading logic added
4. Exact outputs for all commands above
5. A short walkthrough describing:
   - what caused Case 4 to fail and how it was fixed
   - how the no-profit case was separated from slippage
   - what remains deferred to the next phase

Do not go beyond this scope.