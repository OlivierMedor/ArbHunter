Implement Phase 9 on a dedicated branch.

Suggested branch name:
phase-9-wallet-signing-submission

Before doing any code work:
1. Ensure work is being done on branch `phase-9-wallet-signing-submission`
   - if the branch does not exist, create it from current main
   - if it exists, switch to it
2. Do NOT work directly on main

Goal:
Build the wallet, signing, transaction-building, and submission plumbing needed to eventually execute validated execution plans.

Scope:
- wallet abstraction
- signer loading from environment
- nonce management
- EIP-1559 transaction request building
- contract call packaging from ExecutionPlan
- dry-run / disabled-by-default submission path
- metrics and tests for the submission pipeline

This phase is NOT about:
- profitable live trading
- flash loans
- PGA tuning
- mempool warfare
- multi-wallet hot fleet optimization
- autonomous live execution
- real mainnet trading by default

==================================================
PHASE 9 OBJECTIVE
==================================================

By the end of this phase, the system should be able to:

1. Load a wallet/signer safely from local environment variables
2. Build a transaction request from an ExecutionPlan
3. Populate nonce / chain / fee fields correctly
4. Encode contract calls for the Phase 8 executor contract
5. Support a dry-run / disabled-by-default submission mode
6. Expose metrics and logs for the signing/submission pipeline
7. Validate all of the above with unit tests and at least one local-only integration path

This phase is about:
- wallet plumbing
- signing plumbing
- submission plumbing

This phase is NOT about:
- actual profitable execution
- flash loans
- route discovery
- simulation logic (already exists)
- production live trading

==================================================
WORK AREAS
==================================================

Likely files/crates to work in:
- crates/arb_types
- crates/arb_config
- crates/arb_execute
- crates/arb_metrics
- bin/arb_daemon
- contracts/ (only if ABI/interface adjustments are needed)
- docs if needed

==================================================
PART 1 — CONFIG / ENV
==================================================

In `crates/arb_config`:
Add the minimum signing/submission config needed, for example:
- RPC_HTTP_URL (or current provider HTTP URL if reuse is cleaner)
- CHAIN_ID (reuse if already present)
- EXECUTOR_CONTRACT_ADDRESS
- SUBMISSION_MODE
- MAX_FEE_PER_GAS
- MAX_PRIORITY_FEE_PER_GAS
- GAS_LIMIT_OVERRIDE (optional)
- PRIVATE_KEY or SIGNER_PRIVATE_KEY (local env only, never tracked)
- ENABLE_BROADCAST (bool, default false)
- DRY_RUN_ONLY (bool, default true)

Rules:
- secrets must come only from environment
- never print private keys
- never commit secrets
- `.env.example` may include variable names only, no real values
- defaults must be conservative and safe

==================================================
PART 2 — TYPE SYSTEM
==================================================

Add minimal signing/submission-related shared types as needed, for example:
- WalletConfig
- SubmissionMode
- TxBuildRequest
- BuiltTransaction
- SubmissionResult
- SubmissionFailureReason
- NonceState
- FeePolicy

Keep them minimal and serializable where useful.
Do not add strategy logic.

==================================================
PART 3 — WALLET / SIGNER ABSTRACTION
==================================================

In `crates/arb_execute` or a dedicated internal module:
Build a wallet/signer abstraction that can:
- load a signer from env
- derive and expose the signer address safely
- sign transaction payloads
- avoid ever logging secrets

Requirements:
- local-only env loading
- no secret printing
- clear errors when secrets are missing or malformed
- safe-by-default behavior if signing config is absent

Do NOT add multi-wallet fleet logic yet.
One wallet is enough for this phase.

==================================================
PART 4 — NONCE MANAGEMENT
==================================================

Implement a minimal nonce manager.

Requirements:
- fetch current nonce from provider
- build nonce usage policy that is deterministic and testable
- support sequential transaction building
- no advanced parallel nonce scheduling yet

Important:
- no mempool race logic yet
- no replacement spam logic yet
- keep it correct and minimal

==================================================
PART 5 — TRANSACTION BUILDING
==================================================

Build the transaction request/envelope builder from Phase 8 ExecutionPlan.

Requirements:
1. Convert an ExecutionPlan into the exact contract call payload expected by the Phase 8 executor contract
2. Build an EIP-1559 transaction request with:
   - to
   - data
   - value
   - nonce
   - chain_id
   - max_fee_per_gas
   - max_priority_fee_per_gas
   - gas limit (estimated or configured fallback)
3. Keep submission disabled by default
4. If gas estimation is not yet cleanly available, document that and support an explicit safe override

Important:
- no flash-loan path yet
- no broadcasting by default
- no fake values unless clearly documented and only used as safe test placeholders

==================================================
PART 6 — DRY-RUN / DISABLED-BY-DEFAULT SUBMISSION
==================================================

Implement the submission pipeline, but keep it safe by default.

Requirements:
- support a dry-run mode that:
  - builds the full signed transaction or pre-signed payload
  - does NOT broadcast
  - logs a structured SubmissionResult
