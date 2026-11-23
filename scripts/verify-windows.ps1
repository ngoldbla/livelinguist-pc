# Full local verification for native Windows 11 x64.
# Mirrors scripts/verify.sh. See docs/qa/2026-11-22-windows-vm-smoke.md.
[CmdletBinding()]
param(
    [switch]$SkipUi,
    [switch]$SkipEval,
    [switch]$SkipBridge
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

function Resolve-Tool {
    param([Parameter(Mandatory = $true)][string]$Name)
    $command = Get-Command $Name -ErrorAction SilentlyContinue
    if (-not $command) {
        throw "$Name is not on PATH. Open a new terminal after installation; setup is recorded in docs/qa/2026-11-22-windows-vm-smoke.md."
    }
    return $command.Source
}

function Invoke-Native {
    param(
        [Parameter(Mandatory = $true)][string]$FilePath,
        [Parameter(Mandatory = $true)][string[]]$Arguments
    )
    & $FilePath @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "$FilePath $($Arguments -join ' ') failed with exit code $LASTEXITCODE"
    }
}

$cargo = Resolve-Tool cargo
$rustc = Resolve-Tool rustc
$bun = Resolve-Tool bun
$node = Resolve-Tool node

Write-Host "==> toolchain"
Invoke-Native $rustc @('--version')
Invoke-Native $cargo @('--version')
Invoke-Native $bun @('--version')
Invoke-Native $node @('--version')

$cmake = Get-Command cmake -ErrorAction SilentlyContinue
if ($cmake) {
    Invoke-Native $cmake.Source @('--version')
}
$makensis = Get-Command makensis -ErrorAction SilentlyContinue
if ($makensis) {
    Write-Host "makensis: $($makensis.Source)"
} else {
    Write-Host "makensis: not installed (packaging only; W-6)"
}

Write-Host "==> rust format"
Invoke-Native $cargo @('fmt', '--all', '--', '--check')

Write-Host "==> rust clippy"
Invoke-Native $cargo @('clippy', '--workspace', '--all-targets', '--', '-D', 'warnings')

Write-Host "==> rust tests"
Invoke-Native $cargo @('test', '--workspace')

if (-not $SkipEval) {
    Write-Host "==> eval: mock / wav / simplify-only"
    Remove-Item target\verify-eval -Recurse -Force -ErrorAction SilentlyContinue
    Invoke-Native $cargo @(
        'run', '-q', '-p', 'll-eval', '--',
        '--scenario', 'mock', '--language', 'fr-FR',
        '--out', 'target\verify-eval\mock'
    )
    Invoke-Native $cargo @(
        'run', '-q', '-p', 'll-eval', '--',
        '--scenario', 'mock', '--language', 'fr-FR',
        '--wav', 'tests\fixtures\sample.wav',
        '--out', 'target\verify-eval\wav'
    )
    Invoke-Native $cargo @(
        'run', '-q', '-p', 'll-eval', '--',
        '--scenario', 'simplify-only', '--language', 'fr-FR',
        '--text', "Il faudrait que tu viennes nous rendre visite dès que l'occasion se présentera.",
        '--out', 'target\verify-eval\text'
    )
}

if (-not $SkipUi) {
    Write-Host "==> UI install / typecheck / tests / Biome"
    Push-Location ui
    try {
        Invoke-Native $bun @('install', '--frozen-lockfile')
        Invoke-Native $bun @('run', 'typecheck')
        Invoke-Native $bun @('test')
        Invoke-Native $bun @('run', 'check')
    } finally {
        Pop-Location
    }
}

if (-not $SkipBridge) {
    Write-Host "==> bridge build + smoke + 10 s delivery"
    Invoke-Native $cargo @('build', '-p', 'll-bridge', '--release')
    Copy-Item target\release\livelinguist_core.dll tests\bridge\livelinguist-core.node -Force
    Copy-Item target\release\livelinguist_core.dll ui\livelinguist-core.node -Force
    Invoke-Native $node @('tests\bridge\bridge-smoke.mjs')
    $env:LL_BRIDGE_ADDON = (Resolve-Path 'tests\bridge\livelinguist-core.node').Path
    try {
        Invoke-Native $node @('tests\bridge\hardware-probe.mjs')
    } finally {
        Remove-Item Env:LL_BRIDGE_ADDON -ErrorAction SilentlyContinue
    }
    Invoke-Native $node @('tests\bridge\event-delivery.mjs', '10')
    Invoke-Native $bun @('tests\bridge\event-delivery.mjs', '10')
}

Write-Host "==> pass"
