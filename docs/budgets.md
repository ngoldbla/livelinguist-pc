# Budgets (plan §15)

Targets are hypotheses until the Windows reference hardware is measured. Linux
dev numbers validate the plumbing, not product performance.

| Tier | Hardware | Live p50 | On-demand p95 | UI |
|---|---|---|---|---|
| A | Dylan PC, dGPU + 8+ cores, 16 GB+ | <= 1.5 s | <= 8 s | 60 fps |
| B | Dylan PC with dGPU disabled / iGPU, 8-16 GB | <= 2.5 s | <= 8 s | 60 fps |
| C | 4-core/8 GB Windows machine or CPU-limited profile | on-demand only | <= 8 s | >= 30 fps, reduced effects |

Tier C RSS budget: **<= 3.5 GB** total for UI + core + ASR + 0.6B Q4. Hypothesis,
to be measured.

## Measurement definitions

- Word-to-glass: `capture.chunk_start` -> `ui.partial_paint`; in the mock eval we
  use 1.1 s ASR chunk start -> first partial.
- Click-to-simplified: `ui.click` -> `ui.simplified_paint`; metrics use
  `simplify_start` -> `simplified_visible`.
- ASR chunk latency: `asr.chunk_in` -> `asr.segment_out` p95.
- LLM: `llm.prefill_start` -> `llm.first_token` (TTFT), first -> last token.
- UI frame: GPUIX frame stats p95.

## Current Linux dev evidence (demo backend, not product claims)

- Tier 1 p50/p95 = 1.1 s (1.1 s chunk floor).
- Tier 2 p50 0.70 ms / p95 0.85 ms with zero simulated backend latency.
- 0 audio overruns over the 47.3 s demo conversation.
- UI window 1180x800 renders on lavapipe; frame stats not yet asserted.

## Low-end rules

- On-demand default; 0.6B Q4_K_M; 2-3 threads on tier C.
- `mmap` on, `mlock` off; unload 1.7B after idle.
- Disk preflight 6 GB; ~1 GB default models.
- Reduced-effects mode: static field, no motion/glow, 30 fps target.
- Windows UI floor D3D11/12; Linux dev floor Vulkan; WARP/Lavapipe are
  best-effort and never claimed as 60 fps.
