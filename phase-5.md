Implement Phase 5 on a dedicated branch.

Suggested branch name:
phase-5-cl-tickmap-quoter

Before doing any code work:
1. Ensure work is being done on branch `phase-5-cl-tickmap-quoter`
   - if the branch does not exist, create it from current main
   - if it exists, switch to it
2. Do NOT work directly on main

Goal:
Build the concentrated-liquidity depth model and local quote primitives needed before route refinement.

Scope:
- maintain CL tick map state from real V3 events
- maintain liquidity net / liquidity gross where needed
- support local exact-in quote primitives for:
  - reserve-based pools
  - one concentrated-liquidity pool path
- no route search yet
- no strategy logic
- no simulation logic
- no execution logic

==================================================
PHASE 5 OBJECTIVE
==================================================

By the end of this phase, the system should be able to:

1. Maintain a truthful in-memory CL state model sufficient for local quoting
2. Decode the additional V3 event types needed to update CL depth
3. Produce local exact-in quote estimates for:
   - reserve-based pools
   - one CL pool
4. Validate quote behavior with replay fixtures and tests

This phase is about:
- state depth
- tick maps
- quote primitives

This phase is NOT about:
- route search
- candidate filtering
- simulation
- transaction building
- execution

==================================================
DECODING / STATE TARGETS
==================================================

Implement support for the V3 event/state pieces needed to maintain CL depth, at minimum:
- Initialize
- Mint
- Burn
- Swap (already present, integrate with CL state model)

Expected direction:
- track current sqrt price
- track current tick
- track active liquidity
- track liquidity changes at ticks
- maintain enough per-tick structure to support local traversal

You do NOT need:
- full production-grade bitmap optimization yet
- multi-pool routing
- execution

==================================================
WORK AREAS
==================================================

Main crates/files likely involved:
- crates/arb_types
- crates/arb_ingest
- crates/arb_state
- crates/arb_metrics
- bin/arb_daemon
- fixtures/
- docs if necessary

==================================================
PART 1 — TYPES / STATE MODEL
==================================================

Expand state types to support CL depth honestly.

Expected additions:
- CLTickState
- CLPoolState / extensions to PoolStateSnapshot
- liquidity_gross
- liquidity_net
- current_tick
- current_liquidity
- current_sqrt_price_x96

Keep types minimal and serializable where useful.

==================================================
PART 2 — REAL CL EVENT DECODING
==================================================

In `crates/arb_ingest`:
- add decoding for the V3 events needed for CL depth maintenance
- map those events into normalized update types for arb_state

Requirements:
- no substring hacks
- no fake events
- malformed / unsupported logs fail safely
- metrics for unsupported / malformed logs remain honest

==================================================
PART 3 — STATE ENGINE UPDATES
==================================================

In `crates/arb_state`:
- apply CL event updates into the tick map / CL state
- preserve stale-update rejection
- preserve freshness tracking

Do not add route logic.

==================================================
PART 4 — LOCAL QUOTE PRIMITIVES
==================================================

Add local exact-in quote primitives for:
1. reserve-based pools
2. one concentrated-liquidity pool

Important:
- keep the API modular
- this is a quote primitive, not route search
- no cross-pool optimization yet

Be honest:
- if CL quote support is partial, document exactly what assumptions/limitations remain

==================================================
PART 5 — METRICS
==================================================

Add or update metrics honestly as needed, for example:
- arb_cl_ticks_tracked_total
- arb_cl_state_updates_total
- arb_local_quotes_total
- arb_local_quote_errors_total

Do not fake metrics.

==================================================
PART 6 — FIXTURES / TESTS
==================================================

Add replay fixtures and tests for:
- reserve-based quote correctness
- CL state updates from event sequences
- CL quote behavior from known event/state sequences
- stale update rejection still working

Workspace must pass:
- cargo check --workspace
- cargo test --workspace

==================================================
PART 7 — DOCUMENTATION HONESTY
==================================================

Update docs/checklists/walkthrough so they clearly state:
What is real after Phase 5:
- CL tick/depth state foundation
- reserve-based quote primitive
- CL quote primitive
What remains deferred:
- route search
- candidate filtering
- simulation
- execution

Do not oversell beyond what is implemented.

