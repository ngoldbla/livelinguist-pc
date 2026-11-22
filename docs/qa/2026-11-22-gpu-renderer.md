# GPU renderer and accelerator probe (Windows)

- Date: 2026-11-22
- Status: **renderer and accelerator evidence measured; hardware UI gates that
  need a real display/audio path remain open.**
- Branch: `stage-w5-gpu-evidence`
- Plan: `docs/plans/2026-11-21-windows-resume-plan.md` (W-5, W-4 groundwork)
- Machine record: `docs/qa/2026-11-21-windows-hardware.md`

## What changed

1. `RuntimeStatus.uiBackend` (declared from `cfg!(target_os)`) is replaced by a
   measured `RuntimeStatus.ui` object. The new `ll-accel` crate probes DXGI on
   Windows, DRM/Vulkan ICDs on Linux, and the CUDA driver API on Windows.
2. `probeBackends()` no longer guesses from `LL_CUDA_AVAILABLE` or a Linux-only
   path; it reports the measured adapters and accelerator devices.
3. The dev panel shows the measured renderer, adapter, and a software-rendering
   badge; a degraded session surfaces a content-free notice.
4. The UI portfolio is rebuilt through the **GPUIX GPU test renderer**
   (`captureScreenshot`), with the overlay captured from its real 780x250
   window.

## Renderer / accelerator probe

Command: `cargo run -q -p ll-accel --example probe`
Build: `2401677` + this stage's working tree; Rust 1.98, MSVC.

```json
{
  "ui": {
    "backend": "directx",
    "apiAvailable": true,
    "hardware": true,
    "adapter": "NVIDIA GeForce RTX 5060 Ti",
    "driver": null,
    "degradedReason": null,
    "notes": [
      "hardware adapter: NVIDIA GeForce RTX 5060 Ti",
      "software adapter also present: Microsoft Basic Render Driver"
    ]
  },
  "adapters": [
    {
      "name": "NVIDIA GeForce RTX 5060 Ti",
      "vendorId": 4318,
      "deviceId": 11524,
      "dedicatedVramMb": 16050,
      "software": false,
      "kind": "discrete"
    },
    {
      "name": "Microsoft Basic Render Driver",
      "vendorId": 5140,
      "deviceId": 140,
      "dedicatedVramMb": 0,
      "software": true,
      "kind": "software"
    }
  ],
  "vulkan": {
    "available": false,
    "detail": "Vulkan loader present; ICD registry unavailable (WIN32_ERROR(2))",
    "devices": []
  },
  "cuda": {
    "available": true,
    "detail": "CUDA driver 13.5; 1 device(s)",
    "devices": [
      {
        "name": "NVIDIA GeForce RTX 5060 Ti",
        "vramTotalMb": 16283,
        "compute": "12.0"
      }
    ]
  }
}
```

Bridge contract check:

```text
$ node tests\bridge\hardware-probe.mjs
{"ok":true,"ui":"directx","hardware":true,"adapter":"NVIDIA GeForce RTX 5060 Ti",
 "adapters":2,"vulkan":false,"cuda":true,
 "cudaDevices":["NVIDIA GeForce RTX 5060 Ti"]}
```

Interpretation:

- The display class is still the Microsoft Basic Display Adapter, but DXGI sees
  the RTX 5060 Ti as a hardware adapter and CUDA returns one device. The
  previous "no GPU" record was based on `Win32_VideoController` alone and is
  corrected in the hardware record.
- Vulkan is **not** usable: the loader exists, the ICD key is absent.
  `vulkan-strict` must fail loudly, not silently fall back.
- CUDA is available for the W-4 backend path, but no CUDA toolkit or GGUF model
  is installed, so this is device availability only — no TTFT/tokens/s claim.

## Session 0 vs interactive desktop

The agent runs in **session 0 (services)**. In session 0:

```text
[gpui_windows::vsync] Failed to get DWM interval: Access is denied. (0x90070005)
error: Failed to open the GPUI window: Creating DirectX renderer
code: "GenericFailure"
```

DXGI/CUDA probing and `cargo test` still work in session 0; creating the
GPUIX DirectX window does not. The screenshots and GPU renderer check therefore
run in the logged-on desktop through a temporary interactive scheduled task:

```powershell
pwsh -File scripts/run-in-interactive-session.ps1 `
  -FilePath scripts/screenshots-windows.ps1 -WorkingDirectory .
