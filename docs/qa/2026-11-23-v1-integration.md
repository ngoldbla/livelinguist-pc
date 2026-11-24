# v1 integration (S7: T34–T36)

The journey table and soak (T36) are still to come; this file starts with the
privacy check (T35). Content-free: no transcript or model output is recorded here.

## Privacy

| Platform | Date | Command | Result |
|---|---|---|---|
| Windows 11 (reference PC) | 2026-11-24 | `pwsh scripts\privacy-check.ps1` (T12 TTS clip through loopback) | `privacy: PASS` (owner retest step 5, `docs/qa/2026-11-24-windows-retest.md`) |
| Linux dev container | 2026-11-24 | `scripts/privacy-check.sh` (FLEURS en_us clip, `--asr sherpa --llm llama --click-all`) | `privacy: PASS` |

Linux run details:

- Checked 5 output files (`diagnostics.json`, `latency.json`, `summary.json`,
  plus the content-bearing `transcript.md` and `segments.csv`) and the run log
  against 5 distinctive reference words. None appear outside the
  content-bearing exports. The transcript does contain them, so the check is
  not vacuous.
- `strace -e trace=connect`: **0** `connect()` calls during the run (models were
  already in the cache).
- 1 finalized segment, simplified by the real llama simplifier.
- First run failed before any check: `ll-eval` with both `sherpa` and `llama`
  aborted creating the recognizer (`munmap_chunk(): invalid pointer`, a C++
  `std::regex` symbol clash between static onnxruntime and llama.cpp). Fixed
  by linking sherpa-onnx as a shared library on Linux (ADR 0022). Windows is
  unaffected.
