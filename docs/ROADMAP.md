# ArbHunter Roadmap

This document outlines the high-level roadmap and milestone sequence for the ArbHunter high-frequency atomic arbitrage system.

## Project Vision
To build a highly performant, Base-first atomic arbitrage execution engine using a pipeline-oriented Rust architecture and Foundry-based Solidity contracts.

## Milestone Sequence

### Milestone 1: Foundation Setup
- Project scaffolding (`infra/`, `sql/`, `fixtures/`, `crates/`, `docs/`, `contracts/`).
- Core generic primitives (`arb_types`).
- Environment configuration and parsing (`arb_config`).
- Local Foundry environment setup and configuration.

### Milestone 2: Data & Infrastructure
- Non-blocking telemetry and performance tracking (`arb_metrics`).
- Async storage endpoints and schema for Postgres (`arb_storage`).
- RPC failover and latency tracking (`arb_providers`).
- Base Flashblocks and pending state data ingestion (`arb_ingest`).

### Milestone 3: State & Filtering
- In-memory synchronization and structured state management (`arb_state`).
- Fast heuristic opportunity screening and preliminary filtering (`arb_filter`).

### Milestone 4: Logic & Routing
- Pathfinding and execution planning (`arb_route`).
- Transaction payload formulation.
- Tenderly integration (as a later-phase debugging/simulation tool placeholder) (`arb_sim`).

### Milestone 5: Execution Pipeline
- Crafting, signing, and submitting execution transactions (`arb_execute`).
- Complete pipeline integration into the main daemon (`bin/arb_daemon`).
- Solidity smart contract logic for atomic execution (`contracts/`).
- E2E dry-run testing.

## Intentionally Delayed Features
The following are explicitly excluded from the initial development phases to maintain focus on core mechanics:
1. **Live Execution**: Strictly running dry-runs or simulations initially.
2. **Day-One Tenderly Reliance**: Tenderly is reserved for later-phase deep simulation and debugging, not a critical path dependency.
3. **Cloud Deployment**: Initial focus is entirely on local `docker-compose` parity.
4. **Cross-Chain Routing**: The system is strictly Base-first; cross-chain is out of scope.
5. **Advanced MEV Hiding**: Focus is on pure atomic arb execution mechanics first.

## Definition of Done (General)
A milestone is considered complete when:
- Unit and integration tests pass.
- Components successfully integrate without stalling the execution hot path.
- Code adheres to strict constraints (e.g., zero Postgres interaction in the hot path).
- Updated artifacts and documentation reflect the current state.
