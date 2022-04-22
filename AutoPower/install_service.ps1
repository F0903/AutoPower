$User = [Security.Principal.WindowsIdentity]::GetCurrent()
$CurrentPrincipal = New-Object Security.Principal.WindowsPrincipal($User)
$IsAdmin = $CurrentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $IsAdmin) {
    Start-Process "powershell" -Verb RunAs -ArgumentList ('-ExecutionPolicy Bypass -noprofile -file "{0}" -elevated' -f ($MyInvocation.MyCommand.Definition))
    exit
}

$ServiceName = 'AutoPower'
$Dir = "$($PSScriptRoot)\AutoPower.exe"

sc.exe create $ServiceName binPath=$Dir start=delayed-auto
sc.exe start $ServiceName
Write-Output "`r`nDone!`r`n"
Pause