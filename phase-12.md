Implement Phase 12 on a dedicated branch.

Suggested branch name:
phase-12-forked-e2e-harness

Before doing any code work:
1. Ensure work is being done on branch `phase-12-forked-e2e-harness`
   - if the branch does not exist, create it from current main
   - if it exists, switch to it
2. Do NOT work directly on main

Goal:
Build the first full end-to-end execution harness on a local/forked chain so we can prove the entire system works from candidate -> simulation -> execution plan -> signed tx -> contract call -> receipt, without risking real funds.

Scope:
- local/forked chain harness
- deterministic test wallet
- deployment of contract(s) into local/forked environment
- one simple execution path
- receipt / gas / balance / revert validation
- no live mainnet trading
- no aggressive bot loop
- no flash-loan mainnet rollout

This phase is NOT about:
- profitable real trading
- production broadcast policy
- mempool tactics
- private relays
- multi-wallet fleet
- business-entity wallet setup

==================================================
PHASE 12 OBJECTIVE
==================================================

By the end of this phase, the system should be able to:

1. Start a local Anvil or forked-chain environment
2. Deploy the required contract(s) into that environment
3. Fund/use a deterministic local test wallet
4. Take a validated candidate / execution plan
5. Sign and submit a real transaction to the local/forked chain
6. Receive a real receipt
7. Inspect:
   - success/revert
   - gas used
   - revert reason if failed
   - final balances / post-trade outcome
8. Prove the full pipeline works without using real funds

==================================================
WORK AREAS
==================================================

Likely files/crates to work in:
- crates/arb_config
- crates/arb_execute
- bin/arb_daemon
- contracts/
- docker-compose.yml
- Makefile
- docs if needed

==================================================
PART 1 — LOCAL/FORK CHAIN HARNESS
==================================================

Add a project-integrated local execution harness using Anvil.

Preferred approach:
- add an `anvil` service using the official Foundry container
OR
- add a documented docker compose / local run path for Anvil

Requirements:
- support local chain mode
- support optional fork mode using:
  - ANVIL_FORK_URL
  - ANVIL_FORK_BLOCK_NUMBER
- keep it deterministic and safe
- no real mainnet broadcast

Acceptance criteria:
- developer can start a local/forked chain from the repo
- exact commands are documented

==================================================
PART 2 — TEST WALLET / SIGNER SETUP
==================================================

Add support for a deterministic local-only test wallet.

Requirements:
- use TEST_PRIVATE_KEY (or similar) from env
- no real secrets in tracked files
- if using default Anvil wallet, document it honestly
- one wallet is enough for this phase

Do NOT add business-entity wallet logic.
Do NOT add multi-wallet fleet logic.

==================================================
PART 3 — CONTRACT DEPLOYMENT INTO LOCAL/FORK ENV
==================================================

Provide a simple path to deploy the relevant contract(s) into the local/forked environment.

Requirements:
- use Foundry or existing deployment tooling
- deployment must be reproducible
- if the deployment uses local Anvil, document exact commands
- if addresses are needed by Rust, feed them through config/env cleanly

Do not overengineer deployment management.

==================================================
PART 4 — END-TO-END EXECUTION HARNESS
==================================================

Wire the existing pipeline so one validated execution path can be sent end-to-end in the local/forked environment.

Expected flow:
candidate -> simulation -> execution plan -> tx build -> sign -> submit -> receipt

Requirements:
- no live chain broadcast
- at least one controlled execution path
- one success path
- one revert/failure path
- capture:
  - tx hash
  - receipt status
  - gas used
  - revert reason if available
  - final balances / outcome summary

If a mock/simple venue is needed for deterministic success, that is acceptable.
If a real forked route is also practical, include it.

==================================================
PART 5 — TESTING
==================================================

Add and/or run:
1. Foundry tests for local execution behavior
2. Rust-side integration path for:
   - tx build
   - sign
   - submit
   - receipt handling
3. one success scenario
4. one revert scenario
5. optional historical fork scenario if practical

Important:
This phase is about proving the machine works end-to-end, not proving profitability.

==================================================
PART 6 — DOCUMENTATION HONESTY
==================================================

Update docs/walkthrough/checklist so they clearly state:

Real after Phase 12:
- local/forked execution harness exists
- deterministic test wallet exists
- deployment path exists
- signed tx -> on-chain local receipt path exists
- gas/revert/balance inspection exists

Still deferred:
- live mainnet rollout
- production wallet ops
- flash-loan mainnet execution
- mempool tactics
- EV learning / adaptive policy
- aggressive automation

Do not oversell beyond what is implemented.

==================================================
VALIDATION REQUIRED
==================================================

Run and report:

1. Rust validation:
- cargo check --workspace
- cargo test --workspace

2. Foundry validation:
- docker compose run --rm forge forge build
- docker compose run --rm forge forge test

3. Local/fork harness validation:
- docker compose config
- whatever command starts Anvil/fork
- proof that a transaction was actually sent locally
- proof of receipt
- proof of gas used
- proof of success and failure cases

