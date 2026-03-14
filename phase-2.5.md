Implement Phase 2.5 only on a new branch for an observability dashboard.

Suggested branch name:
phase-2_5-observability-dashboard

Goal:
Build a minimal, read-only operator dashboard so I can visually validate that the Base provider + ingestion foundation from Phase 2 is actually working.

This phase is strictly about:
- observability
- health monitoring
- metrics exposure
- dashboard visualization

It is NOT about:
- strategy logic
- routing
- state engine
- simulation
- execution
- trading UI
- control-plane actions

========================================
HIGH-LEVEL GOAL
========================================

I want a simple visual dashboard that lets me answer:

- Is the daemon running?
- Is QuickNode connected?
- Is Alchemy connected?
- Which provider is currently active?
- Are failovers happening?
- Are Flashblock-like events arriving?
- Are pending-log-like events arriving?
- Is the provider -> ingest bridge alive?
- Are replay tests / fixture replays working?

This dashboard must be:
- read-only
- non-blocking
- off the hot path
- safe to run locally
- useful for validating Phase 2 before Phase 3

========================================
ARCHITECTURE REQUIREMENTS
========================================

Build a lightweight observability layer using:
- Prometheus
- Grafana
- existing Rust metrics instrumentation
- Docker Compose integration

Important constraints:
- Do NOT add Postgres to the hot path
- Do NOT add strategy/state/routing/sim/execution logic
- Do NOT make Grafana or Prometheus dependencies of the hot path
- The dashboard must consume exported metrics, not directly inspect hot-path internals in a blocking way
- No secrets in tracked files

========================================
IMPLEMENTATION SCOPE
========================================

Work in these areas as needed:
- crates/arb_metrics
- crates/arb_providers
- crates/arb_ingest
- bin/arb_daemon
- infra/
- docker-compose.yml
- docs/ if needed for observability instructions

========================================
PART 1 — METRICS EXPOSURE
========================================

Enhance the metrics system so the daemon exposes a scrapeable metrics endpoint on:
- METRICS_PORT
(default already expected to be 9090)

Requirements:
1. Add an HTTP metrics endpoint suitable for Prometheus scraping.
2. Reuse existing counters where possible.
3. Add or refine metrics for:

Provider / health metrics:
- provider_connected_total
- provider_disconnected_total
- provider_reconnect_attempts_total
- provider_failover_switches_total
- active_provider (gauge or equivalent state exposure)
- provider_latency_ms if available, or clearly expose placeholder / unavailable state honestly

Ingestion metrics:
- events_ingested_total
- flashblocks_seen_total
- pending_logs_seen_total
- malformed_payloads_total
- replay_events_processed_total (if practical)
- provider_frames_forwarded_total

Daemon/runtime metrics:
- daemon_uptime_seconds
- metrics_requests_total
- daemon_startups_total

Important:
- If queue depth cannot be measured cheaply/safely, do NOT fake it.
- Be honest in what is real vs what is still TODO.

========================================
PART 2 — PROMETHEUS INTEGRATION
========================================

Add Prometheus support in the local stack.

Requirements:
1. Create Prometheus config under infra/ (or similar appropriate location).
2. Configure Prometheus to scrape the daemon metrics endpoint.
3. Keep the config simple and local-development focused.
4. Do not overengineer service discovery.

Expected result:
- `docker compose up` should be able to run the daemon + Prometheus stack cleanly.

========================================
PART 3 — GRAFANA DASHBOARD
========================================

Add Grafana to the local stack.

Requirements:
1. Add Grafana service to docker-compose.
2. Provision Prometheus as a datasource automatically if practical.
3. Create a minimal dashboard JSON or provisioning config with useful panels for:

Required panels:
- daemon up / uptime
- active provider
- provider reconnects over time
- failover switch count
- Flashblock-like events per minute
- pending-log-like events per minute
- malformed payload count
- frames forwarded count

Nice-to-have:
- latency panel if real latency is available
- replay harness events processed

4. Keep the dashboard read-only and operator-focused.

========================================
PART 4 — DAEMON / LOCAL UX
========================================

In `bin/arb_daemon`:
1. Start the metrics endpoint cleanly during daemon startup.
2. Log the metrics URL on startup.
3. Log Prometheus/Grafana local access info if useful.
4. Keep graceful shutdown behavior intact.

No strategy logic.
No controls.
No admin actions.

========================================
PART 5 — TESTING / VALIDATION
========================================

