# Tier C approximation: 4 logical CPUs on the reference host; memory is measured, not limited.
#
# Usage:
#   pwsh scripts\tier-c-profile.ps1 -- <exe> <args...>
# Example:
#   pwsh scripts\tier-c-profile.ps1 -- target\release\ll-eval.exe --scenario click-bench --clicks 50 --language en-US --llm llama --threads 3 --out target\bench-c3
#
# Starts the process, pins it to 4 logical CPUs (affinity 0xF), lowers its
# priority to BelowNormal, samples PeakWorkingSet64 every second, and prints
# `peak_rss_mb=<n>` and `exit_code=<n>` at the end. Content-free: it never
# reads the process output.
$ErrorActionPreference = 'Stop'
# No param() block on purpose: `--`-prefixed eval flags would otherwise bind as
# script parameters when the documented `pwsh scripts\tier-c-profile.ps1 -- ...`
# form is used (pwsh consumes the first `--`, the script still gets `--foo`).
$Command = @($args)
if ($Command.Count -gt 0 -and $Command[0] -eq '--') { $Command = $Command[1..($Command.Count - 1)] }
if ($Command.Count -lt 1) { throw 'usage: tier-c-profile.ps1 -- <exe> <args...>' }
$exe = $Command[0]
$rest = @()
if ($Command.Count -gt 1) { $rest = $Command[1..($Command.Count - 1)] }
$quoted = $rest | ForEach-Object { if ($_ -match '\s') { '"' + $_ + '"' } else { $_ } }
$p = Start-Process -FilePath $exe -ArgumentList $quoted -PassThru -NoNewWindow
$p.ProcessorAffinity = [IntPtr]0xF
$p.PriorityClass = 'BelowNormal'
$peak = 0
while (-not $p.HasExited) {
    $p.Refresh()
    if ($p.PeakWorkingSet64 -gt $peak) { $peak = $p.PeakWorkingSet64 }
    Start-Sleep -Seconds 1
}
$p.WaitForExit()
try { $p.Refresh(); if ($p.PeakWorkingSet64 -gt $peak) { $peak = $p.PeakWorkingSet64 } } catch { }
"peak_rss_mb={0:N0}" -f ($peak / 1MB)
"exit_code=$($p.ExitCode)"
exit $p.ExitCode
