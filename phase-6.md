Implement Phase 6 on a dedicated branch.

Suggested branch name:
phase-6-route-graph-filter

Before doing any code work:
1. Ensure work is being done on branch `phase-6-route-graph-filter`
   - if the branch does not exist, create it from current main
   - if it exists, switch to it
2. Do NOT work directly on main

Goal:
Build the local route graph and candidate filter foundation on top of the existing state engine and quote primitives.

Scope:
- construct a local token/pool graph from canonical pool state
- integrate reserve-based and CL quote primitives into graph edges
- enumerate candidate cyclic routes
- estimate rough gross profit locally
- promote only candidates above configurable thresholds
- expose route/candidate metrics
- validate with replay fixtures/tests

This phase is NOT about:
- transaction execution
- private routing
- simulation
- external router APIs
- strategy optimization beyond local candidate generation
- live trading

==================================================
PHASE 6 OBJECTIVE
==================================================

By the end of this phase, the system should be able to:

1. Build a local directed multigraph of tradable token edges from the current state engine
2. Support reserve-based and CL pool edges in that graph
3. Enumerate at least:
   - 2-hop cyclic routes
   - selected 3-hop cyclic routes
4. Use local quote primitives to estimate rough gross profit for candidate routes
5. Promote candidate opportunities above configurable thresholds
6. Expose candidate-generation metrics and validate via replay fixtures/tests

Keep this phase local-only.
No external routing APIs.
No simulation.
No execution.

==================================================
WORK AREAS
==================================================

Likely files/crates to work in:
- crates/arb_types
- crates/arb_state
- crates/arb_route
- crates/arb_filter
- crates/arb_metrics
- crates/arb_config
- bin/arb_daemon
- fixtures/
- docs if needed

==================================================
PART 1 — TYPE SYSTEM / ROUTE TYPES
==================================================

Add minimal route/candidate types as needed.

Expected types/directions:
- GraphNode / token node abstraction
- GraphEdge / pool edge abstraction
- RouteLeg
- RoutePath
- CandidateOpportunity
- NotionalBucket / QuoteSizeBucket
- CandidateRejectionReason (if useful)
- RootAsset / SearchAnchor config type if useful

Keep these minimal and serializable where useful.
Do not add execution payload types in this phase.

==================================================
PART 2 — POOL METADATA / GRAPH INPUTS
==================================================

A route graph requires truthful token/pool metadata.

If not already fully present in state:
- add an honest metadata layer for pools:
  - token0
  - token1
  - pool kind
  - fee tier / fee bps if available
- this metadata must be real enough for graph construction
- do NOT use empty placeholders
- if some metadata is unavailable in this phase, document the limitation honestly and skip unsupported pools rather than faking them

Graph construction should build:
- token nodes
- directed edges for each supported pool direction
- edge metadata including:
  - pool id
  - pool kind
  - token in / token out
  - fee info if available
  - freshness state
  - quote method availability

==================================================
PART 3 — LOCAL QUOTE WRAPPERS FOR EDGES
==================================================

Integrate existing quote primitives into graph edges.

Requirements:
1. Reserve-based edge quoting:
   - exact-in local quote
2. CL edge quoting:
   - exact-in local quote using the Phase 5 CL state foundation

Rules:
- no external router/API calls
- no simulation
- no gas-precise execution estimates
- no route search optimization yet
- quote functions should return:
  - output amount
  - rough gross delta
  - success/failure
  - failure reason if useful

If CL quote support is still partial:
- keep it honest
- skip unsupported paths safely
- add metrics for quote failures or unsupported routes

==================================================
PART 4 — ROUTE GRAPH CONSTRUCTION
==================================================

In `crates/arb_route`:
Build the route graph from current state snapshots.

Requirements:
- support directed edges
- support multiple edges between the same token pair (multigraph)
- support reserve-based pools
- support CL pools
- keep graph rebuild/update logic modular

Important:
If incremental graph maintenance is too complex for this phase, it is acceptable to:
- rebuild from current state snapshots on demand or on event batches
But document that honestly.

Do NOT add execution logic.

==================================================
PART 5 — CANDIDATE ENUMERATION
==================================================

Build the first candidate generator.

Required route families:
1. 2-hop cyclic routes:
   A -> B -> A
2. Selected 3-hop cyclic routes:
   A -> B -> C -> A

Use configurable root assets / anchors.
Examples could include:
- WETH
- USDC
- other explicitly configured roots

Use configurable notional sizes / quote buckets.
Do NOT overengineer dynamic sizing yet.

For each candidate:
- compute rough gross input/output
- compute gross profit
- compute gross bps
- include pool/path freshness info
- include route family metadata
- include reason if rejected

==================================================
PART 6 — FILTER / PROMOTION
==================================================

