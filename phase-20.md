Implement the next phase using a GitHub-first, anti-freeze workflow.

Branch name:
phase-20-package-feasibility-and-batch-simulation

Before doing any code work:
1. Ensure work is being done on branch `phase-20-package-feasibility-and-batch-simulation`
   - if the branch does not exist, create it from current main
   - if it exists, switch to it
2. Do NOT work directly on main
3. Do NOT create redundant side branches

==================================================
GITHUB-FIRST / ANTI-FREEZE RULES
==================================================

This project must now be run in a GitHub-first way.

That means:
- Large raw files stay LOCAL only
- Small canonical artifacts must be committed to GitHub
- Reviewable outputs must live in the repo so they can be inspected from GitHub later
- Do NOT assume huge local files can be uploaded elsewhere for analysis

Required workflow rules:

1. Keep giant raw files local-only
   Examples:
   - massive JSONL candidate exports
   - giant checkpoint files
   - giant debug logs
   - temporary extraction outputs

2. Commit only compact reviewable artifacts
   Examples:
   - one canonical JSON artifact for the phase
   - small CSV/JSON summary sidecars
   - walkthrough.md
   - phase-20.md
   - ARTIFACT_INDEX.md

3. Do NOT commit huge generated data files
   If a file is large enough to slow the repo, keep it local and summarize it into a compact committed artifact.

4. Do NOT use PowerShell line-by-line processing on huge files.
5. Do NOT use giant editor writes against very large files.
6. Do NOT brute-force rescan giant raw files unless absolutely necessary.
7. Any large-file processing must be:
   - streaming only
   - counters/summaries only
   - checkpointed
   - stopped early if throughput is unreasonable

8. All final reviewable conclusions must be reproducible from:
   - committed canonical artifacts
   - committed walkthrough/docs
   - the exact commit SHA

9. Final docs must contain no local-only `file:///` links.

10. Final response must always provide:
   - exact branch name
   - exact HEAD commit SHA
   - canonical artifact filenames
   - raw proof file path (local temp path is okay)
   - clear statement of which files are committed vs local-only

==================================================
PHASE 20 GOAL
==================================================

Determine whether multiple compatible opportunities can be grouped into one profitable atomic package, using analytical package construction and simulation only.

This phase is analytical only:
- No live trading
- No real broadcasts
- No private orderflow / builder / relay integration
- No actual on-chain batched execution implementation
- No multi-asset live execution logic

Private orderflow / builder / relay integration remains explicitly deferred.

==================================================
PHASE 20 OBJECTIVE
==================================================

By the end of this phase, the system should be able to answer:

1. How often do multiple compatible opportunities occur in the same block/window?
2. How often do they share the same root asset?
3. How often do they avoid destructive pool overlap?
4. If grouped into one package, what is the estimated package-level gross profit?
5. After shared gas / fee overhead, what is the estimated package-level net uplift versus standalone execution?
6. Does batching make the strategy materially more attractive than the standalone method?

==================================================
INPUTS TO REUSE
==================================================

Prefer reusing existing committed canonical artifacts:
- execution_calibration_report.json
- net_profitability_report.json
- gas_calibration_results.json
- historical_replay_full_day_final.json
- fixtures/fork_verification_results.json
- any small committed calibration fixtures already present

Do NOT overwrite earlier phase artifacts.

If extra extraction is needed from local giant files:
- produce a compact local summary
- then commit only the compact summary, not the huge source file

==================================================
PACKAGEABILITY ANALYSIS
==================================================

Build a packageability analyzer that identifies clusters of opportunities by:
- same block or small block window
- same root asset
- low or manageable pool conflict

Analytical outputs should include:
- candidate density by block/window
- same-root-asset clustering frequency
- pool-overlap conflict frequency
- top windows with highest package potential
- package size distribution (2-op / 3-op / 4-op)
- estimated package gross profit
- estimated package net profit
- package uplift vs standalone

This phase must remain analytical only.
Do NOT implement actual batched execution.

==================================================
CANONICAL ARTIFACTS
==================================================

Create one canonical Phase 20 artifact:

- package_batchability_report.json

Optionally create compact committed sidecars if useful, such as:
- block_bucket_density.csv
- route_family_net_summary.csv
- top_package_candidates.csv

Keep all sidecars compact and reviewable.

==================================================
ARTIFACT POLICY
==================================================

Update or create:
- ARTIFACT_INDEX.md

It must list:
- canonical branch by phase
- canonical artifact by phase
- supporting artifacts by phase
- which important raw files are local-only
- the key conclusion of each phase

For Phase 20 specifically, ARTIFACT_INDEX.md must identify:
- `package_batchability_report.json` as the canonical artifact

==================================================
DASHBOARD
==================================================

Add or update a browser-visible dashboard, for example:
- Package Feasibility & Batch Simulation

Required panels:
- candidate density by block/window
- root-asset clustering frequency
- pool conflict rate
- package size distribution
- package uplift vs standalone
- estimated package net by package size
- plain-English recommendation panel:
  - batching justified or not

