# Live Linguist PC

Native, on-device **same-language simplification captions** for x86-64 PCs running
Linux — and, where the stack allows, Windows.

This repo is the PC spin-out of [live-linguist](https://github.com/ngoldbla/live-linguist)
(the macOS/iOS app). The product conceit is unchanged: a learner hears authentic
native-speed speech and reads the same content rendered into the official
easy-language register for that language — **FALC**, **Leichte Sprache**,
**Easy English**, **Lectura Fácil** — with no translation, no cloud, and no audio
leaving the machine.

The Mac app achieves that with Apple-only building blocks. This repo exists to
answer, with evidence, what the same conceit becomes on x86 Linux: which models
run, how fast, on whose hardware, and which parts of the Mac architecture are
portable data versus Apple-specific machinery.

## Scope

**In scope**

- x86-64 Linux desktop (Wayland and X11), CPU-only and vendor-GPU paths
  (Vulkan first; CUDA/ROCm where present).
- The full caption pipeline: audio → VAD → chunking → ASR → segmentation →
  simplification → deterministic validation → two-tier captions.
- On-demand ("click a sentence") simplification as the shipping default, matching
  the Mac app's current UX.
- The reusable parts of the Mac implementation: framework/register packs,
  validator rules, reversion detection, latency instrumentation, export formats.
- Windows as a near-term secondary target, provided it doesn't force a parallel
  stack. See [docs/stack-options.md](docs/stack-options.md).

**Out of scope (for now)**

- ARM Linux, mobile, and any cloud or hosted inference. The privacy contract is
  the product: nothing leaves the device except a model download.
- Translation. Ever. Same-language register rewriting only.
- macOS/iOS: those remain in the reference repo.

## What carries over, and what cannot

| Concern | Mac reference | PC reality |
|---|---|---|
| Register rules, prompts, thresholds | `FrameworkPacks/*.json`, `ValidatorSuite` | **Portable data.** The crown jewel; language-agnostic and test-first. |
| Caption pipeline shape | `CaptionPipeline` (1.1 s chunks, 10 s commit window, tier 1 verbatim + tier 2 simplified) | Portable design; needs a C++/Rust (or Swift-on-Linux) reimplementation. |
| ASR | CoreML **ANE** encoder + MLX RNN-T decoder | **Must be replaced.** No ANE, no MLX on x86. Candidates: sherpa-onnx, ONNX Runtime, whisper.cpp. |
| Simplifier | Qwen3 0.6B/1.7B 4-bit via MLX-Swift | **Must be replaced.** Candidates: llama.cpp GGUF, ONNX Runtime GenAI. |
| Audio capture | AVFoundation mic + CoreAudio process taps | **Different model.** PipeWire/PulseAudio monitor sources or a virtual sink; user setup required. |
| Overlay | NSPanel joining all Spaces | **Harder.** Wayland has no cross-compositor always-on-top; X11 is straightforward. |
| Model delivery | Hugging Face, pinned revisions, SHA-256 | Reusable design; format differs (GGUF/ONNX), same pinning discipline. |

## Budgets

The Mac's numbers are the reference, not the promise. On x86 the live budget must
be set by hardware class and measured, not assumed:

| Tier | Target hardware | Suggested live p50 | Suggested on-demand |
|---|---|---|---|
| A | Discrete GPU (Vulkan/CUDA/ROCm) | ≤ 1.5 s | ≤ 8 s |
| B | Modern CPU, AVX2, 8+ cores | ≤ 2.5 s | ≤ 8 s |
| C | Older/low-power CPU | on-demand only | ≤ 8 s |

These are hypotheses to falsify in M0, not commitments.

## Status

**Pre-M0.** No implementation yet. This repo starts as the decision record and
spike plan: pick the ASR and LLM stacks, prove word-to-glass latency on real x86
hardware, prove audio capture and overlay on both Wayland and X11, then port the
deterministic layer.

## Milestones

- **M0 — feasibility spikes**
  1. ASR: streaming word-to-glass p50/p95 on tier A/B/C hardware (sherpa-onnx vs
     whisper.cpp vs ONNX Runtime), 30-min cache-stability soak.
  2. Simplifier: llama.cpp Qwen3 0.6B/1.7B (and the fine-tuned EasyLanguage GGUFs)
     p50/p95 per segment, CPU vs Vulkan.
  3. Audio capture: PipeWire monitor/virtual sink for mic + system audio on
     GNOME and KDE, Wayland and X11.
  4. Overlay: always-on-top caption window on Wayland (wlr-layer-shell where
     available, KWin rules otherwise) and X11.
- **M1 — headless parity**: CLI that replays audio files through the real
  pipeline and reproduces the Mac eval artifact shape (per-segment CSV, latency
  JSON, summary).
- **M2 — desktop app**: setup, download, captioner, on-demand simplification,
  transcript sidebar/summary, export.
- **M3 — packaging & release**: Flatpak/AppImage/deb, model manager with pinned
  checksums, licenses screen.

## Design invariants (inherited)

- Never block or drop a segment: a late or unsimplified caption beats no caption.
- Validators are confidence signals, not gates; fall back to verbatim with "≡".
- Degradation order: batch → step down model size → skip simplification
  (verbatim) → never drop audio.
- Latency instrumentation is built in, and exports are content-free.
- Pinned model revisions and checksums; no silent model drift.

## Reference

- Mac/iOS implementation: <https://github.com/ngoldbla/live-linguist>
- Product spec: [`PRD.md`](https://github.com/ngoldbla/live-linguist/blob/main/PRD.md)
- Platform PRDs that anticipate this port: `docs/prd/PRD-windows.md`,
  `docs/prd/PRD-android.md` in the reference repo (both assume llama.cpp/GGUF and
  sherpa-onnx, which is a useful prior).
- Live-budget measurements that set expectations:
  `docs/qa/2026-06-10-live-budget-levers.md`.

## Decisions

Architecture decisions and spike results live in
[`docs/decisions/`](docs/decisions/). Start with
[docs/stack-options.md](docs/stack-options.md) for the candidate matrix.
