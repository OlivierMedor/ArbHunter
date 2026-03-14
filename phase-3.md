Implement Phase 3 only on a new branch.

Suggested branch name:
phase-3-state-engine

Goal:
Build the in-memory state engine foundation for ArbHunter.

Scope:
- consume normalized ingest events from Phase 2
- maintain canonical in-memory pool state
- support foundational pool adapters for future reserve-based and concentrated-liquidity DEX models
- track pool freshness / last-updated metadata
- add replay-driven state tests
- no routing logic
- no strategy logic
- no simulation logic
- no execution logic

Work mainly in:
- crates/arb_types
- crates/arb_state
- crates/arb_ingest
- crates/arb_metrics
- bin/arb_daemon

Requirements:

1. arb_types
Add minimal shared state-related structs/enums as needed, such as:
- PoolId
- PoolKind
- TokenAddress or TokenId wrapper
- PoolStateSnapshot
- PoolUpdate
- PoolFreshness
- BlockStamp / EventStamp if useful

Keep them minimal and serializable where useful.

2. arb_state
Build the canonical in-memory state layer:
- pool state store
- apply/update methods that consume normalized ingest events
- freshness tracking
- minimal adapter structure for future:
  - reserve-based pools
  - concentrated-liquidity pools

Important:
- do not implement route search
- do not implement quoting
- do not implement strategy logic
- do not implement execution logic

This phase is about state correctness only.

3. arb_ingest
If needed, add only the smallest changes required so normalized events can be converted into state updates.
Do not expand into strategy logic.

4. arb_metrics
Add state-related counters/gauges if useful, for example:
- state_updates_total
- pools_tracked_total
- stale_pool_events_total
- pool_freshness_age_seconds (if practical and honest)

Do not fake metrics.

5. bin/arb_daemon
Wire ingest output into arb_state so the daemon can:
- receive normalized events
- apply them to in-memory state
- log basic state update counts
- continue graceful shutdown behavior

6. Testing requirements
Add:
- replay fixture tests that drive ingest events into arb_state
- unit tests for state transitions
- tests for freshness tracking
- workspace must pass:
  - cargo check --workspace
  - cargo test --workspace

Constraints:
- Postgres remains off the hot path
- no routing
- no filter logic
- no simulation
- no execution
- keep the design modular and scalable

Artifacts required:
1. Implementation summary
2. Walkthrough artifact
3. Changed-files summary
4. Checklist confirming:
   - state engine added
   - ingest -> state bridge added
   - freshness tracking added
   - replay/state tests added
   - no routing/sim/execution logic added

Do not go beyond Phase 3.