- optionally support a local-only/dev submission path if safe and clearly gated
- `ENABLE_BROADCAST=false` / `DRY_RUN_ONLY=true` should be the safe default

No live trading by default.
No autonomous sending to mainnet by default.

==================================================
PART 7 — METRICS
==================================================

Add or update metrics honestly, for example:
- arb_submission_attempts_total
- arb_submission_signed_total
- arb_submission_broadcast_total
- arb_submission_failed_total
- arb_submission_dry_run_total
- arb_nonce_fetch_total
- arb_nonce_fetch_failures_total
- arb_tx_build_total
- arb_tx_build_failures_total

Rules:
- do not fake metrics
- if broadcast is disabled, reflect that honestly
- no pretend success metrics

==================================================
PART 8 — TESTING
==================================================

Add unit tests for:
1. config parsing of signing/submission settings
2. wallet loading / signer address derivation
3. nonce management behavior
4. execution-plan -> tx request conversion
5. dry-run submission result path
6. failure handling for missing signer / bad config / bad contract address

If practical, add one local-only integration path, for example:
- build and sign a transaction against a local/anvil-style environment
OR
- build and dry-run a signed payload without broadcasting

Keep it honest and safe.
No real mainnet broadcast required for tests.

Validation should include:
- cargo check --workspace
- cargo test --workspace
- forge build
- forge test
- optional local-only submission smoke test if safely gated

==================================================
PART 9 — DOCUMENTATION HONESTY
==================================================

Update docs/checklists/walkthrough so they clearly state:

Real after Phase 9:
- wallet loading exists
- signer exists
- nonce manager exists
- tx request builder exists
- contract calldata packaging exists
- dry-run signing/submission path exists

Still deferred:
- flash loans
- live trading
- production broadcast policy
- multi-wallet fleet
- PGA tuning
- execution heuristics
- EV learning feedback into submission

Do not oversell beyond what is implemented.

==================================================
SOURCE-OF-TRUTH OUTPUTS REQUIRED
==================================================

At the end, include these exact commands and outputs:

1. Git identity:
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-9-wallet-signing-submission
- git status --short
- git log --oneline --decorate -5

2. Grep proofs:
- git grep -n 'PRIVATE_KEY\|SIGNER_PRIVATE_KEY\|EXECUTOR_CONTRACT_ADDRESS\|ENABLE_BROADCAST\|DRY_RUN_ONLY' -- crates/ .env.example
- git grep -n 'ExecutionPlan\|SubmissionResult\|SubmissionFailureReason\|NonceState\|FeePolicy' -- crates/
- git grep -n 'sign\|nonce\|broadcast\|dry_run' -- crates/arb_execute bin/arb_daemon
- git grep -n 'arb_submission_attempts_total\|arb_submission_signed_total\|arb_submission_dry_run_total\|arb_tx_build_total' -- crates/

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
- wallet/signer config added
- signer loading added
- nonce manager added
- execution-plan -> tx request builder added
- dry-run submission path added
- submission metrics added
- no live trading logic added by default

3. Changed-files summary

4. A walkthrough artifact describing:
- how an ExecutionPlan becomes a transaction request
- how signing is handled safely
- how dry-run submission works
- what remains deferred to the next phase

5. The source-of-truth outputs listed above

Do not go beyond Phase 9.


---- updates ----

Do a final Phase 9 merge-readiness pass on the EXISTING branch `phase-9-wallet-signing-submission`.

Do NOT create a new branch.
Do NOT add flash loans, live trading, mempool tactics, or PGA logic.
Do NOT expand scope beyond the 3 concrete issues below.

Goal:
Make Phase 9 merge-ready by fixing signing honesty, builder error handling, and config template alignment.

==================================================
FIX 1 — DRY-RUN MUST BE HONEST
==================================================

Current problem:
In crates/arb_execute/src/submitter.rs, dry_run() still returns:
- a constant tx hash like "0xDRYRUNHASH"
- tx.data.clone() as signed_raw

That is a placeholder, not real signing.

Required fix:
Choose ONE path:

PATH A (preferred):
- Implement real local signing without broadcasting.
- DryRunSuccess should contain a real signed payload and a real derived tx hash.
- No network submission required.

PATH B:
- If true local signing cannot be completed cleanly in this phase, rename/document the dry-run result honestly as a non-signing preview.
- Do NOT pretend signed_raw is actually signed.
- Update docs/checklist/walkthrough to match.

Either path is acceptable, but the code and docs must be honest.

==================================================
FIX 2 — BUILDER MUST NOT SILENTLY ZERO INVALID ADDRESSES
==================================================

Current problem:
In crates/arb_execute/src/builder.rs, address parsing uses `.parse().unwrap_or_default()`.

Required fix:
- Replace all `.unwrap_or_default()` address parsing in the tx builder path with explicit parse handling.
- If a plan contains an invalid address, return a structured error instead of silently using Address::ZERO.

Use a clear error path such as:
- PlanBuildFailureReason::InvalidAddress
or a similarly honest structured failure.