==================================================
SOURCE-OF-TRUTH OUTPUTS REQUIRED
==================================================

At the end, include these exact commands and outputs:

1. Git identity:
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-5-cl-tickmap-quoter
- git status --short
- git log --oneline --decorate -5

2. Grep proofs:
- git grep -n 'Initialize\|Mint\|Burn\|Swap' -- crates/ fixtures/
- git grep -n 'liquidity_gross\|liquidity_net\|current_tick\|sqrt_price' -- crates/
- git grep -n 'quote' -- crates/

3. Validation outputs:
- cargo check --workspace
- cargo test --workspace

==================================================
REQUIRED OUTPUTS
==================================================

When finished, provide:

1. A verdict:
- fully working
- working with known limitations
- blocked (and why)

2. A checklist confirming:
- CL depth/tick state added
- required V3 event decoding added
- reserve-based quote primitive added
- CL quote primitive added
- replay tests added
- no route/sim/execution logic added

3. Changed-files summary

4. A walkthrough artifact describing:
- what CL state is now tracked
- how quote primitives work
- what remains deferred to the next phase

5. The source-of-truth outputs listed above

Do not go beyond Phase 5.


---- Updates ----

Do a final Phase 5 merge-readiness pass on the EXISTING branch `phase-5-cl-tickmap-quoter`.

Do NOT create a new branch.
Do NOT add routing, simulation, or execution logic.
Do NOT expand scope beyond fixing the Phase 5 decoding/state truthfulness issues.

Goal:
Make Phase 5 merge-ready by:
1. making decoded PoolUpdate values truthfully ordered and identifiable,
2. fixing fixture realism/validity,
3. upgrading decoder validation from ignored/deferred to at least one real passing replay-driven test.

==================================================
FIX 1 — REAL ORDERING METADATA
==================================================

Current problem:
Decoded PendingLog-derived PoolUpdate values currently use EventStamp { block_number: 0, log_index: 0 }.

Required fix:
- Extend the PendingLog event path so decoded PoolUpdate values carry real ordering metadata.
- If PendingLogEvent is missing block_number/log_index, add the minimal fields needed and thread them through the ingest path.
- The state engine must not receive zeroed placeholder stamps for real decoded DEX events.

Acceptance criteria:
- decoded V2 and V3 PoolUpdate values no longer use hardcoded zero stamps
- ordering/stale-rejection logic can work truthfully on decoded DEX events

==================================================
FIX 2 — REMOVE EMPTY TOKEN PLACEHOLDERS
==================================================

Current problem:
Decoded PoolUpdate values still set token0 and token1 to empty strings.

Required fix:
Choose one honest path:
PATH A (preferred):
- populate token0/token1 from available fixture/log metadata or pool metadata source if already accessible in-phase

PATH B:
- if token0/token1 cannot be made real in this phase, do NOT pretend they are known;
  instead redesign the update/state path so empty-string token placeholders are not required for merge readiness
  (for example, make them optional where honest and safe)

Do NOT leave empty-string token addresses in “real decoded” updates.

==================================================
FIX 3 — FIX FIXTURE VALIDITY
==================================================

Current problem:
fixtures/pending_logs.jsonl is not a trustworthy replay source.

Required fix:
- make it true JSONL (one valid JSON object per line)
- use valid-looking addresses and realistic topic/data formatting
- keep one V2 Sync-like fixture and one V3 Swap-like fixture
- no malformed addresses, no underscore characters in addresses, no multiple JSON objects on a single line

Acceptance criteria:
- replay harness can read the fixture line-by-line
- each line is valid JSON
- fixture content is honest enough for decoder/state tests

==================================================
FIX 4 — REAL DECODER TESTS
==================================================

Current problem:
Decoder tests are still deferred/ignored.

Required fix:
- add at least one real passing replay-driven test for V2 Sync decoding
- add at least one real passing replay-driven test for V3 Swap decoding OR clearly prove the CL decode path via a real fixture-driven test through the ingest/state pipeline
- remove “ignored” status from at least the core decoder validation path

You do not need perfect exhaustive coverage, but Phase 5 needs at least one honest passing replay-driven decoder test path.

==================================================
VALIDATION REQUIRED
==================================================

