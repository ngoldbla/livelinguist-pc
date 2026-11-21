# LLM backends, calibration, and degradation ladder

- Date: 2026-11-21
- Status: **partial** — mock/demo backend, calibration policy, queue, ladder,
  and prefix-cache bookkeeping implemented and tested; real llama.cpp
  CPU/Vulkan/CUDA numbers are pending a pinned GGUF and the Windows GPU host.
- Decisions: D-011, D-013/014/015, D-040/041/042

## Implemented

- `ll-llm::DemoBackend` uses the pack's curated demo rewrites; `MockBackend`
  scripts responses; `FaultyBackend` injects timeout/error/OOM.
- `CalibrationCache` keyed by CPU/GPU/driver/model hash/backend version, with
  JSON round-trip. `calibrate()` probes CPU and the chosen GPU and keeps the
  faster backend; a cached result is reused.
- `choose_device()` prefers a discrete GPU, then integrated, then CPU.
- The 2-D ladder is exactly:
  `1.7B GPU-all -> 1.7B GPU-partial -> 1.7B CPU -> 0.6B GPU-all -> 0.6B CPU ->
  verbatim` (`ll_llm::ladder`).
- On-demand queue: one active + one queued, newest click replaces the queued
  item, cancel works on active/queued.
- Prefix cache: LRU by locale, records prefill ms.

## Mock measurements (not product claims)

`ll-eval --scenario stress --language fr-FR` on the Linux dev host, demo backend
with zero simulated latency:

- 9 finalized segments, 9 simplified, no fallbacks.
- Tier 1 p50/p95 = 1.1 s (1.1 s ASR chunk floor).
- Tier 2 p50 = 0.70 ms, p95 = 0.85 ms (deterministic demo, not Qwen3).
- 0 overruns.

These numbers validate the event/latency plumbing, not model performance.

## Not yet measured

- Qwen3 0.6B/1.7B GGUF TTFT and tokens/s on CPU/Vulkan/CUDA.
- Calibration accuracy on the real devices.
- Prefix-prefill on tier C.
- OOM/fallback behavior with a real llama.cpp backend.

The model catalog marks the planned GGUF artifacts unpinned; the downloader
refuses them until publish-time SHA-256 values are committed (INV-008). This is
deliberate: no trust-on-first-use and no fake model claims.

## Evidence

```bash
cargo test -p ll-llm
cargo run -p ll-eval -- --scenario stress --language fr-FR --out /tmp/ll-eval-auto
```
