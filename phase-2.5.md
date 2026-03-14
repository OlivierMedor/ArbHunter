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


---- update 2.5 ---

Implement one final Phase 2.5 merge-readiness fix pass on the EXISTING branch `phase-2_5-observability-dashboard`.

Do NOT create a new branch.
Do NOT add strategy, routing, state-engine, simulation, or execution logic.
Do NOT expand scope beyond the 5 concrete fixes below.

Goal:
Make Phase 2.5 actually merge-ready by fixing the remaining observability mismatches.

==================================================
FIX 1 — RESOLVE PORT CONFLICT FOR REAL
==================================================

Current problem:
- docker-compose still exposes Prometheus on host port 9090
- arb_daemon metrics endpoint is also intended to use METRICS_PORT=9090
- walkthrough says Prometheus should be on 9091, but code/config does not match

Required fix:
- Keep arb_daemon metrics endpoint on METRICS_PORT (default 9090)
- Change Prometheus host exposure to 9091:9090
- Keep Grafana on 3000
- Update quick-start / walkthrough text accordingly

Acceptance criteria:
- daemon metrics endpoint = http://localhost:9090/metrics
- Prometheus UI = http://localhost:9091
- Grafana UI = http://localhost:3000

==================================================
FIX 2 — ADD EXPLICIT GRAFANA DATASOURCE UID
==================================================

Current problem:
- Grafana datasource provisioning has no explicit uid
- observability.json mixes datasource uid values "prometheus" and "Prometheus"

Required fix:
- In `infra/grafana/provisioning/datasources/prometheus.yml`, add explicit:
  uid: arbhunter-prometheus
- In `infra/grafana/provisioning/dashboards/files/observability.json`, make every panel use:
  datasource.uid = "arbhunter-prometheus"

Acceptance criteria:
- one consistent datasource uid everywhere
- no mixed-case datasource uid values remain

==================================================
FIX 3 — IMPLEMENT OR REMOVE CLAIMED METRICS HONESTLY
==================================================

Current problem:
The checklist claims metrics exist that do not currently appear implemented.

Required fix:
Implement these metrics in `crates/arb_metrics` and wire them where appropriate:
- provider_frames_forwarded_total
- malformed_payloads_total
- daemon_startups_total
- metrics_requests_total
- daemon_uptime_seconds

Also implement a REAL provider state gauge:
Preferred:
- arb_active_provider{provider="quicknode"} = 1/0
- arb_active_provider{provider="alchemy"} = 1/0
and/or
- arb_provider_connected{provider="quicknode"} = 1/0
- arb_provider_connected{provider="alchemy"} = 1/0

Rules:
- If a metric cannot be made real in this phase, REMOVE it from the checklist/walkthrough claims instead of pretending it exists.
- Do not fake latency. If latency is still TODO, say so honestly.

==================================================
FIX 4 — INCREMENT METRICS IN THE ACTUAL LIVE PATH
==================================================

Required fix:
Ensure the actual provider -> ingest path increments the relevant metrics:

When a provider frame is forwarded:
- increment provider_frames_forwarded_total

When arb_ingest successfully parses a message:
- increment events_ingested_total

When a Flashblock-like message is parsed:
- increment flashblocks_seen_total

When a pending-log-like message is parsed:
- increment pending_logs_seen_total

When parsing fails / malformed payload:
- increment malformed_payloads_total

When daemon starts:
- increment daemon_startups_total

When /metrics is requested:
- increment metrics_requests_total

daemon_uptime_seconds:
- expose as a real uptime gauge/counter based on process start time

==================================================
FIX 5 — REPLACE THE MISLEADING STATUS PANEL
==================================================

Current problem:
The dashboard still derives provider status from:
- arb_provider_connected_total - arb_provider_disconnected_total

That is not a stable live status signal.

Required fix:
Replace that panel with one based on real gauges:
- Active Provider
- QuickNode Connected
- Alchemy Connected

Keep or add these panels:
- Active Provider (stat)
- QuickNode Connected (stat)
- Alchemy Connected (stat)
- Reconnect Attempts (time series)
- Failover Switches (time series or stat)
- Flashblock Events / min
- Pending Log Events / min
- Malformed Payloads
- Provider Frames Forwarded
- Daemon Uptime

==================================================
FILES LIKELY TO CHANGE
==================================================

