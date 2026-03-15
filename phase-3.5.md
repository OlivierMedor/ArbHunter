Implement Phase 3.5 only on a new branch.

Suggested branch name:
phase-3_5-state-dashboard-validation

Goal:
Update the observability layer so the newly merged Phase 3 state engine can be visually validated in the browser, then run a real end-to-end validation pass and fix any issues found.

Scope:
- observability/dashboard updates
- metrics exposure for Phase 3 state engine
- browser validation
- replay/fixture-driven validation if needed
- docs/tests updates if needed
- no routing logic
- no strategy logic
- no simulation logic
- no execution logic

==================================================
PART 1 — ADD STATE ENGINE PANELS TO OBSERVABILITY
==================================================

Update the Grafana dashboard so it shows the new Phase 3 state-engine metrics if they exist in the daemon metrics endpoint.

Add/read panels for:
- Pools Tracked
- State Updates / min
- Stale Pool Events
- Daemon Uptime
- Existing provider/ingest panels should continue to work

If any state metric names differ from expectation, inspect the code and use the actual names.

Do not add fake panels.
Only show metrics that really exist.

==================================================
PART 2 — VERIFY METRICS FLOW END TO END
==================================================

Start and validate:
- daemon metrics endpoint
- Prometheus
- Grafana

Validate in browser:
1. http://localhost:9090/metrics
2. http://localhost:9091 (Prometheus targets)
3. http://localhost:3000 (Grafana dashboard)

Confirm:
- daemon is UP
- Prometheus target is UP
- Grafana datasource works
- dashboard panels render without errors
- new state-engine panels render without datasource/query errors

==================================================
PART 3 — TRIGGER STATE ENGINE ACTIVITY IF NEEDED
==================================================

If the new state metrics remain flat/zero during live run:
- use the existing replay harness / fixture-based path to drive state updates through the pipeline
- do not add new strategy logic
- do not fabricate values
- use the replay path only to validate that ingest -> state -> metrics works

Goal:
Visually prove that state metrics can move when valid events are processed.

==================================================
PART 4 — FIX WHATEVER BLOCKS VALIDATION
==================================================

If browser validation reveals issues, you are allowed to fix:
- broken dashboard queries
- wrong metric names
- missing metric exposure
- broken Prometheus scrape config
- Grafana datasource/dashboard wiring
- replay harness wiring needed for state validation
- docs/tests related to observability and state validation

You are NOT allowed to add:
- route finding
- trade logic
- simulation logic
- execution logic
- state-engine redesign beyond what is needed to expose/validate metrics

==================================================
PART 5 — TESTS / CHECKS
==================================================

Run and report:
- cargo check --workspace
- cargo test --workspace
- docker compose config
- docker compose ps

If useful, add or update:
- metrics smoke test
- state metrics exposure test
- replay-driven validation test

==================================================
PART 6 — SOURCE OF TRUTH OUTPUTS
==================================================

At the end, provide these exact outputs:
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-3_5-state-dashboard-validation
- git status --short
- git log --oneline --decorate -5

Also provide any git grep outputs needed to prove the new dashboard panels and metric names.

==================================================
REQUIRED OUTPUTS
==================================================

When finished, provide:

1. A verdict:
- fully working
- working with known limitations
- blocked (and why)

2. A checklist confirming:
- new state metrics are visible in /metrics
- Prometheus target is UP
- Grafana datasource works
- state dashboard panels render
- state metrics can be observed moving live or via replay
- no routing/sim/execution logic was added

3. Changed-files summary

4. Walkthrough artifact describing:
- what dashboard panels were added/updated
- what browser validation was performed
- whether live traffic moved the state metrics or replay was needed
- what remains deferred to the next phase

5. Validation outputs for:
- cargo check --workspace
- cargo test --workspace
- docker compose config
- docker compose ps

6. Browser artifacts/screenshots for:
- /metrics page
- Prometheus targets page
- Grafana dashboard with new state panels

Do not go beyond Phase 3.5.