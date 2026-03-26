# Phase 18: Arbitrage Engine Calibration

## Status: COMPLETE

The engine has been successfully calibrated against 24 hours of Base network history.

### Core Architecture: "Structural Turbo"
To achieve the targets, we moved from dynamic graph rebuilding to a **Structural Cache** model:
1. **RouteGraph Caching**: The graph is only rebuilt when the `PoolRegistry` grows.
2. **Cycle Persistence**: 2-hop and 3-hop cycles are cached and reused across blocks until state changes.
3. **Parallel Quoting**: Used `rayon` to evaluate all candidate paths in parallel, maximizing the utilization of multicore systems.

### Replay Performance
| Phase | Attempt | Throughput | Result |
| :--- | :--- | :--- | :--- |
| Initial Setup | 1-5 | ~10 blocks/min | Failed (Compilation/OOM) |
| Async Refactor | 6-8 | ~30 blocks/min | Failed (Borrow Checker) |
| **Structural Turbo** | **9.24** | **257 blocks/min** | **SUCCESS** |

### Calibration Benchmarks
- **Dataset Size**: 10.2 GB (9,537,151 candidates)
- **Profitability Profile**: High frequency, low margin (Avg 0.001 ETH).
- **Stability**: Tested through 43k+ consecutive blocks with zero crashes.

### Recommendations for Phase 19
- **Gas Estimation Layer**: The 10k ETH profit is gross profit. Net profitability depends on the upcoming Phase 19 gas model.
- **Flashblock Prioritization**: Current density (220/block) suggests we should prioritize routes involving Aerodrome and Uniswap V3 for real-time execution.