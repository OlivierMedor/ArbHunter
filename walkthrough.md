# ArbHunger Walkthrough

## Phase 20: Package Feasibility & Batch Simulation

### Objective
Determine whether multiple compatible arbitrage opportunities
within the same block can be grouped into atomic packages to
improve net profitability.

### Methodology
- **Stratified Sampling**: 4 windows of 250k lines each across the full 24h candidate dataset
- **Conflict Rule**: Strict — any pool address overlap is a destructive conflict
- **Window**: Same-block only (window = 0)

### Results

| Metric | Value |
|-------|-------|
| Total lines sampled | 1,000,000 |
| Block clusters with >1 opportunity | 3,805 |
| Clusters rejected (pool overlap) | 3,805 (100%) |
| Same-direction overlaps | 159,731 |
| Opposite-direction overlaps | 0 |
| Viable packages | 0 |
| Total uplift | 0.0 ETH |

### Verdict
Packaging is **not feasible** under strict conflict rules at window=0.
All 3,805 block clusters were rejected due to pool overlap.
All overlaps were same-direction, indicating highly correlated opportunities.

### Canonical Artifacts
- `package_batchability_report.json` — Full analytical results
- `scripts/analyzer.py` — Stratified sampling analyzer
- `phase-20.md` — Phase 20 report

### Next Steps
1. Consider relaxing the conflict rule to allow same-direction overlap
2. Explore multi-block windowing for cross-block packages
3. Analyze slippage impact of same-pool batching
