$Global:ServiceName = "AutoPower"
$Global:Dir = "$($PSScriptRoot)\autopower.exe"
$Global:ProxyPath = "$($PSScriptRoot)\autopower_proxy.exe"
$Global:ProxyName = "AutoPower Proxy"

function Assert-Admin($script_definition) {
    $User = [Security.Principal.WindowsIdentity]::GetCurrent()
    $CurrentPrincipal = New-Object Security.Principal.WindowsPrincipal($User)
    $IsAdmin = $CurrentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

    if (-not $IsAdmin) {
        Write-Output "Starting a new shell as admin..."
        Write-Output "Sometimes this window will stay open even after install is finished."
        Start-Process "powershell" -Wait -Verb RunAs -ArgumentList ('-ExecutionPolicy Bypass -noprofile -file "{0}" -elevated' -f ($script_definition))
        exit
    }
}