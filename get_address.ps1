$pk = (Select-String -Path "C:\Users\olivi\Documents\ArbHunger\.env" -Pattern "^SIGNER_PRIVATE_KEY=").Line.Split('=')[1].Trim()
$addr = (& "C:\Users\olivi\Documents\ArbHunger\foundry_bin\cast.exe" wallet address $pk).Trim()
$addr | Out-File -FilePath "C:\Users\olivi\Documents\ArbHunger\signer_address.txt" -Encoding ascii
