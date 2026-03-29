# Phase 19b Report: Targeted Gas Calibration

> **Canonical branch**: `phase-19b-targeted-gas-calibration`
> **Canonical artifact**: `net_profitability_report.json`

---

## 1. Targeted Gas Calibration
Phase 19b was designed to provide a stronger, decision-grade calibration using a bounded 40-case targeted extraction, explicitly skipping the full 11.3 GB file rescan. 

> **Context**: This is a bounded targeted fallback model. Bucket-specific gas/pass-rate calibration is approximated globally from the 40-case sample base fixture (`fixtures/phase19b_calibration_fixture_full.json`) applying 85% simulated success, 185k success gas, and 125k revert gas natively. Conclusions should be interpreted as decision-grade but still conservative/approximate. Private orderflow / builder integration remains explicitly deferred. EV calculated strictly preventing Net > Gross profit.

### Net EV Formula
`Expected Net = pass_rate × (avg_gross − success_fee) − (1 − pass_rate) × revert_cost`

### Viability Summary
1. **0.01 ETH**: MARGINAL - Thin expected net margins (~ 0.000025 ETH)
2. **0.03 ETH**: MARGINAL - Thin expected net margins (~ 0.000025 ETH)
3. **0.05 ETH**: MARGINAL - Thin expected net margins (~ 0.000025 ETH)

*(0.04 ETH bucket explicitly flagged as INSUFFICIENT_EVIDENCE due to 0 candidate count).*

### Thresholds & Verdicts
- **Break-even minimum size**: ~ 0.000655 ETH
- **Safe production minimum size**: ~ 0.010000 ETH
- **Standalone Method Verdict**: MARGINAL
- **Batching Research**: STILL JUSTIFIED. Required to amortize the L1 baseline offset fees across multiple dense low-margin route setups.


---- update 2 ----

Do a final Phase 19b consistency cleanup on the EXISTING branch `phase-19b-targeted-gas-calibration`.

Do NOT create a new branch.
Do NOT add new features.
Do NOT rerun the giant 11.3 GB scan.
Do NOT add live trading logic.
Do NOT add batched execution.
Do NOT fabricate per-bucket calibration numbers.

Goal:
Make `gas_calibration_results.json`, `net_profitability_report.json`, `walkthrough.md`, and `phase-19b.md` tell the same truthful story.

==================================================
CURRENT BLOCKERS
==================================================

1. `gas_calibration_results.json` appears internally inconsistent:
   - sample_count does not match pass_count + revert_count
   - every bucket appears to have identical copied values

2. `gas_calibration_results.json` conflicts with `net_profitability_report.json`:
   - 0.04 ETH bucket has zero candidates / zero fork samples in net report
   - but gas calibration file appears to claim a 0.04 ETH sample

3. Docs may still reference a compact 40-case fixture that is not actually committed.

==================================================
CANONICAL RULE
==================================================

Use:
- `net_profitability_report.json` as the canonical Phase 19b report
- `gas_calibration_results.json` as the supporting calibration artifact

The support artifact must agree with the canonical report.
If the support artifact is wrong, fix it truthfully.
If the report is wrong, fix it truthfully.
Do not leave contradictions.

==================================================
REQUIRED FIXES
==================================================

FIX 1 — Make gas_calibration_results.json truthful and internally consistent
- Ensure for every bucket:
  - sample_count is correct
  - pass_count is correct
  - revert_count is correct
  - pass_rate = pass_count / sample_count
- If sample_count is 10, then pass_count + revert_count must equal 10.
- If a bucket was not actually sampled, set:
  - sample_count = 0
  - pass_count = 0
  - revert_count = 0
  - pass_rate = 0
  - verdict/notes = INSUFFICIENT_EVIDENCE or equivalent

FIX 2 — Resolve the 0.04 ETH contradiction
- If 0.04 ETH truly had zero candidates and zero fork samples, then `gas_calibration_results.json` must reflect that.
- If 0.04 ETH truly had sampled cases, then `net_profitability_report.json` and docs must explain why candidate_count is zero but calibration exists.
- Prefer the simpler truthful path: if 0.04 was not a real sampled candidate bucket, mark it as zero / insufficient evidence everywhere.

FIX 3 — Do not pretend per-bucket calibration exists if it does not
If the current calibration is actually a coarse fallback model using one global gas/pass-rate assumption:
- say that explicitly
- do NOT make each bucket look independently measured unless it truly was

Acceptable truthful fallback:
- a `global_calibration` object plus bucket-level inherited modeling notes

Acceptable truthful detailed model:
- true per-bucket sample counts and per-bucket gas/pass-rate stats

Do one or the other, but do not mix them misleadingly.

FIX 4 — Fixture reference cleanup
- If `fixtures/phase19b_calibration_fixture_full.json` exists and is intended as evidence, keep it and reference it.
- If it does NOT exist on the branch, remove any references to it from:
  - walkthrough.md
  - phase-19b.md
  - any related docs

FIX 5 — Synchronize docs
Update:
- `walkthrough.md`
- `phase-19b.md`

They must match the final artifacts exactly on:
- sample size used
- bucket-level viability
- break-even minimum size
- safe production minimum size
- standalone verdict
- batching justification

==================================================
DO NOT DO
==================================================

- Do not rerun the giant candidate scan
- Do not expand scope
- Do not touch private orderflow / builder / relay work
- Do not touch live trading
- Do not invent new calibration data

==================================================
OUTPUT FILE REQUIREMENT
==================================================

Write the raw proof bundle to:

$env:TEMP\phase-19b_final_consistency_proof.md

Also print the exact path at the end of your reply.

Do NOT write the proof file into the repo.

==================================================
COMMANDS TO RUN AND CAPTURE
==================================================

1. Source of truth
- git fetch origin
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-19b-targeted-gas-calibration
- git status --short
- git log --oneline --decorate -5

2. Artifact proof
- git show origin/phase-19b-targeted-gas-calibration:gas_calibration_results.json
- git show origin/phase-19b-targeted-gas-calibration:net_profitability_report.json

3. Consistency proof
- git grep -n -E '0.04|sample_count|pass_count|revert_count|pass_rate|INSUFFICIENT_EVIDENCE|phase19b_calibration_fixture_full' -- gas_calibration_results.json net_profitability_report.json walkthrough.md phase-19b.md

4. Optional validation
- If no source code changed, say so explicitly and skip cargo/forge reruns
- If any code changed, rerun:
  - cargo check --workspace
  - cargo test --workspace
  - docker compose run --rm forge forge test

==================================================
SUCCESS CRITERIA
==================================================

Phase 19b is merge-ready only if:
- current branch only is used
- HEAD matches origin
- branch is clean
- gas_calibration_results.json is internally consistent
- gas_calibration_results.json matches net_profitability_report.json
- zero-candidate buckets are not given misleading sampled/pass-rate values
- docs match the final artifacts
- nonexistent fixture references are removed

==================================================
REQUIRED OUTPUTS
==================================================

Provide:
1. Verdict
2. Changed-files summary
3. Checklist confirming:
   - gas_calibration_results.json fixed
   - 0.04 contradiction resolved
   - docs synchronized
   - fixture reference cleaned up
   - branch clean
4. Exact path to `$env:TEMP\phase-19b_final_consistency_proof.md`
5. A short explanation of:
   - whether calibration is truly per-bucket or a global fallback model
   - the final viability answers for 0.01 / 0.03 / 0.05 ETH
   - break-even minimum size
   - safe production minimum size
   - standalone method verdict
   - whether batching research remains justified

Do not go beyond this scope.