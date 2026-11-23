# ASR coverage and demo evidence (S-3A partial)

- Date: 2026-11-21
- Status: **partial** - mock/demo path implemented and exercised. The model is
  now pinned (Nemotron 3.5 560 ms int8) and the WER gate is pending the mirror
  and the `sherpa` runtime; see `docs/qa/2026-11-22-model-pins.md` and ADR 0019.
  The "candidate" column below is historical.
- Decision: D-031, D-032, INV-015

## What runs today

- `ll-asr` exposes a streaming `AsrEngine` seam, a scripted demo engine, and an
  `Off` engine. A `sherpa` feature exists but fails with an explicit
  `ASR_MODEL_MISSING` when no pinned model is installed; it never pretends to
  listen.
- `ll-eval --scenario mock --language fr-FR` drives a 9-line demo conversation
  through audio -> VAD -> chunker -> ASR -> segmenter and writes 9 finalized
  segments (`/tmp/ll-eval-out/segments.csv`).
- `ll-eval --wav tests/fixtures/sample.wav` replays a 3.0 s synthetic non-PII WAV
  and writes 1 finalized segment.
- `ll-eval --scenario simplify-only --text "…"` exercises the typed/pasted path
  through the real simplification engine.

## Per-locale matrix (v1 default)

All rows are `tbd` because no WER gate has been measured yet. `tbd` is not
shippable as `ok`; the effective v1 behavior is `simplify_only` for every row
until S-3A records a fixture result. The UI must show the paste/type notice.

| Locale | Candidate ASR | Streaming | WER fixture | Status | UI behavior |
|---|---|---|---|---|---|
| de | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| fr | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| es | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| en | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| ar | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| da | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| et | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| fi | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| hi | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| it | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| ja | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| ko | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| nl | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| pt-BR | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| pt-PT | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| ru | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| sk | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| sv | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| tr | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| vi | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| zh-CN | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| uk | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| ro | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| bg | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| pl | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| hr | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| nb | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| hu | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |
| cs | Nemotron 3.5 / zipformer TBD | not measured | none | tbd | simplify only |

`CoverageMatrix::v1()` in `ll-core/src/coverage.rs` is the single source of
truth; `set_status` promotes a row only after the WER gate.

## Blockers for the real S-3A gate

1. The planned `ndgold/nemotron-3.5-asr-streaming-0.6b-onnx` artifact is not
   reachable from this environment; no ONNX export/model files are committed.
2. Building the `sherpa` feature and downloading a real streaming zipformer
   needs a model pin, license check, and a WER fixture per locale.
3. Windows WASAPI/real-call audio acceptance is pending the Windows side.

These are recorded as remaining work, not claimed as passing.
