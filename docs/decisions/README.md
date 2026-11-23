# Decisions

One file per decision, numbered. Each records the context, the options
considered, the choice, and what measurement would overturn it. Spike results go
in the same file as a dated addendum. The master plan
(`docs/plans/2026-11-21-live-linguist-pc-master-plan.md`) is the integrated index;
these ADRs are canonical for the rationale and evidence of their decision.

Status labels: `proposed`, `accepted`, `baseline` (working default), `superseded`.

| # | Decision | Plan IDs | Status |
|---|---|---|---|
| [0001](0001.md) | ASR runtime: sherpa-onnx | D-012 | accepted |
| [0002](0002.md) | Simplifier runtime: llama.cpp GGUF | D-011 | accepted |
| [0003](0003.md) | System-audio capture | D-007, §13 | baseline |
| [0004](0004.md) | Overlay strategy: Win32 primary | D-007 | accepted |
| [0005](0005.md) | Core implementation language: Rust | D-002 | accepted |
| [0006](0006.md) | Live budget per hardware tier | D-051 | baseline |
| [0007](0007.md) | UI framework: GPUIX + React | D-001 | baseline |
| [0008](0008.md) | Window modes and overlay policy | D-004/008/009/010 | baseline |
| [0009](0009.md) | Inference backends and calibration | D-013/014/015/040 | baseline |
| [0010](0010.md) | Developer settings and strict scenarios | D-018 | accepted |
| [0011](0011.md) | Per-locale ASR coverage | D-031 | accepted |
| [0012](0012.md) | Event delivery and envelope | D-033/038 | baseline |
| [0013](0013.md) | Platform order: Windows-first | D-023/053 | accepted |
| [0014](0014.md) | GUI floor: D3D11/12 / Vulkan | D-024 | accepted |
| [0015](0015.md) | Windows-only v1 scope | D-053 | accepted |
| [0016](0016.md) | Reference hardware: Dylan PC | D-051 | accepted |
| [0017](0017.md) | Measure the renderer and accelerators; never declare them | D-054 | accepted |
| [0018](0018.md) | v1.0 is single-screen, system-audio, CPU-first | D-055 | accepted |

## Superseded

None yet.

## Adding or changing a decision

1. Copy the template in the master plan §22.5.
2. Give it the next number and the D-ID it records.
3. Move the old entry to `Superseded` with a link.
4. Update the decision log in the master plan.
