You are working in the ArbHunter repo.

Start from:
main

Create a new branch, for example:
phase-24-controlled-base-live-canary

Phase 24 objective:
Implement a controlled Base live-canary capability on top of the current Phase 23 safety layer.

Critical safety requirements:
- Do NOT start the daemon or execute any live-trading process as part of this task.
- Do NOT run the strategy automatically.
- Do NOT enable live trading by default.
- Do NOT enable real broadcasts by default.
- Do NOT broaden to new chains.
- Do NOT expand route families.
- Do NOT raise canary limits.
- Do NOT silently switch to any unfinished/mock flash-loan live path.
- Keep the repo safe/off by default even after this phase is implemented.

Current posture to preserve unless explicitly documented otherwise:
- route family allowlist: multi
- route family blocklist: direct, unknown
- max trade size: 0.03 ETH
- max concurrent trades: 1
- stop after 3 consecutive reverts
- review threshold: 30 attempts
- cumulative realized loss cap: 0.05 ETH
- Tenderly should be a late-stage final pre-send safety gate
- live mode must remain explicit and operator-controlled

What I want in Phase 24

A. Live-canary activation path, but OFF by default
Implement the code/config needed for a real Base live canary while keeping default behavior disabled.

Requirements:
- keep default config non-live
- live mode should require explicit operator activation
- fail fast on startup if live mode is requested but required config is missing
- do not allow an ambiguous half-live posture

At minimum, validate on startup for live mode:
- ENABLE_BROADCAST=true
- DRY_RUN_ONLY=false
- CANARY_LIVE_MODE_ENABLED=true
- signer/private key present
- executor contract address present
- RPC HTTP URL present
- required websocket provider present
- TENDERLY_ENABLED=true
- TENDERLY_API_KEY present
- TENDERLY_ACCOUNT_SLUG present
- TENDERLY_PROJECT_SLUG present

If any required live-canary setting is missing, the process must refuse to enter live mode with a clear actionable error.

B. Durable canary state across restarts
Implement persistence for live-canary state, including at least:
- cumulative realized P&L
- cumulative realized loss
- consecutive reverts
- halted state
- attempt/review counters
- in-flight / pending live tx tracking as needed

Requirements:
- persist state to a durable local file or similarly simple durable mechanism
- restore canary state on restart before evaluating new live trades
- a restart must not wipe out the loss cap or revert streak
- document the persistence format and location

C. Pending tx reconciliation / restart recovery
Implement:
- persistence or journaling of pending live txs
- startup reconciliation for unresolved tx hashes
- safe handling of unknown final state on restart
- in-flight counter recovery so max_concurrent_trades=1 remains trustworthy

D. Real post-trade attribution from actual chain outcomes
Implement a real attribution path for broadcasted live transactions:
- wait for transaction receipt / confirmation
- capture tx hash
- capture receipt status
- capture actual gas used
- capture actual effective gas price / actual paid tx fee where available
- attempt to capture Base-specific fee components, especially L1 data/security fee, when available from provider/receipt/RPC extensions
- if some Base fee component is not available, record it explicitly as unknown or estimated rather than pretending it is known

Important:
- use actual receipt-based costs for live canary accounting whenever possible
- reverted live txs should count actual burned network fee as realized loss
- successful live txs should use actual realized outcome, not only pre-broadcast simulation

E. Use preflight gas results to set final tx gas settings
The current live path should not rely on fixed broadcast gas settings if better preflight estimates are available.

Implement:
- derive final live tx gas_limit from preflight estimate with a configurable safety multiplier
- add configurable min/max caps
- log the chosen gas limit and fee settings clearly
- document how the final gas settings are chosen

Important:
- if Tenderly returns gas_used and RPC estimate_gas is unavailable, use the best available source carefully
- do not fake precision
- keep the final choice explainable in logs and docs

F. Success-path execution attribution
Audit the current executor and add the minimal instrumentation needed for reliable live attribution.

If needed, add Solidity event(s) for:
- execution success
- amount in
- amount out
- root asset / target token
- actual profit before network fees
- whether repayment / flash-loan path was used
- tx-relevant metadata needed for attribution
- ownership transferred
- withdrawal/rescue actions

Then:
- parse those events off-chain
- connect them to tx hash / candidate / canary attempt
- use them in realized P&L reporting

Keep this clean and minimal.
If you add contract events, update tests and docs and clearly note any deploy/migration implications.

G. Move live canary accounting to actual outcome timing
In live mode:
- do NOT finalize canary P&L/loss/revert accounting purely from simulated outcome before broadcast
- update canary outcome from the actual post-broadcast result when receipt/outcome is known
- keep simulated/predicted values as prediction telemetry, not as final realized truth

In shadow/sim mode:
- keep the existing simulated accounting behavior as appropriate
- but make the distinction between predicted and realized explicit

H. Tenderly usage should be late-stage, measurable, and configurable
Tenderly should run only after a candidate has already passed:
- normal filtering
- canary eligibility
- local simulation / earlier cheap checks

Requirements:
- do not spam Tenderly on low-quality candidates
- add explicit metrics/logs for:
  - Tenderly invoked count
  - Tenderly success/failure/timeout count
  - Tenderly latency / duration
  - preflight stage durations overall
- make Tenderly timeout configurable
- document the default timeout and rationale
- in live mode, if Tenderly is required and fails/times out, the tx must not be sent