==================================================
SOURCE-OF-TRUTH OUTPUTS REQUIRED
==================================================

At the end, include these exact commands and outputs:

1. Git identity:
- git fetch origin
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-12-forked-e2e-harness
- git status --short
- git log --oneline --decorate -5

2. Grep proofs:
- git grep -n 'ANVIL|FORK|TEST_PRIVATE_KEY' -- .env.example crates/ docker-compose.yml docs/
- git grep -n 'receipt|gas_used|revert|broadcast|submit' -- crates/arb_execute bin/ docs/
- git grep -n 'anvil|forge build|forge test' -- docker-compose.yml Makefile docs/

3. Validation outputs:
- cargo check --workspace
- cargo test --workspace
- docker compose config
- docker compose run --rm forge forge build
- docker compose run --rm forge forge test
- command/output showing local/fork chain started
- command/output showing a real local transaction receipt

==================================================
REQUIRED OUTPUTS
==================================================

Provide:

1. A verdict:
- fully working
- working with known limitations
- blocked (and why)

2. A checklist confirming:
- local/fork harness added
- deterministic test wallet configured
- contract deployment path added
- signed tx -> receipt path proven
- success and failure cases proven
- no real mainnet trading logic added

3. Changed-files summary

4. A walkthrough artifact describing:
- how to start the local/forked environment
- how the wallet is used safely
- how deployment works
- how a candidate becomes a real local transaction
- how receipt/gas/revert are inspected
- what remains deferred to the next phase

5. The source-of-truth outputs listed above

Do not go beyond Phase 12.


---- updates 1 ----

Do a final Phase 12 merge-readiness pass on the EXISTING branch `phase-12-forked-e2e-harness`.

Do NOT create a new branch.
Do NOT add new features.
Do NOT add execution strategy, flash-loan changes, or live trading logic.
Do NOT expand scope beyond fixing the concrete merge blockers below.

Goal:
Make Phase 12 merge-ready by:
1. fixing the failing workspace test,
2. removing real provider data from `.env.example`,
3. leaving the branch clean and commit-ready.

==================================================
FIX 1 — REPAIR arb_providers TEST CONFIG
==================================================

Current problem:
`cargo test --workspace` fails because a test in:
- crates/arb_providers/src/lib.rs

is instantiating `arb_config::Config` without the new Phase 12 fields:
- local_rpc_url
- test_private_key

Required fix:
- update the test Config literal so it matches the current `arb_config::Config`
- use safe placeholder values only
- no real secrets
- keep the fix minimal and local to the test path if possible

==================================================
FIX 2 — REMOVE REAL QUICKNODE URL FROM .env.example
==================================================

Current problem:
`.env.example` currently contains a real QuickNode endpoint/token in:
- ANVIL_FORK_URL=...

Required fix:
- replace any real provider URL in `.env.example` with a blank or obvious placeholder
- keep only template-safe values
- do not commit real endpoint tokens in tracked files

Acceptable:
ANVIL_FORK_URL=
ANVIL_FORK_BLOCK_NUMBER=
ANVIL_RPC_URL=http://127.0.0.1:8545
TEST_PRIVATE_KEY=<anvil_default_or_leave_blank>
ENABLE_BROADCAST=false
DRY_RUN_ONLY=true

If you keep TEST_PRIVATE_KEY in the example, it must be clearly documented as a local-only Anvil test key, not a production key.

==================================================
FIX 3 — CLEAN BRANCH STATE
==================================================

Current problem:
The branch is not clean.

Required fix:
- ensure all intended changes are either committed or reverted
- final `git status --short` must be clean

==================================================
VALIDATION REQUIRED
==================================================

Run and report:

1. Source of truth:
- git fetch origin
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-12-forked-e2e-harness
- git status --short
- git log --oneline --decorate -5

2. Proof commands:
- git grep -n 'ANVIL_FORK_URL|ANVIL_FORK_BLOCK_NUMBER|TEST_PRIVATE_KEY' -- .env.example crates/arb_config/src/lib.rs
- git grep -n 'Config {' -- crates/arb_providers/src/lib.rs
- git grep -n 'local_rpc_url|test_private_key' -- crates/arb_providers/src/lib.rs crates/arb_config/src/lib.rs

3. Validation:
- cargo check --workspace
- cargo test --workspace
- docker compose config
- docker compose up -d anvil
- docker compose run --rm forge forge build
- docker compose run --rm forge forge test

==================================================
REQUIRED OUTPUTS
==================================================

Provide:
1. Verdict
2. Changed-files summary
3. Checklist confirming:
   - failing arb_providers test fixed
   - .env.example no longer contains a real provider URL
   - branch is clean
   - cargo check passes
   - cargo test passes
   - dockerized Foundry validation passes
   - no new execution strategy logic added
4. Exact outputs for all source-of-truth and proof commands above
5. A short walkthrough describing:
   - how the local/fork harness now works
   - how secrets/templates are handled safely
   - what remains deferred to the next phase

Do not go beyond this scope.