In `crates/arb_filter` (or equivalent):
Add a local-only candidate promotion layer.

Requirements:
- configurable minimum gross threshold
- configurable maximum staleness
- configurable root assets
- configurable max path length for this phase
- promote only candidates that pass all checks

No external APIs.
No simulation.
No execution.

If possible, return:
- promoted candidates
- rejected candidates with reason (for testing/metrics)

==================================================
PART 7 — METRICS / OBSERVABILITY
==================================================

Add metrics honestly for candidate generation.

Suggested metrics:
- arb_route_nodes_total
- arb_route_edges_total
- arb_candidates_considered_total
- arb_candidates_promoted_total
- arb_quote_failures_total
- arb_stale_pool_skips_total
- arb_candidate_gross_profit_estimate_total or equivalent if useful
- arb_candidate_cycles_2hop_total
- arb_candidate_cycles_3hop_total

Rules:
- do not fake metrics
- if a metric is not real, do not claim it

You may update observability wiring if needed, but do NOT turn this into a dashboard phase.
Metrics exposure is enough for now.

==================================================
PART 8 — DAEMON INTEGRATION
==================================================

In `bin/arb_daemon`:
Wire the current state engine into the route graph / candidate generation pipeline.

Expected shape:
provider -> ingest -> state -> graph -> candidate filter -> metrics/logging

Requirements:
- no execution
- no simulation
- no transaction building
- no database use on the hot path
- keep graceful shutdown intact

If helpful, add a lightweight dev/replay mode that prints top promoted candidates.

==================================================
PART 9 — FIXTURES / TESTS
==================================================

Add fixtures and tests that prove candidate generation works.

Required:
1. fixture-driven graph construction test
2. at least one 2-hop candidate generation test
3. at least one 3-hop candidate generation test
4. freshness gating test
5. quote-failure or unsupported-pool test
6. replay-driven end-to-end test:
   ingest -> state -> graph -> candidate promotion

Important:
At least one fixture/test should produce a positive promoted candidate.
No fabricated hidden shortcuts; keep fixtures honest enough for what they prove.

==================================================
PART 10 — DOCUMENTATION HONESTY
==================================================

Update docs/walkthrough/checklist so they clearly state:

Real after Phase 6:
- local graph exists
- candidate generation exists
- rough gross profit estimation exists
- promotion/filtering exists

Still deferred:
- simulation
- transaction building
- execution
- external router discovery/refinement
- dynamic EV learning layer
- production-grade incremental graph optimization (if not done here)

Do not oversell beyond what is implemented.

==================================================
VALIDATION REQUIRED
==================================================

Run and report:
- cargo check --workspace
- cargo test --workspace
- docker compose config (only if touched)
- if useful, a replay-driven candidate-generation smoke test

==================================================
SOURCE-OF-TRUTH OUTPUTS REQUIRED
==================================================

At the end, include these exact commands and outputs:

1. Git identity:
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-6-route-graph-filter
- git status --short
- git log --oneline --decorate -5

2. Grep proofs:
- git grep -n '2-hop\|2hop\|3-hop\|3hop' -- crates/ docs/
- git grep -n 'CandidateOpportunity\|RoutePath\|RouteLeg' -- crates/
- git grep -n 'arb_route_nodes_total\|arb_route_edges_total\|arb_candidates_considered_total\|arb_candidates_promoted_total' -- crates/
- git grep -n 'quote' -- crates/arb_route crates/arb_filter crates/arb_state

3. Validation outputs:
- cargo check --workspace
- cargo test --workspace
- docker compose config (if changed)

==================================================
REQUIRED OUTPUTS
==================================================

Provide:

1. A verdict:
- fully working
- working with known limitations
- blocked (and why)

2. A checklist confirming:
- route graph added
- reserve-based edges supported
- CL edges supported
- 2-hop candidate generation added
- 3-hop candidate generation added
- promotion/filtering added
- replay tests added
- no simulation/execution logic added

3. Changed-files summary

4. A walkthrough artifact describing:
- how graph nodes/edges are built
- how candidates are generated
- how rough gross profit is estimated
- how filtering works
- what remains deferred to the next phase

5. The source-of-truth outputs listed above

Do not go beyond Phase 6.

---- updates ----

Do a final Phase 6 merge-readiness pass on the EXISTING branch `phase-6-route-graph-filter`.

Do NOT create a new branch.
Do NOT add simulation, execution, or external router logic.
Do NOT expand scope beyond configuration cleanup + documentation honesty + required proof outputs.

Goal:
Make Phase 6 merge-ready by removing hardcoded route/filter settings from the daemon and making them configurable via arb_config.

==================================================
FIX 1 — MOVE ROUTE/FILTER SETTINGS INTO CONFIG
==================================================

