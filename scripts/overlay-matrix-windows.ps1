# W-1 overlay matrix on Windows (real window, real DPI).
#
# Runs in the logged-on desktop. Measures:
#   - window policy per mode (topmost / no-activate / toolwindow / MA_NOACTIVATE);
#   - topmost over a maximized foreground app (Notepad proxy) and focus retention;
#   - unfocused click-through and WindowFromPoint z-order;
#   - 100/125/150% per-monitor DPI via DisplayConfig, with captures;
#   - monitor topology, overlay move/capture per monitor (when >1 exists);
#   - current virtual desktop membership (documented COM interface).
#
# It restores the original DPI scale in a finally block. Dev tooling only; no
# transcript content is read or written.
#
#   pwsh -File scripts/overlay-matrix-windows.ps1
#   pwsh -File scripts/overlay-matrix-windows.ps1 -SkipDpi

[CmdletBinding()]
param(
    [string]$OutDir = "",
    [string]$ResultsPath = "",
    [int[]]$DpiScales = @(100, 125, 150),
    [switch]$SkipDpi,
    [switch]$SkipFocus
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
if (-not $OutDir) { $OutDir = Join-Path $Root "docs/screenshots/m2-windows/overlay" }
if (-not $ResultsPath) { $ResultsPath = Join-Path $Root "docs/qa/2026-11-22-overlay-matrix.json" }

Add-Type -AssemblyName System.Drawing
Add-Type -TypeDefinition (Get-Content (Join-Path $PSScriptRoot "win32/OverlayMatrix.cs") -Raw) -Language CSharp
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

$Monitors = [LlDisplay]::Monitors()
$Primary = $Monitors | Where-Object Primary | Select-Object -First 1
if (-not $Primary) { throw "no primary monitor" }

# Recover from a run that was killed before its finally block restored scaling.
$stateFile = Join-Path $env:TEMP "ll-overlay-matrix-dpi.txt"
if (Test-Path $stateFile) {
    $stale = 0
    if ([int]::TryParse((Get-Content $stateFile -Raw).Trim(), [ref]$stale)) {
        [LlDisplay]::SetScalePercent($Primary.Device, $stale) | Out-Null
        Start-Sleep -Milliseconds 800
        Write-Host "recovered stale DPI scale $stale% from an interrupted run"
    }
    Remove-Item $stateFile -Force -ErrorAction SilentlyContinue
}

$Monitors = [LlDisplay]::Monitors()
$Sources = [LlDisplay]::Sources()
$PrimarySource = $Sources | Where-Object Device -eq $Primary.Device | Select-Object -First 1
if (-not $PrimarySource) { throw "no active display source for $($Primary.Device)" }
$OriginalScale = $PrimarySource.ScalePercent
Set-Content -Path $stateFile -Value $OriginalScale

$result = [ordered]@{
    date = (Get-Date).ToString("yyyy-MM-ddTHH:mm:ssK")
    commit = (git -C $Root rev-parse --short HEAD)
    host = $env:COMPUTERNAME
    monitors = @($Monitors | ForEach-Object {
        [ordered]@{
            device = $_.Device
            description = $_.Description
            rect = "$($_.Left),$($_.Top),$($_.Right),$($_.Bottom)"
            dpi = $_.Dpi
            primary = $_.Primary
        }
    })
    sources = @($Sources | ForEach-Object {
        [ordered]@{ device = $_.Device; scalePercent = $_.ScalePercent }
    })
    originalScalePercent = $OriginalScale
    modes = @()
    dpi = @()
    multiMonitor = [ordered]@{ count = $Monitors.Count; status = "not run" }
    focus = [ordered]@{ status = "not run" }
    drag = [ordered]@{ status = "not run" }
    notes = @()
}

function Wait-Window {
    param([System.Diagnostics.Process]$Process, [int]$TimeoutSeconds = 40)
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        $Process.Refresh()
        if ($Process.HasExited) { throw "app exited with code $($Process.ExitCode)" }
        if ($Process.MainWindowHandle -ne 0) { return $Process.MainWindowHandle }
        Start-Sleep -Milliseconds 300
    }
    throw "no GPUIX window after ${TimeoutSeconds}s"
}

