Implement Phase 1 only.

Goal:
Create the approved project scaffolding for ArbHunter.

Also:
Fix the docs boundary issue by updating docs/PHASES.md so that:
- Phase 0 = docs/planning only
- Phase 1 = scaffolding creation

Create:
- Cargo.toml workspace root
- crates/
  - arb_types
  - arb_config
  - arb_metrics
  - arb_providers
  - arb_ingest
  - arb_state
  - arb_filter
  - arb_route
  - arb_sim
  - arb_execute
  - arb_storage
- bin/arb_daemon
- contracts/ Foundry project skeleton
- infra/
- sql/
- fixtures/
- .env.example
- Makefile
- docker-compose.yml placeholder
- update README if needed

Requirements:
- no business logic yet
- no provider logic yet
- no strategy logic yet
- no live execution logic
- no secrets
- compile or stub cleanly
- placeholder tests where appropriate
- keep Postgres off the hot path
- produce a walkthrough artifact when done

Output:
- the scaffolding files
- the docs/PHASES.md correction
- a walkthrough artifact summarizing exactly what was created