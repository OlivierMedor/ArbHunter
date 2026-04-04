# Phase 24 Canary Daemon Smoke Test (Fork-Local)
$env:CANARY_LIVE_MODE_ENABLED = "false"
$env:DRY_RUN_ONLY = "true"
$env:ENABLE_BROADCAST = "false"
$env:RPC_HTTP_URL = "http://localhost:8545"
$env:EXECUTOR_CONTRACT_ADDRESS = "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512"

# Run the daemon for a short burst and redirect output
& "cargo" run --bin arb_daemon -- --smoke-test > daemon_smoke.log 2>&1
