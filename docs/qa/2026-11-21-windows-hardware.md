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
| GPU driver | Display class: Microsoft Basic Display Adapter 10.0.26100.1; CUDA driver 13.5 (`nvcuda.dll`); no `nvcc` | TBD | n/a |
| Vulkan API/device | Loader `vulkan-1.dll` present; **no ICD registered** → unavailable | TBD | n/a |
| CUDA toolkit | Driver API 13.5; toolkit/`nvcc` not installed | n/a | n/a |
| Display(s) / scaling | 1024x768, 100% (96 DPI), single monitor (`WinDisc`) | TBD | TBD |
| Microphone | **none enumerated** (two render endpoints only) | TBD | TBD |
| Output/loopback device | `LG HDR WFHD (NVIDIA High Definition Audio)`, `Digital Audio (S/PDIF)` | TBD | TBD |
| Storage free | 130.3 GB on C: | TBD | TBD |

## Build record

| Field | Value |
|---|---|
| Repo commit | `2401677` + this stage's working tree |
| Rust / MSVC | rustc 1.98.0 `x86_64-pc-windows-msvc` |
| Bun / Node | Bun 1.4.5 / Node 22.23.5 |
| Visual Studio Build Tools / SDK | 17.14 (MSVC 14.44.35207), Windows 11 SDK |
| NSIS | not installed (W-6) |
| cargo-packager | not installed (W-6) |

## Display / GPU caveat

`Win32_VideoController` reports the Microsoft Basic Display Adapter, but DXGI
enumerates the RTX 5060 Ti as a hardware adapter and the CUDA driver API returns
one device (16283 MB, compute capability 12.0). The earlier "no GPU" note in
`2026-11-22-windows-vm-smoke.md` was based on the display class only and is
corrected here. The display stack is still Basic Display, so:

- DirectX rendering and the GPUIX GPU test renderer work in the logged-on
  desktop session (see `2026-11-22-gpu-renderer.md`).
- Overlay-over-fullscreen, mixed-DPI/multi-monitor, frame pacing, and WASAPI
  capture are **not** validated by that evidence.

## Notes

- Tier C is an approximation on this host; label it as such.
- Record driver + display scaling with every audio/GPU measurement.
- Audio has render endpoints but no capture endpoint; W-2 microphone/loopback
  still needs hardware or a documented virtual-device decision.
