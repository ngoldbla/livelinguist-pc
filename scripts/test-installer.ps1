# Release gate: install the built setup.exe silently as a normal user, check
# files/shortcuts/registry, run the installed app's --self-test, upgrade over
# itself, uninstall, and check that the downloaded models are kept by default.
# Also self-tests the portable ZIP. Run after scripts/build-windows.ps1.
#
#   pwsh scripts/test-installer.ps1 -Version 1.0.0
param([Parameter(Mandatory = $true)][string]$Version)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Out = Join-Path $Root "dist"
$Setup = Join-Path $Out "LiveLinguist-$Version-setup.exe"
$Zip = Join-Path $Out "LiveLinguist-$Version-portable.zip"
$Dir = Join-Path $env:LOCALAPPDATA "Programs\LiveLinguist"
$Models = Join-Path $env:LOCALAPPDATA "LiveLinguist\models"
$UninstKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\LiveLinguist"
$StartLnk = Join-Path ([Environment]::GetFolderPath("Programs")) "Live Linguist.lnk"
$DesktopLnk = Join-Path ([Environment]::GetFolderPath("Desktop")) "Live Linguist.lnk"
$Files = @("livelinguist.exe", "livelinguist-core.node", "gpuix-native.win32-x64-msvc.node",
  "uninstall.exe", "licenses\LICENSE.txt", "licenses\THIRD_PARTY.md")

function Check([bool]$Condition, [string]$What) {
  if (-not $Condition) { throw "FAIL: $What" }
  Write-Host "ok: $What"
}

function Run([string]$Exe, [string[]]$Arguments, [string]$What) {
  $Proc = Start-Process -FilePath $Exe -ArgumentList $Arguments -Wait -PassThru
  Check ($Proc.ExitCode -eq 0) "$What (exit $($Proc.ExitCode))"
}

function SelfTest([string]$Exe, [string]$What) {
  $Report = Join-Path $Out "self-test-$([guid]::NewGuid()).json"
  $Proc = Start-Process -FilePath $Exe -ArgumentList "--self-test=$Report" -Wait -PassThru
  if (Test-Path $Report) { Get-Content $Report | Write-Host }
  Check ($Proc.ExitCode -eq 0) "$What self-test"
}

Check (Test-Path $Setup) "installer exists: $Setup"
Check (-not (Test-Path (Join-Path $Dir "livelinguist.exe"))) "no previous install"

# 1. Fresh silent install, no elevation.
Run $Setup @("/S") "silent install"
foreach ($File in $Files) { Check (Test-Path (Join-Path $Dir $File)) "installed $File" }
Check (Test-Path $StartLnk) "Start menu shortcut"
Check (Test-Path $DesktopLnk) "desktop shortcut"
$Reg = Get-ItemProperty $UninstKey
Check ($Reg.DisplayVersion -eq $Version) "Installed apps entry shows $Version"
Check ($Reg.EstimatedSize -gt 0) "Installed apps entry has a size"
SelfTest (Join-Path $Dir "livelinguist.exe") "installed app"

# 2. Upgrade over the existing install (same version is enough to exercise it).
New-Item -ItemType Directory -Force -Path $Models | Out-Null
Set-Content -Path (Join-Path $Models "keep.txt") -Value "placeholder"
Run $Setup @("/S") "silent upgrade over existing install"
SelfTest (Join-Path $Dir "livelinguist.exe") "upgraded app"

# 3. Silent uninstall. NSIS re-launches the uninstaller from %TEMP% and
#    returns at once, so wait for the files to go.
Run (Join-Path $Dir "uninstall.exe") @("/S") "silent uninstall"
$Deadline = (Get-Date).AddSeconds(60)
while ((Test-Path (Join-Path $Dir "livelinguist.exe")) -and (Get-Date) -lt $Deadline) {
  Start-Sleep -Milliseconds 500
}
Start-Sleep -Seconds 2
foreach ($File in $Files) { Check (-not (Test-Path (Join-Path $Dir $File))) "removed $File" }
Check (-not (Test-Path $StartLnk)) "Start menu shortcut removed"
Check (-not (Test-Path $DesktopLnk)) "desktop shortcut removed"
Check (-not (Test-Path $UninstKey)) "Installed apps entry removed"
Check (Test-Path (Join-Path $Models "keep.txt")) "downloaded models kept by default"
Remove-Item -Recurse -Force (Join-Path $env:LOCALAPPDATA "LiveLinguist")

# 4. Portable ZIP runs from any folder.
$Portable = Join-Path $Out "portable-test"
Expand-Archive -Path $Zip -DestinationPath $Portable -Force
SelfTest (Join-Path $Portable "livelinguist.exe") "portable app"
Remove-Item -Recurse -Force $Portable

Write-Host "installer test: PASS"
