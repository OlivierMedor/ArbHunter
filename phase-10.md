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


---- updates ----

Do a final Phase 10 merge-readiness pass on the EXISTING branch `phase-10-preflight-nonce-broadcast`.

Do NOT create a new branch.
Do NOT add flash loans, live trading, mempool tactics, PGA logic, private relay logic, or execution strategy changes.
Do NOT expand scope beyond the concrete Phase 10 preflight/config wiring issues below.

Goal:
Make Phase 10 merge-ready by ensuring the new config safety flags actually control runtime behavior, and by providing source-of-truth outputs proving the branch is clean and passing.

==================================================
FIX 1 — WIRE PRELIGHT FLAGS INTO REAL BEHAVIOR
==================================================

Current problem:
`arb_config` exposes:
- require_preflight
- require_gas_estimate
- require_eth_call

But the actual submitter/preflight path appears to still behave like:
- one boolean gate for "preflight yes/no"
- and PreflightChecker always runs both eth_call and estimate_gas

Required fix:
Make runtime behavior honor the config fields explicitly.

Expected behavior:
1. If require_preflight = false
   - no preflight checks are run
   - submission path may continue (subject to other safety gates)

2. If require_preflight = true
   - preflight runs
   - but it must respect the two sub-flags below

3. If require_eth_call = true
   - eth_call preflight must run
4. If require_eth_call = false
   - eth_call preflight must be skipped honestly

5. If require_gas_estimate = true
   - gas estimate preflight must run
6. If require_gas_estimate = false
   - gas estimate preflight must be skipped honestly

Important:
- do not fake results
- do not silently run disabled checks
- do not silently ignore enabled checks

==================================================
FIX 2 — STRUCTURED PREFLIGHT RESULT HONESTY
==================================================

Current problem:
If preflight has multiple sub-checks, the result needs to clearly distinguish:
- success
- failed check
- skipped check

Required fix:
Ensure the preflight result structure or result mapping can honestly represent:
- eth_call passed / failed / skipped
- gas estimate passed / failed / skipped
- overall preflight outcome

If your current types are too coarse, add the minimal type changes needed.
Keep them small and honest.
Do not overengineer.

==================================================
FIX 3 — SUBMITTER / EXECUTION FLOW ALIGNMENT
==================================================

In the submitter or execution path:
- thread the new config values from arb_config into the submitter/preflight layer
- ensure dry-run mode still works
- ensure broadcast gating still works
- ensure broadcast is still disabled by default
- ensure failed required preflight blocks broadcast
- ensure skipped preflight due to config is treated honestly, not as success-by-default unless policy allows it

No live trading logic.
No flash-loan logic.

==================================================
FIX 4 — TESTS
==================================================

Add or update tests for these cases:

1. require_preflight = false
   - no preflight checks are required
   - dry-run/submission path behaves consistently

2. require_preflight = true, require_eth_call = true, require_gas_estimate = true
   - both checks run

3. require_preflight = true, require_eth_call = true, require_gas_estimate = false
   - only eth_call runs
   - gas estimate is skipped

4. require_preflight = true, require_eth_call = false, require_gas_estimate = true
   - only gas estimate runs
   - eth_call is skipped

5. preflight failure blocks broadcast when required
6. dry-run path still works and remains safe by default

Tests can be unit tests or local/mock-provider tests.
Do not require real mainnet broadcasting.

==================================================
FIX 5 — DOCUMENTATION HONESTY
==================================================

Update walkthrough/checklist/docs so they accurately state:
- which preflight checks exist
- which config flags control them
- how skipped vs failed vs passed checks are represented
- that broadcast remains disabled by default unless explicitly enabled

Do not oversell beyond what is implemented.

==================================================
VALIDATION REQUIRED
==================================================

Run and report:

1. Source of truth:
- git fetch origin
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-10-preflight-nonce-broadcast
- git status --short
- git log --oneline --decorate -5

2. Proof commands:
- git grep -n 'require_preflight|require_gas_estimate|require_eth_call' -- crates/arb_config crates/arb_execute bin/
- git grep -n 'eth_call|estimate_gas|Preflight' -- crates/arb_execute bin/
- git grep -n 'DryRun|Broadcast|Skipped' -- crates/arb_execute

3. Build/test:
- cargo check --workspace
- cargo test --workspace

4. If contracts were NOT changed:
- explicitly report that Foundry validation is unchanged from the previous phase and was not required for this pass

==================================================
REQUIRED OUTPUTS
==================================================

Provide:

1. Verdict:
- fully working
- working with known limitations
- blocked (and why)

2. Changed-files summary

3. Checklist confirming:
- preflight flags are actually wired into behavior
- eth_call can be independently enabled/disabled
- gas estimate can be independently enabled/disabled
- skipped vs failed vs passed states are represented honestly
- dry-run still works
- broadcast remains safely gated
- no flash-loan/live-trading logic added

4. Exact outputs for all source-of-truth and proof commands above

5. A short walkthrough describing:
- how config controls preflight now
- how the preflight result model works
- how the safe broadcast path behaves
- what remains deferred to the next phase

Do not go beyond this scope.


---- update 2 ----


Do a final Phase 10 test-fix pass on the EXISTING branch `phase-10-preflight-nonce-broadcast`.

Do NOT create a new branch.
Do NOT add any new features.
Do NOT add flash loans, live trading, mempool tactics, or PGA logic.
Do NOT expand scope beyond fixing the failing workspace test and preserving Phase 10 behavior.

Goal:
Make Phase 10 merge-ready by fixing the failing `cargo test --workspace` issue in `bin/arb_daemon/src/main.rs`.

==================================================
FIX 1 — REPAIR arb_daemon TEST WALLET CREATION
==================================================

Current problem:
`cargo test --workspace` fails because `bin/arb_daemon/src/main.rs` references:
- `Wallet::from_random()`

but `arb_execute::Wallet` does not provide that function.

Required fix:
- update the test in `bin/arb_daemon/src/main.rs` so it constructs a valid test wallet using the actual available wallet APIs
- do NOT add a fake helper just for convenience unless truly necessary
- prefer one of:
  1. construct a `PrivateKeySigner` from a known deterministic test private key and wrap it in `Wallet`
  2. use `Wallet::from_env()` with temporary test env setup if that is cleaner and safe
- keep the test deterministic and local
- no real secrets
- no network calls required for this test

==================================================
FIX 2 — DO NOT CHANGE PHASE 10 SCOPE
==================================================

Do NOT add:
- new execution features
- new signing features
- broadcast changes
- strategy logic
- flash loans
- extra plumbing unrelated to the failing test

Only fix the test and any tiny supporting code required to make the workspace test suite pass.

==================================================
VALIDATION REQUIRED
==================================================

Run and report:

1. Source of truth:
- git fetch origin
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-10-preflight-nonce-broadcast
- git status --short
- git log --oneline --decorate -5

2. Proof commands:
- git grep -n 'from_random' -- bin/arb_daemon crates/arb_execute
- git grep -n 'from_env|PrivateKeySigner|Wallet {' -- bin/arb_daemon crates/arb_execute

3. Build/test:
- cargo check --workspace
- cargo test --workspace

==================================================
REQUIRED OUTPUTS
==================================================

Provide:
1. Verdict
2. Changed-files summary
3. Checklist confirming:
   - failing arb_daemon test fixed
   - cargo check passes
   - cargo test passes
   - no new execution logic added
4. Exact outputs for all source-of-truth and proof commands above
5. A short walkthrough describing:
   - how the test wallet is now created
   - why the fix is deterministic and safe
   - what remains deferred to the next phase

Do not go beyond this scope.