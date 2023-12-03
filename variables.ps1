$Global:ServiceName = "AutoPower"
$Global:Dir = "$($PSScriptRoot)\autopower.exe"
$Global:NotifierPath = "$($PSScriptRoot)\autopower_notification_provider.exe"
$Global:NotifierName = "AutoPower Notification Provider"

function Assert-Admin($script_definition) {
    $User = [Security.Principal.WindowsIdentity]::GetCurrent()
    $CurrentPrincipal = New-Object Security.Principal.WindowsPrincipal($User)
    $IsAdmin = $CurrentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

    if (-not $IsAdmin) {
        Write-Output "Starting a new shell as admin..."
        Start-Process "powershell" -Wait -Verb RunAs -ArgumentList ('-ExecutionPolicy Bypass -noprofile -file "{0}" -elevated' -f ($script_definition))
        exit
    }
}