Run and report:
- cargo check --workspace
- cargo test --workspace
- git grep -n 'EventStamp { block_number: 0, log_index: 0 }' -- crates/
- git grep -n 'TokenAddress(\"\"' -- crates/
- git show origin/phase-5-cl-tickmap-quoter:fixtures/pending_logs.jsonl

==================================================
SOURCE-OF-TRUTH OUTPUTS REQUIRED
==================================================

At the end, include:
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-5-cl-tickmap-quoter
- git status --short
- git log --oneline --decorate -5

==================================================
REQUIRED OUTPUTS
==================================================

Provide:
1. Verdict:
- fully working
- working with known limitations
- blocked (and why)

2. Checklist confirming:
- real ordering metadata added
- empty token placeholders removed or honestly redesigned
- pending_logs.jsonl fixed to valid JSONL
- at least one core decoder test path now passes unignored
- no routing/sim/execution logic added

3. Changed-files summary

4. Walkthrough artifact describing:
- how decoded events now become truthful PoolUpdate values
- how fixture realism was improved
- what remains deferred to the next phase

5. Exact outputs for:
- cargo check --workspace
- cargo test --workspace
- git grep -n 'EventStamp { block_number: 0, log_index: 0 }' -- crates/
- git grep -n 'TokenAddress(\"\"' -- crates/
- git show origin/phase-5-cl-tickmap-quoter:fixtures/pending_logs.jsonl

Do not go beyond Phase 5.

----- Updates -----

Do one final Phase 5 merge-readiness fix pass on the EXISTING branch `phase-5-cl-tickmap-quoter`.

Do NOT create a new branch.
Do NOT add routing, simulation, or execution logic.
Do NOT expand scope beyond the 3 concrete blockers below.

Goal:
Make Phase 5 merge-ready by fixing the remaining truthfulness and fixture issues.

==================================================
FIX 1 — REAL ORDERING METADATA
==================================================

Current problem:
Decoded PoolUpdate values still use:
EventStamp { block_number: 0, log_index: 0 }

Required fix:
- Thread real block_number and log_index from PendingLogEvent into decoded PoolUpdate.stamp
- Remove all hardcoded zero EventStamp placeholders from decoded DEX event paths
- Keep state ordering logic honest

Acceptance criteria:
The command below must return NO output:
git grep -n 'EventStamp { block_number: 0, log_index: 0 }' -- crates/

==================================================
FIX 2 — FIX FIXTURE VALIDITY
==================================================

Current problem:
fixtures/pending_logs.jsonl still contains invalid-looking Ethereum addresses.

Required fix:
- Replace the invalid addresses with valid-looking 0x-prefixed 40-hex-character addresses
- Keep one JSON object per line
- Keep one V2 Sync-like fixture and one V3 Initialize/Swap-like fixture
- Preserve valid JSONL formatting

Acceptance criteria:
- fixtures/pending_logs.jsonl contains only valid-looking 0x... addresses
- no underscores
- no invalid characters like letter l in place of hex digits

==================================================
FIX 3 — REMOVE STALE DEFERRED TEST COMMENT
==================================================

Current problem:
Code still says DEX decoder tests are deferred to Phase 6 even though real decoder tests now pass.

Required fix:
- remove or rewrite the stale comment
- make code/comments honest about current test status

Acceptance criteria:
The command below should either return no output or only updated honest wording:
git grep -n 'deferred|ignored' -- crates/arb_ingest/src/lib.rs

==================================================
VALIDATION REQUIRED
==================================================

Run and report:

1. Source of truth:
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-5-cl-tickmap-quoter
- git status --short
- git log --oneline --decorate -5

2. Proof commands:
- git grep -n 'EventStamp { block_number: 0, log_index: 0 }' -- crates/
- git grep -n 'TokenAddress\(\"\"\)' -- crates/
- git grep -n 'deferred|ignored' -- crates/arb_ingest/src/lib.rs
- git show origin/phase-5-cl-tickmap-quoter:fixtures/pending_logs.jsonl

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
   - zero-stamp placeholders removed
   - fixture addresses fixed
   - stale deferred/ignored comment fixed
   - no routing/sim/execution logic added
4. Exact outputs for all validation commands above

Do not go beyond this scope.