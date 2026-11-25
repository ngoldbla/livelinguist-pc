# Budgets (plan §15)

Targets are hypotheses until the Windows reference hardware is measured. Linux
dev numbers validate the plumbing, not product performance.

| Tier | Hardware | Live p50 | Click-to-simplified p95 | UI |
|---|---|---|---|---|
| A | Dylan PC, dGPU + 8+ cores, 16 GB+ | <= 1.5 s | <= 8 s | 60 fps |
| B | Integrated/low-end GPU, 8-16 GB | <= 2.5 s | <= 8 s | 60 fps |
| C | 4-core/8 GB Windows machine or CPU-limited profile | live if ASR RTF <= 0.5; otherwise chunked with a banner | <= 8 s | >= 30 fps, reduced effects |

v1.0 inference is **CPU-only** (sherpa-onnx ASR + llama.cpp 0.6B); GPU
acceleration is post-1.0. Simplification is **click-only**: there is no
automatic or live simplification, so the click-to-simplified budget is the only
LLM latency gate.

Tier C RSS budget: **<= 3.5 GB** total for UI + core + ASR + 0.6B Q4. Hypothesis,
to be measured.

## Measurement definitions

- Word-to-glass: `capture.chunk_start` -> `ui.partial_paint`; in the mock eval we
  use the 560 ms ASR chunk start -> first partial.
- Click-to-simplified: `ui.click` -> `ui.simplified_paint`; metrics use
  `simplify_start` -> `simplified_visible`.
- ASR chunk latency: `asr.chunk_in` -> `asr.segment_out` p95.
- LLM: `llm.prefill_start` -> `llm.first_token` (TTFT), first -> last token.
- UI frame: GPUIX frame stats p95.

## Product-level end-to-end baseline

The Rust-side Tier 1 stamp is a diagnostic approximation, not proof of paint.
The Windows product baseline is the media-clock → UIA-visible-caption benchmark
in `autoresearch/` (D-061). It reports partial/final p50/p95, cue coverage,
token similarity, and polling quantization. See
`docs/qa/2026-11-24-caption-latency-baseline.md` for measured numbers.

Measured on the Windows reference PC (2026-11-24/25):

| Path | Partial p95 | Final p95 | Coverage |
|---|---:|---:|---:|
| Installed RC, live view | 3657 ms | 12671 ms | 82.6% partial |
| Installed RC, overlay (final-only) | — | 12658 ms | 78.3% final |
| Overlay partial + 80 ms input chunk (experiment) | 2278 ms | 12523 ms | 93.8% partial |
| Overlay partial + **560 ms** (shipping default) | 2685 ms | 12523 ms | 93.8% partial |

The 560 ms input chunk is the shipping default since 2026-11-25. On the second
voice it measures 5208 ms mean p95 (4418 ms median) with 94.7% coverage; the
80 ms experiment wins the lecture but regresses the second-voice p95 to
6436–11102 ms. Evidence: `docs/qa/2026-11-25-asr-chunk-confirmation.md`.

## Current Linux dev evidence (demo backend, not product claims)

- Tier 1 p50/p95 = 0.56 s (560 ms chunk floor).
- Tier 2 p50 0.70 ms / p95 0.85 ms with zero simulated backend latency.
- 0 audio overruns over the 47.3 s demo conversation.
- UI window 1180x800 renders on lavapipe; frame stats not yet asserted.

## Low-end rules

- Click-only simplification; 0.6B Q4_K_M; 2-3 threads on tier C.
- `mmap` on, `mlock` off; unload the model after idle (optional post-1.0).
- Disk preflight 6 GB; ~1 GB default models.
- Reduced-effects mode: static field, no motion/glow, 30 fps target.
- Windows UI floor D3D11/12; Linux dev floor Vulkan; WARP/Lavapipe are
  best-effort and never claimed as 60 fps.
- No GPU inference in 1.0; do not rescue a slow CPU with a GPU dependency.
