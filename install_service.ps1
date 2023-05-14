$User = [Security.Principal.WindowsIdentity]::GetCurrent()
$CurrentPrincipal = New-Object Security.Principal.WindowsPrincipal($User)
$IsAdmin = $CurrentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $IsAdmin) {
    Write-Output "Starting a new shell as admin..."
    Start-Process "powershell" -Wait -Verb RunAs -ArgumentList ('-ExecutionPolicy Bypass -noprofile -file "{0}" -elevated' -f ($MyInvocation.MyCommand.Definition))
    exit
}

$ServiceName = 'AutoPower'
$Dir = "$($PSScriptRoot)\auto_power.exe"

sc.exe create $ServiceName binPath=$Dir start=delayed-auto
sc.exe start $ServiceName
Write-Output "`r`nDone!`r`n"
Pause