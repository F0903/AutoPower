. "$($PSScriptRoot)\variables.ps1"

Assert-Admin($MyInvocation.MyCommand.Definition)

sc.exe start $ServiceName
Start-Process -FilePath $ProxyPath -ErrorAction Stop
Write-Host "Success!"
Write-Host "It's recommended to restart your system as it otherwise sometimes won't work reliably."
Write-Host "Press any key to exit..."
$null = $Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown');