Implement Phase 7 on a dedicated branch.

Suggested branch name:
phase-7-pending-sim-validation

Before doing any code work:
1. Ensure work is being done on branch `phase-7-pending-sim-validation`
   - if the branch does not exist, create it from current main
   - if it exists, switch to it
2. Do NOT work directly on main

Goal:
Build the pending-state simulation and candidate validation layer on top of the existing route graph and candidate filter.

Scope:
- validate promoted candidates against pending-state / current-state simulation
- produce structured dry-run outcomes
- add simulation metrics and observability
- keep all logic pre-execution
- no transaction signing
- no submission
- no live trading

This phase is NOT about:
- execution
- flash loans
- wallet management
- PGA tuning
- live trade sending

==================================================
PHASE 7 OBJECTIVE
==================================================

By the end of this phase, the system should be able to:

1. Take promoted CandidateOpportunity values from Phase 6
2. Reconstruct the route in a simulation-friendly form
3. Simulate or validate the route against current/pending state
4. Produce a structured result such as:
   - success/fail
   - expected out
   - expected gas estimate if available
   - failure reason
   - confidence / validation result
5. Emit metrics and logs for simulation outcomes
6. Support replay-driven and local validation tests

This phase is about:
- candidate truth-checking
- simulation pipeline
- dry-run outcome recording

This phase is NOT about:
- sending transactions
- signing transactions
- execution contracts
- live arbitrage

==================================================
WORK AREAS
==================================================

Likely files/crates to work in:
- crates/arb_types
- crates/arb_route
- crates/arb_filter
- crates/arb_sim
- crates/arb_metrics
- crates/arb_config
- bin/arb_daemon
- fixtures/
- docs if needed

==================================================
PART 1 — TYPE SYSTEM
==================================================

Add minimal simulation-related shared types as needed, such as:
- SimulatedRoute
- SimulationRequest
- SimulationResult
- SimulationFailureReason
- CandidateValidationResult
- SimOutcomeStatus

Keep them minimal and serializable where useful.
Do not add execution payload types yet.

==================================================
PART 2 — arb_sim FOUNDATION
==================================================

Implement `crates/arb_sim` as the simulation/validation crate.

Requirements:
1. Accept promoted candidates from Phase 6
2. Reconstruct a simulation request from RoutePath / RouteLeg data
3. Support at least one local simulation/validation path:
   - deterministic local validation against current state and quote primitives
4. If current architecture already supports richer RPC-based pending checks, add them carefully, but keep this phase execution-free
5. Produce structured SimulationResult values with:
   - status
   - expected_output
   - expected_profit
   - optional gas estimate if available
   - failure reason

Important:
- No external router APIs
- No execution
- No flash loans
- No wallet/signing logic

==================================================
PART 3 — CANDIDATE VALIDATION FLOW
==================================================

Integrate the Phase 6 promoted candidates into the new simulation lane.

Expected daemon flow now:
provider -> ingest -> state -> graph -> candidate filter -> simulation -> metrics/logging

Requirements:
- only promoted candidates should be simulated
- simulation failures should be categorized
- simulation successes should be surfaced cleanly
- keep all this off the execution path because there is no execution path yet

==================================================
PART 4 — METRICS
==================================================

Add or update metrics honestly, for example:
- arb_simulations_total
- arb_simulations_success_total
- arb_simulations_failed_total
- arb_simulation_failures_by_reason
- arb_candidates_validated_total
- arb_candidates_validation_success_total
- arb_candidates_validation_failed_total

If gas estimation is added honestly:
- arb_simulated_gas_estimate_total or similar

Do not fake metrics.

==================================================
PART 5 — DAEMON INTEGRATION
==================================================

In `bin/arb_daemon`:
- wire promoted candidates into arb_sim
- log simulation results
- keep graceful shutdown intact
- no execution, no wallet, no transaction sending

If useful, add a lightweight dev mode that prints:
- top promoted candidates
- simulation pass/fail
- expected profit

==================================================
PART 6 — FIXTURES / TESTS
==================================================

Add tests that prove the simulation layer works.

Required:
1. candidate -> simulation request conversion test
2. successful validation test for a positive candidate
3. failed validation test for an invalid/stale candidate
4. replay-driven end-to-end test:
   ingest -> state -> graph -> filter -> simulation

At least one test should prove:
- a candidate is promoted
- then simulation marks it valid
- then a structured SimulationResult is produced

Workspace must pass:
- cargo check --workspace
- cargo test --workspace

==================================================
PART 7 — DOCUMENTATION HONESTY
==================================================

Update docs/checklists/walkthrough so they clearly state:

