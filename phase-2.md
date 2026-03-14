Implement Phase 2 only.

Important change:
We are using REAL provider endpoints in this phase, not mock-only provider wiring.

I will provide real environment variables locally (not in the repo) for:
- QUICKNODE_WSS_URL
- ALCHEMY_WSS_URL

Do not hardcode any URLs or secrets.
Do not write real secrets into any file.
Only read them from environment/config at runtime.

Goal:
Build the provider and ingestion foundation for ArbHunter on Base using:
- QuickNode as primary provider
- Alchemy as failover provider

Scope:
- provider abstraction
- real WSS provider connection foundation
- failover logic
- provider health + latency metrics
- Base Flashblocks-aware ingestion foundation
- pending log ingestion foundation
- replay/test harness for ingestion
- no strategy logic
- no route logic
- no simulation logic
- no execution logic

Work only in the relevant crates:
- crates/arb_types
- crates/arb_config
- crates/arb_metrics
- crates/arb_providers
- crates/arb_ingest
- bin/arb_daemon

Requirements:

1. arb_types
Create minimal shared types for:
- ProviderKind
- ProviderHealth
- ProviderLatencySample
- IngestEvent
- PendingLogEvent
- FlashblockEvent
- ProviderStatusSnapshot

Keep these types minimal and serializable where useful.

2. arb_config
Add typed environment/config loading for:
- QUICKNODE_WSS_URL
- ALCHEMY_WSS_URL
- CHAIN_ID
- LOG_LEVEL
- METRICS_PORT
- optional feature flags:
  - ENABLE_FLASHBLOCKS
  - ENABLE_PENDING_LOGS
  - ENABLE_FAILOVER

Rules:
- startup-only config parsing
- no disk reads in hot path
- fail clearly if required env vars are missing for live provider mode

3. arb_metrics
Add a small metrics/logging layer for:
- provider_connected
- provider_disconnected
- provider_reconnect_attempts
- provider_latency_ms
- failover_switches
- events_ingested_total
- flashblocks_seen_total
- pending_logs_seen_total

Keep it simple and non-blocking.

4. arb_providers
Create a provider abstraction layer that supports:
- QuickNode primary
- Alchemy failover
- real websocket connection foundation
- reconnect logic
- health tracking
- latency measurement
- failover decision logic

Important:
- no trading logic
- no routing logic
- no simulation logic
- no business decisions here

Also:
- do not make the architecture depend only on WebSocket subscriptions
- keep the provider abstraction compatible with pending-state RPC methods and fallback behavior later

5. arb_ingest
Create ingestion foundation for:
- Base Flashblocks-aware subscription plumbing
- pending log ingestion plumbing
- normalized internal event conversion
- rolling in-memory event channel/buffer
- replay harness support from local fixture files

Important:
- support the concept of:
  - flashblock events
  - pending logs
  - future V2/V3 DEX event decoding
- but do not implement full DEX decoding yet
- this phase is about ingestion infrastructure, not strategy

6. bin/arb_daemon
Wire together:
- config loading
- provider startup
- metrics startup
- ingest pipeline startup
- graceful shutdown
- basic startup logs

No strategy engine yet.
No execution path yet.

Testing requirements:
- unit tests for config parsing
- unit tests for provider health/failover logic
- unit tests for normalized event conversion
- replay harness test using fixture input
- workspace must compile cleanly

Constraints:
- Postgres remains off the hot path
- no arb_storage usage in ingestion hot path
- no Tenderly in this phase
- no execution logic in this phase
- keep everything modular and pipeline-oriented

Artifacts required:
1. Implementation summary
2. Walkthrough artifact
3. Repo tree or changed-files summary
4. Checklist showing:
   - provider abstraction added
   - QuickNode + Alchemy config added
   - real provider connection foundation added
   - failover logic added
   - Flashblocks/pending ingestion foundation added
   - replay harness added
   - no strategy/execution logic added

Do not go beyond Phase 2.

environment variables



----- UPDATES ----

Implement a final Phase 2 cleanup + completion pass on branch `phase-2-providers-ingest`.

Goal:
Make this branch safe, honest, and close to merge-ready by:
1. removing secret/artifact clutter,
2. fixing repo hygiene,
3. replacing the current mock-heavy ingest parsing with real structured parsing for at least one provider message shape,
4. keeping Phase 2 strictly limited to providers + ingestion foundation.

IMPORTANT CONSTRAINTS:
- Do NOT add strategy logic.
- Do NOT add routing logic.
- Do NOT add simulation logic.
- Do NOT add execution logic.
- Do NOT add Postgres to the hot path.
- Do NOT require real credentials to compile or run tests.
- Do NOT write any real secrets to tracked files.
- Do NOT create or commit a real `.env` file.
- Keep `.env.example` only as a template.

========================================
PART 1 — REPO CLEANUP / SECURITY HYGIENE
========================================

Remove these tracked files/directories from the branch if present:
- .env
- target/
- build_errors.txt
- build_errors2.txt
- diff_output.txt
- force_lf_write.py
- repo_tree.txt
- test_err.txt
- verify.py
- verify_output.txt

