Implement Phase 4 on a dedicated branch.

Suggested branch name:
phase-4-dex-event-decoding

Before doing any code work:
1. Ensure work is being done on branch `phase-4-dex-event-decoding`
   - if the branch does not exist, create it from current main
   - if it exists, switch to it
2. Do NOT work directly on main

Goal:
Replace the current synthetic Flashblock-derived state updates with real DEX event decoding as the primary state input path.

Scope:
- decode real pending-log DEX events into PoolUpdate values
- update the in-memory state engine to support real reserve-based and minimal concentrated-liquidity state updates
- add replay fixtures and tests for these decoders
- keep state metrics honest
- no routing logic
- no strategy logic
- no simulation logic
- no execution logic

==================================================
PHASE 4 OBJECTIVE
==================================================

By the end of this phase, the system should be able to:

1. Consume normalized PendingLog-style events from Phase 2/3
2. Decode at least:
   - one real reserve-based DEX event shape
   - one real concentrated-liquidity DEX event shape
3. Convert those decoded logs into real PoolUpdate values
4. Feed those updates into the state engine
5. Update state/freshness metrics accordingly
6. Prove correctness via replay fixtures and tests

This phase is about:
- real DEX event decoding
- real state updates

This phase is NOT about:
- route search
- arb filtering
- simulation
- transaction building
- execution

==================================================
DECODING TARGETS
==================================================

Implement support for at least these real event families:

A. Reserve-based pools
- Uniswap V2-style Sync(uint112 reserve0, uint112 reserve1)
- decode from PendingLog payload
- convert to PoolUpdate with reserve-based state

B. Concentrated-liquidity pools
- Uniswap V3-style Swap(...)
- minimally decode the fields required to track top-level CL state:
  - sqrtPriceX96
  - liquidity
  - tick
- convert to PoolUpdate with CL-style state snapshot

Important:
- You do NOT need full tick bitmap decoding in this phase
- You do NOT need full quote math in this phase
- You do NOT need routing in this phase
- We only want enough decoded state to maintain a truthful canonical pool snapshot

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
PART 1 — TYPE SYSTEM / STATE MODEL
==================================================

In `crates/arb_types` and/or `crates/arb_state`, add the minimal state types needed to support real DEX-driven state updates.

Expected direction:
- PoolId
- PoolKind
- EventStamp
- PoolFreshness
- PoolStateSnapshot
- PoolUpdate

Expand the state model so PoolStateSnapshot can represent at least:
1. Reserve-based snapshot:
   - reserve0
   - reserve1
2. Concentrated-liquidity snapshot:
   - sqrt_price_x96
   - liquidity
   - tick

Keep the types minimal and honest.
No quote math yet.

==================================================
PART 2 — REAL PENDING LOG DECODING
==================================================

In `crates/arb_ingest`:

Replace the current “pending-log ignored for state” limitation by implementing real decoding for at least:
- V2-style Sync event logs
- V3-style Swap event logs

Requirements:
1. Match by real topic0/event signatures
2. Decode event data structurally
3. Produce normalized internal decoded event representations
4. Convert decoded events into PoolUpdate values suitable for arb_state

Important:
- no substring hacks
- no fake decoding
- no invented reserve values
- malformed / unsupported logs must fail gracefully and increment metrics if appropriate

If needed:
- introduce an intermediate enum like `DecodedDexEvent`
- add helper modules for V2/V3 log decoding

==================================================
PART 3 — STATE ENGINE INTEGRATION
==================================================

In `crates/arb_state` and daemon wiring:

1. Feed real decoded DEX events into StateEngine::apply()
2. Keep stale-update rejection via EventStamp ordering
3. Keep freshness tracking
4. Keep synthetic Flashblock-derived updates OUT of the primary state path

If you keep synthetic Flashblock state support at all:
- it must be behind an explicit feature flag or dev-only path
- it must be clearly marked as fallback/dev-only
- it must not be the default primary state input anymore

==================================================
PART 4 — METRICS
==================================================

Add or update metrics honestly in `arb_metrics` as needed.

Suggested useful metrics:
- arb_dex_sync_events_total
- arb_dex_cl_swap_events_total
- arb_pending_logs_state_updates_total
- arb_unsupported_dex_logs_total
- arb_malformed_payloads_total (reuse if already present)
- existing state metrics should continue to work:
  - arb_state_updates_total
  - arb_pools_tracked_total
  - arb_stale_pool_events_total

Rules:
- do not fake metrics
- if a metric is not real, don’t claim it exists

==================================================
PART 5 — FIXTURES / REPLAY TESTS
==================================================

Under `fixtures/`, add representative replay inputs for:
- at least one V2-style Sync log payload
- at least one V3-style Swap log payload

In tests:
- replay these fixture logs through ingest -> decode -> state
- assert that state changes correctly
- assert reserve-based pool updates are applied correctly
- assert CL snapshot updates (sqrtPrice/liquidity/tick) are applied correctly
- assert out-of-order stale updates are rejected
- assert unsupported logs are handled safely

==================================================
PART 6 — DAEMON WIRING
==================================================

In `bin/arb_daemon`:
- wire the decoded pending-log path into arb_state
- preserve graceful shutdown
- preserve observability wiring
- do not add routing / execution logic

If needed, update startup logs to mention:
- real DEX log decoding enabled
- synthetic Flashblock state path disabled by default (if still present)

==================================================
PART 7 — DOCUMENTATION HONESTY
==================================================

Update docs/walkthrough/checklist so they clearly state:

What is real after Phase 4:
- V2 Sync decoding
- V3 Swap top-level state decoding
- pending-log -> state bridge
- state engine applying real DEX-derived updates

What remains deferred:
- full CL tick map / bitmap handling
- quote math
- route finding
- simulation
- execution

Do not oversell beyond what is implemented.

==================================================
VALIDATION REQUIRED
==================================================

Run and report:
- cargo check --workspace
- cargo test --workspace
- docker compose config (only if relevant files changed)
- if useful, a replay-driven smoke validation

==================================================
SOURCE-OF-TRUTH OUTPUTS REQUIRED
==================================================

At the end, include these exact commands and outputs:

1. Git branch identity:
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-4-dex-event-decoding
- git status --short
- git log --oneline --decorate -5

2. Grep proofs:
- git grep -n 'Sync' -- crates/ fixtures/
- git grep -n 'Swap' -- crates/ fixtures/
- git grep -n 'sqrt_price\|sqrtPrice\|liquidity\|tick' -- crates/
- git grep -n 'arb_dex_sync_events_total\|arb_dex_cl_swap_events_total\|arb_pending_logs_state_updates_total\|arb_unsupported_dex_logs_total' -- crates/

3. Validation outputs:
- cargo check --workspace
- cargo test --workspace
- docker compose config (if changed)

==================================================
REQUIRED OUTPUTS
==================================================

When finished, provide:

1. A verdict:
- fully working
- working with known limitations
- blocked (and why)

2. A checklist confirming:
- real V2 Sync decoding added
- real V3 Swap top-level decoding added
- pending-log -> state bridge is real
- state engine applies real DEX-derived updates
- synthetic Flashblock state path is no longer primary
- replay tests added
- no routing/sim/execution logic added

3. Changed-files summary

4. A walkthrough artifact describing:
- what event types are now decoded
- how those decoded logs become PoolUpdate values
- how state changes are validated
- what remains deferred to the next phase

5. The source-of-truth outputs listed above

Do not go beyond Phase 4.