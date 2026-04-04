$signer = (powershell -File c:\Users\olivi\Documents\ArbHunger\get_address.ps1).Trim()
$nonce = (& "C:\Users\olivi\Documents\ArbHunger\foundry_bin\cast.exe" nonce $signer --rpc-url http://localhost:8545).Trim()
Write-Output "Signer: $signer"
Write-Output "Nonce: $nonce"
