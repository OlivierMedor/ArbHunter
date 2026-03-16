Implement Phase 11 on a dedicated branch.

Suggested branch name:
phase-11-flashloan-atomic-path

Before doing any code work:
1. Ensure work is being done on branch `phase-11-flashloan-atomic-path`
   - if the branch does not exist, create it from current main
   - if it exists, switch to it
2. Do NOT work directly on main

Goal:
Add flash-loan-capable atomic execution support on top of the existing execution-plan + contract foundation, while keeping everything safe, testable, and local-first.

Scope:
- extend execution-plan types to support flash-loan funding
- extend the Solidity executor contract for atomic flash-loan-assisted execution
- add repayment/profit/slippage guards
- add Rust-side plan building / ABI alignment for flash-loan paths
- add Foundry tests for atomic success + failure cases
- no live mainnet trading rollout yet
- no mempool/PGA tactics
- no private relay logic

This phase is NOT about:
- production live trading
- aggressive broadcast automation
- EV learning
- multi-wallet fleet logic
- builder/relay integration

==================================================
PHASE 11 OBJECTIVE
==================================================

By the end of this phase, the system should be able to:

1. Represent a flash-loan-backed atomic execution plan in Rust
2. Encode that plan into the contract call path
3. Support a minimal Solidity atomic execution path:
   - receive flash-loaned funds
   - execute the route
   - repay lender
   - enforce min-out / slippage guard
   - enforce profit guard
4. Prove the full path with local/Foundry tests
5. Keep everything disabled from real trading by default unless explicitly tested in a safe environment

This phase is about:
- atomic execution path foundation
- flash-loan-capable contract interface
- repayment safety
- local correctness

This phase is NOT about:
- profitable mainnet deployment
- execution race tactics
- production funding strategy

==================================================
WORK AREAS
==================================================

Likely files/crates to work in:
- crates/arb_types
- crates/arb_execute
- contracts/
- possibly crates/arb_config
- docs if needed

Try to keep changes narrowly focused.

==================================================
PART 1 — TYPE SYSTEM
==================================================

Add minimal flash-loan-related execution types as needed.

Expected direction:
- FlashLoanSpec
- FlashLoanProviderKind
- AtomicExecutionPlan
- AtomicExecutionLeg (if distinct from ExecutionLeg)
- RepaymentGuard
- ProfitGuard
- AtomicExecutionFailureReason

Requirements:
- keep them minimal and serializable where useful
- do not add live trading policy logic
- do not overengineer provider abstraction if one concrete path is enough for this phase

==================================================
PART 2 — RUST-SIDE PLAN BUILDER
==================================================

In `crates/arb_execute`:
Extend the planner/builder path so a validated candidate can become:
- normal execution plan
- or flash-loan-backed atomic execution plan

Requirements:
1. Preserve deterministic route order
2. Include:
   - loan asset
   - loan amount
   - route legs
   - expected output
   - repayment expectations
   - min-out / slippage guard
   - expected profit
3. If a plan cannot be safely represented for the flash-loan path, return a structured error
4. Keep the current non-flash execution plan path intact

No broadcast changes required beyond compatibility.
No live provider logic changes unless absolutely needed.

==================================================
PART 3 — SOLIDITY CONTRACT FOUNDATION
==================================================

In `contracts/`:
Extend the executor contract (or add a clearly named new contract) to support an atomic flash-loan execution path.

Expected direction:
- authorized caller model remains
- one explicit entrypoint for atomic execution
- internal callback/repayment path
- balance-before / balance-after profit guard
- min-out / slippage checks
- repayment must be enforced before successful completion

Important:
- keep the contract minimal and testable
- do not put strategy logic in the contract
- do not do route discovery in the contract
- do not add unrelated features

If a concrete flash-loan provider interface is used in tests, keep it isolated and mocked if needed.
A mock lender is acceptable for local tests if that is the cleanest safe approach.

==================================================
PART 4 — ABI / PLAN ALIGNMENT
==================================================

Ensure Rust and Solidity stay aligned.

Required:
- execution-plan encoding must match contract expectations
- ABI/parameter layout must be explicitly tested
- if bindings are generated, keep them minimal
- if bindings are deferred, document that honestly

At minimum prove:
- Rust atomic execution plan -> encoded calldata/params
- Solidity side accepts that structure and executes the expected path in tests

==================================================
PART 5 — FOUNDRY TESTS
==================================================

Add/expand Foundry tests for the flash-loan atomic path.

Required tests:
1. unauthorized caller revert
2. slippage/minOut revert
3. insufficient repayment revert
4. no-profit / negative-profit revert
5. successful atomic execution path
6. ABI/parameter alignment test if helpful

Use mocks where appropriate.
The goal is correctness, not protocol realism at all costs.

==================================================
PART 6 — RUST TESTS
==================================================

Add Rust-side tests for:
1. validated candidate -> flash-loan atomic execution plan success
2. unsupported/invalid candidate -> structured failure
3. atomic plan encoding alignment
4. repayment/profit guard fields being populated correctly

Keep tests deterministic and local.

==================================================
PART 7 — CONFIG / SAFETY
==================================================

Only add config if necessary.

If you add config, keep it minimal, for example:
- ENABLE_FLASHLOAN_PATH
- FLASHLOAN_PROVIDER_KIND
- default-safe values

Rules:
- no real secrets beyond what already exists
- no mainnet addresses hardcoded unless clearly documented and safe
- if provider selection is still mock/local-only, say so honestly

==================================================
PART 8 — DOCUMENTATION HONESTY
==================================================

Update docs/checklists/walkthrough so they clearly state:

Real after Phase 11:
- flash-loan-capable atomic execution plan types
- contract callback / repayment foundation
- Rust/contract ABI alignment
- local atomic success/failure test coverage

Still deferred:
- live flash-loan provider rollout
- real mainnet execution
- provider-specific operational hardening
- MEV protection / private relays
- production broadcast policy

Do not oversell beyond what is implemented.

==================================================
VALIDATION REQUIRED
==================================================

Run and report:
- cargo check --workspace
- cargo test --workspace
- forge build
- forge test

If any contract/tool issue blocks Foundry validation, report it honestly.
Do not fake success.

==================================================
SOURCE-OF-TRUTH OUTPUTS REQUIRED
==================================================

At the end, include these exact commands and outputs:

1. Git identity:
- git fetch origin
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-11-flashloan-atomic-path
- git status --short
- git log --oneline --decorate -5

2. Grep proofs:
- git grep -n 'FlashLoanSpec|AtomicExecutionPlan|RepaymentGuard|ProfitGuard|AtomicExecutionFailureReason' -- crates/
- git grep -n 'flash|repay|minOut|slippage|profit' -- contracts/
- git grep -n 'executePlan|flash' -- crates/arb_execute contracts/

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
- flash-loan-capable plan types added
- Rust planner supports atomic execution plans
- Solidity contract foundation supports atomic path
- repayment/slippage/profit guards added
- Rust tests added
- Foundry tests added
- no live trading logic added

3. Changed-files summary

4. A walkthrough artifact describing:
- how a validated candidate becomes an atomic execution plan
- how the contract callback/repayment path works
- how safety guards work
- what remains deferred to the next phase

5. The source-of-truth outputs listed above

Do not go beyond Phase 11.