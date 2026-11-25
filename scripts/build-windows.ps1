# Build the Windows 11 x64 release: app payload, per-user NSIS installer,
# portable ZIP, and SHA256SUMS.txt. Run on Windows with Rust (MSVC), Bun and
# NSIS installed. Release CI (.github/workflows/release.yml) runs exactly this.
#
#   pwsh scripts/build-windows.ps1                # build + self-test + package
#   pwsh scripts/build-windows.ps1 -SkipInstaller # payload + ZIP only
#
# Layout (ADR 0021): livelinguist.exe with gpuix-native.win32-x64-msvc.node and
# livelinguist-core.node beside it. No models (downloaded on first run), no GPU
# backends, no .pdb. Packs and UI locales are embedded in the binaries.
param([switch]$SkipInstaller)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Out = Join-Path $Root "dist"
$Dist = Join-Path $Out "windows"

# Version: workspace Cargo.toml is the single source (tag must match in CI).
$Cargo = Get-Content (Join-Path $Root "Cargo.toml") -Raw
if ($Cargo -notmatch '(?ms)^\[workspace\.package\].*?^version\s*=\s*"([^"]+)"') {
  throw "workspace.package version not found in Cargo.toml"
}
$Version = $Matches[1]
# Windows file versions are four numbers; drop any pre-release suffix.
$NumericVersion = (($Version -split '-')[0]) + ".0"
Write-Host "==> Live Linguist $Version"

if (Test-Path $Out) { Remove-Item -Recurse -Force $Out }
New-Item -ItemType Directory -Force -Path $Dist | Out-Null

Write-Host "==> Rust core"
# The shipped addon needs real capture, ASR and the simplifier; without these
# features system audio has no captions and clicks never load the GGUF.
cargo build -p ll-bridge --release --features capture,sherpa,llama
if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
Copy-Item (Join-Path $Root "target\release\livelinguist_core.dll") (Join-Path $Dist "livelinguist-core.node")

Write-Host "==> Bun UI binary"
$Icon = Join-Path $Root "packaging\icons\livelinguist.ico"
Push-Location (Join-Path $Root "ui")
try {
  bun install --frozen-lockfile
  if ($LASTEXITCODE -ne 0) { throw "bun install failed" }
  # --windows-hide-console: no black console window behind the app.
  bun build --compile src/main.tsx --outfile (Join-Path $Dist "livelinguist.exe") `
    --windows-hide-console `
    --windows-icon="$Icon" `
    --windows-title="Live Linguist" `
    --windows-publisher="Live Linguist contributors" `
    --windows-description="Live Linguist" `
    --windows-copyright="MIT License" `
    --windows-version="$NumericVersion"
  if ($LASTEXITCODE -ne 0) { throw "bun build --compile failed" }

  # GPUIX native addon beside the exe; src/packaged.ts points the napi loader
  # at it through NAPI_RS_NATIVE_LIBRARY_PATH.
  $GpuixPkg = Join-Path (Get-Location) "node_modules\@gpuix\native-win32-x64-msvc"
  $GpuixNode = Get-ChildItem -Path $GpuixPkg -Filter "*.node" | Select-Object -First 1
  if (-not $GpuixNode) { throw "GPUIX win32-x64 addon not found in $GpuixPkg" }
  Copy-Item $GpuixNode.FullName (Join-Path $Dist "gpuix-native.win32-x64-msvc.node")
} finally {
  Pop-Location
}

Write-Host "==> licenses"
$Licenses = Join-Path $Dist "licenses"
Copy-Item -Recurse (Join-Path $Root "packaging\licenses") $Licenses
Copy-Item (Join-Path $Root "LICENSE") (Join-Path $Licenses "LICENSE.txt")
Copy-Item (Join-Path $Root "NOTICE") (Join-Path $Licenses "NOTICE.txt")

Write-Host "==> self-test (payload)"
# Loads both addons without opening a window; a broken payload stops here.
$Report = Join-Path $Out "self-test-payload.json"
$Proc = Start-Process -FilePath (Join-Path $Dist "livelinguist.exe") `
  -ArgumentList "--self-test=$Report" -Wait -PassThru
if (Test-Path $Report) { Get-Content $Report | Write-Host }
if ($Proc.ExitCode -ne 0) { throw "self-test failed (exit $($Proc.ExitCode))" }

$Bytes = (Get-ChildItem -Recurse -File $Dist | Measure-Object -Property Length -Sum).Sum
Write-Host ("payload: {0:N1} MB (models excluded)" -f ($Bytes / 1MB))

$Artifacts = @()

if (-not $SkipInstaller) {
  Write-Host "==> installer"
  $Makensis = Get-Command makensis -ErrorAction SilentlyContinue
  if (-not $Makensis) {
    $Candidate = Join-Path ${env:ProgramFiles(x86)} "NSIS\makensis.exe"
    if (Test-Path $Candidate) { $Makensis = Get-Item $Candidate }
  }
  if (-not $Makensis) { throw "makensis not found; install NSIS (or pass -SkipInstaller)" }
  $Setup = Join-Path $Out "LiveLinguist-$Version-setup.exe"
  & $Makensis.Source /V2 /WX "/DVERSION=$Version" "/DNUMVERSION=$NumericVersion" `
    "/DSRCDIR=$Dist" "/DOUTFILE=$Setup" (Join-Path $Root "packaging\nsis\livelinguist.nsi")
  if ($LASTEXITCODE -ne 0) { throw "makensis failed" }
  $Artifacts += $Setup
}

Write-Host "==> portable ZIP"
$Zip = Join-Path $Out "LiveLinguist-$Version-portable.zip"
Compress-Archive -Path (Join-Path $Dist "*") -DestinationPath $Zip
$Artifacts += $Zip

Write-Host "==> SHA256SUMS.txt"
# sha256sum format, so `sha256sum -c` and Get-FileHash both work.
$Sums = foreach ($File in $Artifacts) {
  $Hash = (Get-FileHash -Algorithm SHA256 $File).Hash.ToLowerInvariant()
  "$Hash  $(Split-Path -Leaf $File)"
}
$Sums | Set-Content -Encoding ascii (Join-Path $Out "SHA256SUMS.txt")
$Sums | Write-Host
