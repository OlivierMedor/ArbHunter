Implement Phase 8 on a dedicated branch.

Suggested branch name:
phase-8-execution-plan-contract

Before doing any code work:
1. Ensure work is being done on branch `phase-8-execution-plan-contract`
   - if the branch does not exist, create it from current main
   - if it exists, switch to it
2. Do NOT work directly on main

Goal:
Build the execution-plan layer and minimal contract interface foundation so validated candidates can be transformed into deterministic on-chain execution plans.

Scope:
- define execution-plan types
- build deterministic plan generation from validated candidates
- add a minimal Solidity executor contract foundation
- add Rust-side ABI/plan compatibility
- add Foundry tests and Rust tests
- no transaction signing
- no transaction submission
- no flash-loan implementation yet
- no live trading

This phase is NOT about:
- sending transactions
- private keys
- nonce management
- PGA tuning
- flash-loan callbacks
- live execution

==================================================
PHASE 8 OBJECTIVE
==================================================

By the end of this phase, the system should be able to:

1. Take a validated candidate from Phase 7
2. Convert it into a deterministic ExecutionPlan
3. Represent route steps in a contract-compatible way
4. Build calldata or encoded plan payloads for a minimal executor contract
5. Validate that Rust-side planning and Solidity-side interfaces are aligned
6. Prove correctness with unit tests and Foundry tests

This phase is about:
- execution planning
- contract interface alignment
- calldata / plan serialization

This phase is NOT about:
- actually broadcasting transactions
- using flash loans
- live arbitrage

==================================================
WORK AREAS
==================================================

Likely files/crates to work in:
- crates/arb_types
- crates/arb_sim
- crates/arb_execute
- contracts/
- possibly crates/arb_config
- docs if needed

==================================================
PART 1 — TYPE SYSTEM
==================================================

Add minimal execution-related types as needed.

Expected types/directions:
- ExecutionPlan
- ExecutionLeg
- ExecutionPath
- SlippageGuard
- MinOutConstraint
- ExpectedOutcome
- PlanBuildFailureReason
- Optional FlashLoanSpec type placeholder (type only, no implementation required)

Keep them minimal and serializable where useful.

Do NOT add signing/submission types yet.

==================================================
PART 2 — RUST EXECUTION PLAN BUILDER
==================================================

In `crates/arb_execute`:
Build the planner that converts:
- CandidateOpportunity
- CandidateValidationResult / SimulationResult
into:
- ExecutionPlan

Requirements:
1. Only validated candidates can become execution plans
2. Preserve route order exactly
3. Include:
   - input asset
   - output asset
   - path steps
   - amount in
   - expected amount out
   - min out guard
   - expected profit
4. If the route family is unsupported for execution planning, return a structured failure reason

No transaction sending.
No wallet logic.
No flash-loan implementation yet.

==================================================
PART 3 — MINIMAL SOLIDITY EXECUTOR FOUNDATION
==================================================

In `contracts/`:
Add a minimal executor contract foundation.

Expected direction:
- authorized caller model
- deterministic execution entrypoint
- balance-before / balance-after profit guard
- slippage/minOut guard support
- no strategy logic in contract
- no routing discovery in contract
- no live flash-loan logic yet

This contract can be a minimal foundation/skeleton for the later SwitchHitter-style path, but keep it small and honest.

Requirements:
- compile with Foundry
- tested with Foundry
- no privileged assumptions beyond authorized caller control

==================================================
PART 4 — ABI / PLAN ALIGNMENT
==================================================

Ensure Rust plan generation and Solidity interface expectations align.

Required:
- plan serialization/encoding strategy is explicit
- ABI compatibility is tested or proven
- if bindings are generated, keep them minimal
- if bindings are deferred, document that honestly

At minimum, prove:
- Rust plan -> encoded calldata/parameters
- Solidity contract interface accepts the expected structure

==================================================
PART 5 — TESTING
==================================================

Add both Rust and Foundry tests.

Required Rust-side tests:
1. validated candidate -> execution plan success
2. unsupported candidate -> structured plan build failure
3. minOut / slippage guard values are encoded correctly

