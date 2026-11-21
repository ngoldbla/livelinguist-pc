# Build the Windows 11 x64 app + NSIS installer. Run on Windows with
# Rust (MSVC), Bun, Node, and NSIS installed.
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Dist = Join-Path $Root "dist\windows"

if (Test-Path $Dist) { Remove-Item -Recurse -Force $Dist }
New-Item -ItemType Directory -Force -Path $Dist | Out-Null

Write-Host "==> Rust core"
cargo build -p ll-bridge --release
Copy-Item (Join-Path $Root "target\release\livelinguist_core.dll") (Join-Path $Dist "livelinguist-core.node")

Write-Host "==> Bun UI binary"
Push-Location (Join-Path $Root "ui")
bun install --frozen-lockfile
bun build --compile src/main.tsx --outfile (Join-Path $Dist "livelinguist.exe")
Pop-Location

Write-Host "==> assets"
Copy-Item -Recurse (Join-Path $Root "assets") (Join-Path $Dist "assets")
Copy-Item -Recurse (Join-Path $Root "packaging\licenses") (Join-Path $Dist "licenses")
if (Test-Path (Join-Path $Root "backends")) {
  Copy-Item -Recurse (Join-Path $Root "backends") (Join-Path $Dist "backends")
}

Write-Host "==> installer"
Push-Location (Join-Path $Root "packaging\nsis")
if (Get-Command makensis -ErrorAction SilentlyContinue) {
  makensis livelinguist.nsi
} else {
  Write-Warning "makensis not found; install NSIS to produce the installer."
}
Pop-Location

Write-Host "==> portable ZIP"
$Zip = Join-Path $Root "dist\LiveLinguist-0.1.0-windows-x64-portable.zip"
if (Test-Path $Zip) { Remove-Item -Force $Zip }
Compress-Archive -Path (Join-Path $Dist "*") -DestinationPath $Zip
Write-Host "built $Zip"
