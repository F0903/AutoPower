. "$($PSScriptRoot)\variables.ps1"

Assert-Admin($MyInvocation.MyCommand.Definition)

sc.exe start $ServiceName
Start-Process -FilePath $ProxyPath -ErrorAction Stop