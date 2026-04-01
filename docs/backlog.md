# ArbHunter Future Backlog

Items deferred from Phase 23. Do not implement these until the corresponding gate conditions are met.

## 1. Adaptive Canary Ramping
**Gate:** After ≥ 100 live canary attempts with pass_rate ≥ 0.80 and no halt events.
- Gradually increase `max_trade_size_wei` from 0.03 ETH toward 0.05 ETH
- Gradually increase `max_daily_volume_eth`
- Implement auto-ramp controller in `arb_canary`

## 2. Dynamic Route + Size Learning from Live Results
**Gate:** After Phase 3+ live canary data with real realized PnL records.
- Compare live realized/predicted ratio per route family + bucket
- Feed back to `CanaryPolicy` thresholds dynamically
- May require a separate analytics pipeline or shadow journal upgrade

## 3. Real Flash-Loan Integration Decision
**Gate:** After standalone canary proves profitable without flash loans.
- Evaluate BalancerV2 vs Uniswap V2 flash loan providers
- Assess execution-cost overhead vs capital efficiency gain
- Implement in a new `arb_flashloan` crate only if the math justifies it

## 4. Re-evaluate Direct Routes (0.01 ETH)
**Gate:** When the committed fork-verified fixture for `direct` route family reaches n ≥ 30 cases.
- Currently blocked: n=15, MARGINAL, LOW_CONFIDENCE
- Run a new targeted calibration batch focused on direct routes
- Update `CanaryPolicy.route_family_blocklist` only if evidence supports

## 5. Re-evaluate 0.05 ETH Size Expansion
**Gate:** After 0.03 ETH canary has operated cleanly for ≥ 30 live attempts.
- Expand `max_trade_size_wei` to 50_000_000_000_000_000 (0.05 ETH)
- Run a new stratified calibration against the 0.05 ETH bucket
- Update `canary_policy.json` and config defaults

## 6. More Venues / Additional Chains
**Gate:** Only after Base chain strategy is stable and profitable in live canary mode.
- Do NOT expand venue coverage or add new chains during canary phase
- Candidates: additional Base DEXes, Optimism, Arbitrum
- Requires new `arb_ingest` + `arb_state` adapters per venue

---
*Created: Phase 23 — sim-safety-loss-cap*
*Do not delete or significantly modify this file without a phase review.*
