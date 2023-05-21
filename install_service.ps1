. "$($PSScriptRoot)\variables.ps1"

Assert-Admin($MyInvocation.MyCommand.Definition)

# Some of these are a shot in the dark...
# WpnService IS REQUIRED
sc.exe create $ServiceName binPath=$Dir start=auto depend=LanmanServer/LanmanWorkstation/LSM/Power/SessionEnv/DcomLaunch/WpnService
New-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Run" -Name $NotifierName -Value $NotifierPath -PropertyType "String" -ErrorAction Stop

& "$($PSScriptRoot)\start_service.ps1"

Write-Output "`r`nDone!`r`n"
Pause