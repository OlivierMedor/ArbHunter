# ArbHunter Operating Rules

This document outlines the strict technical constraints and operating guidelines for ArbHunter development.

## 1. Hot-Path Explicit Rules
The "hot path" encompasses all logic from network ingestion (`arb_ingest`) through to transaction submission (`arb_execute`).

- **Zero I/O Blocking**: The hot path must NEVER block on disk I/O, network requests (other than critical payload submissions), or console logging.
- **Zero Database Reads/Writes**: Postgres is strictly for post-execution analytics, telemetry, and non-blocking historical state preservation. Interactions with `arb_storage` must occur asynchronously (e.g., via non-blocking channels).
- **Stateless Pipeline**: Each pipeline stage operates on shared or concurrently passed state data. Stages do not hold blocking locks across module boundaries.
- **Minimize Heap Allocations**: Pre-allocate buffers and reuse standard data structures wherever possible during steady-state hot-path execution.

## 2. Dependencies & External Services
- **RPC Logic**: Primary execution on QuickNode; secondary failover on Alchemy. Managed internally by `arb_providers` without stalling the execution sequence.
- **Tenderly / Simulation**: Tenderly is specifically relegated to a later-phase debugging and simulation tool. Components like `arb_sim` start as off-path validation or shadow-mode tools. They must not be an obligatory day-one hot path dependency.
- **Database**: PostgreSQL only. Handled asynchronously.

## 3. General Development Constraints
- **Secrets**: No API keys, private keys, or mnemonic seeds committed to code. Use `.env` or system environment variables.
- **Docker-First**: All infrastructure (Postgres, Metrics, Daemon) must have a reliable `docker-compose.yml` local-mode equivalent to ensure parity.
- **Dockerized Foundry**: The project uses the official `ghcr.io/foundry-rs/foundry:latest` container via `docker-compose.yml` (`forge` service). A host `forge` installation is not required. Developers must use `make forge-build` or `make forge-test` (which alias to `docker compose run --rm forge forge build` and `docker compose run --rm forge forge test`) rather than installing binaries locally. Do NOT vendor or commit binaries (`foundry_bin/`, `forge.exe`) to the repository.
- **Modularity**: Code must remain tightly scoped within its respective pipeline crate. Shared logic belongs in `arb_types` or `arb_config` if it escapes a specific module's domain.