Add or update `.gitignore` so that these are ignored going forward:
- .env
- .env.local
- target/
- *.txt debug outputs like:
  - build_errors*.txt
  - diff_output.txt
  - test_err.txt
  - verify_output.txt
  - repo_tree.txt
- one-off helper scripts that are not part of the product
- OS/editor junk

Rules:
- Keep `.env.example`
- Do not delete legitimate project files
- If any tracked secrets/artifact files are removed, include them in the changed-files summary

========================================
PART 2 — PROVIDER LAYER CLEANUP
========================================

In `crates/arb_providers`:

Current problem:
- provider connection is partly real, but health/latency logic is still placeholder-heavy.

Required fixes:
1. Keep real `tokio-tungstenite` websocket connection foundation.
2. Remove fake hardcoded latency constants where possible.
3. If true latency measurement is not yet fully implemented, replace fake values with clearly named placeholder state and TODOs that do not pretend to be real measurements.
4. ProviderManager must remain responsible for:
   - primary provider = QuickNode
   - failover provider = Alchemy
   - reconnect attempts
   - provider health transitions
   - failover switching
5. The code should clearly separate:
   - connection foundation
   - health tracking
   - failover policy

Do not try to build trading logic here.

========================================
PART 3 — INGEST LAYER: REMOVE MOCK SUBSTRING PARSING
========================================

In `crates/arb_ingest`:

Current problem:
- ingest currently relies on simplistic substring matching / mock event creation.

Required fixes:
Replace substring-based mock parsing with structured parsing for at least:
1. one real Flashblocks-style message shape
2. one real pending-log style message shape

Requirements:
- Use `serde` / `serde_json` with minimal real message structs or enums.
- Normalize parsed messages into the existing internal `IngestEvent` representation.
- Keep the design extensible for future V2/V3 DEX decoding.
- It is okay if only a subset of the provider message is parsed for now, but it must be real structured parsing, not substring matching.

Implementation guidance:
- Add fixture files under `fixtures/` containing representative sample payloads for:
  - one flashblock-like payload
  - one pending-log-like payload
- ReplayHarness should be able to read those fixtures and feed them through the parser.

Do NOT add DEX-specific strategy logic yet.
This phase is still about ingest infrastructure only.

========================================
PART 4 — TESTING / VALIDATION
========================================

Add or improve tests for:
1. `arb_config`
   - required env parsing
   - optional feature flags
2. `arb_providers`
   - provider health transitions
   - failover decision behavior
3. `arb_ingest`
   - structured parsing from fixture payloads
   - normalization into internal event types
4. replay harness
   - reading fixture files and emitting normalized events

The project must still compile cleanly with:
- `cargo check --workspace`
- `cargo test --workspace`

Do NOT require live provider credentials for unit tests.

========================================
PART 5 — DOCUMENTATION / ARTIFACT HONESTY
========================================

Update the Phase 2 walkthrough / summary language so it is accurate.

Important:
Do NOT oversell this phase as “fully real production ingest” if it is still only a foundation.

The final wording should make clear:
- real provider connection foundation exists,
- failover foundation exists,
- structured ingest foundation exists for at least one real message shape,
- strategy / route / sim / execution are still intentionally absent.

========================================
REQUIRED OUTPUTS
========================================

When finished, produce:

1. A checklist confirming:
- secret/artifact clutter removed
- `.gitignore` updated
- `.env` is not tracked
- provider layer cleaned up
- fake substring ingest parsing removed
- structured parsing added for at least one flashblock-like and one pending-log-like message shape
- fixture-based replay test added
- no strategy/sim/execution logic added

2. A changed-files summary

3. A walkthrough artifact summarizing:
- what was cleaned up
- what provider logic is now real vs still future work
- what ingest parsing is now real vs still future work
- what tests were added

4. Any follow-up TODOs that should belong to Phase 3, not Phase 2

Do not go beyond Phase 2.


---- Updates part 3 ----

Implement a final Phase 2 completion pass on branch `phase-2-providers-ingest`.

Goal:
Make Phase 2 truly merge-ready by:
1. fixing repo hygiene/security,
2. completing the provider -> ingest integration,
3. making failover behavior real (not just conceptual),
4. keeping the scope strictly limited to provider + ingestion foundation.

IMPORTANT CONSTRAINTS
- Do NOT add strategy logic.
- Do NOT add routing logic.
- Do NOT add simulation logic.
- Do NOT add execution logic.
- Do NOT add Postgres to the hot path.
- Do NOT require real credentials for unit tests.
- Do NOT print or expose any secret values in output, logs, artifacts, code, docs, or diffs.
- Keep `.env.example` as the only tracked env template.
- Preserve local developer usability, but `.env` must not remain tracked in git.

================================================================
PART 1 — REPO HYGIENE / SECRET CLEANUP
================================================================

Current issue:
The branch still appears to track secret/artifact clutter and debug leftovers.

