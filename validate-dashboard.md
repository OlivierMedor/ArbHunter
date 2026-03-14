Perform a full Phase 2.5 end-to-end validation and repair pass on the CURRENT branch.

Goal:
Start the local observability stack and the daemon, validate that the browser-visible observability flow is actually working end to end, and fix any issues that prevent it from working.

Scope is STRICTLY limited to:
- provider / ingest / observability / dashboard validation
- Docker / Prometheus / Grafana / metrics endpoint issues
- docs / tests needed to support this validation
- no strategy logic
- no routing logic
- no state-engine logic
- no simulation logic
- no execution logic

Do NOT create a new branch.
Do NOT merge anything.
Do NOT commit secrets.
Do NOT print secrets from .env.
Do NOT rewrite unrelated code.

==================================================
ASSUMPTIONS / RULES
==================================================

1. Use the local existing `.env` if present, but NEVER print its contents.
2. Do not create or commit a real `.env`.
3. Keep `.env.example` as template only.
4. If required credentials are missing and the system cannot run, stop and report that clearly instead of fabricating values.
5. Use the browser for actual validation, not just code inspection.
6. If something is broken, inspect logs, browser errors, and config, then fix only what is necessary.
7. Stay within Phase 2.5 observability/provider-ingest scope only.

==================================================
VALIDATION TARGETS
==================================================

You must validate all of the following:

A. Metrics endpoint
- Start the daemon
- Confirm http://localhost:9090/metrics loads in the browser
- Confirm it returns Prometheus plaintext metrics
- Confirm expected metrics are visible, including if implemented:
  - arb_provider_connected
  - arb_active_provider
  - arb_flashblocks_seen_total
  - arb_pending_logs_seen_total
  - arb_provider_frames_forwarded_total
  - arb_malformed_payloads_total
  - arb_daemon_uptime_seconds
  - arb_metrics_requests_total

B. Prometheus
- Start Prometheus via Docker Compose
- Open http://localhost:9091 in browser
- Navigate to Status -> Targets
- Confirm the daemon target is UP
- If it is DOWN, diagnose and fix the cause

C. Grafana
- Start Grafana via Docker Compose
- Open http://localhost:3000 in browser
- Confirm Grafana loads
- Confirm Prometheus datasource works
- Confirm the ArbHunter observability dashboard exists and loads
- Confirm panels render without datasource errors
- Confirm the panel wiring matches the current metric names/UID configuration

D. Runtime/log validation
- Confirm the daemon starts cleanly
- Confirm provider connections initialize
- Confirm there are no immediate crashes/panics
- Confirm graceful shutdown still works if practical

==================================================
EXECUTION PLAN
==================================================

Perform the work in this order:

1. Inspect current branch and relevant files
2. Start Docker services needed for observability
   - prometheus
   - grafana
3. Start arb_daemon locally
4. Validate metrics endpoint in browser
5. Validate Prometheus target health in browser
6. Validate Grafana dashboard in browser
7. If anything fails:
   - inspect terminal output
   - inspect docker compose logs
   - inspect browser/network/errors
   - patch code/config/docs/tests as needed
   - rerun validation
8. Repeat until the observability stack works or until blocked by missing local credentials/config

==================================================
ALLOWED FILES TO MODIFY
==================================================

Modify only what is necessary, likely including:
- docker-compose.yml
- infra/prometheus/prometheus.yml
- infra/grafana/provisioning/datasources/prometheus.yml
- infra/grafana/provisioning/dashboards/files/observability.json
- crates/arb_metrics/src/lib.rs
- crates/arb_providers/src/lib.rs
- crates/arb_ingest/src/lib.rs
- bin/arb_daemon/src/main.rs
- docs / quick-start / walkthrough / checklist files
- tests related to metrics/provider/ingest/observability

Do NOT modify unrelated strategy/execution files.

==================================================
IF ISSUES ARE FOUND
==================================================

You are allowed to fix:
- bad ports
- broken docker config
- broken Grafana datasource wiring
- broken dashboard panel references
- missing metric increments
- broken provider -> ingest observability hooks
- docs that no longer match reality
- tests needed to verify observability behavior

You are NOT allowed to add:
- route finding
- trade logic
- arbitrage logic
- state engine logic
- simulation logic
- transaction submission logic

==================================================
TESTS / CHECKS TO RUN
==================================================

After any fixes, run and report:

1. cargo check --workspace
2. cargo test --workspace
3. docker compose config
4. docker compose ps
5. docker compose logs prometheus --tail=100
6. docker compose logs grafana --tail=100
7. if useful: daemon startup logs / relevant terminal output

If practical, also add/update:
- metrics smoke test
- daemon startup smoke test
- observability config sanity test

==================================================
REQUIRED BROWSER VALIDATION ARTIFACTS
==================================================

Provide evidence from the browser, such as:
- screenshot or artifact of /metrics loading
- screenshot or artifact of Prometheus targets page showing UP
- screenshot or artifact of Grafana dashboard loading correctly

If the browser cannot complete a step, say exactly why.

==================================================
REQUIRED OUTPUTS
==================================================

When finished, provide ALL of the following:

1. A concise verdict:
- fully working
- working with known limitations
- blocked (and why)

2. A checklist confirming:
- daemon starts
- /metrics loads at localhost:9090
- Prometheus loads at localhost:9091
- Prometheus target is UP
- Grafana loads at localhost:3000
- Grafana datasource works
- dashboard loads without datasource errors
- no strategy/sim/execution logic was added

3. A changed-files summary

4. A walkthrough artifact describing:
- what was validated
- what was broken
- what was fixed
- what remains deferred to Phase 3

5. Validation outputs for:
- cargo check --workspace
- cargo test --workspace
- docker compose config
- docker compose ps

6. Browser validation artifacts/screenshots for:
- /metrics
- Prometheus target page
- Grafana dashboard

Do not go beyond this scope.