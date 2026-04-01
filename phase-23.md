You are working in the ArbHunter repo.

Start from branch:
phase-22-high-value-evidence-and-canary-gate

Create a new working branch for this phase, for example:
phase-23-sim-safety-loss-cap

Phase 23 objective:
This phase is STILL simulation / shadow / evidence-hardening only by default.
Do NOT enable real trading.
Do NOT enable real broadcasts.
Do NOT change the repo into a live-trading posture unless there is a separate, explicit live-gate sign-off.
Any live-oriented logic you add must be inert by default.

High-level goal:
Turn the current Phase 22 policy/artifact conclusions into enforceable code paths, more accurate simulation economics, better telemetry, and cleaner docs, while keeping the project in sim/shadow mode.

Important product decisions already made:
1. Phase 23 remains sim/shadow-first, not live trading.
2. Replace the fixed daily canary volume cap concept with a cumulative realized loss cap concept for future live use.
3. Initial future live canary loss cap should be 0.05 ETH cumulative realized loss.
4. Keep route-family posture at start:
   - multi allowed
   - direct excluded until re-evaluated
5. Keep max trade size at 0.03 ETH for the initial posture.
6. Keep max concurrent trades = 1.
7. Keep stop after 3 consecutive reverts.
8. Keep a review checkpoint after a meaningful sample count (target: 30 attempts / cases).
9. Future loss accounting should focus on actual realized net loss / paid execution costs, especially gas + fees. Flash-loan principal itself is not “loss” when the tx reverts; count true non-recoverable costs.

What I want you to do in this phase:

A. Audit current implementation vs current Phase 22 policy
- Inspect the repo and identify what is already enforced in code versus what is still only documented in JSON/markdown.
- Identify any mismatches between:
  - canary_policy.json
  - phase-22.md
  - walkthrough.md
  - any runtime policy/config code
- Especially verify whether current runtime code truly enforces:
  - route-family allow/block lists
  - max trade size
  - revert streak stop
  - canary gating behavior
  - no-live / no-broadcast defaults

B. Implement runtime policy enforcement plumbing
- Add or improve code so the canary/safety policy is enforceable in runtime code, not just docs.
- Keep it simulation/shadow-safe by default.
- Add support for a cumulative realized loss cap parameter for future live use:
  - initial value: 0.05 ETH
- Preserve these policy constraints:
  - multi only
  - direct blocked
  - max trade size 0.03 ETH
  - max concurrent trades 1
  - stop after 3 consecutive reverts
  - review threshold around 30 attempts
- If the system is still shadow-only, wire the policy/telemetry so that later live activation is straightforward, but do not activate live broadcasts.

C. Correct the economics / fee model
- Improve profit/loss modeling to explicitly account for real execution costs as much as practical in this phase.
- Include:
  - Base L2 execution fee
  - Base L1 data/security fee if/where it should be modeled
  - priority fee handling if relevant in current code path
  - flash-loan fee handling
  - revert-burn / failed-attempt cost accounting
- Make sure predicted-vs-realized accounting is clearer and more accurate.
- If some pieces cannot be perfectly measured in current sim mode, document the approximation and where it enters the code.

D. Tenderly integration for simulation
- Add Tenderly as an optional simulation backend / pre-broadcast validation option.
- This is for simulation safety checks, not for enabling live trading in this phase.
- Prefer the integration shape to support:
  - final pre-send safety simulation
  - optional “their tx then our tx” / same-block bundle simulation flow
- Keep existing provider stack for ordinary reads unless there is a strong reason to change it.
- If credentials are required and not available, add the config scaffolding, interface integration, mocks/fallbacks, and documentation for how to wire it later.

E. Evidence hardening for the intended future live path
- Focus evidence on the chosen posture:
  - 0.03 ETH
  - multi routes
- Improve replay / calibration / reporting so we can better compare predicted vs realized outcomes for that path.
- Validate or challenge the current pass-rate assumptions on that chosen bucket.
- If you can only partially automate this, at least create the scripts/reporting hooks and document exactly how to run them.

F. Documentation reconciliation
- Make the docs consistent with the actual policy posture.
- Remove stale or conflicting wording that implies:
  - 0.01 ETH direct is part of the initial canary
  - Phase 23 is already live
  - real broadcasts are enabled by default
- Update or create the relevant phase document for Phase 23.

G. Telemetry / scoreboard
- Add or improve telemetry/reporting for:
  - predicted profit
  - realized profit
  - realized/predicted ratio
  - revert count
  - revert streak
  - cumulative realized canary P&L / loss
  - route family
  - size bucket
- Make the telemetry useful for later dynamic route/size learning, even if we are not implementing the full adaptive system in this phase.

H. Preserve future backlog in-repo
- Create or update a backlog/planning note in the repo so later tasks do not get lost.
- Include these later items explicitly:
  1. adaptive canary ramping
  2. dynamic route + size learning from live results
  3. real flash-loan integration decision
  4. re-evaluate direct routes
  5. re-evaluate 0.05 ETH size
  6. only after Base is stable: more venues/chains

Constraints:
- Do not enable real trading.
- Do not enable real broadcasts.
- Do not broaden scope into multi-chain expansion.
- Do not do large unrelated refactors.
- Keep changes as minimal and clean as possible while still making the policy real and useful.
- Prefer code + tests + docs over vague notes.

Deliverables:
1. Code changes implementing the Phase 23 simulation/safety work.
2. Tests for the new policy enforcement and accounting behavior.
3. Updated docs/policy artifacts with no internal contradictions.
4. A concise implementation report that includes:
   - what was already present
   - what you changed
   - what remains incomplete
   - exact files changed
   - exact commands/tests run
   - any manual steps needed for Tenderly credentials/config
5. A short “recommended next step after Phase 23” section.

Acceptance criteria:
- Default posture remains simulation/shadow only.
- No default code path performs real broadcasts.
- Future live-canary policy supports cumulative realized loss cap = 0.05 ETH.
- multi-only / direct-blocked posture is enforceable in runtime code or clearly scaffolded where execution is intentionally disabled.
- 0.03 ETH max trade size is enforced or clearly wired for enforcement.
- 3-consecutive-reverts stop is enforced or clearly wired for enforcement.
- Fee model is more realistic than Phase 22 and documented.
- Tenderly integration exists at least as a usable scaffold with config/docs if full live credentials are unavailable.
- Docs and JSON/policy artifacts agree with each other.

Work in a careful, repo-aware way:
- First inspect and summarize the current state.
- Then implement.
- Then run tests / validation.
- Then produce the implementation report.

Do not stop at analysis only. Make the changes.