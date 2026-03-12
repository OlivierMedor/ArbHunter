# ArbHunter Implementation Phases

This document details the progressive implementation phases for ArbHunter and defines the explicit "Definition of Done" for each phase.

## Phase 0: Planning & Setup
**Focus**: Project scaffolding, documentation, and foundational architecture.
**Definition of Done**:
- Phase 0 documentation (`ROADMAP.md`, `ARCHITECTURE.md`, `OPERATING_RULES.md`, `PHASES.md`) reviewed and committed.

*(Note: Workspace and folder scaffolding belong to the next setup phase. Phase 0 is strictly a docs-only completion phase.)*

## Phase 1: Core Primitives & Infrastructure
**Focus**: Types, configuration, telemetry, async storage, and basic node connectivity.
**Definition of Done**:
- Primary types defined in `arb_types`.
- Environment loads properly via `arb_config`.
- Postgres schema applied via `sql/` migrations and async inserts operational in `arb_storage`.
- RPC connectivity, latency tracking, and failover implemented in `arb_providers`.
- All Phase 1 logic unit tested successfully.

## Phase 2: Ingestion & State Sync
**Focus**: Reading from Base data sources and maintaining precise in-memory representations.
**Definition of Done**:
- `arb_ingest` successfully streams Base Flashblocks.
- `arb_state` accurately processes stream data into usable pool representations without latency spikes.
- Integration tests confirm memory state matches on-chain state closely.

## Phase 3: Routing & Filtering
**Focus**: Identifying actionable opportunities and drawing paths.
**Definition of Done**:
- `arb_filter` eliminates non-profitable states rapidly.
- `arb_route` outputs optimal transaction route data.
- E2E dry-run benchmarks confirm hot-path execution speed meets targets.

## Phase 4: Execution Pipeline & Simulation
**Focus**: Packaging transactions, simulation, and dry-run execution.
**Definition of Done**:
- `arb_sim` integrates basic Tenderly debugging features.
- `arb_execute` crafts and signs valid Base transactions.
- Smart contracts (`contracts/src/`) handle the generated payload payloads accurately.
- Pipeline from start to finish completes without live-fire (purely localized or dry-run validation).

## Phase 5: Live Trading & Tuning
**Focus**: Deployment to production, live fire, and MEV tuning.
**Definition of Done**:
- Live Base execution begins.
- Docker-compose setups migrated/synced with remote infrastructure.
- Profitability metrics reliably logged to Postgres.
