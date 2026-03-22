Implement Phase 15 on a dedicated branch.

Suggested branch name:
phase-15-live-shadow-mode

Before doing any code work:
1. Ensure work is being done on branch `phase-15-live-shadow-mode`
   - if the branch does not exist, create it from current main
   - if it exists, switch to it
2. Do NOT work directly on main

Goal:
Build a live shadow-mode / paper-trading layer that runs the real pipeline against live data but never broadcasts transactions.

Scope:
- live shadow mode
- candidate journaling
- plan journaling
- delayed outcome checking
- prediction vs observed drift tracking
- no live broadcasts
- no real trading
- no mempool/PGA tactics

This phase is NOT about:
- real-money execution
- live canary trades
- private relays
- EV learning policy changes
- multi-wallet scaling

==================================================
PHASE 15 OBJECTIVE
==================================================

By the end of this phase, the system should be able to:

1. Run the live pipeline:
   provider -> ingest -> state -> graph -> filter -> simulation -> execution plan
2. Decide whether a candidate would have qualified for execution
3. Record a structured shadow-trade journal entry
4. Re-check the same opportunity after a short delay window
5. Compare:
   - predicted profit
   - predicted output
   - observed later state / later profitability estimate
   - drift / decay
6. Expose useful shadow-mode metrics
7. Never broadcast a real transaction

==================================================
WORK AREAS
==================================================

Likely files/crates to work in:
- crates/arb_types
- crates/arb_config
- crates/arb_metrics
- crates/arb_sim
- crates/arb_execute
- bin/arb_daemon
- docs if needed

==================================================
PART 1 — CONFIG
==================================================

Add minimal config for shadow mode, for example:
- ENABLE_SHADOW_MODE
- SHADOW_RECHECK_DELAY_MS
- SHADOW_MIN_PROFIT_THRESHOLD
- SHADOW_MAX_CANDIDATES_PER_WINDOW
- SHADOW_WRITE_JOURNAL
- SHADOW_JOURNAL_PATH (if file output is used)

Safe defaults:
- ENABLE_SHADOW_MODE=true only when explicitly set
- ENABLE_BROADCAST must remain false by default
- shadow mode must never send transactions

==================================================
PART 2 — TYPE SYSTEM
==================================================

Add minimal shared types such as:
- ShadowDecision
- ShadowCase
- ShadowJournalEntry
- ShadowOutcome
- DriftSummary
- ShadowRecheckResult

Fields should capture:
- candidate id / route id
- predicted amount out
- predicted profit
- predicted gas
- route family
- decision timestamp
- delayed recheck timestamp
- observed outcome / decay
- decision reason

Keep them minimal and serializable.

==================================================
PART 3 — SHADOW JOURNAL
==================================================

Build a shadow journaling path that records:
- candidate considered
- candidate promoted
- simulation result
- execution plan built
- would_trade yes/no
- reason
- later recheck result

The journal can be:
- JSONL
- structured file output
- or in-memory + report output

Keep it simple and inspectable.

==================================================
PART 4 — DELAYED RECHECK
==================================================

Implement a delayed recheck mechanism.

For each shadow-trade candidate:
- wait a configurable delay
- re-evaluate the opportunity on the newer state
- compare against the original prediction

Record:
- still profitable? yes/no
- profit drift
- amount-out drift
- error / decay
- reason if invalidated

Do NOT broadcast.
Do NOT fake execution.

==================================================
PART 5 — METRICS
==================================================

Add or update metrics honestly, for example:
- arb_shadow_candidates_total
- arb_shadow_promoted_total
- arb_shadow_would_trade_total
- arb_shadow_rechecks_total
- arb_shadow_still_profitable_total
- arb_shadow_invalidated_total
- arb_shadow_avg_profit_drift
- arb_shadow_avg_output_drift

Do not fake metrics.

==================================================
PART 6 — DAEMON INTEGRATION
==================================================

In `bin/arb_daemon`:
Wire shadow mode into the live pipeline.

Expected behavior:
- if shadow mode enabled, run the pipeline live
- build candidates and plans
- record journal entries
- schedule delayed rechecks
- never submit transactions

Make logging clear and concise.

==================================================
PART 7 — DOCUMENTATION HONESTY
==================================================

Update docs/checklists/walkthrough so they clearly state:

Real after Phase 15:
- live shadow mode exists
- candidates/plans are journaled
- delayed rechecks exist
- drift/decay can be measured
- no live trades are sent

Still deferred:
- live canaries
- real-money execution
- private relays
- EV learning policy automation
- production fleet scaling

==================================================
VALIDATION REQUIRED
==================================================

Run and report:
- cargo check --workspace
- cargo test --workspace
- docker compose config
- one live shadow-mode run for a short bounded window
- sample journal output
- sample delayed recheck results

==================================================
SOURCE-OF-TRUTH OUTPUTS REQUIRED
==================================================

At the end, include these exact commands and outputs:

