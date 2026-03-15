Implement Phase 10 on a dedicated branch.

Suggested branch name:
phase-10-preflight-nonce-broadcast

Before doing any code work:
1. Ensure work is being done on branch `phase-10-preflight-nonce-broadcast`
   - if the branch does not exist, create it from current main
   - if it exists, switch to it
2. Do NOT work directly on main

Goal:
Build the provider-backed preflight, nonce-sync, and safe broadcast plumbing on top of the existing Phase 8/9 execution-plan + signing/submission foundation.

Scope:
- provider-backed nonce fetching
- preflight validation via eth_call / estimate_gas (or honest equivalents through provider APIs)
- transaction request hydration from ExecutionPlan
- safe broadcast path, disabled by default
- metrics and tests for nonce/preflight/submission
- no flash loans yet
- no profitable live trading assumptions
- no mempool/PGA tactics yet

This phase is NOT about:
- flash loans
- private RPC / Flashbots / builder integrations
- replacement spam
- production high-frequency trading
- EV learning layer
- strategy changes

==================================================
PHASE 10 OBJECTIVE
==================================================

By the end of this phase, the system should be able to:

1. Take an ExecutionPlan
2. Fetch a real nonce from a provider
3. Build a provider-ready transaction request
4. Run a real preflight check before any broadcast:
   - eth_call / equivalent validation
   - gas estimate / equivalent validation
5. Produce a structured PreflightResult / SubmissionResult
6. Support a safe, explicitly gated broadcast mode
7. Keep dry-run and non-broadcast paths working

This phase is about:
- provider-backed transaction safety
- nonce correctness
- preflight correctness

This phase is NOT about:
- actual profitable live trading
- flash loans
- race tactics

==================================================
WORK AREAS
==================================================

Likely files/crates to work in:
- crates/arb_types
- crates/arb_config
- crates/arb_execute
- crates/arb_metrics
- bin/arb_daemon
- docs if needed

Do not touch unrelated strategy/state/route code unless absolutely necessary.

==================================================
PART 1 — CONFIG / SAFETY GATES
==================================================

In `crates/arb_config`, add or finalize provider-backed execution safety config, for example:
- RPC_HTTP_URL (if not already available in the right place)
- CHAIN_ID
- EXECUTOR_CONTRACT_ADDRESS
- ENABLE_BROADCAST
- DRY_RUN_ONLY
- REQUIRE_PREFLIGHT
- REQUIRE_GAS_ESTIMATE
- REQUIRE_ETH_CALL
- GAS_LIMIT_OVERRIDE (optional)
- MAX_FEE_PER_GAS
- MAX_PRIORITY_FEE_PER_GAS
- NONCE_MODE (simple/manual/remote if useful)
- ALLOWED_BROADCAST_CHAIN_ID (optional safety guard)

Rules:
- safe defaults only
- broadcasting disabled by default
- secrets only from environment
- no secret logging
- no real values in tracked files
- `.env.example` must include any new variable names

==================================================
PART 2 — TYPE SYSTEM
==================================================

Add minimal types as needed, such as:
- PreflightRequest
- PreflightResult
- PreflightFailureReason
- NonceFetchResult
- BroadcastResult
- TxEnvelope or ProviderTxRequest
- BroadcastPolicy

Keep them minimal and serializable where useful.
Do not add flash-loan-specific types yet.

==================================================
PART 3 — NONCE FETCHING
==================================================

In `crates/arb_execute`:
Implement provider-backed nonce fetching.

Requirements:
- fetch current account nonce from provider
- support a deterministic nonce path for transaction building
- return structured errors on provider failure
- do not overengineer parallel nonce reservation yet
- keep it safe and simple

Tests:
- unit tests for nonce fetch handling via provider abstraction or mock
- explicit failure path test

==================================================
PART 4 — PREFLIGHT VALIDATION
==================================================

Implement provider-backed preflight for built transactions.

Requirements:
- before broadcast, the system should be able to run:
  - eth_call (or provider equivalent) against the transaction request
  - gas estimate (or provider equivalent)
- capture structured success/failure
- map failures into PreflightFailureReason
- if preflight is required by config and fails, do not broadcast

Important:
- no fake gas estimates
- if gas estimate cannot be obtained honestly, return a structured failure or respect an explicit override
- dry-run mode should still work without broadcasting

Tests:
- unit tests / mocked provider tests for:
  - successful preflight
  - failed eth_call
  - failed gas estimate
  - preflight-required gating

==================================================
PART 5 — SAFE BROADCAST PATH
==================================================

Build a safe broadcast path on top of the existing signer/submitter.

