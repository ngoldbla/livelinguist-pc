# Windows reference hardware record

- Status: **template — fill on the Windows side before any measurement.**
- Decision: D-051 (Dylan PC is tier A; GPU-less/limited is tier B/C approximation)

Record exact values in every measurement; a performance claim without this
record is invalid.

| Field | Tier A (dGPU) | Tier B (iGPU / dGPU disabled) | Tier C approximation |
|---|---|---|---|
| Machine | Dylan PC Windows 11 dual boot | same | low-end 4-core/8 GB or CPU-limited profile |
| Windows build (winver) | TBD | TBD | TBD |
| CPU / cores / threads | TBD | TBD | TBD |
| RAM | TBD | TBD | TBD |
| GPU | TBD | TBD | TBD |
| GPU driver | TBD | TBD | TBD |
| Vulkan API/device | TBD | TBD | TBD |
| CUDA toolkit | TBD | n/a | n/a |
| Display(s) / scaling | TBD | TBD | TBD |
| Microphone | TBD | TBD | TBD |
| Output/loopback device | TBD | TBD | TBD |
| Storage free | TBD | TBD | TBD |

## Build record

| Field | Value |
|---|---|
| Repo commit | TBD |
| Rust / MSVC | TBD |
| Bun / Node | TBD |
| Visual Studio Build Tools / SDK | TBD |
| NSIS | TBD |
| cargo-packager | TBD |

## Notes

- Tier C is an approximation on the Dylan PC; label it as such.
- Record driver + display scaling with every audio/GPU measurement.
