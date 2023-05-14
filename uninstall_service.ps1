$User = [Security.Principal.WindowsIdentity]::GetCurrent()
$CurrentPrincipal = New-Object Security.Principal.WindowsPrincipal($User)
$IsAdmin = $CurrentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $IsAdmin) {
    Write-Output "Starting a new shell as admin..."
    Start-Process "powershell" -Wait -Verb RunAs -ArgumentList ('-ExecutionPolicy Bypass -noprofile -file "{0}" -elevated' -f ($MyInvocation.MyCommand.Definition))
    exit
}

$ServiceName = 'AutoPower'
sc.exe stop $ServiceName
sc.exe delete $ServiceName
Write-Output "`r`n$($ServiceName) has now been deleted. In some cases it is neccessary to restart your PC to complete the removal. Afterwards you can delete this directory.`r`n"
Pause