Only modify what is necessary, likely:
- docker-compose.yml
- infra/grafana/provisioning/datasources/prometheus.yml
- infra/grafana/provisioning/dashboards/files/observability.json
- crates/arb_metrics/src/lib.rs
- crates/arb_providers/src/lib.rs
- crates/arb_ingest/src/lib.rs
- bin/arb_daemon/src/main.rs
- phase-2.5 docs/walkthrough if needed

==================================================
VALIDATION REQUIRED
==================================================

After the fix pass, provide:

1. Checklist confirming:
- Prometheus moved to 9091
- datasource uid standardized to arbhunter-prometheus
- missing metrics implemented or honestly removed from claims
- provider_frames_forwarded_total is wired in the real path
- malformed_payloads_total is wired in the real path
- daemon_startups_total is real
- metrics_requests_total is real
- daemon_uptime_seconds is real
- status panels now use real gauges
- no strategy/sim/execution logic was added

2. Changed-files summary

3. Walkthrough artifact describing:
- what ports are used now
- how metrics flow from daemon -> Prometheus -> Grafana
- which metrics are real now
- what remains deferred to Phase 3

4. Validation results for:
- cargo check --workspace
- cargo test --workspace
- docker compose config

Do not go beyond Phase 2.5.


--- updates 3.5 ---

Implement one FINAL targeted fix pass on the EXISTING branch `phase-2_5-observability-dashboard`.

Do NOT create a new branch.
Do NOT add any new features.
Do NOT add strategy logic, routing, state engine, simulation, or execution logic.
Do NOT expand scope beyond the 4 concrete fixes below.

Goal:
Make Phase 2.5 actually merge-ready by fixing the exact mismatches between the checklist/walkthrough and the current code/config.

IMPORTANT:
Do not claim success unless the verification section at the end passes exactly.

==================================================
FIX 1 — PROMETHEUS PORT CONFLICT
==================================================

Current problem:
- arb_daemon metrics endpoint is intended to run on METRICS_PORT=9090
- docker-compose still exposes Prometheus on host port 9090
- quick-start says Prometheus should be on 9091

Required fix:
- Keep arb_daemon metrics endpoint on port 9090
- Change Prometheus host mapping to:
  9091:9090
- Keep Grafana on 3000
- Update walkthrough / quick-start text to match this exactly

Acceptance criteria:
- daemon metrics endpoint URL = http://localhost:9090/metrics
- Prometheus UI URL = http://localhost:9091
- Grafana UI URL = http://localhost:3000

==================================================
FIX 2 — GRAFANA DATASOURCE UID CONSISTENCY
==================================================

Current problem:
- datasource provisioning does not define an explicit uid
- observability.json mixes "Prometheus" and "prometheus" datasource uid values

Required fix:
- In:
  infra/grafana/provisioning/datasources/prometheus.yml
  define the datasource with explicit:
    uid: arbhunter-prometheus

- In:
  infra/grafana/provisioning/dashboards/files/observability.json
  every panel must use exactly:
    datasource.uid = "arbhunter-prometheus"

Rules:
- No mixed-case uid variants
- No panel should rely on ambiguous datasource names
- Standardize every panel to the same explicit uid

==================================================
FIX 3 — IMPLEMENT THE MISSING METRICS FOR REAL
==================================================

Current problem:
The checklist claims metrics exist that do not currently appear implemented or wired.

Required metrics to implement and wire honestly:
- metrics_requests_total
- daemon_uptime_seconds
- provider_frames_forwarded_total
- malformed_payloads_total
- arb_active_provider gauge(s)
- arb_provider_connected gauge(s)

Preferred provider gauge model:
- arb_active_provider{provider="quicknode"} = 1 or 0
- arb_active_provider{provider="alchemy"} = 1 or 0
- arb_provider_connected{provider="quicknode"} = 1 or 0
- arb_provider_connected{provider="alchemy"} = 1 or 0

Required wiring:
- metrics_requests_total increments every time /metrics is requested
- daemon_startups_total increments at daemon startup
- daemon_uptime_seconds reflects real process uptime
- provider_frames_forwarded_total increments when a provider frame is forwarded downstream
- malformed_payloads_total increments when ingest fails to parse malformed payloads
- active provider gauges update when provider manager changes active provider state
- connected gauges update when provider sockets connect/disconnect

Rules:
- If any metric cannot be made real in this phase, remove it from checklist/walkthrough claims instead of faking it
- Do NOT fake latency
- If latency is still TODO, keep it explicitly deferred

==================================================
FIX 4 — REPLACE THE MISLEADING STATUS PANEL
==================================================

Current problem:
The dashboard still uses:
  arb_provider_connected_total - arb_provider_disconnected_total