I. Keep the current route/size posture
Do NOT broaden strategy scope in Phase 24.

Specifically keep:
- multi only
- direct blocked
- unknown blocked
- max trade size 0.03 ETH
- max concurrent trades 1
- 3 consecutive revert halt
- 30-attempt review threshold
- 0.05 ETH cumulative loss cap

If you think a limit should change, document it, but do not silently change it.

J. Do not rely on unfinished/mock flash-loan live execution
Audit the current standard live execution path vs atomic path.

Requirements:
- if the atomic/flash-loan path still depends on mocks or incomplete lender integration, do not use it for live canary
- keep the live canary on the proven/current standard execution path unless a fully real alternative is implemented and clearly justified

If flash-loan live support is incomplete:
- document that clearly
- keep it out of live-canary activation scope

K. Contract hardening before any real live deployment
Audit and improve the executor contract for production safety.

Required contract/admin additions:
- owner-only ERC20 withdraw/rescue
- owner-only native asset withdraw/rescue if applicable
- transferOwnership
- events for ownership transfer and withdrawals/rescues

Critical callback/security work:
- harden uniswapV3SwapCallback so unauthorized callers cannot drain tokens
- validate the caller against the expected pool / active execution context
- add negative tests proving an unauthorized callback cannot pull funds

Token transfer safety:
- replace or harden bare token transfer flows so failed ERC20 transfers cannot silently pass
- use safe transfer semantics or explicit return-value checks
- add tests for non-standard transfer behavior where practical

Optional only if low-risk and small:
- a pause/emergency-stop mechanism
Do not let this optional item delay the required callback/auth/withdraw fixes.

L. Operator runbook and env example
Create/update:
- phase-24.md as an implementation report
- a concise live-canary runbook / activation checklist
- an env example for live canary

The runbook should cover:
- required accounts / credentials
- Tenderly setup fields
- required RPC/provider fields
- signer funding prerequisites
- executor funding / token prerequisites if relevant
- safe startup order
- dry-run/shadow smoke test before live
- explicit live enable sequence
- emergency stop / halt / reset procedure
- how to review first 30 attempts
- what to inspect if loss cap or revert halt is hit

M. Truthful policy artifacts
Update policy/docs so they match the real Phase 24 state.

Important:
- do not claim that live canary has already been run if it has not
- do not mark the phase complete with fake profitability claims
- distinguish clearly between:
  - live-capable code path
  - operator-ready configuration
  - actual live deployment results

N. Multi-wallet is NOT part of Phase 24
Do NOT implement multi-wallet parallel execution in this phase.

Instead:
- add a short design note describing the future options:
  - one executor per wallet
  - multiple authorized operators on one executor
- note the tradeoffs
- keep current live canary single-wallet / single-lane

O. Tests and validation
Add/extend tests for:
- startup refusal when live mode is misconfigured
- persistence and restore of canary state
- pending tx reconciliation logic
- live canary accounting updates from actual outcomes
- receipt-based fee handling paths
- Tenderly timeout/failure behavior in live mode
- owner-only withdraw/rescue
- transferOwnership
- unauthorized V3 callback rejection
- safe token transfer handling
- operator safeguards remaining intact

Run at minimum:
- cargo check --workspace
- cargo test -p arb_canary
- cargo test -p arb_execute
- cargo test -p arb_daemon if relevant
- any additional package tests you modify
- any Solidity/Foundry tests relevant to new events or contract changes

Constraints
- Base only
- no new DEX expansion
- no new chain expansion
- no broad refactor unless truly required
- no pretend live performance claims
- no hidden policy loosening
- keep default posture safe/off
- do not weaken existing shadow+broadcast safety protections
- do not start the daemon or any live process as part of this task

Deliverables
1. Code changes for controlled live-canary enablement
2. Durable canary-state persistence
3. Pending tx recovery/reconciliation
4. Receipt-based live attribution and fee accounting
5. Improved gas-setting logic based on preflight estimates
6. Contract hardening (withdraw/rescue, ownership transfer, callback security, transfer safety)
7. Any minimal contract instrumentation needed for attribution
8. Updated docs/runbook/env examples
9. Tests and validation results
10. A concise implementation report with:
   - what you changed
   - exact files changed
   - exact commands/tests run
   - what still remains incomplete
   - whether the repo is:
     a) live-capable but default-off
     b) operator-ready for first canary
     c) still blocked by any unresolved issue

Acceptance criteria
- default repo posture remains non-live unless explicitly enabled
- live mode refuses to start without full required config
- canary state persists across restarts
- restart does not erase the loss cap / revert streak / halt state
- live outcome accounting is based on actual chain results, not only pre-broadcast simulation
- reverted live txs count real burned fee as realized loss
- successful live txs have a reliable attribution path
- Tenderly is used late in the pipeline and is measurable/configurable
- final tx gas settings are derived from preflight estimates with documented safeguards
- executor contract has owner withdrawal/rescue and ownership transfer
- unauthorized V3 callback cannot drain funds
- no silent switch into mock/incomplete flash-loan live mode
- docs accurately reflect what is real, what is default-off, and what is still incomplete
- task does not auto-start live trading

Work in this order:
1. Audit current main state
2. Implement the minimal safe Phase 24 changes
3. Validate with tests/builds
4. Write the implementation report and runbook

Do not stop at analysis only.
Make the changes.