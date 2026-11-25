[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][int]$ProcessId,
    [Parameter(Mandatory = $true)][ValidateSet("live", "overlay")][string]$Mode,
    [Parameter(Mandatory = $true)][string]$StopFile,
    [Parameter(Mandatory = $true)][string]$ReadyUrl,
    [int]$PollMs = 20,
    [int]$ClickAtMs = 0,
    [int]$ClickEveryN = 0,
    [int]$PlaceX = -1,
    [int]$PlaceY = -1
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class LlBenchMouse {
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int X, int Y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
  public static void Click(int x, int y) {
    SetCursorPos(x, y);
    mouse_event(0x0002, 0, 0, 0, UIntPtr.Zero);
    mouse_event(0x0004, 0, 0, 0, UIntPtr.Zero);
  }
  public static void Move(int x, int y) { SetCursorPos(x, y); }
  public static void Wheel(int delta) {
    mouse_event(0x0800, 0, 0, unchecked((uint)delta), UIntPtr.Zero);
  }
}
public static class LlBenchWindow {
  [DllImport("user32.dll")] public static extern bool SetWindowPos(
    IntPtr hWnd, IntPtr after, int X, int Y, int cx, int cy, uint flags);
}
'@

function Get-AppRoot {
    $process = Get-Process -Id $ProcessId -ErrorAction Stop
    if ($process.MainWindowHandle -eq 0) { return $null }
    return [System.Windows.Automation.AutomationElement]::FromHandle($process.MainWindowHandle)
}

function Find-Button {
    param($Root, [string]$Name)
    $condition = [System.Windows.Automation.AndCondition]::new(
        [System.Windows.Automation.PropertyCondition]::new(
            [System.Windows.Automation.AutomationElement]::NameProperty, $Name
        ),
        [System.Windows.Automation.PropertyCondition]::new(
            [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
            [System.Windows.Automation.ControlType]::Button
        )
    )
    return $Root.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $condition)
}

function Find-ButtonContaining {
    param($Root, [string]$Text)
    $buttons = $Root.FindAll(
        [System.Windows.Automation.TreeScope]::Descendants,
        [System.Windows.Automation.PropertyCondition]::new(
            [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
            [System.Windows.Automation.ControlType]::Button
        )
    )
    foreach ($button in $buttons) {
        if ($button.Current.Name -like "$Text*") { return $button }
    }
    return $null
}

function Find-CaptionButton {
    param($Root, [string]$Text, [switch]$AnyState)
    $buttons = $Root.FindAll(
        [System.Windows.Automation.TreeScope]::Descendants,
        [System.Windows.Automation.PropertyCondition]::new(
            [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
            [System.Windows.Automation.ControlType]::Button
        )
    )
    foreach ($button in $buttons) {
        $name = [string]$button.Current.Name
        if (-not $name.StartsWith($Text)) { continue }
        if (-not $AnyState -and -not $name.Contains("Click to simplify")) { continue }
        return $button
    }
    return $null
}

function Test-ElementInWindow {
    param($Root, $Element)
    $window = $Root.Current.BoundingRectangle
    $rect = $Element.Current.BoundingRectangle
    if ($rect.Width -le 0 -or $rect.Height -le 0) { return $false }
    return (
        $rect.X -ge $window.X -and
        $rect.Y -ge $window.Y -and
        ($rect.X + $rect.Width) -le ($window.X + $window.Width) -and
        ($rect.Y + $rect.Height) -le ($window.Y + $window.Height)
    )
}

function Wait-Button {
    param($Name, [int]$TimeoutMs = 15000)
    $deadline = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds() + $TimeoutMs
    do {
        $root = Get-AppRoot
        if ($null -ne $root) {
            $button = Find-Button -Root $root -Name $Name
            if ($null -ne $button) { return $button }
        }
        Start-Sleep -Milliseconds 50
    } while ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds() -lt $deadline)
    throw "UI button did not appear: $Name"
}

function Click-Element {
    param($Element)
    $rect = $Element.Current.BoundingRectangle
    if ($rect.Width -le 0 -or $rect.Height -le 0) {
        throw "UI button has no bounds"
    }
    [LlBenchMouse]::Click(
        [int]($rect.X + $rect.Width / 2),
        [int]($rect.Y + $rect.Height / 2)
    )
}

function Wait-CaptionScreen {
    param([int]$TimeoutMs = 30000)
    $deadline = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds() + $TimeoutMs
    do {
        $root = Get-AppRoot
        if ($null -ne $root) {
            $names = $root.FindAll(
                [System.Windows.Automation.TreeScope]::Descendants,
                [System.Windows.Automation.Condition]::TrueCondition
            ) | ForEach-Object { $_.Current.Name }
            if ($names -contains "CAPTIONS" -or $names -contains "Listening") { return }
        }
        Start-Sleep -Milliseconds 50
    } while ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds() -lt $deadline)
    throw "caption screen did not become ready"
}

