# Windows reference hardware record

- Status: **measured 2026-11-22 on the Windows 11 dev host.** Tier B/C rows
  remain to be measured; no performance claim is valid without them.
- Decision: D-051 (Dylan PC is tier A; GPU-less/limited is tier B/C approximation)
- Probe used: `cargo run -p ll-accel --example probe` (DXGI, Vulkan ICD,
  CUDA driver API). Raw output is in `docs/qa/2026-11-22-gpu-renderer.md`.

| Field | Tier A (dGPU) | Tier B (iGPU / dGPU disabled) | Tier C approximation |
|---|---|---|---|
| Machine | Windows 11 Pro host (MS-7B79, Ryzen 9 3900X) | same | low-end 4-core/8 GB or CPU-limited profile |
| Windows build (winver) | 10.0.26300 x64 | TBD | TBD |
| CPU / cores / threads | AMD Ryzen 9 3900X, 12 / 24 | TBD | TBD |
| RAM | 128 GB (137,357,053,952 bytes) | TBD | TBD |
| GPU | NVIDIA GeForce RTX 5060 Ti, 16 GB (DXGI, PCI 10DE:2D04) | TBD | TBD |
| GPU driver | NVIDIA 32.0.16.2124 (display + CUDA 13.5); no `nvcc` | TBD | n/a |
| Vulkan API/device | Loader `vulkan-1.dll` present; **no ICD registered** → unavailable | TBD | n/a |
| CUDA toolkit | Driver API 13.5; toolkit/`nvcc` not installed | n/a | n/a |
| Display(s) / scaling | `\\.\DISPLAY3`, 2560x1080, 100% (96 DPI), single monitor | TBD | TBD |
| Microphone | **none enumerated** (two render endpoints only) | TBD | TBD |
| Output/loopback device | `LG HDR WFHD (NVIDIA High Definition Audio)`, `Digital Audio (S/PDIF)` | TBD | TBD |
| Storage free | 130.3 GB on C: | TBD | TBD |

## Build record

| Field | Value |
|---|---|
| Repo commit | `55a91e1` + this stage's working tree |
| Rust / MSVC | rustc 1.98.0 `x86_64-pc-windows-msvc` |
| Bun / Node | Bun 1.4.5 / Node 22.23.5 |
| Visual Studio Build Tools / SDK | 17.14 (MSVC 14.44.35207), Windows 11 SDK |
| NSIS | not installed (W-6) |
| cargo-packager | not installed (W-6) |

## Display / GPU caveat

The earlier W-0 pass saw only the Microsoft Basic Display Adapter and recorded
"no GPU". The NVIDIA driver was subsequently installed: `Win32_VideoController`,
DXGI, and CUDA all report the RTX 5060 Ti, driving a 2560x1080 LG display at
100%. Vulkan still has no registered ICD.

- DirectX rendering, the GPUIX GPU test renderer, and the real overlay window
  work (`2026-11-22-gpu-renderer.md`, `2026-11-22-overlay-matrix.md`).
- Mixed-DPI and multi-monitor, overlay-over-exclusive-fullscreen, frame pacing,
  and WASAPI capture are **not** validated by that evidence; there is one
  display and no microphone endpoint.

## Notes

- Tier C is an approximation on this host; label it as such.
- Record driver + display scaling with every audio/GPU measurement.
- Audio has render endpoints but no capture endpoint. W-2 now starts with
  WASAPI **system-audio loopback** (the initial Zoom/Teams/Meet/Skype/YouTube
  use case); microphone capture is a follow-up, not a blocker.