1. Git identity:
- git fetch origin
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-15-live-shadow-mode
- git status --short
- git log --oneline --decorate -5

2. Grep proofs:
- git grep -n -E 'ENABLE_SHADOW_MODE|SHADOW_RECHECK_DELAY_MS|SHADOW_' -- crates/arb_config .env.example bin/
- git grep -n -E 'ShadowJournalEntry|ShadowOutcome|DriftSummary|ShadowRecheckResult' -- crates/ bin/
- git grep -n -E 'shadow|recheck|would_trade|drift' -- crates/arb_metrics bin/arb_daemon

3. Validation outputs:
- cargo check --workspace
- cargo test --workspace
- docker compose config
- output from one short shadow-mode run
- sample journal lines / report output

==================================================
REQUIRED OUTPUTS
==================================================

Provide:

1. A verdict:
- fully working
- working with known limitations
- blocked (and why)

2. A checklist confirming:
- live shadow mode added
- journal entries added
- delayed rechecks added
- drift reporting added
- no live trading logic added

3. Changed-files summary

4. A walkthrough artifact describing:
- how shadow mode works
- how candidates are journaled
- how delayed rechecks work
- what the drift metrics mean
- what remains deferred to the next phase

5. The source-of-truth outputs listed above

Do not go beyond Phase 15.


---- update ----

Run a Phase 15 validation pass on the EXISTING branch `phase-15-live-shadow-mode`.

Do NOT create a new branch.
Do NOT modify source code.
Do NOT change configs permanently.
Do NOT commit anything.
Do NOT summarize results in your reply only — you must write the full raw outputs to a file I can copy/paste.

Goal:
Run the exact validation commands, capture the raw outputs, and write them to a single file outside the repo so the working tree stays clean.

==================================================
OUTPUT FILE REQUIREMENT
==================================================

Write the final report to:

$env:TEMP\phase-15_validation_output.md

Also print the full path at the end of your reply.

The file must contain:
- the exact commands run
- the exact raw output for each command
- no omitted failures
- no paraphrased summaries in place of raw output
- section headers are fine, but keep the command outputs intact

Do NOT write the output file inside the git repo.

==================================================
RULES
==================================================

- Use PowerShell commands.
- Stay on `phase-15-live-shadow-mode`.
- Do not modify tracked files.
- Any temporary logs/journal samples should go in `$env:TEMP`.
- If a command fails, capture the failure output exactly.
- If a file is missing, capture that fact exactly.
- Do not redact outputs unless they contain secrets; if a secret would be printed, mask only the secret token portion and note that you masked it.

==================================================
COMMANDS TO RUN
==================================================

First capture git/source-of-truth:

1.
git fetch origin
git branch --show-current
git rev-parse HEAD
git rev-parse origin/phase-15-live-shadow-mode
git status --short
git log --oneline --decorate -5

2. Check whether the expected Phase 15 docs/artifacts exist on the remote-tracking branch:
git ls-tree -r --name-only origin/phase-15-live-shadow-mode | Select-String 'walkthrough\.md|phase-15\.md|checklist\.md|changed_files_summary\.md'

3. Grep proofs:
git grep -n -E 'ENABLE_SHADOW_MODE|SHADOW_RECHECK_DELAY_MS|SHADOW_MAX_CANDIDATES_PER_WINDOW|SHADOW_WRITE_JOURNAL|SHADOW_JOURNAL_PATH' -- crates/arb_config/src/lib.rs .env.example bin/
git grep -n -E 'ShadowJournalEntry|ShadowOutcome|ShadowRecheckResult|DriftSummary|ShadowDecision' -- crates/arb_types crates/arb_metrics bin/
git grep -n -E 'shadow_|would_trade|recheck|drift|journal' -- crates/arb_metrics/src/lib.rs bin/arb_daemon/src/main.rs
git grep -n -E 'SHADOW_MOCK_INJECTION|v2_v3_mixed' -- bin/arb_daemon/src/main.rs .env.example

4. Workspace / compose validation:
cargo check --workspace
cargo test --workspace
docker compose config
docker compose run --rm forge forge test

==================================================
BOUNDED LIVE SHADOW-MODE RUN
==================================================

Now do one bounded live shadow-mode run with temporary environment overrides.

Use these temporary PowerShell env vars for this process only:

$env:ENABLE_SHADOW_MODE="true"
$env:ENABLE_BROADCAST="false"
$env:DRY_RUN_ONLY="true"
$env:SHADOW_RECHECK_DELAY_MS="500"
$env:SHADOW_MAX_CANDIDATES_PER_WINDOW="5"
$env:SHADOW_WRITE_JOURNAL="true"
$env:SHADOW_JOURNAL_PATH="$env:TEMP\shadow_journal_test.jsonl"
$env:RUST_LOG="info"

Before running, remove old temp files if present:
Remove-Item "$env:TEMP\shadow_journal_test.jsonl" -ErrorAction SilentlyContinue
Remove-Item "$env:TEMP\shadow_stdout.log" -ErrorAction SilentlyContinue
Remove-Item "$env:TEMP\shadow_stderr.log" -ErrorAction SilentlyContinue