function Set-AppPosition {
    param([int]$X, [int]$Y)
    $process = Get-Process -Id $ProcessId -ErrorAction Stop
    if ($process.MainWindowHandle -eq 0) { return }
    $flags = 0x0001 -bor 0x0004 -bor 0x0010  # NOSIZE | NOZORDER | NOACTIVATE
    [LlBenchWindow]::SetWindowPos(
        $process.MainWindowHandle, [IntPtr]::Zero, $X, $Y, 0, 0, $flags
    ) | Out-Null
}

function Scroll-ListToBottom {
    param($Root)
    $rect = $Root.Current.BoundingRectangle
    if ($rect.Width -le 0 -or $rect.Height -le 0) { return }
    $x = [int]($rect.X + $rect.Width / 2)
    $y = [int]($rect.Y + $rect.Height * 0.55)
    [LlBenchMouse]::Move($x, $y)
    for ($i = 0; $i -lt 30; $i += 1) {
        [LlBenchMouse]::Wheel(-120)
        Start-Sleep -Milliseconds 12
    }
    Start-Sleep -Milliseconds 150
}

try {
    $deadline = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds() + 15000
    do {
        $root = Get-AppRoot
        if ($null -ne $root) { break }
        Start-Sleep -Milliseconds 50
    } while ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds() -lt $deadline)
    if ($null -eq $root) { throw "app window did not appear" }

    Click-Element (Wait-Button -Name "System audio")
    Start-Sleep -Milliseconds 250
    Click-Element (Wait-Button -Name "Start captions")
    Wait-CaptionScreen
    if ($Mode -eq "overlay") {
        Click-Element (Wait-Button -Name "Caption overlay")
        Start-Sleep -Milliseconds 400
    }

    if ($PlaceX -ge 0 -and $PlaceY -ge 0) {
        Set-AppPosition -X $PlaceX -Y $PlaceY
        Start-Sleep -Milliseconds 250
    }

    $process = Get-Process -Id $ProcessId -ErrorAction Stop
    $root = Get-AppRoot
    if ($null -eq $root) { throw "app window did not appear" }
    $rect = $root.Current.BoundingRectangle
    $ready = [ordered]@{
        ready = $true
        title = $process.MainWindowTitle
        rect = [ordered]@{
            x = [int]$rect.X
            y = [int]$rect.Y
            width = [int]$rect.Width
            height = [int]$rect.Height
        }
    }
    [Console]::Out.WriteLine(($ready | ConvertTo-Json -Compress -Depth 4))
    [Console]::Out.Flush()

    $previousPartial = [System.Collections.Generic.HashSet[string]]::new()
    $previousFinal = [System.Collections.Generic.HashSet[string]]::new()
    $allFinals = [System.Collections.Generic.HashSet[string]]::new()
    $readyAt = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
    $clicked = $false
    $lastFinalText = ""
    $finalCount = 0
    $pending = @{}

    while (-not (Test-Path -LiteralPath $StopFile)) {
        $scanStart = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
        try {
            $process = Get-Process -Id $ProcessId -ErrorAction Stop
            if ($process.HasExited -or $process.MainWindowHandle -eq 0) { break }
            $root = [System.Windows.Automation.AutomationElement]::FromHandle($process.MainWindowHandle)
            $elements = $root.FindAll(
                [System.Windows.Automation.TreeScope]::Descendants,
                [System.Windows.Automation.Condition]::TrueCondition
            )
            $texts = [System.Collections.Generic.List[string]]::new()
            $partialFlags = [System.Collections.Generic.List[bool]]::new()
            $pendingPartial = $false
            foreach ($element in $elements) {
                if (
                    $element.Current.ControlType -eq [System.Windows.Automation.ControlType]::Group -and
                    $element.Current.Name -in @("Live partial", "Listening — keep speaking")
                ) {
                    $pendingPartial = $true
                    continue
                }
                if ($element.Current.ControlType -eq [System.Windows.Automation.ControlType]::Text) {
                    $texts.Add([string]$element.Current.Name)
                    $partialFlags.Add($pendingPartial)
                    $pendingPartial = $false
                }
            }
            $scanEnd = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
            $stamp = [long]($scanStart + ($scanEnd - $scanStart) / 2)
            $scanMs = $scanEnd - $scanStart

            $currentPartial = [System.Collections.Generic.HashSet[string]]::new()
            $currentFinal = [System.Collections.Generic.HashSet[string]]::new()
            $newItems = [System.Collections.Generic.List[object]]::new()

            for ($i = 0; $i -lt $texts.Count; $i += 1) {
                $text = $texts[$i]
                if ([string]::IsNullOrWhiteSpace($text) -or $text -eq "Live partial") { continue }
                $next = if ($i + 1 -lt $texts.Count) { $texts[$i + 1] } else { "" }
                $previous = if ($i -gt 0) { $texts[$i - 1] } else { "" }

                if ($partialFlags[$i] -or $next -match "^Listening .*keep speaking$") {
                    [void]$currentPartial.Add($text)
                    if (-not $previousPartial.Contains($text)) {
                        $newItems.Add([ordered]@{
                            t = $stamp; kind = "partial"; text = $text; scan_ms = $scanMs
                        })
                    }
                } elseif ($previous -eq "verbatim" -or $next -match "^Tap a sentence") {
                    [void]$currentFinal.Add($text)
                    $lastFinalText = $text
                    if (-not $previousFinal.Contains($text) -and $allFinals.Add($text)) {
                        $newItems.Add([ordered]@{
                            t = $stamp; kind = "final"; text = $text; scan_ms = $scanMs
                        })
                    }
                }
            }

            foreach ($item in $newItems) {
                [Console]::Out.WriteLine(($item | ConvertTo-Json -Compress))
                [Console]::Out.Flush()
            }

            if ($ClickEveryN -gt 0) {
                $targets = [System.Collections.Generic.List[object]]::new()
                foreach ($item in $newItems) {
                    if ($item.kind -ne "final") { continue }
                    $index = $finalCount
                    $finalCount += 1
                    if (($index % $ClickEveryN) -eq 0) {
                        $targets.Add([ordered]@{ index = $index; text = [string]$item.text })
                    }
                }
                foreach ($target in $targets) {
                    $targetRoot = Get-AppRoot
                    if ($null -eq $targetRoot) { break }
                    if ($Mode -eq "live") {
                        Scroll-ListToBottom -Root $targetRoot
                        $targetRoot = Get-AppRoot
                    }
                    $button = if ($null -ne $targetRoot) {
                        Find-CaptionButton -Root $targetRoot -Text $target.text
                    } else {
                        $null
                    }
                    if ($null -eq $button -or -not (Test-ElementInWindow -Root $targetRoot -Element $button)) {
                        [Console]::Out.WriteLine((
                            [ordered]@{
                                t = $stamp; kind = "click"; index = $target.index; ok = $false
                                in_window = $false; scan_ms = $scanMs
                            } | ConvertTo-Json -Compress
                        ))
                        [Console]::Out.Flush()
                        continue
                    }
                    Click-Element $button
                    $clickedAt = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
                    $pending[$target.index] = @{ text = $target.text; t = $clickedAt }
                    [Console]::Out.WriteLine((
                        [ordered]@{
                            t = $clickedAt; kind = "click"; index = $target.index; ok = $true
                            in_window = $true; scan_ms = $scanMs
                        } | ConvertTo-Json -Compress
                    ))
                    [Console]::Out.Flush()
                }
            }

            foreach ($index in @($pending.Keys)) {
                $entry = $pending[$index]
                $button = Find-CaptionButton -Root $root -Text $entry.text -AnyState
                $name = if ($null -ne $button) { [string]$button.Current.Name } else { "" }
                if ($name.Contains("Simplified:")) {
                    [Console]::Out.WriteLine((
                        [ordered]@{
                            t = $stamp; kind = "simplified"; index = $index; scan_ms = $scanMs
                        } | ConvertTo-Json -Compress
                    ))
                    [Console]::Out.Flush()
                    $pending.Remove($index)
                    if ($Mode -eq "live") { Scroll-ListToBottom -Root $root }
                } elseif (($stamp - $entry.t) -gt 20000) {
                    [Console]::Out.WriteLine((
                        [ordered]@{
                            t = $stamp; kind = "click_timeout"; index = $index; scan_ms = $scanMs
                        } | ConvertTo-Json -Compress
                    ))
                    [Console]::Out.Flush()
                    $pending.Remove($index)
                }
            }

            if (
                $ClickEveryN -eq 0 -and
                -not $clicked -and
                $ClickAtMs -gt 0 -and
                $lastFinalText -ne "" -and
                ($stamp - $readyAt) -ge $ClickAtMs
            ) {
                $button = if ($Mode -eq "live") {
                    Find-ButtonContaining -Root $root -Text $lastFinalText
                } else {
                    Find-Button -Root $root -Name $lastFinalText
                }
                if ($null -ne $button) {
                    Click-Element $button
                    $clicked = $true
                    [Console]::Out.WriteLine((
                        [ordered]@{ t = $stamp; kind = "click"; ok = $true; scan_ms = $scanMs } |
                        ConvertTo-Json -Compress
                    ))
                    [Console]::Out.Flush()
                }
            }
            $previousPartial = $currentPartial
            $previousFinal = $currentFinal
        } catch {
            if ($_.Exception.Message -match "process|window|handle") { break }
        }
        Start-Sleep -Milliseconds $PollMs
    }
} catch {
    [Console]::Error.WriteLine($_.Exception.Message)
    [Console]::Out.WriteLine('{"error":"ui-probe-failed"}')
    exit 1
}
