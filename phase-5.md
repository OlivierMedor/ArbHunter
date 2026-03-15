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