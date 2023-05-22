. "$($PSScriptRoot)\variables.ps1"

Assert-Admin($MyInvocation.MyCommand.Definition)

sc.exe create $ServiceName binPath=$Dir start=auto
New-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Run" -Name $NotifierName -Value $NotifierPath -PropertyType "String" -ErrorAction Stop

& "$($PSScriptRoot)\start_service.ps1"

Write-Output "`r`nDone!`r`n"
Pause