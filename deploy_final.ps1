$rpc = "http://localhost:8545"
$pk = (Select-String -Path .env -Pattern "^SIGNER_PRIVATE_KEY=").Line.Split('=')[1]
Push-Location contracts
& "..\foundry_bin\forge.exe" create src/ArbExecutor.sol:ArbExecutor --rpc-url $rpc --private-key $pk --evm-version london --gas-limit 5000000 --json
Pop-Location
