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