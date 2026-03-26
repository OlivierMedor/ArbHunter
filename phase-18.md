# Phase 18: Arbitrage Engine Calibration

## Status: COMPLETE — `phase-18-final-calibration`

> **Canonical artifact**: `execution_calibration_report.json`
> All numbers in this document come directly from that file.

---

## Replay Window

| Field | Value |
| :--- | :--- |
| Start block | 43,680,550 |
| End block | 43,723,750 |
| Total blocks in window | 43,201 |
| Blocks with candidates | 40,717 (94.3% of window) |

---

## Core Architecture: "Structural Turbo"

To hit calibration goals we moved from dynamic graph rebuilding to a **Structural Cache** model:

1. **RouteGraph Caching** — the graph is only rebuilt when the `PoolRegistry` grows.
2. **Cycle Persistence** — 2-hop and 3-hop cycles are cached and reused across blocks until state changes.
3. **Parallel Quoting** — `rayon` is used to evaluate all candidate paths in parallel.

---

## Canonical Results (`execution_calibration_report.json`)

### Candidate Volume

| Metric | Value |
| :--- | :--- |
| Total unique candidates | 10,708,460 |
| Block density | 247.88 candidates / block |
| Blocks with at least one candidate | 40,717 (94.3%) |

### Route Family Breakdown

| Family | Count |
| :--- | :--- |
| Multi-hop (3-leg) | 10,016,271 (93.5%) |
| Direct (2-leg) | 692,189 (6.5%) |

### Size-Bucket Breakdown

| Bucket | Count | % of total |
| :--- | :--- | :--- |
| 0.01 ETH input | 8,102,605 | 75.7% |
| 0.03 ETH input | 1,710,091 | 16.0% |
| 0.05 ETH input | 895,764 | 8.4% |

**Explicit answers to the size questions:**

- **Were there any 0.03 ETH opportunities?** Yes — 1,710,091 candidates identified at this size.
- **Were there any 0.04 ETH opportunities?** Not evaluated as a discrete bucket; 0.03 and 0.05 were the adjacent thresholds tested.
- **Were there any 0.05 ETH+ opportunities?** Yes — 895,764 candidates identified at this size.

### Profitability (Gross, Pre-Gas)

| Metric | Wei | ETH |
| :--- | :--- | :--- |
| Total simulated gross profit | 11,512,619,578,721,224,175,061 | 11,512.62 ETH |
| Average profit / trade | 1,075,095,726,063,432 | ~0.00108 ETH |
| Peak profit / trade | 5,000,072,552,514,093 | ~0.00500 ETH |

> **Note**: All figures are **gross simulated profit** before gas costs. Phase 19 introduces the gas fee layer.

---

## Batchability Analysis

**Were multiple small opportunities common in the same block?**

Yes. With a block density of 247.88 candidates/block and 94.3% of blocks containing at least one candidate, co-occurrence is the norm rather than the exception. The 10M+ multi-hop count relative to 692k direct-hop indicates the engine consistently surfaces compound paths in the same block as simpler ones.

**Does the data justify future batched execution research?**

Yes, analytically. The density data supports a hypothesis that atomic batch bundles (2–3 routes per block) could reduce per-trade gas overhead and increase net profitability. This is not yet implemented and is recommended as the primary research direction for Phase 20.

---

## Replay Performance

| Phase | Attempt | Throughput | Result |
| :--- | :--- | :--- | :--- |
| Initial setup | 1–5 | ~10 blocks/min | Failed (compilation/OOM) |
| Async refactor | 6–8 | ~30 blocks/min | Failed (borrow checker) |
| **Structural Turbo** | **9.24** | **257 blocks/min** | **SUCCESS** |

---

## Recommendations for Phase 19

- **Gas Layer**: All profit is gross. Phase 19 must subtract L2 gas at current Base gwei prices to get net profitability.
- **Flashblock Prioritization**: Current density (247.88/block) suggests prioritizing routes on Aerodrome and Uniswap V3.
- **Batch Research (Phase 20)**: Multi-opportunity co-occurrence per block justifies a batched execution study.