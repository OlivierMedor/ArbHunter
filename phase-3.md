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


--- update 1 ---

Do a final Phase 3 merge-readiness cleanup pass on the EXISTING branch `phase-3-state-engine`.

Do NOT create a new branch.
Do NOT add features.
Do NOT modify state logic unless required for cleanup.

Goal:
Make the branch merge-ready by removing tracked runtime log artifacts and tightening repo hygiene.

Required fixes:
1. Remove these tracked files from git if present:
- daemon_stdout.log
- daemon_stderr.log

2. Update `.gitignore` so these do not get tracked again:
- daemon_stdout.log
- daemon_stderr.log
- optionally other runtime log patterns if appropriate

3. Do NOT remove any legitimate source files.

4. After cleanup, run and report:
- cargo check --workspace
- cargo test --workspace
- git status --short

Required outputs:
1. Changed-files summary
2. Checklist confirming:
   - daemon_stdout.log no longer tracked
   - daemon_stderr.log no longer tracked
   - .gitignore updated if needed
   - no state/routing/sim/execution logic added
3. Validation outputs for:
   - cargo check --workspace
   - cargo test --workspace
   - git status --short


--- update 2 ---

Do a final Phase 3 merge-readiness cleanup pass on the EXISTING branch `phase-3-state-engine`.

Do NOT create a new branch.
Do NOT add routing, simulation, or execution logic.
Do NOT add real DEX decoding yet.

Goal:
Make the branch merge-ready by cleaning tracked runtime log files and ensuring the docs are honest about what Phase 3 actually implements.

Required fixes:

1. Remove these tracked files from git if present:
- daemon_stdout.log
- daemon_stderr.log

2. Update .gitignore so these do not get tracked again:
- daemon_stdout.log
- daemon_stderr.log
- optionally *.log if appropriate

3. Review the Phase 3 checklist / walkthrough / implementation summary and make sure they clearly state:
- the state engine is real
- freshness tracking is real
- stale update rejection is real
- BUT the current daemon still converts Flashblock events into synthetic block-level PoolUpdate values
- PendingLog events are still ignored for state in this phase
- real DEX event-to-state decoding is deferred to the next phase

4. Do NOT add any new strategy, routing, simulation, or execution logic.

5. After cleanup, run and report:
- cargo check --workspace
- cargo test --workspace
- git status --short

Required outputs:
1. Changed-files summary
2. Checklist confirming:
   - daemon log files are no longer tracked
   - .gitignore updated if needed
   - Phase 3 docs are honest about synthetic state inputs
   - no routing/sim/execution logic was added
3. Validation outputs for:
   - cargo check --workspace
   - cargo test --workspace
   - git status --short

   --- update 3 ---

   Do a final Phase 3 merge-readiness cleanup pass on the EXISTING branch `phase-3-state-engine`.

Do NOT create a new branch.
Do NOT add routing, filtering, simulation, or execution logic.
Do NOT add new features.
Do NOT expand scope beyond cleanup + documentation alignment.

Goal:
Make Phase 3 merge-ready by:
1. removing tracked runtime log files,
2. fixing project-phase documentation so it matches the actual implementation order.

==================================================
PART 1 — REMOVE TRACKED RUNTIME LOG FILES
==================================================

If present and tracked, remove these from git:
- daemon_stdout.log
- daemon_stderr.log

Update `.gitignore` if needed so they do not get tracked again.
Keep the rest of the repo intact.

Acceptance criteria:
- `git ls-files daemon_stdout.log` returns nothing
- `git ls-files daemon_stderr.log` returns nothing

==================================================
PART 2 — FIX docs/PHASES.md TO MATCH REAL PROJECT PHASES
==================================================

Current problem:
`docs/PHASES.md` does not match the actual implemented sequence.

It must be revised so the phase ordering matches the project as actually built so far:

- Phase 0 = Planning / docs
- Phase 1 = Scaffold / repo foundations
- Phase 2 = Providers + ingestion foundation
- Phase 2.5 = Observability dashboard
- Phase 3 = State engine
- Later phases can continue from there

For Phase 3 specifically, the document should clearly reflect:
- in-memory canonical state layer
- freshness tracking
- stale update rejection
- ingest -> state bridge
- no routing/filtering/simulation/execution yet

Do not oversell.
Be accurate.

==================================================
PART 3 — VALIDATION
==================================================

After making the cleanup changes, run and report:
- cargo check --workspace
- cargo test --workspace
- git status --short
- git ls-files daemon_stdout.log
- git ls-files daemon_stderr.log

==================================================
REQUIRED OUTPUTS
==================================================

Provide:

1. Changed-files summary
2. Checklist confirming:
- daemon_stdout.log no longer tracked
- daemon_stderr.log no longer tracked
- docs/PHASES.md now matches the real project sequence
- no routing/filtering/sim/execution logic was added

3. Validation outputs for:
- cargo check --workspace
- cargo test --workspace
- git status --short
- git ls-files daemon_stdout.log
- git ls-files daemon_stderr.log

4. A short walkthrough summarizing:
- what was cleaned up
- what phase numbering/doc wording was corrected
- what remains deferred to the next phase