```

`run-in-interactive-session.ps1` refuses to run unelevated, creates a one-shot
`/IT` task for the current account, streams its log, deletes the task, and
returns its exit code. It is dev tooling only.

## GPUIX GPU test renderer

`hasTestGpuixRenderer()` is `true` in the installed `@gpuix/native` 0.11.0
Windows build. In the interactive session, `createTestRoot` + `captureScreenshot`
rendered the real React app offscreen through GPUIX's DirectX renderer.

Command:

```powershell
pwsh -File scripts/run-in-interactive-session.ps1 `
  -FilePath scripts/screenshots-windows.ps1 -WorkingDirectory .
```

Result (2026-11-22):

| Screen | Capture | Canvas | a11y nodes | Unlabeled controls | Keyboard stops |
|---|---|---|---|---|---|
| onboarding-welcome | `onboarding-welcome.png` | 1536x1061 | 35 | 0 | — |
| onboarding-language | `onboarding-language.png` | 1536x1061 | 139 | 0 | — |
| onboarding-models | `onboarding-models.png` | 1536x1061 | 35 | 0 | — |
| console | `console.png` | 1536x1061 | 67 | 0 | 4 |
| console-arabic | `console-arabic.png` | 1536x1061 | 67 | 0 | — |
| live | `live.png` | 1536x1061 | 86 | 0 | — |
| expanded | `expanded.png` | 1536x1061 | 86 | 0 | — |
| settings | `settings.png` | 1536x1061 | 39 | 0 | — |
| settings-developer | `settings-developer.png` | 1536x1061 | 91 | 0 | — |
| summary | `summary.png` | 1536x1061 | 77 | 0 | — |
| licenses | `licenses.png` | 1536x1061 | 55 | 0 | — |
| console → Start captions (interaction) | `console-start-captions.png` | 1536x1061 | — | — | — |
| overlay (real window) | `overlay.png` | 796x258 | n/a | n/a | — |

Notes:

- The offscreen test window is fixed at 1536x1061 physical pixels on this host
  regardless of the requested size, so all offscreen captures share that canvas.
  The overlay is captured from its real window to keep its 780x250 form factor.
- `settings-developer.png` shows the measured values in the dev panel:
  `Renderer: directx · GPU`, `Adapter: NVIDIA GeForce RTX 5060 Ti`,
  `Window: console · directx · GPU`.
- The a11y count uses GPUI's painted a11y tree; "unlabeled controls" means a
  node with role button/checkbox/switch/slider/textbox/link/tab and no label.
  The console Tab pass moved focus through 4 distinct stops.
- `console-start-captions.png` is an interaction pass, not just a static
  render: a click at the measured bounds of "Start captions" went through GPUI
  hit testing, React, the transport, and the live screen (9 verbatim
  segments in on-demand mode).
- Raw a11y trees are not persisted because live screens include transcript text.

## Non-claims

- This is **not** the W-1 overlay matrix: topmost, focus retention, unfocused
  click/hover, drag, mixed DPI, multi-monitor, virtual desktops, and
  overlay-over-fullscreen are not exercised.
- The display adapter is Basic Display. The test renderer proves the DirectX
  code path and GPU adapter enumeration; it does not prove frame pacing or that
  each frame was presented by the RTX adapter.
- No WASAPI capture: the host has render endpoints but no microphone endpoint.
- No ASR/LLM measurement: no models. CUDA availability is not a tokens/s claim.
- The test-renderer PNGs are not substituted for real-window acceptance.

## Reproduce

```powershell
cargo run -q -p ll-accel --example probe
cargo test -p ll-accel
cargo build -p ll-bridge --release
Copy-Item target\release\livelinguist_core.dll tests\bridge\livelinguist-core.node -Force
node tests\bridge\hardware-probe.mjs

cd ui
bun run typecheck
bun test
bun run check
cd ..

pwsh -File scripts/run-in-interactive-session.ps1 `
  -FilePath scripts/screenshots-windows.ps1 -WorkingDirectory .
```

## Follow-ups

- W-1: run the overlay matrix on the interactive desktop.
- W-4: with a pinned GGUF and a CUDA-capable llama.cpp backend, measure
  TTFT/tokens/s on the RTX 5060 Ti; Vulkan remains unavailable until an ICD is
  installed.
- W-2: a microphone endpoint or a documented virtual-device decision is still
  required.
