# Privacy check, Windows (T35). Linux dev version: scripts/privacy-check.sh.
#
# Plays a known speech clip through the speakers while ll-eval captures system
# audio with the real models (sherpa ASR, llama simplifier, every segment
# clicked). Then checks:
#   1. no reference word appears in any output file, the run log, or the app
#      log directory, except the content-bearing exports (transcript.md,
#      segments.csv, simplification.txt, hyp.txt);
#   2. no network: Get-NetTCPConnection for the ll-eval PID every 500 ms; any
#      remote address fails the check (models are already downloaded);
#   3. not vacuous: the transcript export contains the words.
# Prints `privacy: PASS` or the failing file/connection. It never prints the
# matched words.
#
# Usage: pwsh scripts\privacy-check.ps1 [-Clip target\asr-smoke\en.wav] [-Reference target\asr-smoke\en.ref.txt] [-NoBuild]
param(
    [string]$Clip = 'target\asr-smoke\en.wav',
    [string]$Reference = 'target\asr-smoke\en.ref.txt',
    [switch]$NoBuild
)
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path -Parent $PSScriptRoot)
if (-not (Test-Path $Clip) -or -not (Test-Path $Reference)) { 'privacy: FAIL (clip or reference missing)'; exit 2 }
if (-not $NoBuild) { cargo build -q --release -p ll-eval --features capture,sherpa,llama; if ($LASTEXITCODE) { exit 2 } }
$out = 'target\privacy'
$runlog = 'target\privacy-run.log'
Remove-Item -Recurse -Force $out, $runlog -ErrorAction SilentlyContinue
$contentBearing = @('transcript.md', 'segments.csv', 'simplification.txt', 'hyp.txt')
$words = (Get-Content -Raw $Reference).ToLower().Split() |
    ForEach-Object { $_.Trim('.,;:!?"''') } |
    Where-Object { $_ -match '^\p{L}{6,}$' } | Sort-Object -Unique |
    Sort-Object Length -Descending | Select-Object -First 5
if ($words.Count -lt 3) { 'privacy: FAIL (reference has too few distinctive words)'; exit 1 }

$args = @('--scenario', 'mock', '--language', 'en-US', '--source', 'loopback', '--seconds', '25',
          '--asr', 'sherpa', '--llm', 'llama', '--click-all', '--out', $out)
$p = Start-Process -FilePath 'target\release\ll-eval.exe' -ArgumentList $args -PassThru -NoNewWindow `
    -RedirectStandardOutput $runlog -RedirectStandardError "$runlog.err"
Start-Sleep -Seconds 2
$player = New-Object System.Media.SoundPlayer (Resolve-Path $Clip)
$player.Play()
$failures = @()
while (-not $p.HasExited) {
    $remote = Get-NetTCPConnection -OwningProcess $p.Id -ErrorAction SilentlyContinue |
        Where-Object { $_.RemoteAddress -and $_.RemoteAddress -notin @('0.0.0.0', '::', '127.0.0.1', '::1') }
    foreach ($c in $remote) { $failures += "network: connection to $($c.RemoteAddress):$($c.RemotePort)" }
    Start-Sleep -Milliseconds 500
}
$player.Stop()
if ($p.ExitCode -ne 0) { "privacy: FAIL (ll-eval exited $($p.ExitCode); see $runlog)"; exit 1 }

$scan = @(Get-ChildItem -Recurse -File $out) + @(Get-Item $runlog, "$runlog.err")
$appLogs = Join-Path $env:LOCALAPPDATA 'LiveLinguist\logs'
if (Test-Path $appLogs) { $scan += Get-ChildItem -Recurse -File $appLogs }
$found = 0
foreach ($file in $scan) {
    $text = (Get-Content -Raw $file.FullName -ErrorAction SilentlyContinue)
    if (-not $text) { continue }
    $text = $text.ToLower()
    for ($i = 0; $i -lt $words.Count; $i++) {
        if ($text -match ('\b' + [regex]::Escape($words[$i]) + '\b')) {
            if ($contentBearing -contains $file.Name) { $found++ } else { $failures += "$($file.Name): reference word #$($i + 1)" }
        }
    }
}
if ($found -eq 0) { $failures += 'vacuous: the transcript export has none of the reference words' }
if ($failures.Count) { $failures | Sort-Object -Unique | ForEach-Object { "privacy: FAIL $_" }; exit 1 }
'privacy: PASS'
