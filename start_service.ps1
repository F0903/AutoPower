. "$($PSScriptRoot)\variables.ps1"

Assert-Admin($MyInvocation.MyCommand.Definition)

sc.exe start $ServiceName
Start-Sleep -Milliseconds 500
Start-Process -FilePath $NotifierPath -ErrorAction Stop