Add or improve tests for:
1. metrics registry behavior
2. metrics endpoint smoke test
3. daemon startup with metrics enabled
4. docker compose config validity if practical
5. optional integration test that increments counters and verifies scrape output contains expected metric names

Important:
- tests must not require real QuickNode/Alchemy credentials unless explicitly marked as manual/local smoke tests
- unit/integration tests should still pass in CI/local dev without live endpoints

========================================
PART 6 — DOCUMENTATION
========================================

Update docs as needed so the repo clearly explains:
- what Phase 2.5 is
- how to start the local observability stack
- where Prometheus and Grafana are exposed
- what the dashboard validates
- what it does NOT validate yet

Be explicit that this is:
- an operator health dashboard
- not a trading dashboard
- not proof of profitable execution
- not Phase 3 state validation yet

========================================
DELIVERABLES
========================================

When finished, provide:

1. A checklist confirming:
- metrics endpoint added
- Prometheus added
- Grafana added
- daemon metrics exported
- provider/ingest health metrics exposed
- dashboard panels created
- no strategy/sim/execution logic added

2. A changed-files summary

3. A walkthrough artifact describing:
- how metrics flow from daemon to Prometheus to Grafana
- what each dashboard panel means
- what is real vs still placeholder
- how to run and verify the stack locally

4. A quick start section with exact commands for:
- starting the stack
- opening Prometheus
- opening Grafana
- verifying the metrics endpoint manually

Do not go beyond Phase 2.5.
Do not start Phase 3 state work yet.


--- update 1 ---

Implement a final Phase 2.5 observability fix pass on the EXISTING branch `phase-2_5-observability-dashboard`.

Do NOT create a new branch.
Do NOT add strategy, routing, simulation, state-engine, or execution logic.
Do NOT expand scope beyond observability correctness and dashboard wiring.

Goal:
Make Phase 2.5 merge-ready by fixing the concrete observability issues identified in review.

==================================================
SCOPE
==================================================

This is a targeted fix pass for:
1. port conflicts
2. Grafana datasource/dashboard consistency
3. missing metrics promised by the checklist
4. ingest metrics not being incremented in the real path
5. a misleading provider-status panel

Do not change anything outside this scope unless absolutely required.

==================================================
ISSUE 1 — FIX PORT CONFLICT
==================================================

Current problem:
- The daemon metrics endpoint binds to METRICS_PORT (expected 9090)
- Prometheus is also exposed on host port 9090 in docker-compose
- The quick start instructions therefore conflict

Required fix:
- Keep the daemon metrics endpoint on METRICS_PORT (default 9090)
- Move Prometheus host exposure to a different port, e.g. 9091
- Keep Grafana on 3000
- Update docs/walkthrough/quick-start accordingly

Acceptance criteria:
- daemon metrics endpoint available at http://localhost:9090/metrics
- Prometheus UI available at http://localhost:9091
- Grafana UI available at http://localhost:3000

==================================================
ISSUE 2 — FIX GRAFANA DATASOURCE UID CONSISTENCY
==================================================

Current problem:
- Datasource provisioning does not define one explicit uid consistently
- Dashboard JSON uses inconsistent datasource uid/name values

Required fix:
- In Grafana datasource provisioning, define one explicit datasource UID:
  arbhunter-prometheus
- In every panel in observability.json, use that same datasource UID consistently
- Do not mix "Prometheus", "prometheus", or other variants

Acceptance criteria:
- datasource provisioning explicitly sets uid = arbhunter-prometheus
- all panels reference exactly that uid
- no panel depends on ambiguous datasource names

==================================================
ISSUE 3 — IMPLEMENT OR CORRECT MISSING METRICS
==================================================

Current problem:
The checklist/walkthrough promise metrics that are not fully implemented.

Required fix:
Add or expose these metrics in `crates/arb_metrics` and wire them honestly:

Required metrics:
- provider_connected_total
- provider_disconnected_total
- provider_reconnect_attempts_total
- provider_failover_switches_total
- provider_frames_forwarded_total
- events_ingested_total
- flashblocks_seen_total
- pending_logs_seen_total
- malformed_payloads_total
- daemon_startups_total
- metrics_requests_total
- daemon_uptime_seconds

Provider status metrics:
Implement a real active-provider / connection-health gauge.
Preferred implementation:
- arb_active_provider{provider="quicknode"} = 1/0
- arb_active_provider{provider="alchemy"} = 1/0
- arb_provider_connected{provider="quicknode"} = 1/0
- arb_provider_connected{provider="alchemy"} = 1/0

