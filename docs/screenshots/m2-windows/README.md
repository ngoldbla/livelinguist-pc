# Windows screenshots

Portfolio rebuilt 2026-11-22 on the Windows 11 host with the GPUIX 0.11 GPU
test renderer (DirectX). Evidence and non-claims:
`docs/qa/2026-11-22-gpu-renderer.md`.

| File | Path |
|---|---|
| `onboarding-welcome.png` | GPUIX GPU test renderer |
| `onboarding-language.png` | GPUIX GPU test renderer |
| `onboarding-models.png` | GPUIX GPU test renderer |
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

**Not** claimed by this folder: the W-1 overlay matrix (topmost, focus
retention, unfocused click/hover, drag, mixed DPI, multi-monitor, virtual
desktops, overlay-over-fullscreen), frame pacing, WASAPI capture, or any
model/ASR/LLM measurement. The display class is the Microsoft Basic Display
Adapter even though DXGI enumerates the RTX 5060 Ti. Keep the Linux portfolio in
`docs/screenshots/m2/` untouched.