Real after Phase 7:
- local graph and candidate generation
- promoted candidate validation
- simulation/dry-run results
- no execution

Still deferred:
- transaction building
- signing
- flash loans
- execution
- live trading
- EV learning layer

Do not oversell beyond what is implemented.

==================================================
VALIDATION REQUIRED
==================================================

Run and report:
- cargo check --workspace
- cargo test --workspace
- docker compose config (only if touched)

==================================================
SOURCE-OF-TRUTH OUTPUTS REQUIRED
==================================================

At the end, include these exact commands and outputs:

1. Git identity:
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-7-pending-sim-validation
- git status --short
- git log --oneline --decorate -5

2. Grep proofs:
- git grep -n 'SimulationResult\|SimulationRequest\|CandidateValidationResult\|SimulationFailureReason' -- crates/
- git grep -n 'arb_simulations_total\|arb_simulations_success_total\|arb_simulations_failed_total\|arb_candidates_validated_total' -- crates/
- git grep -n 'simulate\|validation' -- crates/arb_sim crates/arb_filter bin/arb_daemon

3. Validation outputs:
- cargo check --workspace
- cargo test --workspace

==================================================
REQUIRED OUTPUTS
==================================================

Provide:

1. A verdict:
- fully working
- working with known limitations
- blocked (and why)

2. A checklist confirming:
- simulation crate foundation added
- promoted candidates flow into simulation
- structured simulation results added
- simulation metrics added
- replay tests added
- no execution logic added

3. Changed-files summary

4. A walkthrough artifact describing:
- how promoted candidates are validated
- how simulation results are represented
- what remains deferred to the next phase

5. The source-of-truth outputs listed above

Do not go beyond Phase 7.


----- update 1 ----

Do a final Phase 7 merge-readiness pass on the EXISTING branch `phase-7-pending-sim-validation`.

Do NOT create a new branch.
Do NOT add execution, signing, flash loans, or transaction submission logic.
Do NOT expand scope beyond the 3 issues below.

Goal:
Make Phase 7 merge-ready by removing fake gas reporting and strengthening simulation validation proof.

==================================================
FIX 1 — REMOVE FAKE GAS ESTIMATE
==================================================

Current problem:
The simulator still returns a hardcoded dummy gas estimate:
expected_gas_used: Some(150_000)

Required fix:
Choose ONE:
PATH A (preferred):
- replace the fake gas estimate with a real estimate if there is already a clean, honest way to do so in this phase

PATH B:
- set expected_gas_used to None
- update docs/checklist/walkthrough so they clearly state gas estimation is deferred

Do NOT leave a fake hardcoded gas estimate in merge-ready code.

==================================================
FIX 2 — ADD A POSITIVE SIMULATION TEST
==================================================

Current problem:
The visible arb_sim tests are too weak.

Required fix:
Add at least one test that proves:
- a valid candidate enters the simulator
- simulation returns success
- SimulationResult is populated with expected_amount_out and expected_profit
- CandidateValidationResult.is_valid is true

Keep it deterministic and local.

==================================================
FIX 3 — ADD / PROVE REPLAY-DRIVEN END-TO-END VALIDATION
==================================================

Current problem:
Phase 7 requires more than just unit tests inside arb_sim.

Required fix:
Add or prove at least one replay-driven test path that covers:
ingest -> state -> graph -> filter -> simulation

This can be:
- a real test in code, OR
- an existing test path plus exact proof outputs

But it must be explicit and honest.

==================================================
VALIDATION REQUIRED
==================================================

Run and report:

1. Source of truth:
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-7-pending-sim-validation
- git status --short
- git log --oneline --decorate -5

2. Proof commands:
- git grep -n 'expected_gas_used' -- crates/arb_sim
- git grep -n 'mock dummy gas estimate' -- crates/arb_sim
- git grep -n 'test_.*simulate\|test_.*validation\|test_.*replay' -- crates/arb_sim crates/arb_filter crates/arb_route bin/

3. Build/test:
- cargo check --workspace
- cargo test --workspace

==================================================
REQUIRED OUTPUTS
==================================================

Provide:
1. Which gas path was chosen:
- PATH A = real gas estimate
- PATH B = no gas estimate / deferred honestly

2. Changed-files summary

3. Checklist confirming:
- fake gas estimate removed
- positive simulation test added
- replay-driven validation test/proof added
- no execution logic added

4. Exact outputs for all source-of-truth and proof commands above

5. A short walkthrough describing:
- how successful validation is now proven
- whether gas estimation is real or deferred
- what remains deferred to the next phase

Do not go beyond this scope.