You are working in the ArbHunter repo.

Branch:
main

This is the REAL BASE DEPLOYMENT PREP pass.
Do NOT start live trading.
Do NOT execute production arbitrage transactions.
Do NOT plan only.
Actually prepare the repo and environment for real Base deployment of ArbExecutor, then leave the system safe/default-off.

Use the operator’s CURRENT rotated secrets and CURRENT clean local env.
Do NOT print secrets.
Do NOT echo:
- SIGNER_PRIVATE_KEY
- TENDERLY_API_KEY
- full .env contents

Goal:
1. Verify main is healthy after the 0.01 ETH cap change.
2. Deploy/redeploy ArbExecutor on REAL Base using the operator’s new signer.
3. Write the real EXECUTOR_CONTRACT_ADDRESS into the operator’s local production env.
4. Leave the system SAFE / DEFAULT-OFF after deployment.
5. Produce the operator checklist for the first real live canary, but do NOT trigger it.

Critical rules:
- do not start live trading
- do not intentionally execute arbitrage on mainnet
- do not flip the daemon into active canary execution without an explicit final operator step
- do not use old leaked keys
- use the new signer only
- do not print secret values

A. Verify main branch and environment
Run:
- cargo check --workspace --all-targets
- cargo test -p arb_execute
- cargo test -p arb_canary
- cargo test -p arb_daemon
- relevant Foundry tests for ArbExecutor

Check env presence only (yes/no) for:
- SIGNER_PRIVATE_KEY
- TENDERLY_ENABLED
- TENDERLY_API_KEY
- TENDERLY_ACCOUNT_SLUG
- TENDERLY_PROJECT_SLUG
- QUICKNODE_WSS_URL
- RPC_HTTP_URL or QUICKNODE_HTTP_URL

Also derive and report:
- public signer address only

B. Reconfirm active canary posture on main
Verify current active posture:
- multi only
- direct blocked
- max trade size 0.03 ETH
- max concurrent trades 1
- stop after 3 consecutive reverts
- cumulative loss cap 0.01 ETH
- live-capable/default-off

C. Deploy ArbExecutor on REAL Base
1. Use the operator’s current signer from SIGNER_PRIVATE_KEY
2. Deploy/redeploy ArbExecutor to real Base mainnet
3. Capture and report:
   - deployed contract address
   - public deployer/signer address
   - exact deploy command used
4. Verify:
   - contract owner equals the signer/deployer
5. If deployment fails:
   - fix only what is necessary
   - rerun deployment
   - report exact blocker/resolution

D. Update local production env safely
1. Write the real Base deployed contract address into the operator’s local production env as:
   - EXECUTOR_CONTRACT_ADDRESS=0x...
2. Do NOT commit or track any env file
3. Do NOT modify fork-local overlay files unless needed for cleanup

E. Keep system default-off after deployment
Desired safe post-deploy posture:
- CANARY_LIVE_MODE_ENABLED=false
- ENABLE_BROADCAST=false
- DRY_RUN_ONLY=true

If documenting a ready-to-flip config, document it only; do not leave the daemon in active live mode.

F. Produce the first live-canary operator checklist
Include:
1. deployed contract address confirmed
2. signer address confirmed
3. wallet balance check
4. Tenderly credentials confirmed
5. provider endpoints confirmed
6. exact live flags to change when ready:
   - CANARY_LIVE_MODE_ENABLED=true
   - ENABLE_BROADCAST=true
   - DRY_RUN_ONLY=false
7. monitoring items for the first live canary:
   - revert count
   - cumulative realized loss
   - tx submission / polling behavior
   - Tenderly/preflight status
8. explicit stop conditions:
   - 3 consecutive reverts
   - cumulative loss cap hit
   - incomplete attribution / ambiguous tx state

G. Final report
Return:
1. Branch verdict
Use exactly one:
- READY FOR REAL BASE DEPLOYMENT
- DEPLOYED TO REAL BASE / SAFE DEFAULT-OFF
- NEEDS MINOR FIXES
- NOT READY

2. Exact commands run

3. Verification results
- pass/fail by command

4. Env presence check
- yes/no only
- no secret values

5. Derived signer
- public signer address only

6. Deployment result
- deployed contract address
- public deployer/signer address
- whether owner matches signer

7. Files changed
- if any non-env tracked files changed, explain why

8. Operator checklist
- exact next safe steps before first live canary

