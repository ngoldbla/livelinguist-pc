# Windows screenshots

Portfolio rebuilt 2026-11-22 on the Windows 11 host with the GPUIX 0.11 GPU
test renderer (DirectX). Evidence and non-claims:
`docs/qa/2026-11-22-gpu-renderer.md`.

| File | Path |
|---|---|
| `onboarding-welcome.png` | GPUIX GPU test renderer |
| `onboarding-language.png` | GPUIX GPU test renderer |
| `language-picker-coverage.png` | GPUIX GPU test renderer (T33: coverage badges) |
| `onboarding-models.png` | GPUIX GPU test renderer |
| `onboarding-download-progress.png` | **real** window, native addon (T16: live download) |
| `onboarding-models-ready.png` | **real** window, native addon (T16: models ready) |
| `console.png` | GPUIX GPU test renderer |
| `console-start-captions.png` | GPUIX click path console → live |
| `console-arabic.png` | GPUIX GPU test renderer |
| `live.png` | GPUIX GPU test renderer |
| `expanded.png` | GPUIX GPU test renderer |
| `settings.png` | GPUIX GPU test renderer |
| `settings-developer.png` | GPUIX GPU test renderer (measured renderer/adapter) |
| `summary.png` | GPUIX GPU test renderer |
| `licenses.png` | GPUIX GPU test renderer |
| `overlay.png` | **real** 780x250 GPUIX window, PrintWindow capture |
| `console-dev-vm.png` | earlier client-area capture (kept) |

The offscreen test window is fixed at 1536x1061 physical pixels on this host, so
those captures share that canvas. `settings-developer.png` shows
`Renderer: directx · GPU` and `Adapter: NVIDIA GeForce RTX 5060 Ti`, measured by
DXGI.

Reproduce from an elevated PowerShell on the logged-on desktop:

```powershell
pwsh -File scripts/run-in-interactive-session.ps1 `
  -FilePath scripts/screenshots-windows.ps1 -WorkingDirectory .
```

## Overlay matrix captures

`overlay/` holds the W-1 run (`docs/qa/2026-11-22-overlay-matrix.md`):

| File | What it shows |
|---|---|
| `overlay-100.png`, `-125`, `-150` | real 780x250 overlay at 100/125/150% scaling |
| `console-100.png`, `-125`, `-150` | console reflow at the same scales |
| `overlay-100-over-foreground.png` | overlay above a full-screen foreground proxy |
| `overlay-100-after-drag.png` | after dragging the overlay grip +120,+80 |

Mixed-DPI and multi-monitor are **not** in this folder: the host has one
2560x1080 display. The harness records the blocked row and will capture
`overlay-monitor-N.png` when a second display is present.

**Not** claimed by this folder: mixed-DPI or multi-monitor results (one display),
the virtual-desktop switch, overlay-over-exclusive-fullscreen, frame pacing,
WASAPI capture, or any model/ASR/LLM measurement. The NVIDIA driver now drives
the 2560x1080 display; the earlier portfolio was captured before that change.
Keep the Linux portfolio in `docs/screenshots/m2/` untouched.