Then start the daemon in the background and let it run for about 45 seconds:

$p = Start-Process cargo -ArgumentList 'run','--bin','arb_daemon' -RedirectStandardOutput "$env:TEMP\shadow_stdout.log" -RedirectStandardError "$env:TEMP\shadow_stderr.log" -PassThru
Start-Sleep -Seconds 45
Stop-Process -Id $p.Id

After that, capture:

Get-Content "$env:TEMP\shadow_stdout.log" -Tail 80
Get-Content "$env:TEMP\shadow_stderr.log" -Tail 80
Get-Content "$env:TEMP\shadow_journal_test.jsonl" -Tail 20
Select-String -Path "$env:TEMP\shadow_journal_test.jsonl" -Pattern 'would_trade|recheck|drift|invalidated'
(Get-Item "$env:TEMP\shadow_journal_test.jsonl").Length

If the journal file was not created, capture that exactly.

==================================================
FILE FORMAT
==================================================

The markdown file should be structured like:

# Phase 15 Validation Output

## Source of Truth
[command]
[raw output]

## Remote Docs/Artifacts Check
[command]
[raw output]

## Grep Proofs
[command]
[raw output]

## Workspace Validation
[command]
[raw output]

## Live Shadow Run
[command]
[raw output]

## Journal Sample
[command]
[raw output]

Do not replace raw output with prose.

==================================================
FINAL REPLY
==================================================

In your reply to me:
1. confirm the file was written
2. give the exact path
3. briefly note whether the shadow run produced journal entries
4. do not paste the full contents unless there was an error writing the file


---- update 2 ----
Do a final Phase 15 proof pass on the EXISTING branch `phase-15-live-shadow-mode`.

Do NOT create a new branch.
Do NOT add new features unless absolutely required for proof.
Do NOT add live trading logic.
Do NOT expand scope beyond producing a real journaled shadow-mode proof.

Goal:
Make Phase 15 merge-ready by producing at least one real shadow journal entry and one delayed recheck result from a bounded daemon run.

==================================================
CURRENT BLOCKER
==================================================

The last validation file showed:
- daemon started
- but Journal Length: 0

So the live shadow-mode proof is still incomplete.

==================================================
OPTION A — PREFERRED
==================================================

Try to produce at least one real shadow journal entry from a bounded live run by adjusting only runtime parameters:

Use a bounded run with:
- ENABLE_SHADOW_MODE=true
- ENABLE_BROADCAST=false
- DRY_RUN_ONLY=true
- SHADOW_RECHECK_DELAY_MS=500
- SHADOW_MAX_CANDIDATES_PER_WINDOW increased modestly
- SHADOW_MIN_PROFIT_THRESHOLD relaxed if necessary
- runtime increased to 180–300 seconds

Do NOT change source logic if config/runtimes are enough.

Success criteria:
- at least one journal entry is written
- at least one delayed recheck result is present
- sample journal lines show would_trade / recheck / drift fields

==================================================
OPTION B — ACCEPTABLE FALLBACK
==================================================

If a real bounded live run still produces zero candidates, use a clearly labeled deterministic daemon smoke run with the existing mock-injection path (if present).

Requirements:
- this must run through the actual daemon path, not only a unit/integration test
- it must produce a real shadow_journal.jsonl file
- it must include at least one journal entry and one delayed recheck
- it must be explicitly labeled as:
  "deterministic shadow smoke test"
  not "live opportunity capture"

==================================================
SOURCE-OF-TRUTH OUTPUTS REQUIRED
==================================================

Run and paste exact outputs for:

1. Git identity:
- git fetch origin
- git branch --show-current
- git rev-parse HEAD
- git rev-parse origin/phase-15-live-shadow-mode
- git status --short
- git log --oneline --decorate -5

2. Validation:
- cargo check --workspace
- cargo test --workspace
- docker compose config
- docker compose run --rm forge forge test

3. Bounded daemon proof:
- exact command used for the daemon run
- stdout tail
- stderr tail
- journal tail
- Select-String over the journal for:
  would_trade|recheck|drift|invalidated
- journal file length in bytes

==================================================
SUCCESS CRITERIA
==================================================

Phase 15 is only merge-ready if the final outputs show:
- branch clean
- tests passing
- and at least one journal entry plus one delayed recheck result from the daemon path

==================================================
REQUIRED OUTPUTS
==================================================

Provide:
1. Verdict
2. Whether Option A or Option B was used
3. Changed-files summary (if any)
4. Checklist confirming:
   - daemon path produced journal entries
   - delayed recheck was captured
   - no live transactions were broadcast
   - branch is clean
5. Exact raw outputs for all commands above
6. A short walkthrough describing:
   - how the proof run was produced
   - whether it was live shadow capture or deterministic smoke test
   - what remains deferred