# Build the Windows 11 x64 release: app payload, per-user NSIS installer,
# portable ZIP, and SHA256SUMS.txt. Run on Windows with Rust (MSVC), Bun and
# NSIS installed. Release CI (.github/workflows/release.yml) runs exactly this.
#
#   pwsh scripts/build-windows.ps1                # build + self-test + package
#   pwsh scripts/build-windows.ps1 -SkipInstaller # payload + ZIP only
#   pwsh scripts/build-windows.ps1 -Stage payload # build + self-test, stop before packaging
#   pwsh scripts/build-windows.ps1 -Stage package # package an existing payload
#   pwsh scripts/build-windows.ps1 -Stage sums    # rewrite SHA256SUMS.txt only
#
# Release CI uses the stages to sign between them: Authenticode changes bytes,
# so the payload is signed before packaging and the installer after (ADR 0020).
#
# Layout (ADR 0021): livelinguist.exe with gpuix-native.win32-x64-msvc.node and
# livelinguist-core.node beside it. No models (downloaded on first run), no GPU
# backends, no .pdb. Packs and UI locales are embedded in the binaries.
param(
  [switch]$SkipInstaller,
  [ValidateSet("all", "payload", "package", "sums")][string]$Stage = "all"
)

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

$DoPayload = $Stage -eq "all" -or $Stage -eq "payload"
$DoPackage = $Stage -eq "all" -or $Stage -eq "package"

# sha256sum format, so `sha256sum -c` and Get-FileHash both work.
function Write-Sums([string[]]$Files) {
  $Lines = foreach ($File in $Files) {
    $Hash = (Get-FileHash -Algorithm SHA256 $File).Hash.ToLowerInvariant()
    "$Hash  $(Split-Path -Leaf $File)"
  }
  $Lines | Set-Content -Encoding ascii (Join-Path $Out "SHA256SUMS.txt")
  $Lines | Write-Host
}

# Nothing built: rewrite the checksums over the files at hand. Release CI runs
# this after SignPath returns a signed installer, because signing changes bytes.
if ($Stage -eq "sums") {
  $Files = @(
    (Join-Path $Out "LiveLinguist-$Version-setup.exe"),
    (Join-Path $Out "LiveLinguist-$Version-portable.zip")
  ) | Where-Object { Test-Path $_ }
  if (-not $Files) { throw "no release files in $Out; run -Stage package first" }
  Write-Host "==> SHA256SUMS.txt"
  Write-Sums $Files
  exit 0
}

if ($DoPayload) {
  if (Test-Path $Out) { Remove-Item -Recurse -Force $Out }
  New-Item -ItemType Directory -Force -Path $Dist | Out-Null
}
elseif (-not (Test-Path (Join-Path $Dist "livelinguist.exe"))) {
  throw "no payload in $Dist; run -Stage payload first"
}
else {
  New-Item -ItemType Directory -Force -Path $Out | Out-Null
}

if ($DoPayload) {
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

    # Bundled rounded caption face; src/App.tsx looks for fonts\ beside the exe.
    Copy-Item -Recurse (Join-Path $Root "ui\assets\fonts") (Join-Path $Dist "fonts")
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
}

if ($DoPackage) {
  $Bytes = (Get-ChildItem -Recurse -File $Dist | Measure-Object -Property Length -Sum).Sum
  Write-Host ("payload: {0:N1} MB (models excluded)" -f ($Bytes / 1MB))

  $Artifacts = @()

  if (-not $SkipInstaller) {
    Write-Host "==> installer"
    $Makensis = $null
    $OnPath = Get-Command makensis -ErrorAction SilentlyContinue
    if ($OnPath) { $Makensis = $OnPath.Source }
    foreach ($Base in @(${env:ProgramFiles(x86)}, $env:ProgramFiles)) {
      if (-not $Makensis -and $Base -and (Test-Path (Join-Path $Base "NSIS\makensis.exe"))) {
        $Makensis = Join-Path $Base "NSIS\makensis.exe"
      }
    }
    if (-not $Makensis) { throw "makensis not found; install NSIS (or pass -SkipInstaller)" }
    $Setup = Join-Path $Out "LiveLinguist-$Version-setup.exe"
    & $Makensis /V2 /WX "/DVERSION=$Version" "/DNUMVERSION=$NumericVersion" `
      "/DSRCDIR=$Dist" "/DOUTFILE=$Setup" (Join-Path $Root "packaging\nsis\livelinguist.nsi")
    if ($LASTEXITCODE -ne 0) { throw "makensis failed" }
    $Artifacts += $Setup
  }

  Write-Host "==> portable ZIP"
  $Zip = Join-Path $Out "LiveLinguist-$Version-portable.zip"
  Compress-Archive -Path (Join-Path $Dist "*") -DestinationPath $Zip
  $Artifacts += $Zip

  Write-Host "==> SHA256SUMS.txt"
  Write-Sums $Artifacts
}