The dashboard must clearly distinguish:
- standalone stats
- package stats
- estimated uplift

==================================================
DOCUMENTATION
==================================================

Update:
- walkthrough.md
- phase-20.md
- ARTIFACT_INDEX.md

Docs must clearly state:
- this phase is analytical only
- no live batched execution was implemented
- no private orderflow / builder / relay integration was added
- what the packageability analysis found
- whether future batched execution is promising enough to build

No local-only `file:///` links are allowed.

==================================================
REQUIRED PLAIN-ENGLISH CONCLUSIONS
==================================================

The final docs/artifact must explicitly answer:

1. Are multiple compatible opportunities common in the same block/window?
2. Are they common enough to make package construction practical?
3. Does packaging materially improve net profitability vs standalone?
4. Is future batched execution research still justified?
5. Is batched execution now the more promising path than the standalone method?

==================================================
VALIDATION REQUIRED
==================================================

Run and report:

1. Source of truth:
- git fetch origin
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-20-package-feasibility-and-batch-simulation
- git status --short
- git log --oneline --decorate -5

2. Grep proofs:
- git grep -n -E 'package|batch|uplift|cluster|compatibility|overlap|same_root' -- walkthrough.md phase-20.md package_batchability_report.json docs/ crates/ bin/
- git grep -n -E 'private orderflow|builder|relay|live trading|broadcast|file:///' -- walkthrough.md phase-20.md docs/

3. Validation:
- cargo check --workspace
- cargo test --workspace
- docker compose config
- docker compose up -d prometheus grafana
- docker compose run --rm forge forge test

4. Package-analysis proof:
- exact commands used
- exact input artifacts used
- exact local-only raw inputs referenced
- path to package_batchability_report.json
- excerpt of the final report

5. Browser validation:
- dashboard name
- panels checked
- values shown
- explicit answers to the packageability questions above

==================================================
OUTPUT FILE REQUIREMENT
==================================================

Write the raw proof bundle to:

$env:TEMP\phase-20_packageability_proof.md

Also print the exact path at the end of your reply.

Do NOT write the proof file into the repo.

==================================================
SUCCESS CRITERIA
==================================================

Phase 20 is merge-ready only if:
- current branch only is used
- branch is clean
- HEAD matches origin
- package_batchability_report.json is truthful
- walkthrough.md and phase-20.md match the artifact exactly
- ARTIFACT_INDEX.md is updated
- no local-only links remain
- no giant-file freeze-prone method was used unnecessarily
- no live trading logic added
- private orderflow / builder / relay integration remains deferred

==================================================
REQUIRED OUTPUTS
==================================================

Provide:
1. Verdict
2. Canonical artifact filename
3. Changed-files summary
4. Checklist confirming:
   - packageability analysis added
   - package uplift analysis added
   - ARTIFACT_INDEX.md updated
   - dashboard validated in browser
   - no live batched execution added
5. Exact raw outputs for all commands above
6. Exact HEAD commit SHA
7. A short summary explaining:
   - whether compatible package opportunities are common
   - whether packaging materially improves net profitability
   - whether future batched execution research is justified
   - whether batching now looks more promising than the standalone method
   - which files are committed vs local-only

Do not go beyond this scope.
## Phase 20 Results: Package Feasibility & Batch Simulation

### Execution Summary

- **Branch**: `phase-20-package-feasibility-and-batch-simulation`
- **Method**: Stratified sampling (4x250k lines) across the full 24h candidate dataset
- **Conflict Rule**: Strict — ANY pool overlap = destructive conflict
- **Window**: Same-block only (window = 0)

### Key Metrics

| Metric | Value |
|-------|-------|
| Total lines sampled | 1,000,000 |
| Block clusters with >1 opportunity | 3,805 |
| Clusters rejected (pool overlap) | 3,805 (100%) |
| Same-direction overlaps | 159,731 |
| Opposite-direction overlaps | 0 |
| Viable packages | 0 |
| Total uplift | 0.0 ETH |

### Package Size Distribution

| Cluster Size | Count |
|--------------|-------|
| 28 | 1 |
| 122 | 1 |
| 150 | 1 |
| 151 | 1 |
| 165 | 1 |
| 248 | 1 |
| 262 | 1 |
| 263 | 3,798 |

### Analytical Verdict

**Packaging is NOT feasible under strict conflict rules.**

Every single block cluster was rejected due to pool overlap. This means
that same-block arbitrage opportunities are highly correlated — they
share the same liquidity pools. All 159,731 overlaps were same-direction,
meaning multiple opportunities were trying to exploit the same price
dislocation via the same pool.

### Implications

1. **Same-block batching is not viable** under conservative conflict rules
2. **Opportunities are highly correlated** — they compete for the same liquidity
3. **Relaxed rules could unlock packaging** — allowing same-direction overlap
   would permit all 3,805 clusters, but this requires careful slippage analysis
4. **Multi-block windowing** may create cross-block packages with less overlap

### Canonical Artifacts

- `package_batchability_report.json` — Full analytical results
- `scripts/analyzer.py` — Stratified sampling analyzer
- `phase-20.md` — This report