function Start-App {
    param([string]$Screen, [string]$Mode = "on_demand")
    for ($attempt = 0; $attempt -lt 2; $attempt++) {
        $stamp = [Guid]::NewGuid().ToString("N").Substring(0, 8)
        $stdout = Join-Path $env:TEMP "ll-overlay-$stamp.out"
        $stderr = Join-Path $env:TEMP "ll-overlay-$stamp.err"
        $env:LL_SCREEN = $Screen
        $env:LL_MODE = $Mode
        $env:LL_LANGUAGE = "fr-FR"
        $env:LL_UI_LOCALE = "en"
        $env:GPUIX_BACKGROUND = "1"
        $proc = Start-Process -FilePath (Get-Command bun).Source `
            -ArgumentList "src/main.tsx" `
            -WorkingDirectory (Join-Path $Root "ui") `
            -PassThru `
            -RedirectStandardOutput $stdout `
            -RedirectStandardError $stderr
        try {
            $hwnd = Wait-Window -Process $proc
            return [pscustomobject]@{ Process = $proc; Hwnd = $hwnd; Stdout = $stdout; Stderr = $stderr }
        } catch {
            $detail = (Get-Content $stderr -ErrorAction SilentlyContinue | Select-Object -Last 12) -join " | "
            if (-not $proc.HasExited) { Stop-Process -Id $proc.Id -Force }
            Remove-Item $stdout, $stderr -Force -ErrorAction SilentlyContinue
            if ($attempt -eq 0) {
                Write-Host "  $Screen launch failed ($($_.Exception.Message)); retrying once"
                Start-Sleep -Milliseconds 1000
                continue
            }
            throw "$_ ($detail)"
        }
    }
}

function Stop-App {
    param($App)
    if ($App -and -not $App.Process.HasExited) { Stop-Process -Id $App.Process.Id -Force }
    if ($App) { Remove-Item $App.Stdout, $App.Stderr -Force -ErrorAction SilentlyContinue }
}

function Get-Policy {
    param([IntPtr]$Hwnd)
    $ex = [LlWindow]::ExStyle($Hwnd)
    return [ordered]@{
        topmost = (($ex -band [LlWindow]::WS_EX_TOPMOST) -ne 0)
        noActivate = (($ex -band [LlWindow]::WS_EX_NOACTIVATE) -ne 0)
        toolWindow = (($ex -band [LlWindow]::WS_EX_TOOLWINDOW) -ne 0)
        virtualDesktop = [LlWindow]::OnCurrentVirtualDesktop($Hwnd)
        title = [LlWindow]::Title($Hwnd)
    }
}

function Get-Rect {
    param([IntPtr]$Hwnd)
    $rect = New-Object LlWindow+RECT
    if (-not [LlWindow]::TryRect($Hwnd, [ref]$rect)) { throw "GetWindowRect failed" }
    return $rect
}

function Save-Capture {
    param([IntPtr]$Hwnd, [string]$Name)
    $rect = Get-Rect $Hwnd
    $width = $rect.Right - $rect.Left
    $height = $rect.Bottom - $rect.Top
    if ($width -le 0 -or $height -le 0) { throw "empty window rect for $Name" }
    $bitmap = New-Object System.Drawing.Bitmap $width, $height
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $hdc = $graphics.GetHdc()
    $ok = [LlWindow]::Capture($Hwnd, $hdc)
    $graphics.ReleaseHdc($hdc)
    $graphics.Dispose()
    $path = Join-Path $OutDir "$Name.png"
    $bitmap.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
    $bitmap.Dispose()
    if (-not $ok) { throw "PrintWindow failed for $Name" }
    $relative = [System.IO.Path]::GetRelativePath($Root, $path).Replace('\', '/')
    return [pscustomobject]@{ Path = $path; Relative = $relative; Width = $width; Height = $height }
}

function Wait-Dpi {
    param([string]$Device, [uint32]$Expected, [int]$TimeoutSeconds = 8)
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        $monitor = [LlDisplay]::Monitors() | Where-Object Device -eq $Device | Select-Object -First 1
        if ($monitor -and $monitor.Dpi -eq $Expected) { return $true }
        Start-Sleep -Milliseconds 250
    }
    return $false
}

function Wait-OverlayPolicy {
    param([IntPtr]$Hwnd, [int]$TimeoutSeconds = 6)
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        $ex = [LlWindow]::ExStyle($Hwnd)
        if ((($ex -band [LlWindow]::WS_EX_TOPMOST) -ne 0) -and
            (($ex -band [LlWindow]::WS_EX_NOACTIVATE) -ne 0)) {
            return $true
        }
        Start-Sleep -Milliseconds 200
    }
    return $false
}

function Run-DragCheck {
    param([IntPtr]$OverlayHwnd, [string]$Name)
    $before = Get-Rect $OverlayHwnd
    # At 100% the overlay is 796x258 including 8px borders; the four header
    # icons are 22px with 8px gaps, right-aligned inside the 10+14px padding.
    # Scale the grip offset with the captured window so the check also works
    # at 125/150%.
    $scale = ($before.Right - $before.Left) / 796.0
    $x = $before.Right - [int](133 * $scale)
    $y = $before.Top + [int](41 * $scale)
    $sent = [LlWindow]::Drag($x, $y, $x + 120, $y + 80)
    Start-Sleep -Milliseconds 700
    $after = Get-Rect $OverlayHwnd
    $dx = $after.Left - $before.Left
    $dy = $after.Top - $before.Top
    $capture = Save-Capture $OverlayHwnd "$Name-after-drag"
    [LlWindow]::Move($OverlayHwnd, $before.Left, $before.Top, ($before.Right - $before.Left), ($before.Bottom - $before.Top), $true) | Out-Null
    Start-Sleep -Milliseconds 400
    return [ordered]@{
        grip = "$x,$y"
        sent = $sent
        moved = ([Math]::Abs($dx) -gt 40 -or [Math]::Abs($dy) -gt 40)
        delta = "$dx,$dy"
        capture = $capture.Relative
    }
}

function Run-FocusCheck {
    param([IntPtr]$OverlayHwnd, [string]$Name)
    $monitor = [LlDisplay]::Monitors() | Where-Object Primary | Select-Object -First 1
    $proxy = [LlProxyWindow]::Show($monitor.Left, $monitor.Top, $monitor.Right, $monitor.Bottom)
    if ($proxy -eq [IntPtr]::Zero) { throw "proxy window creation failed" }
    try {
        [LlWindow]::ForceForeground($proxy) | Out-Null
        Start-Sleep -Milliseconds 700
        $rect = Get-Rect $OverlayHwnd
        $x = [int](($rect.Left + $rect.Right) / 2)
        $y = [int](($rect.Top + $rect.Bottom) / 2)
        $beforeForeground = [LlWindow]::Foreground()
        $beforeHit = [LlWindow]::At($x, $y)
        $overlayVisible = [LlWindow]::Alive($OverlayHwnd)
        $clicked = [LlWindow]::Click($x, $y)
        Start-Sleep -Milliseconds 700
        $afterForeground = [LlWindow]::Foreground()
        $afterHit = [LlWindow]::At($x, $y)
        $capture = Save-Capture $OverlayHwnd "$Name-over-foreground"
        return [ordered]@{
            proxy = "borderless full-screen window (LlProxyWindow)"
            overlayVisible = $overlayVisible
            overlayAboveApp = ($beforeHit -eq $OverlayHwnd -and $afterHit -eq $OverlayHwnd)
            clickSent = $clicked
            focusRetained = ($afterForeground -eq $proxy)
            foregroundBeforeIsApp = ($beforeForeground -eq $proxy)
            foregroundAfterIsApp = ($afterForeground -eq $proxy)
            capture = $capture.Relative
            captureRect = "$($capture.Width)x$($capture.Height)"
        }
    } finally {
        [LlProxyWindow]::Close($proxy)
    }
}

try {
    if (-not $SkipDpi) {
        foreach ($percent in $DpiScales) {
            if (-not [LlDisplay]::SetScalePercent($Primary.Device, $percent)) {
                $result.dpi += [ordered]@{ scalePercent = $percent; status = "set failed" }
                continue
            }
            $expectedDpi = [uint32]($percent * 96 / 100)
            $applied = Wait-Dpi -Device $Primary.Device -Expected $expectedDpi
            Start-Sleep -Milliseconds 600
            $run = [ordered]@{
                scalePercent = $percent
                expectedDpi = $expectedDpi
                applied = $applied
                overlay = $null
                console = $null
            }

            $overlay = Start-App -Screen "overlay" -Mode "live"
            try {
                $policyReady = Wait-OverlayPolicy $overlay.Hwnd
                if (-not $policyReady) { Write-Host "  overlay policy did not appear within 6s" }
                $policy = Get-Policy $overlay.Hwnd
                $capture = Save-Capture $overlay.Hwnd "overlay-$percent"
                $run.overlay = [ordered]@{
                    policy = $policy
                    rect = "$($capture.Width)x$($capture.Height)"
                    capture = $capture.Relative
                }
                if (-not $SkipFocus -and $percent -eq 100) {
                    $result.drag = Run-DragCheck -OverlayHwnd $overlay.Hwnd -Name "overlay-$percent"
                    $result.focus = Run-FocusCheck -OverlayHwnd $overlay.Hwnd -Name "overlay-$percent"
                }
            } finally {
                Stop-App $overlay
            }

            $console = Start-App -Screen "console"
            try {
                Start-Sleep -Milliseconds 900
                $capture = Save-Capture $console.Hwnd "console-$percent"
                $run.console = [ordered]@{
                    rect = "$($capture.Width)x$($capture.Height)"
                    capture = $capture.Relative
                }
            } finally {
                Stop-App $console
            }

            $result.dpi += $run
        }

        [LlDisplay]::SetScalePercent($Primary.Device, $OriginalScale) | Out-Null
        Wait-Dpi -Device $Primary.Device -Expected ([uint32]($OriginalScale * 96 / 100)) | Out-Null
    }

    if ($Monitors.Count -gt 1) {
        $overlay = Start-App -Screen "overlay" -Mode "live"
        try {
            $index = 0
            $perMonitor = @()
            foreach ($monitor in $Monitors) {
                $rect = Get-Rect $overlay.Hwnd
                $width = $rect.Right - $rect.Left
                $height = $rect.Bottom - $rect.Top
                $x = [int]($monitor.Left + (($monitor.Right - $monitor.Left) - $width) / 2)
                $y = [int]($monitor.Top + (($monitor.Bottom - $monitor.Top) - $height) / 2)
                [LlWindow]::Move($overlay.Hwnd, $x, $y, $width, $height, $true) | Out-Null
                Start-Sleep -Milliseconds 900
                $capture = Save-Capture $overlay.Hwnd "overlay-monitor-$index"
                $perMonitor += [ordered]@{
                    index = $index
                    device = $monitor.Device
                    dpi = $monitor.Dpi
                    capture = $capture.Relative
                }
                $index += 1
            }
            $result.multiMonitor = [ordered]@{ count = $Monitors.Count; status = "passed"; perMonitor = $perMonitor }
        } finally {
            Stop-App $overlay
        }
    } else {
        $result.multiMonitor = [ordered]@{
            count = $Monitors.Count
            status = "blocked"
            reason = "one monitor attached; mixed-DPI/multi-monitor needs two displays"
        }
    }

    foreach ($mode in @(
        @{ name = "console"; screen = "console"; mode = "on_demand" },
        @{ name = "overlay"; screen = "overlay"; mode = "live" },
        @{ name = "expanded"; screen = "expanded"; mode = "live" }
    )) {
        $app = Start-App -Screen $mode.screen -Mode $mode.mode
        try {
            Start-Sleep -Milliseconds 2500
            $result.modes += [ordered]@{
                mode = $mode.name
                policy = (Get-Policy $app.Hwnd)
            }
        } finally {
            Stop-App $app
        }
    }

    $result.notes += "Drag and live virtual-desktop switching are not automated in this run."
    $result.notes += "Focus uses a borderless full-screen Win32 proxy window as the foreground app."
    $result.notes += "WM_MOUSEACTIVATE is verified by the ll-window same-process Win32 test; a cross-process SendMessage is not a valid probe."
} finally {
    [LlDisplay]::SetScalePercent($Primary.Device, $OriginalScale) | Out-Null
    Remove-Item $stateFile -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 500
}

$result | ConvertTo-Json -Depth 10 | Set-Content -Path $ResultsPath -Encoding UTF8
Write-Host "overlay matrix written: $ResultsPath"
Write-Host "captures: $OutDir"
