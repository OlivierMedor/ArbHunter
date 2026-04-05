$pk = (Select-String -Path "C:\Users\olivi\Documents\ArbHunger\.env" -Pattern "^SIGNER_PRIVATE_KEY=").Line.Split('=')[1]
cd "C:\Users\olivi\Documents\ArbHunger\contracts"
& "C:\Users\olivi\Documents\ArbHunger\foundry_bin\forge.exe" create src/ArbExecutor.sol:ArbExecutor --rpc-url http://localhost:8545 --private-key $pk
