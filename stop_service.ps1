. "$($PSScriptRoot)\variables.ps1"

Assert-Admin($MyInvocation.MyCommand.Definition)

sc.exe stop $ServiceName
Stop-Process -Name $ProxyName