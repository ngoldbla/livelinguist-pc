# Capture a real GPUIX window (not the offscreen test renderer) as a PNG.
#
# Run this from an interactive desktop session. It launches the real app,
# waits for the window named "Live Linguist - <pid>", and uses PrintWindow with
# PW_RENDERFULLCONTENT so GPU-composited windows capture correctly. Tooling
# only; it never reads or writes transcript content.

[CmdletBinding()]
param(
    [ValidateSet("onboarding", "console", "live", "expanded", "overlay", "settings", "summary", "licenses")]
    [string]$Screen = "overlay",
    [ValidateSet("on_demand", "live")]
    [string]$Mode = "live",
    [string]$Language = "fr-FR",
    [string]$UiLocale = "en",
    [string]$Tab = "",
    [string]$Beat = "",
    [Parameter(Mandatory = $true)][string]$Out,
    [int]$WaitMs = 3500,
    [int]$TimeoutSeconds = 30
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Ui = Join-Path $Root "ui"

Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class LlWindowCapture {
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr hwnd, IntPtr hdc, uint flags);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
}
"@

$env:LL_SCREEN = $Screen
$env:LL_MODE = $Mode
$env:LL_LANGUAGE = $Language
$env:LL_UI_LOCALE = $UiLocale
$env:GPUIX_BACKGROUND = "1"
if ($Tab) { $env:LL_TAB = $Tab } else { Remove-Item Env:LL_TAB -ErrorAction SilentlyContinue }
if ($Beat) { $env:LL_ONBOARDING_BEAT = $Beat } else { Remove-Item Env:LL_ONBOARDING_BEAT -ErrorAction SilentlyContinue }

$stamp = [Guid]::NewGuid().ToString("N").Substring(0, 8)
$stdout = Join-Path $env:TEMP "ll-window-$stamp.out"
$stderr = Join-Path $env:TEMP "ll-window-$stamp.err"
$proc = Start-Process -FilePath (Get-Command bun).Source `
    -ArgumentList "src/main.tsx" `
    -WorkingDirectory $Ui `
    -PassThru `
    -RedirectStandardOutput $stdout `
    -RedirectStandardError $stderr

try {
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        Start-Sleep -Milliseconds 400
        $proc.Refresh()
        if ($proc.MainWindowHandle -ne 0) { break }
    }
    $proc.Refresh()
    if ($proc.MainWindowHandle -eq 0) {
        $detail = (Get-Content $stderr -ErrorAction SilentlyContinue | Select-Object -Last 12) -join " | "
        throw "no GPUIX window for $Screen; stderr: $detail"
    }

    Start-Sleep -Milliseconds $WaitMs
    $rect = New-Object LlWindowCapture+RECT
    [void][LlWindowCapture]::GetWindowRect($proc.MainWindowHandle, [ref]$rect)
    $width = $rect.Right - $rect.Left
    $height = $rect.Bottom - $rect.Top
    if ($width -le 0 -or $height -le 0) { throw "window rect is empty: ${width}x${height}" }

    $bitmap = New-Object System.Drawing.Bitmap $width, $height
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $hdc = $graphics.GetHdc()
    $ok = [LlWindowCapture]::PrintWindow($proc.MainWindowHandle, $hdc, 2)
    $graphics.ReleaseHdc($hdc)
    $graphics.Dispose()
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $Out) | Out-Null
    $bitmap.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
    $bitmap.Dispose()
    if (-not $ok) { throw "PrintWindow failed for $Screen" }
    Write-Host "captured $Screen -> $Out (${width}x${height})"
} finally {
    if (-not $proc.HasExited) { Stop-Process -Id $proc.Id -Force }
    Remove-Item $stdout, $stderr -Force -ErrorAction SilentlyContinue
}