as a status proxy.

Required fix:
Remove that misleading panel/query.

Replace with panels based on real gauges:
- Active Provider (stat)
- QuickNode Connected (stat)
- Alchemy Connected (stat)

Also keep/create these useful panels:
- Reconnect Attempts
- Failover Switches
- Flashblock Events / min
- Pending Log Events / min
- Malformed Payloads
- Provider Frames Forwarded
- Daemon Uptime

Rules:
- Use real gauge metrics, not derived pseudo-status math
- Keep dashboard read-only and operator-focused

==================================================
FILES LIKELY TO CHANGE
==================================================

Only modify what is necessary, likely:
- docker-compose.yml
- infra/grafana/provisioning/datasources/prometheus.yml
- infra/grafana/provisioning/dashboards/files/observability.json
- crates/arb_metrics/src/lib.rs
- crates/arb_providers/src/lib.rs
- crates/arb_ingest/src/lib.rs
- bin/arb_daemon/src/main.rs
- any observability docs/walkthrough text if needed

Do NOT touch unrelated files.

==================================================
VALIDATION REQUIRED
==================================================

After making the fixes, provide ALL of the following:

1. Checklist confirming:
- Prometheus now maps to 9091:9090
- datasource uid is exactly arbhunter-prometheus everywhere
- metrics_requests_total is real
- daemon_uptime_seconds is real
- provider_frames_forwarded_total is real
- malformed_payloads_total is real
- arb_active_provider gauge is real
- arb_provider_connected gauge is real
- misleading provider status panel was removed/replaced
- no strategy/sim/execution logic was added

2. Changed-files summary

3. Walkthrough artifact describing:
- exact ports now used
- how metrics flow from daemon -> Prometheus -> Grafana
- which metrics are newly real
- what remains deferred to Phase 3

4. Validation command results for:
- cargo check --workspace
- cargo test --workspace
- docker compose config

5. Exact proof snippets (VERY IMPORTANT):
Show the relevant snippets or outputs proving:
- docker-compose.yml contains 9091:9090 for Prometheus
- prometheus.yml datasource provisioning contains uid: arbhunter-prometheus
- observability.json references arbhunter-prometheus consistently
- metrics code contains metrics_requests_total
- metrics code contains daemon_uptime_seconds
- metrics code contains provider_frames_forwarded_total
- metrics code contains malformed_payloads_total
- metrics code contains arb_active_provider
- metrics code contains arb_provider_connected

Do not go beyond Phase 2.5.
Do not claim success unless all proof snippets are included.


--- updates 4.5 ---
Do one FINAL targeted fix pass on the EXISTING branch `phase-2_5-observability-dashboard`.

Do NOT create a new branch.
Do NOT add any strategy, routing, state-engine, simulation, or execution logic.
Do NOT add unrelated features.
Do NOT change repo structure.

Goal:
Fix the exact remaining mismatches between the branch contents and the checklist/walkthrough so the branch becomes honestly merge-ready.

IMPORTANT:
Do not claim success unless the exact proof commands at the end show the fixes in the actual files.

==================================================
FIX 1 — PROMETHEUS HOST PORT
==================================================

Current problem:
`docker-compose.yml` still exposes Prometheus on host port 9090, which conflicts with the daemon metrics endpoint that is supposed to be on 9090.

Required fix:
In `docker-compose.yml`:
- keep daemon metrics endpoint on 9090
- change Prometheus port mapping to:
  9091:9090
- keep Grafana on 3000

Also update any quick-start / walkthrough text to match:
- daemon metrics = http://localhost:9090/metrics
- Prometheus UI = http://localhost:9091
- Grafana UI = http://localhost:3000

==================================================
FIX 2 — GRAFANA DATASOURCE UID
==================================================

Current problem:
`infra/grafana/provisioning/datasources/prometheus.yml` has no explicit uid, and the dashboard JSON still references the wrong datasource uid/name.

Required fix:
In `infra/grafana/provisioning/datasources/prometheus.yml`:
- add:
  uid: arbhunter-prometheus

In `infra/grafana/provisioning/dashboards/files/observability.json`:
- every panel must use datasource.uid = "arbhunter-prometheus"
- remove any remaining "Prometheus" / "prometheus" uid variants

Do not leave any panel using an ambiguous datasource name.

==================================================
FIX 3 — METRICS CLAIMS MUST MATCH REAL CODE
==================================================

Current problem:
The checklist/walkthrough claim metrics that do not visibly exist in `crates/arb_metrics/src/lib.rs`.

