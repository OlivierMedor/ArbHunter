$rpc = (Select-String -Path .env -Pattern "^QUICKNODE_HTTP_URL=").Line.Split('=')[1]
& "C:\Users\olivi\Documents\ArbHunger\foundry_bin\anvil.exe" --fork-url $rpc --chain-id 8453 --block-time 2 --fork-retry-backoff 1000