Current problem:
The daemon currently hardcodes:
- root asset
- minimum gross profit
- minimum gross bps
- freshness requirement
- quote size buckets

Required fix:
Add Phase 6 route/filter config to `crates/arb_config`, for example:
- ROOT_ASSETS (or one ROOT_ASSET initially)
- MIN_GROSS_PROFIT
- MIN_GROSS_BPS
- REQUIRE_FRESH
- QUOTE_BUCKETS

Use simple env-driven parsing.
Keep it honest and minimal.
Do not overengineer.

Then update `bin/arb_daemon` to use these config values instead of hardcoded ones.

==================================================
FIX 2 — DOCUMENTATION HONESTY
==================================================

Update walkthrough/checklist/docs so they clearly state:
- route graph exists
- local candidate generation exists
- filter thresholds are now configurable
- route generation is still local-only
- simulation/execution remain deferred

Remove stale wording if Phase 5 walkthrough text is still mixed into the branch outputs.

==================================================
FIX 3 — SOURCE-OF-TRUTH OUTPUTS
==================================================

At the end, include these exact command outputs:

- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-6-route-graph-filter
- git status --short
- git log --oneline --decorate -5

And these grep proofs:

- git grep -n 'MIN_GROSS_PROFIT\|MIN_GROSS_BPS\|REQUIRE_FRESH\|ROOT_ASSET\|ROOT_ASSETS\|QUOTE_BUCKETS' -- crates/ bin/
- git grep -n 'QuoteSizeBucket\|CandidateOpportunity\|RoutePath\|RouteLeg' -- crates/
- git grep -n 'arb_route_nodes_total\|arb_route_edges_total\|arb_candidates_considered_total\|arb_candidates_promoted_total' -- crates/

Validation:
- cargo check --workspace
- cargo test --workspace

==================================================
REQUIRED OUTPUTS
==================================================

Provide:
1. Verdict
2. Changed-files summary
3. Checklist confirming:
   - route/filter config moved into arb_config
   - daemon no longer hardcodes root asset / thresholds / buckets
   - docs updated honestly
   - no simulation/execution logic added
4. Exact source-of-truth outputs listed above

---- updates 2 ----

Do a final Phase 6 merge-readiness pass on the EXISTING branch `phase-6-route-graph-filter`.

Do NOT create a new branch.
Do NOT add simulation, execution, or external router logic.
Do NOT expand scope beyond configuration cleanup + documentation honesty + proof outputs.

Goal:
Make Phase 6 merge-ready by removing hardcoded route/filter settings from the daemon and moving them into arb_config.

==================================================
FIX 1 — MOVE ROUTE/FILTER SETTINGS INTO CONFIG
==================================================

Current problem:
The daemon still hardcodes:
- root asset
- minimum gross profit
- minimum gross bps
- freshness requirement
- quote size buckets

Required fix:
Add route/filter settings to `crates/arb_config`:
- ROOT_ASSET (single string is fine for now)
- MIN_GROSS_PROFIT
- MIN_GROSS_BPS
- REQUIRE_FRESH
- QUOTE_BUCKETS

Use simple env parsing.
Keep it minimal and honest.

Then update `bin/arb_daemon/src/main.rs` to read those values from Config.
Remove the current hardcoded root asset / thresholds / buckets from the daemon.

==================================================
FIX 2 — DOCUMENTATION HONESTY
==================================================

Update the walkthrough/checklist/docs so they clearly state:
- local graph exists
- candidate generation exists
- filter thresholds are now config-driven
- route generation is still local-only
- simulation/execution remain deferred

Do not oversell beyond what is implemented.

==================================================
VALIDATION REQUIRED
==================================================

Run and report:

- cargo check --workspace
- cargo test --workspace

Source-of-truth outputs:
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-6-route-graph-filter
- git status --short
- git log --oneline --decorate -5

Proof greps:
- git grep -n 'ROOT_ASSET\|MIN_GROSS_PROFIT\|MIN_GROSS_BPS\|REQUIRE_FRESH\|QUOTE_BUCKETS' -- crates/arb_config/src/lib.rs bin/arb_daemon/src/main.rs
- git grep -n 'QuoteSizeBucket\|CandidateOpportunity\|RoutePath\|RouteLeg' -- crates/
- git grep -n 'arb_route_nodes_total\|arb_route_edges_total\|arb_candidates_considered_total\|arb_candidates_promoted_total' -- crates/

==================================================
REQUIRED OUTPUTS
==================================================

Provide:
1. Verdict
2. Changed-files summary
3. Checklist confirming:
   - route/filter config moved into arb_config
   - daemon no longer hardcodes root asset / thresholds / buckets
   - docs updated honestly
   - no simulation/execution logic added
4. Exact outputs for all source-of-truth and grep commands