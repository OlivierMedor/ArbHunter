# ArbHunter Implementation Phases

This document details the progressive implementation phases for ArbHunter and defines the explicit "Definition of Done" for each phase.

## Phase 0: Planning & Setup
**Focus**: Documentation and foundational architecture planning.
**Definition of Done**:
- Phase 0 documentation (`ROADMAP.md`, `ARCHITECTURE.md`, `OPERATING_RULES.md`, `PHASES.md`) reviewed and committed.

## Phase 1: Core Primitives & Infrastructure
**Focus**: Project scaffolding, types, configuration, telemetry, async storage, and basic node connectivity.
**Definition of Done**:
- Rust workspace and pipeline crates initialized with empty lib stubs.
- Foundry project initialized in `contracts/`.
- Local Foundry environment configured.
- Primary types defined in `arb_types`.
- Environment loads properly via `arb_config`.
- Postgres schema applied via `sql/` migrations and async inserts operational in `arb_storage`.
- RPC connectivity, latency tracking, and failover implemented in `arb_providers`.
- All Phase 1 logic unit tested successfully.

## Phase 2: Providers & Ingestion Foundation
**Focus**: Robust WebSocket connectivity and normalized event intake.
**Definition of Done**:
- `arb_providers` implements `ProviderManager` with failover support (QuickNode/Alchemy).
- `arb_ingest` successfully parses and normalizes Base Flashblocks and pending logs.
- `arb_metrics` tracks frame intake and provider connectivity status.

## Phase 2.5: Observability Dashboard
**Focus**: Real-time visibility into the daemon's internal state and pipeline health.
**Definition of Done**:
- Prometheus scrape endpoint operational in `arb_daemon`.
- Grafana dashboard provisioned with metrics for provider health, ingestion volume, and uptime.
- Docker Compose setup verified for local monitoring stack.

## Phase 3: State Engine Foundation
**Focus**: Canonical in-memory representation and freshness management.
**Definition of Done**:
- `arb_state` crate implements `PoolStore` and `StateEngine` with `RwLock` concurrency.
- **Freshness tracking**: Monotonic `EventStamp` ordering rejects stale updates; wall-clock sweeping marks pools stale after 30s.
- **Ingest Bridge**: Daemon wires `IngestEvent` stream to `StateEngine`.
- **Honesty Note**: Phase 3 uses synthetic block-level updates derived from Flashblocks; real DEX Sync/Swap log decoding is deferred to Phase 4.
- **Minimal Scope**: No routing, filtering, simulation, or execution logic added.

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
