# Run a command in the logged-on user's interactive desktop session.
#
# The GPU path needs a real desktop: DXGI adapter enumeration works in session 0
# (services), but GPUIX cannot create its DirectX window there. On a dev VM
# whose agent runs in session 0, this creates a temporary interactive scheduled
# task, runs the command once, streams its log, and deletes the task.
#
# Requires an elevated PowerShell and a logged-on user with the same account.
# Tooling only; the product never does this.

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$FilePath,
    [string]$ArgumentString = "",
    [string]$WorkingDirectory = (Get-Location).Path,
    [int]$TimeoutSeconds = 240
)

$ErrorActionPreference = "Stop"

$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
    [Security.Principal.WindowsBuiltInRole]::Administrator
)
if (-not $isAdmin) {
    throw "run-in-interactive-session.ps1 needs an elevated PowerShell (to create the temporary task)."
}
if ((Get-Process -Id $PID).SessionId -ne 0) {
    Write-Host "Already in an interactive session; running directly."
    Push-Location $WorkingDirectory
    try {
        $direct = "'" + ($FilePath -replace "'", "''") + "' " + $ArgumentString
        Invoke-Expression $direct
        exit $LASTEXITCODE
    } finally {
        Pop-Location
    }
}

$stamp = [Guid]::NewGuid().ToString("N").Substring(0, 8)
$taskName = "LL-Interactive-$stamp"
$log = Join-Path $env:TEMP "ll-interactive-$stamp.log"
$exitFile = Join-Path $env:TEMP "ll-interactive-$stamp.exit"
$wrapper = Join-Path $env:TEMP "ll-interactive-$stamp.ps1"
$pwsh = (Get-Command pwsh).Source

$argList = ($Arguments | ForEach-Object { "'" + ($_ -replace "'", "''") + "'" }) -join ", "
$escapedDir = $WorkingDirectory -replace "'", "''"
$escapedFile = $FilePath -replace "'", "''"
@"
`$ErrorActionPreference = "Continue"
Set-Location '$escapedDir'
& '$escapedFile' $ArgumentString *>&1 | Tee-Object -FilePath '$log'
`$code = `$LASTEXITCODE
"exit=`$code" | Add-Content '$log'
Set-Content -Path '$exitFile' -Value `$code
"@ | Set-Content -Path $wrapper -Encoding UTF8

schtasks /create /tn $taskName /tr "`"$pwsh`" -NoProfile -ExecutionPolicy Bypass -File `"$wrapper`"" /sc once /st 23:59 /it /ru "$env:USERNAME" /f | Out-Null
try {
    schtasks /run /tn $taskName | Out-Null
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while (-not (Test-Path $exitFile) -and (Get-Date) -lt $deadline) {
        Start-Sleep -Milliseconds 500
    }
    if (Test-Path $log) {
        Get-Content $log
    }
    if (-not (Test-Path $exitFile)) {
        throw "interactive command timed out after ${TimeoutSeconds}s; see $log"
    }
    $code = [int](Get-Content $exitFile)
    if ($code -ne 0) {
        throw "interactive command failed with exit code $code; see $log"
    }
} finally {
    schtasks /delete /tn $taskName /f 2>$null | Out-Null
    Remove-Item $exitFile, $wrapper -Force -ErrorAction SilentlyContinue
}
