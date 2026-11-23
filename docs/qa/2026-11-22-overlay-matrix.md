# Win32 overlay matrix (Windows)

- Date: 2026-11-22
- Status: **single-monitor matrix passes at 100/125/150%; mixed-DPI and
  multi-monitor are blocked on this host (one display).**
- Branch: `stage-w5-gpu-evidence`
- Plan: `docs/plans/2026-11-21-windows-resume-plan.md` (W-1)
- Machine: `docs/qa/2026-11-21-windows-hardware.md`
- Raw results: `docs/qa/2026-11-22-overlay-matrix.json`
- Captures: `docs/screenshots/m2-windows/overlay/`

## Host

| Item | Value |
|---|---|
| OS | Windows 11 Pro 10.0.26300 x64 |
| GPU | NVIDIA GeForce RTX 5060 Ti, driver 32.0.16.2124 |
| Display | `\\.\DISPLAY3`, 2560x1080, 100% (96 DPI), one monitor |
| Session | interactive desktop (session 1) |
| Vulkan | not involved (no ICD) |

## Command

```powershell
pwsh -File scripts/overlay-matrix-windows.ps1
```

The script sets per-monitor scaling through `DisplayConfigGet/SetDeviceInfo`
(`DISPLAYCONFIG_DEVICE_INFO_GET/SET_DPI_SCALE`), restarts the real GPUIX app for
each scale, captures the window with `PrintWindow(PW_RENDERFULLCONTENT)`, and
restores the original scale in a `finally` block. It never reads transcript
content.

## Per-DPI results

| Scale | Source scale | Effective DPI | Overlay capture | Console capture | Policy |
|---|---|---|---|---|---|
| 100% | 100% | 96 | 796x258 | 1196x828 | topmost, no-activate, toolwindow, MA_NOACTIVATE |
| 125% | 125% | 120 | 993x322 | 1493x1034 | topmost, no-activate, toolwindow, MA_NOACTIVATE |
| 150% | 150% | 144 | 1192x386 | 1792x1106 | topmost, no-activate, toolwindow, MA_NOACTIVATE |

The overlay scales cleanly and keeps its form factor. At 150% on a 1080p
display the console window is 1106 px tall, taller than the screen; the bottom
of the console is clipped. That is a layout finding for W-5, not an overlay
regression.

Files: `overlay-100.png`, `overlay-125.png`, `overlay-150.png`,
`console-100.png`, `console-125.png`, `console-150.png`.

## Mode policy

| Mode | Topmost | No-activate | Toolwindow | Notes |
|---|---|---|---|---|
| console | no | no | no | decorated, taskbar visible |
| overlay | yes | yes | yes | subclass answers `WM_MOUSEACTIVATE` with `MA_NOACTIVATE` |
| expanded | yes | no | yes | keyboard focus allowed |

The same-process Win32 integration test
(`ll_window::win32::tests::overlay_window_sets_styles_and_answers_mouse_activate`)
also verifies that leaving overlay mode removes the subclass and clears
`WS_EX_NOACTIVATE`, and that console clears topmost/toolwindow. A cross-process
`SendMessage(WM_MOUSEACTIVATE)` is not a valid probe; the real click test below
is authoritative.

## Focus retention and unfocused click

- Proxy: a borderless, non-topmost full-screen Win32 window ("video call")
  created in the test process.
- Before the click: the proxy is foreground; the overlay is above it.
- Click: a real `SendInput` click at the overlay center while the proxy is
  foreground.
- After the click: the proxy is still foreground, the overlay is still above
  it, and the overlay is visible.

Result (100%): `overlayVisible=true`, `overlayAboveApp=true`,
`foregroundBeforeIsApp=true`, `foregroundAfterIsApp=true`,
`focusRetained=true`. Capture: `overlay-100-over-foreground.png`.

## Drag

- Grip: added to the overlay header (`aria-label="Move window"`); on mouse-down
  the bridge starts a custom topmost drag loop that follows the cursor without
  activating the window. `DefWindowProc(WM_NCLBUTTONDOWN)` was not usable
  because a `WS_EX_NOACTIVATE` window does not get the activation the modal move
  loop expects.
- Test: press the grip, move 120x80, release.
- Result: the window moved exactly `+120,+80` (`drag.moved=true`). Capture:
  `overlay-100-after-drag.png`.

## Virtual desktops

`IVirtualDesktopManager::IsWindowOnCurrentVirtualDesktop` reports `true` for the
overlay in every run. Live desktop switching was **not** exercised; doing so
would change the user's desktop. The "all workspaces" goal in ADR 0008 is not
yet proven on Windows.

## Multi-monitor and mixed DPI

**Blocked on this host:** one 2560x1080 monitor. Windows cannot present two
monitors with different scales without a second display (or a virtual/indirect
display driver). No fake mixed-DPI result is recorded.

The harness already:

- enumerates monitors and active sources (`EnumDisplayMonitors`,
  `QueryDisplayConfig`, `GetDpiForMonitor`);
- moves the overlay to each monitor and captures per monitor when the count is
  greater than one;
- records each source's scale, so a mixed-DPI host produces a per-monitor table.

To close the row, attach a second display (or a monitor in picture-by-picture
mode with two active inputs) and rerun the command. Expected new files:
`overlay-monitor-0.png`, `overlay-monitor-1.png`, and `multiMonitor.status =
"passed"`.

## Non-claims

- No mixed-DPI or multi-monitor result: not available on this host.
- No virtual-desktop switch result: not exercised.
- No overlay-over-exclusive-fullscreen-video result: the proxy is a borderless
  full-screen window, not a hardware overlay/exclusive-mode video surface.
- No frame-pacing or 60 fps claim.
- The captures are one monitor at a time at 100/125/150%, not two monitors at
  different scales simultaneously.

## Reproduce

```powershell
cargo test -p ll-window --lib
cargo build -p ll-bridge --release
Copy-Item target\release\livelinguist_core.dll ui\livelinguist-core.node -Force
pwsh -File scripts/overlay-matrix-windows.ps1
```
