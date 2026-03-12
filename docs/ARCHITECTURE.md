# ArbHunter Architecture

ArbHunter is designed as a pipeline-oriented Rust application combined with highly optimized Solidity smart contracts. It targets the Base network exclusively.

## General Architecture Paradigm
The architecture separates concerns into a sequential pipeline, ensuring zero IO-blocking dependencies on the "hot path". 

- **Rust Workspace (`crates/`)**: Houses the modular pipeline components.
- **Smart Contracts (`contracts/`)**: Contains the Foundry-managed solidity execution interfaces.
- **Orchestrator (`bin/arb_daemon`)**: Wires the pipeline components and manages lifecycle.

## Pipeline Layout

The execution sequence follows a strict pipeline layout to minimize latency:

1. **`arb_ingest`**: Subscribes to Base Flashblocks and pending transaction data. Pushes events downstream.
2. **`arb_state`**: Applies ingested events to an in-memory representation of network state (reserves, pools).
3. **`arb_filter`**: Discards non-actionable state changes rapidly.
4. **`arb_route`**: Calculates optimal cyclic paths and sizing for arbitrage.
5. **`arb_sim`**: (Later phase) Evaluates execution success locally or against Tenderly before committing gas.
6. **`arb_execute`**: Signs and broadcasts the final transaction payload to RPC nodes.

## Supporting Crates
- `arb_types`: System-wide primitives.
- `arb_config`: Environment and startup configuration.
- `arb_metrics`: Performance tracking and asynchronous logging.
- `arb_providers`: RPC client management, failover logic, and latency profiling.
- `arb_storage`: Purely asynchronous PostgreSQL handlers for historical and analytical data.

## Build Order Sequence
1. Foundation: `arb_types`, `arb_config`
2. Core Infrastructure: `arb_storage`, `arb_metrics`
3. Network Layer: `arb_providers`, `arb_ingest`
4. State & Logic Layer: `arb_state`, `arb_filter`
5. Action Layer: `arb_route`, `arb_sim`, `arb_execute`
6. Final Assembly: `bin/arb_daemon`, `contracts/`
