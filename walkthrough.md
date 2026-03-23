# Phase 16 Walkthrough: Historical Shadow Calibration (Path B)

This walkthrough documents the final, merge-ready proof for Phase 16 on the
`phase-16-historical-shadow-calibration-dashboard` branch.

## 1. Goal

The goal of Phase 16 is to provide a truthful historical replay calibration system that:

- replays confirmed historical Base Mainnet activity through the real pipeline
- measures candidate frequency and delayed profitability
- exposes those results through Prometheus metrics
- displays those metrics in Grafana for browser-based review

For this branch, the final proof uses **Path B: an honest 1-hour calibration slice**.

A full **24h+ historical replay** and **fork verification** remain deferred.

---

## 2. Final Proof Strategy

### Path B: Honest 1-Hour Calibration Slice

To ensure the branch ends in a truthful, high-signal, merge-ready state, the final proof uses a bounded 1-hour replay window rather than claiming a full 24h+ run.

This means the final proof on this branch is:

- **bounded**
- **truthful**
- **non-zero**
- **browser-visible through Grafana**

It does **not** claim that a full 24h+ final replay was completed on this branch.

---

## 3. Replay Configuration

Final replay window:

- **Start Block:** `43638000`
- **End Block:** `43639800`
- **Total Blocks:** `1801`
- **Network:** Base Mainnet
- **Recheck Delay:** `1 block`

Canonical final artifact:

- **File:** `historical_replay_calibration_final.json`

This file is the single source of truth for the final replay numbers on this branch.

---

## 4. Canonical Results Summary

The final replay results stored in `historical_replay_calibration_final.json` are:

- **Total Logs Processed:** `6602`
- **Candidates Considered:** `4,353,720`
- **Promoted Candidates:** `8,905`
- **Would-Trade Candidates:** `8,905`
- **Still Profitable After Recheck:** `8,905`
- **Invalidated:** `0`
- **Average Profit Drift:** `0 wei`

These numbers represent the final merge-ready calibration proof for Phase 16.

---

## 5. Dashboard Validation

The Grafana dashboard used for validation is:

- **Dashboard Name:** `Historical Shadow Calibration`

The following dashboard panels were checked against the canonical artifact:

- **Total Candidates**
- **Would Trade**
- **Still Profitable**
- **Invalidated**
- **Average Profit Drift**

Observed dashboard values matched the canonical artifact:

- **Total Candidates:** ~`4.35M`
- **Would Trade:** ~`8.91K`
- **Still Profitable:** ~`8.91K`
- **Invalidated:** `0`
- **Average Profit Drift:** `0`

This confirms that:

1. the replay metrics endpoint is serving meaningful data
2. Prometheus is scraping those metrics correctly
3. Grafana is displaying the same values as the canonical artifact

---

## 6. Metrics / Observability Notes

Phase 16 restores and validates the historical replay metrics flow:

- replay process emits `arb_hist_*` metrics
- Prometheus scrapes the replay endpoint
- Grafana renders those metrics in the browser

This phase demonstrates that the historical calibration pipeline is visible and inspectable in the dashboard, not just through JSON output.

---

## 7. Safety / Honesty Notes

The final Phase 16 proof on this branch is intentionally conservative and honest:

- **No live trading logic was added**
- **No real transaction broadcast was performed**
- **No hardcoded provider URL remains in source**
- **The final proof is a 1-hour calibration slice, not a 24h+ replay**
- **Fork verification is deferred and not claimed as complete in this final proof**

---

## 8. Deferred Items

The following remain deferred beyond this branch:

- **Full 24h+ historical replay as the final proof artifact**
- **Automatic fork verification / spot-check execution**
- **Live canaries**
- **Real-money execution**
- **Private relays / builder integration**
- **Adaptive EV policy automation**
- **Production fleet scaling**

---

## 9. Final Merge-Readiness Statement

Phase 16 is merge-ready **as a truthful historical calibration slice**.

What this branch proves:

- historical replay discovery works
- delayed recheck accounting works
- non-zero calibration data is produced
- metrics are exposed correctly
- dashboard values match the canonical artifact

What this branch does **not** claim:

- that a full 24h+ replay is the final proof
- that fork verification is complete
- that live execution is enabled

The canonical final artifact for this branch is:

- `historical_replay_calibration_final.json`