Required action:
Choose ONE of these two paths, but be honest.

PATH A (preferred):
Implement these metrics for real and wire them honestly:
- arb_metrics_requests_total
- arb_daemon_uptime_seconds
- arb_provider_frames_forwarded_total
- arb_malformed_payloads_total
- arb_active_provider (gauge or gauge vec)
- arb_provider_connected (gauge or gauge vec)

Required behavior if implementing:
- arb_metrics_requests_total increments when /metrics is requested
- arb_daemon_uptime_seconds reflects real process uptime
- arb_provider_frames_forwarded_total increments when a provider frame is forwarded downstream
- arb_malformed_payloads_total increments on malformed payload parse failures
- arb_active_provider updates when the provider manager changes active provider
- arb_provider_connected updates on connect/disconnect state

PATH B (allowed only if you cannot implement PATH A cleanly in this phase):
- remove all claims about these metrics from:
  - checklist
  - walkthrough
  - quick-start
  - dashboard panels
- do NOT pretend they exist if they do not

Whichever path you choose, the repo contents and docs must agree exactly.

==================================================
FIX 4 — REMOVE THE MISLEADING STATUS PANEL
==================================================

Current problem:
The dashboard still uses a misleading derived status panel based on:
provider_connected_total - provider_disconnected_total

Required fix:
Remove that pseudo-status logic.

Replace with panels based on real gauges ONLY.
If PATH A above is chosen and the gauges are implemented, use:
- Active Provider
- QuickNode Connected
- Alchemy Connected

If PATH B above is chosen and those gauges are not implemented, then:
- remove the misleading status panel and remove any checklist/walkthrough claims that such live status exists

==================================================
FILES ALLOWED TO CHANGE
==================================================

Only modify what is necessary, likely:
- docker-compose.yml
- infra/grafana/provisioning/datasources/prometheus.yml
- infra/grafana/provisioning/dashboards/files/observability.json
- crates/arb_metrics/src/lib.rs
- crates/arb_providers/src/lib.rs
- crates/arb_ingest/src/lib.rs
- bin/arb_daemon/src/main.rs
- checklist.md / walkthrough.md / quick-start text if needed

Do not touch unrelated files.

==================================================
REQUIRED VERIFICATION — MUST INCLUDE ACTUAL OUTPUT
==================================================

After making the fixes, run and include the exact outputs of:

1. Prove the Prometheus host port mapping:
- `git grep -n '9091:9090' -- docker-compose.yml`

2. Prove the datasource uid exists in provisioning:
- `git grep -n 'uid: arbhunter-prometheus' -- infra/grafana/provisioning/datasources/prometheus.yml`

3. Prove the dashboard JSON references that uid:
- `git grep -n 'arbhunter-prometheus' -- infra/grafana/provisioning/dashboards/files/observability.json`

4. If PATH A was chosen, prove each metric exists in code:
- `git grep -n 'arb_metrics_requests_total' -- crates/`
- `git grep -n 'arb_daemon_uptime_seconds' -- crates/ bin/`
- `git grep -n 'arb_provider_frames_forwarded_total' -- crates/ bin/`
- `git grep -n 'arb_malformed_payloads_total' -- crates/ bin/`
- `git grep -n 'arb_active_provider' -- crates/ bin/`
- `git grep -n 'arb_provider_connected' -- crates/ bin/`

5. If PATH B was chosen, prove the claims were removed:
- `git grep -n 'metrics_requests_total\|daemon_uptime_seconds\|provider_frames_forwarded_total\|malformed_payloads_total\|active provider\|QuickNode Connected\|Alchemy Connected' -- checklist.md walkthrough.md docs/ infra/grafana/provisioning/dashboards/files/observability.json`

6. Validation commands:
- `cargo check --workspace`
- `cargo test --workspace`
- `docker compose config`

==================================================
OUTPUT FORMAT
==================================================

When finished, provide ONLY:

1. Which path was chosen:
- PATH A (implemented metrics)
or
- PATH B (removed unsupported claims)

2. Changed-files summary

3. Checklist confirming:
- Prometheus port conflict fixed
- datasource uid fixed
- dashboard datasource references fixed
- misleading status panel removed/replaced
- docs/checklist now match actual code
- no strategy/sim/execution logic added

4. The exact verification command outputs listed above

5. A short walkthrough:
- exact ports now used
- what is now real in observability
- what remains deferred to Phase 3

Do not go beyond this scope.
Do not claim success without the grep outputs.