Do NOT allow malformed addresses to build a transaction successfully.

==================================================
FIX 3 — .env.example MUST MATCH CONFIG
==================================================

Current problem:
crates/arb_config/src/lib.rs expects Phase 9 settings like:
- SIGNER_PRIVATE_KEY
- EXECUTOR_CONTRACT_ADDRESS
- ENABLE_BROADCAST
- DRY_RUN_ONLY

But `.env.example` does not currently document them.

Required fix:
Update `.env.example` to include the Phase 9 variable names as placeholders only.
Do NOT include any real values or secrets.
Keep it consistent with Config.

==================================================
VALIDATION REQUIRED
==================================================

Run and report:

1. Source of truth:
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-9-wallet-signing-submission
- git status --short
- git log --oneline --decorate -5

2. Proof commands:
- git grep -n 'DRYRUNHASH|signed_raw: tx.data.clone|simplified signing proof' -- crates/arb_execute
- git grep -n 'unwrap_or_default()' -- crates/arb_execute/src/builder.rs
- git grep -n 'SIGNER_PRIVATE_KEY|EXECUTOR_CONTRACT_ADDRESS|ENABLE_BROADCAST|DRY_RUN_ONLY' -- crates/arb_config/src/lib.rs .env.example

3. Build/test:
- cargo check --workspace
- cargo test --workspace
- forge build
- forge test

==================================================
REQUIRED OUTPUTS
==================================================

Provide:
1. Which dry-run path was chosen:
- PATH A = real local signing
- PATH B = honest non-signing preview

2. Changed-files summary

3. Checklist confirming:
- dry-run behavior is now honest
- builder no longer silently zeroes invalid addresses
- .env.example matches Phase 9 config
- no live trading logic added

4. Exact outputs for all source-of-truth and proof commands above

5. A short walkthrough describing:
- how dry-run now works
- how invalid address handling now works
- how config/env docs now match reality
- what remains deferred to the next phase

Do not go beyond this scope.


---- update 1 ----

Do a final Phase 9 test-fix pass on the EXISTING branch `phase-9-wallet-signing-submission`.

Do NOT create a new branch.
Do NOT add any new features.
Do NOT add flash loans, live trading, mempool tactics, or PGA logic.
Do NOT expand scope beyond making the workspace test suite pass and documenting Foundry validation honestly.

Goal:
Make Phase 9 merge-ready by fixing the failing Rust workspace tests caused by the updated Config shape.

==================================================
FIX 1 — REPAIR arb_providers TEST CONFIG
==================================================

Current problem:
`cargo test --workspace` fails because a test in:
- crates/arb_providers/src/lib.rs

is still instantiating `arb_config::Config` using the old field set.

Error shown:
missing fields `dry_run_only`, `enable_broadcast`,
`executor_contract_address`, and another newly added field.

Required fix:
- update the test(s) in `crates/arb_providers/src/lib.rs` so the `Config` initializer matches the current `arb_config::Config`
- use explicit placeholder values appropriate for tests
- do not use real secrets
- if there is a clean helper/default constructor pattern, use it
- otherwise populate all required fields directly

==================================================
FIX 2 — KEEP PHASE 9 SCOPE CLEAN
==================================================

Do NOT modify unrelated strategy logic.
Do NOT add execution features.
Do NOT touch route/sim logic unless absolutely required by the failing test.

Only fix what is necessary to make:
- cargo check --workspace
- cargo test --workspace
pass cleanly.

==================================================
FIX 3 — FOUNDRY VALIDATION HONESTY
==================================================

Current local environment does not have `forge` on PATH.

Required action:
- do NOT fake Foundry validation
- if Foundry-specific validation cannot be run in this environment, update the walkthrough/checklist/docs honestly to say:
  - Rust workspace validation was run locally
  - Foundry validation requires local Foundry install or explicit binary path
- If the repo already has a documented non-PATH way to run Foundry, mention it clearly
- Do NOT bundle new binaries into the repo

==================================================
VALIDATION REQUIRED
==================================================

Run and report:

1. Source of truth:
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-9-wallet-signing-submission
- git status --short
- git log --oneline --decorate -5

2. Build/test:
- cargo check --workspace
- cargo test --workspace

3. Optional Foundry validation:
- forge build
- forge test
OR, if unavailable:
- report clearly that forge is not on PATH and no fake success is claimed

4. Proof of the config-test fix:
- git grep -n 'Config {' -- crates/arb_providers/src/lib.rs
- git grep -n 'signer_private_key|executor_contract_address|enable_broadcast|dry_run_only' -- crates/arb_providers/src/lib.rs crates/arb_config/src/lib.rs

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
- arb_providers test config fixed
- cargo check passes
- cargo test passes
- no new execution logic added
- Foundry validation status is reported honestly

4. Exact outputs for all source-of-truth and proof commands above

5. A short walkthrough describing:
- what test was fixed
- how the Config shape mismatch was resolved
- whether Foundry was actually run or deferred due local setup
- what remains deferred to the next phase

Do not go beyond this scope.