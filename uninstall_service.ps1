. "$($PSScriptRoot)\variables.ps1"

Assert-Admin($MyInvocation.MyCommand.Definition)

& "$($PSScriptRoot)\stop_service.ps1"

sc.exe delete $ServiceName

Remove-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Run" -Name "AutoPower Notification Provider"

Remove-Item -LiteralPath "$env:TEMP\autopower" -Force -Recurse
Write-Output "Deleted autopower log directory."

Write-Output "`r`n$($ServiceName) has now been deleted. In some cases it is neccessary to restart your PC to complete the removal. Afterwards you can delete this directory.`r`n"
Pause