Do not stop at planning.
Actually perform the real Base deployment prep and leave the system in a safe default-off state.

---- update ----

You are working in the ArbHunter repo.

Branch:
main

Important:
Phase 24 is already merged into main, and the cumulative loss cap has already been lowered to 0.01 ETH on main.
Do NOT create another implementation plan.
Do NOT try to merge phase-24-recovery-restore again.
Do NOT do live trading.
Actually perform the deployment-prep work.

Goal:
Prepare the system for real Base deployment of ArbExecutor using the operator’s rotated signer and rotated provider/Tenderly keys, then leave everything SAFE / DEFAULT-OFF.

Critical rules:
- do not start live trading
- do not intentionally execute arbitrage on mainnet
- do not flip the daemon into active live canary execution without an explicit final operator step
- do not print secrets
- do not echo SIGNER_PRIVATE_KEY, TENDERLY_API_KEY, or full .env contents
- use only the operator’s new/rotated signer and keys

A. Verify current main branch health
Run:
- cargo check --workspace --all-targets
- cargo test -p arb_execute
- cargo test -p arb_canary
- cargo test -p arb_daemon
- relevant Foundry tests for ArbExecutor

If something fails:
- fix only what is necessary
- rerun the affected commands
- clearly separate fixes from verification

B. Safe environment presence check
Check presence only (yes/no) for:
- SIGNER_PRIVATE_KEY
- TENDERLY_ENABLED
- TENDERLY_API_KEY
- TENDERLY_ACCOUNT_SLUG
- TENDERLY_PROJECT_SLUG
- QUICKNODE_WSS_URL
- RPC_HTTP_URL or QUICKNODE_HTTP_URL

Also:
- derive and report the public signer address only
- do NOT print secret values

C. Reconfirm active canary posture on main
Verify and report the active posture on main:
- multi only
- direct blocked
- max trade size 0.03 ETH
- max concurrent trades 1
- stop after 3 consecutive reverts
- cumulative loss cap 0.01 ETH
- live-capable/default-off

D. Deploy ArbExecutor on REAL Base
1. Use the operator’s current signer from SIGNER_PRIVATE_KEY
2. Deploy/redeploy ArbExecutor to real Base mainnet
3. Capture and report:
   - deployed contract address
   - public deployer/signer address
   - exact deploy command used
4. Verify:
   - contract owner equals the signer/deployer
5. If deployment fails:
   - fix only what is necessary
   - rerun deployment
   - report exact blocker/resolution

E. Update local production env safely
1. Write the real Base deployed contract address into the operator’s local production env as:
   - EXECUTOR_CONTRACT_ADDRESS=0x...
2. Do NOT commit or track any env file
3. Do NOT modify fork-local overlay files unless needed for cleanup

F. Keep the system SAFE / DEFAULT-OFF after deployment
Desired safe post-deploy posture:
- CANARY_LIVE_MODE_ENABLED=false
- ENABLE_BROADCAST=false
- DRY_RUN_ONLY=true

If documenting a ready-to-flip config, document it only; do not leave the daemon in active live mode.

G. Produce the first live-canary operator checklist
Include:
1. deployed contract address confirmed
2. signer address confirmed
3. wallet balance check
4. Tenderly credentials confirmed
5. provider endpoints confirmed
6. exact live flags to change when ready:
   - CANARY_LIVE_MODE_ENABLED=true
   - ENABLE_BROADCAST=true
   - DRY_RUN_ONLY=false
7. monitoring items for the first live canary:
   - revert count
   - cumulative realized loss
   - tx submission / polling behavior
   - Tenderly/preflight status
8. explicit stop conditions:
   - 3 consecutive reverts
   - cumulative loss cap hit
   - incomplete attribution / ambiguous tx state

H. Final report
Return:
1. Branch verdict
Use exactly one:
- READY FOR REAL BASE DEPLOYMENT
- DEPLOYED TO REAL BASE / SAFE DEFAULT-OFF
- NEEDS MINOR FIXES
- NOT READY

2. Exact commands run

3. Verification results
- pass/fail by command

4. Env presence check
- yes/no only
- no secret values

5. Derived signer
- public signer address only

6. Deployment result
- deployed contract address
- public deployer/signer address
- whether owner matches signer

7. Files changed
- if any non-env tracked files changed, explain why

8. Operator checklist
- exact next safe steps before first live canary

Do not stop at planning.
Actually perform the real Base deployment prep and leave the system in a safe default-off state.