Latency:
- If real latency is not implemented yet, do NOT fake it
- Either expose a clearly honest placeholder metric/state or omit it from dashboards
- Keep TODOs explicit

Rules:
- Be honest
- If a metric cannot be made real in this phase, remove it from the checklist/dashboard claims instead of faking it

==================================================
ISSUE 4 — INCREMENT INGEST METRICS IN THE REAL PATH
==================================================

Current problem:
Metrics exist, but the real provider -> ingest path may not increment them correctly.

Required fix:
Ensure the following happen in the actual live path:

When a provider frame is forwarded downstream:
- increment provider_frames_forwarded_total

When arb_ingest successfully parses and normalizes an event:
- increment events_ingested_total

When a Flashblock-like event is normalized:
- increment flashblocks_seen_total

When a pending-log-like event is normalized:
- increment pending_logs_seen_total

When a payload is malformed / fails parse:
- increment malformed_payloads_total

If replay harness processes fixture lines:
- optionally increment replay-specific counts if already supported
- otherwise do not invent new claims

Important:
These increments must happen in the actual provider -> ingest flow, not just in tests.

==================================================
ISSUE 5 — REPLACE MISLEADING STATUS PANEL
==================================================

Current problem:
A panel currently derives provider health from connected_total - disconnected_total, which is not a stable status signal.

Required fix:
Replace the misleading provider status panel with one based on a real gauge:
- active provider
and/or
- provider connected state

Suggested dashboard panels:
1. Active Provider (stat)
2. QuickNode Connected (stat)
3. Alchemy Connected (stat)
4. Reconnect Attempts (time series)
5. Failover Switch Count (stat or time series)
6. Flashblock Events / min (time series)
7. Pending Log Events / min (time series)
8. Malformed Payloads (time series/stat)
9. Provider Frames Forwarded (time series)
10. Daemon Uptime (stat)

If latency is still not real, keep it out of the primary dashboard or clearly mark it as TODO.

==================================================
FILES MOST LIKELY TO CHANGE
==================================================

You may modify only what is necessary, likely including:
- crates/arb_metrics/src/lib.rs
- crates/arb_providers/src/lib.rs
- crates/arb_ingest/src/lib.rs
- bin/arb_daemon/src/main.rs
- docker-compose.yml
- infra/prometheus/prometheus.yml
- infra/grafana/provisioning/datasources/prometheus.yml
- infra/grafana/provisioning/dashboards/files/observability.json
- docs / walkthrough / quick-start text if needed

Do not add unrelated files or features.

==================================================
TESTING / VALIDATION
==================================================

Add or update validation so the branch is honestly merge-ready.

Required validation:
1. cargo check --workspace
2. cargo test --workspace
3. docker compose config
4. Metrics endpoint smoke test if practical
5. Verify that dashboard JSON and datasource provisioning are consistent

If possible, include a lightweight test or assertion that:
- metrics_requests_total increments when /metrics is hit
- provider_frames_forwarded_total increments when a provider frame is forwarded
- malformed_payloads_total increments on invalid payload

Do not require real credentials for unit tests.

==================================================
DOCUMENTATION HONESTY
==================================================

Update the checklist / walkthrough / quick start so they are precise:

Must be true after this phase:
- daemon metrics endpoint is real
- Prometheus is wired
- Grafana is wired
- provider -> ingest metrics are real
- active provider / connection status is real
- malformed payload tracking is real

Must NOT be overstated:
- true latency measurement if still TODO
- anything related to strategy, routing, simulation, or execution

Also update the quick-start URLs so they match the corrected ports.

==================================================
REQUIRED OUTPUTS
==================================================

When finished, provide:

1. A checklist confirming:
- port conflict resolved
- datasource uid is consistent
- missing metrics implemented or honestly removed from claims
- ingest metrics increment in the real path
- provider status panel now uses real gauges
- no strategy/sim/execution logic was added

2. A changed-files summary

3. A walkthrough artifact explaining:
- what ports are now used
- how metrics flow from daemon -> Prometheus -> Grafana
- which metrics are real now
- what is still explicitly deferred to Phase 3

4. A quick validation section with outputs/results for:
- cargo check --workspace
- cargo test --workspace
- docker compose config

Do not go beyond Phase 2.5.