Requirements:
- broadcasting must remain disabled by default
- only broadcast if all of the following are satisfied:
  - config allows it
  - chain safety checks pass
  - preflight passes (if required)
  - signed transaction is available
- return structured BroadcastResult / SubmissionResult
- keep dry-run mode intact

Do NOT add:
- retry loops
- replacement transactions
- mempool tactics
- private relay logic

This is a correctness phase, not a speed phase.

==================================================
PART 6 — METRICS
==================================================

Add or update metrics honestly, for example:
- arb_nonce_fetch_total
- arb_nonce_fetch_failures_total
- arb_preflight_total
- arb_preflight_success_total
- arb_preflight_failed_total
- arb_preflight_eth_call_failed_total
- arb_preflight_gas_estimate_failed_total
- arb_submission_attempts_total
- arb_submission_signed_total
- arb_submission_broadcast_total
- arb_submission_failed_total
- arb_submission_dry_run_total

Rules:
- do not fake metrics
- reflect disabled broadcast honestly
- do not mark broadcast success unless it really happened

==================================================
PART 7 — DAEMON INTEGRATION
==================================================

In `bin/arb_daemon`:
Wire the current flow into the new preflight/broadcast safety path.

Expected Phase 10 daemon flow:
provider -> ingest -> state -> graph -> filter -> simulation -> execution plan -> preflight -> dry-run/broadcast decision

Requirements:
- no flash loans yet
- no multi-wallet fleet yet
- no live trading loop assumptions beyond safe gated plumbing
- preserve graceful shutdown

If useful, add clear logging like:
- candidate validated
- execution plan built
- nonce fetched
- preflight passed/failed
- broadcast skipped because dry-run
- broadcast skipped because config gate
- broadcast attempted

==================================================
PART 8 — TESTING
==================================================

Add tests for:
1. config parsing for new execution safety fields
2. nonce fetch success/failure
3. preflight success/failure
4. execution-plan -> provider tx request conversion
5. dry-run path remains functional
6. safe broadcast gating:
   - disabled by default
   - blocked on preflight failure
   - blocked on wrong chain if safety checks exist

If practical, add one local-only integration path using a mock provider abstraction or local environment.
Do not require real mainnet broadcasting for tests.

Validation should include:
- cargo check --workspace
- cargo test --workspace

If Solidity/contracts are touched:
- forge build
- forge test
Otherwise do not claim unnecessary Foundry validation.

==================================================
PART 9 — DOCUMENTATION HONESTY
==================================================

Update docs/checklists/walkthrough so they clearly state:

Real after Phase 10:
- provider-backed nonce fetching
- provider-backed preflight
- safe broadcast gating
- dry-run and signed submission plumbing

Still deferred:
- flash loans
- private relay / builder paths
- nonce replacement strategy
- high-frequency live execution
- EV learning layer
- production fee optimization

Do not oversell beyond what is implemented.

==================================================
SOURCE-OF-TRUTH OUTPUTS REQUIRED
==================================================

At the end, include these exact commands and outputs:

1. Git identity:
- git fetch origin
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-10-preflight-nonce-broadcast
- git status --short
- git log --oneline --decorate -5

2. Grep proofs:
- git grep -n 'RPC_HTTP_URL|ENABLE_BROADCAST|DRY_RUN_ONLY|REQUIRE_PREFLIGHT|REQUIRE_GAS_ESTIMATE|REQUIRE_ETH_CALL' -- crates/arb_config/src/lib.rs .env.example
- git grep -n 'nonce|preflight|eth_call|estimate_gas|broadcast' -- crates/arb_execute bin/arb_daemon
- git grep -n 'arb_nonce_fetch_total|arb_preflight_total|arb_preflight_success_total|arb_preflight_failed_total|arb_submission_broadcast_total|arb_submission_dry_run_total' -- crates/arb_metrics/src/lib.rs

3. Validation outputs:
- cargo check --workspace
- cargo test --workspace
- forge build
- forge test
(Only if contracts were touched; otherwise report honestly that no contract changes required Foundry rerun)

==================================================
REQUIRED OUTPUTS
==================================================

Provide:

1. A verdict:
- fully working
- working with known limitations
- blocked (and why)

2. A checklist confirming:
- nonce fetching added
- provider-backed preflight added
- safe broadcast gating added
- dry-run path still works
- metrics added
- no flash-loan/live-trading logic added

3. Changed-files summary

4. A walkthrough artifact describing:
- how ExecutionPlan becomes a provider-backed tx request
- how nonce/preflight/broadcast flow works
- how safety gating works
- what remains deferred to the next phase

5. The source-of-truth outputs listed above

Do not go beyond Phase 10.