Required Solidity / Foundry tests:
1. contract compiles
2. authorized caller restriction works
3. execution entrypoint rejects invalid/slippage-failing inputs
4. balance-before / balance-after guard works in a controlled test environment

Do not add live execution tests.
Do not add wallet/private key logic.

Workspace/build validation should include:
- cargo check --workspace
- cargo test --workspace
- forge build
- forge test

==================================================
PART 6 — DOCUMENTATION HONESTY
==================================================

Update docs/checklists/walkthrough so they clearly state:

Real after Phase 8:
- validated candidates can become execution plans
- a minimal executor contract foundation exists
- Rust/contract interface alignment exists
- no live transaction sending yet

Still deferred:
- signing
- nonce management
- transaction submission
- flash loans
- live execution
- PGA tuning

Do not oversell beyond what is implemented.

==================================================
SOURCE-OF-TRUTH OUTPUTS REQUIRED
==================================================

At the end, include these exact commands and outputs:

1. Git identity:
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-8-execution-plan-contract
- git status --short
- git log --oneline --decorate -5

2. Grep proofs:
- git grep -n 'ExecutionPlan\|ExecutionLeg\|ExpectedOutcome\|PlanBuildFailureReason' -- crates/
- git grep -n 'authorized\|balance_before\|balance_after\|minOut\|slippage' -- contracts/
- git grep -n 'simulate\|execution plan\|plan build' -- crates/arb_execute crates/arb_sim

3. Validation outputs:
- cargo check --workspace
- cargo test --workspace
- forge build
- forge test

==================================================
REQUIRED OUTPUTS
==================================================

Provide:

1. A verdict:
- fully working
- working with known limitations
- blocked (and why)

2. A checklist confirming:
- execution plan types added
- validated candidates flow into execution plans
- minimal executor contract added
- Rust/contract interface aligned
- Rust tests added
- Foundry tests added
- no signing/submission/live execution logic added

3. Changed-files summary

4. A walkthrough artifact describing:
- how validated candidates become execution plans
- what the contract foundation does
- how ABI/plan alignment is verified
- what remains deferred to the next phase

5. The source-of-truth outputs listed above

Do not go beyond Phase 8.

---- updates ----

Do a final Phase 8 merge-readiness cleanup pass on the EXISTING branch `phase-8-execution-plan-contract`.

Do NOT create a new branch.
Do NOT add any new features.
Do NOT change execution-plan or contract logic unless absolutely necessary for cleanup.
Do NOT expand scope beyond repo hygiene + documentation honesty.

Goal:
Make Phase 8 merge-ready by removing tracked artifact/tooling files that do not belong on main.

==================================================
REMOVE TRACKED ARTIFACT / TOOL FILES
==================================================

Remove these from git tracking if present:
- check_err.json
- foundry.zip
- foundry_bin/anvil.exe
- foundry_bin/cast.exe
- foundry_bin/chisel.exe
- foundry_bin/forge.exe
- verify_hash.rs

Update `.gitignore` if needed so they do not get tracked again.

Rules:
- Keep legitimate source files only
- Do not delete real project code
- Do not remove the contracts/ source or tests
- Do not remove docs that matter

==================================================
DOCUMENTATION HONESTY
==================================================

If any docs imply Foundry/tool binaries are bundled with the repo, correct that.
Prefer docs to assume:
- Foundry is installed locally by the developer
- the repo does not vendor binaries into main

==================================================
VALIDATION REQUIRED
==================================================

Run and report:

1. Source of truth:
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-8-execution-plan-contract
- git status --short
- git log --oneline --decorate -5

2. Proof of cleanup:
- git ls-tree -r --name-only origin/phase-8-execution-plan-contract | Select-String 'foundry_bin|foundry\.zip|check_err\.json|verify_hash\.rs'

This should return no output.

3. Build/test:
- cargo check --workspace
- cargo test --workspace
- forge build
- forge test

==================================================
REQUIRED OUTPUTS
==================================================

Provide:
1. Verdict
2. Changed-files summary
3. Checklist confirming:
   - artifact/tool files removed from tracking
   - docs adjusted if needed
   - no execution-plan or contract behavior changed
4. Exact outputs for all source-of-truth and proof commands above