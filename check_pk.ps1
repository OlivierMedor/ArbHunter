$pk = (Select-String -Path "C:\Users\olivi\Documents\ArbHunger\.env" -Pattern "^SIGNER_PRIVATE_KEY=").Line.Split('=')[1].Trim()
Write-Output "Key Length: $($pk.Length)"
