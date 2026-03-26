# Phase 18 Walkthrough: Arbitrage Engine Calibration

> **Canonical branch**: `phase-18-final-calibration`
> **Canonical artifact**: `execution_calibration_report.json`
> All numbers in this document come directly from `execution_calibration_report.json`.

---

## 1. Replay Window

- **Blocks**: 43,680,550 → 43,723,750
- **Total blocks in window**: 43,201
- **Blocks with at least one candidate**: 40,717 (94.3%)

The "Structural Turbo" architecture was used:
- `RouteGraph` cached and rebuilt only when the pool registry grows.
- 2-hop and 3-hop cycles persisted between blocks.
- `rayon` parallel evaluation of all candidate paths.

---

## 2. Canonical Results

### Candidate Volume

| Metric | Value |
| :--- | :--- |
| Total unique candidates | 10,708,460 |
| Block density | 247.88 candidates / block |
| Blocks with candidates | 40,717 (94.3%) |

### Size-Bucket Breakdown (Input Size, Not Profit)

| Bucket | Count | % |
| :--- | :--- | :--- |
| 0.01 ETH input | 8,102,605 | 75.7% |
| 0.03 ETH input | 1,710,091 | 16.0% |
| 0.05 ETH input | 895,764 | 8.4% |

### Route Family

| Family | Count |
| :--- | :--- |
| Multi-hop (3-leg) | 10,016,271 (93.5%) |
| Direct (2-leg) | 692,189 (6.5%) |

### Profitability (Gross, Pre-Gas)

| Metric | ETH |
| :--- | :--- |
| Total simulated gross profit | 11,512.62 ETH |
| Average profit per trade | ~0.00108 ETH |
| Peak profit per trade | ~0.00500 ETH |

---

## 3. Size Questions

- **Were there 0.03 ETH opportunities?** Yes — 1,710,091 candidates with 0.03 ETH input size.
- **Were there 0.04 ETH opportunities?** Not a discrete bucket tested; thresholds were 0.01, 0.03, and 0.05.
- **Were there 0.05 ETH+ opportunities?** Yes — 895,764 candidates with 0.05 ETH input size.

---

## 4. Batchability Findings

With 247.88 candidates/block and 94.3% block coverage, multiple opportunities co-occurring in the same block is the norm. The 10M+ multi-hop count vs. 692k direct-hop indicates compound paths frequently surface alongside simpler ones per block.

**Conclusion**: Batched execution research (Phase 20) is analytically justified by this density data. Not implemented here.

---

## 5. What Is NOT Here

- Gas cost subtraction (Phase 19 scope).
- Live execution logic (not added).
- The 11.3 GB `historical_replay_full_day_candidates.jsonl` is local-only (exceeds GitHub 100 MB limit).

---

## Source of Truth

`execution_calibration_report.json`