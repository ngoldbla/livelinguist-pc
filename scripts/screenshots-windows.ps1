# Rebuild the Windows UI screenshot portfolio.
#
# Run from an interactive desktop session (see run-in-interactive-session.ps1):
#   pwsh -File scripts/run-in-interactive-session.ps1 `
#     -FilePath scripts/screenshots-windows.ps1 -WorkingDirectory .
#
# Screens other than the overlay use the GPUIX GPU test renderer (offscreen,
# deterministic). The overlay keeps its real 780x250 window form factor and is
# captured from the running window with PrintWindow.

[CmdletBinding()]
param([switch]$SkipOffscreen, [switch]$SkipOverlay)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Out = Join-Path $Root "docs/screenshots/m2-windows"
$bun = (Get-Command bun).Source

if (-not $SkipOffscreen) {
    Push-Location (Join-Path $Root "ui")
    try {
        & $bun run scripts/screenshots-windows.tsx --skip=overlay
        if ($LASTEXITCODE -ne 0) {
            throw "offscreen screenshot run failed with exit code $LASTEXITCODE"
        }
    } finally {
        Pop-Location
    }
}

if (-not $SkipOverlay) {
    & pwsh -NoProfile -File (Join-Path $PSScriptRoot "capture-gpui-window.ps1") `
        -Screen overlay -Mode live -Out (Join-Path $Out "overlay.png")
    if ($LASTEXITCODE -ne 0) {
        throw "overlay capture failed with exit code $LASTEXITCODE"
    }
}

Write-Host "Windows screenshot portfolio complete: $Out"