Required fixes:
1. Remove these from git tracking if present:
- .env
- target/
- build_errors.txt
- build_errors2.txt
- diff_output.txt
- force_lf_write.py
- repo_tree.txt
- test_err.txt
- verify.py
- verify_output.txt

2. Update `.gitignore` so these remain ignored going forward:
- .env
- .env.local
- target/
- debug output files
- temporary verification/helper scripts
- OS/editor junk

3. IMPORTANT:
- If `.env` exists locally and contains real credentials, do NOT print it and do NOT destroy useful local values.
- Untrack `.env` from git while preserving local developer workflow if possible.
- Keep `.env.example` only as the tracked template.

Acceptance criteria:
- `git ls-files .env` returns nothing
- tracked junk/artifact files above are removed from version control
- `.gitignore` reflects the ignore policy

================================================================
PART 2 — FINISH PROVIDER LAYER
================================================================

Current issue:
Provider connection exists, but incoming websocket frames are discarded and failover is still partly conceptual.

Required fixes in `crates/arb_providers`:
1. Keep the real websocket connection foundation using tokio-tungstenite.
2. Stop discarding all incoming frames.
3. Expose a clean internal stream/channel API so provider messages can be consumed by `arb_ingest`.
4. Make failover behavior operational:
   - QuickNode = primary
   - Alchemy = backup
   - ProviderManager must actually update active provider state when failover occurs
   - Emit/provider-status metrics on transitions
5. Keep latency logic honest:
   - if real ping/pong latency is not fully implemented yet, do NOT fake it with hardcoded values
   - use clear placeholder/TODO state where needed
   - connection detachment and reconnect behavior should still work
6. Keep provider logic modular:
   - connection management
   - provider health
   - active provider selection
   - message forwarding
   should be cleanly separated

Do NOT add any business logic here.

================================================================
PART 3 — REAL PROVIDER -> INGEST BRIDGE
================================================================

Current issue:
`arb_ingest` can parse fixture-based payloads, but live provider frames are not actually bridged into the ingest pipeline.

Required fixes:
1. Wire provider output into `arb_ingest` so live websocket messages can flow into the ingest pipeline.
2. `arb_ingest` should accept raw provider payloads and normalize them into internal `IngestEvent` values.
3. Keep support for:
   - flashblock-like messages
   - pending-log-like messages
4. At least one real provider message shape for each of:
   - flashblock-style payload
   - pending-log-style payload
   must be structurally parsed with serde / serde_json
5. Keep fixture replay support intact under `fixtures/`.

Important:
- This phase is still ingestion infrastructure only.
- Do NOT add DEX strategy decoding, route finding, or execution decisions.
- Do NOT overclaim support beyond the message shapes actually implemented.

================================================================
PART 4 — DAEMON INTEGRATION
================================================================

In `bin/arb_daemon`:

Required fixes:
1. Wire together:
   - config
   - metrics
   - provider manager
   - live message bridge
   - ingest pipeline
2. On startup:
   - log provider mode and enabled features
   - initialize provider connections
   - initialize ingest consumer
3. On shutdown:
   - perform graceful teardown
4. Keep it as a foundation only:
   - no strategy
   - no route logic
   - no transaction sending

================================================================
PART 5 — TESTS / VALIDATION
================================================================

Add or improve tests for:
1. `arb_config`
   - required env parsing
   - optional feature flags
2. `arb_providers`
   - provider health transitions
   - active provider switching
   - reconnect behavior (to the extent testable)
3. `arb_ingest`
   - structured parsing from fixture payloads
   - normalization into internal event types
4. replay harness
   - fixture files can be consumed end-to-end
5. daemon smoke-level validation if practical without live secrets

The branch must still pass:
- `cargo check --workspace`
- `cargo test --workspace`

Do NOT require real provider credentials for unit tests.

================================================================
PART 6 — DOCUMENTATION / HONESTY
================================================================

Update the Phase 2 summary/walkthrough/checklist so they are honest and precise.

The final wording should clearly state:
- real websocket connection foundation exists
- active provider switching exists
- provider -> ingest bridge exists
- structured parsing exists for at least one flashblock-like and one pending-log-like payload shape
- strategy/route/sim/execution are still intentionally absent

Do NOT oversell beyond what is implemented.

================================================================
REQUIRED OUTPUTS
================================================================

When finished, provide:

1. A checklist confirming:
- `.env` is no longer tracked
- junk/artifact files were removed from git
- `.gitignore` updated
- provider manager now forwards real messages instead of discarding them
- active provider switching is real
- provider -> ingest bridge is real
- structured parsing exists for at least one flashblock-like payload and one pending-log-like payload
- replay harness still works
- no strategy/sim/execution logic was added

2. A changed-files summary

3. A walkthrough artifact describing:
- what was cleaned up
- how provider messages now flow into ingest
- what failover now really does
- what is still deferred to Phase 3

4. Any follow-up TODOs that belong to Phase 3, not Phase 2

Do not go beyond Phase 2.