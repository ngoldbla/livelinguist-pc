---
name: live-linguist-core
description: Use when implementing or changing the Live Linguist PC Rust core, pipeline, protocol, models, or eval. Covers the 17 invariants, the deterministic porting order, event envelope, degradation, privacy rules, and acceptance checks.
---

# Live Linguist core

## Invariants (must hold in every change)

| ID | Rule |
|---|---|
| INV-001 | Same-language only; never translate. |
| INV-002 | Two caption tiers: verbatim partial + simplified final. |
| INV-003 | On-demand simplification is the shipping default. |
| INV-004 | Validators are confidence signals, not gates. Show amber + flags, never hide text. |
| INV-005 | Fallback to verbatim with the `≡` marker; test the path. |
| INV-006 | Never drop a finalized transcript. Overflow is counted, surfaced, degrades. |
| INV-007 | On-device; network only for downloads + opt-in update checks. |
| INV-008 | Pinned model revisions and publish-time SHA-256. |
| INV-009 | No secrets in app or repo. |
| INV-010 | Transcripts are memory-only until explicit export. |
| INV-011 | Exports/diagnostics are content-free; no segment text in logs/crashes. |
| INV-012 | Accessibility + cognitive clarity are release gates. |
| INV-013 | Latency instrumentation is built in. |
| INV-014 | One window, one process in v1. |
| INV-015 | Every locale has an explicit ASR status. No fake listening. |
| INV-016 | Every event is `{v, seq, ts, event}`; UI applies monotonically. |
| INV-017 | Audio overflow is visible to the user and diagnostics. |

## Porting order

1. Copy `assets/packs/*.json` verbatim from the Mac repo
   (`Sources/LiveLinguistKit/Resources/FrameworkPacks/`). Never hand-edit a
   pack's prompt to change behavior.
2. Port `ll-core`: `Language`/`Segment` models, `FrameworkPack`, `PromptBuilder`,
   `CaptionDecoding`, `DisfluencyCleaner`, `Segmenter`, `TextAnalysis`,
   `LanguageGuard`, `FidelityGuard`, `NumberGuard`, `LanguageRuleCheckers`,
   `ValidatorSuite`, `ReversionDetector`, `SessionStore`, `LatencyMetrics`,
   `SessionCSVExporter`, `TranscriptExporter`, `DemoSimplifierBackend`.
3. The Swift tests are the spec. Port them by suite; do not "improve" a
   threshold without a new decision and golden update.
4. Then protocol/config/pipeline, audio/models/bridge/eval, then UI.

## Pipeline rules

- Audio callback: no allocation, locks, logging, or JS.
- VAD gates feeding ASR; sherpa endpointing ends utterances; the segmenter
  splits sentences for the UI (not the ASR chunker).
- Ring buffer 10 s at 16 kHz mono f32; overflow increments `AUDIO_OVERRUN` and
  starts the degradation ladder.
- Event queue bounded; coalesce stale partials; never drop finalized segments,
  errors, or state changes.
- On-demand: one active + one queued; newest click replaces the queued item;
  live yields to on-demand; `cancelSimplify(segmentId)` works.
- 2-D ladder: 1.7B GPU-full -> 1.7B GPU-partial -> 1.7B CPU -> 0.6B GPU-full ->
  0.6B CPU -> verbatim.
- Qwen3 `enable_thinking=false`; think-block stripping is a safety net.
- Any locale not `ok`/`degraded` is `simplify_only`: paste/type + simplify,
  never a listening state.

## Protocol

```ts
type Envelope<T> = { v: number; seq: number; ts: number; event: T }
```

`getSnapshot() -> { state, lastSeq }`. The UI drops `seq <= lastSeq`. Stable
error codes: `ASR_INIT_FAILED`, `ASR_MODEL_MISSING`, `LLM_OOM`,
`MODEL_HASH_MISMATCH`, `AUDIO_DEVICE_LOST`, `AUDIO_OVERRUN`, `BACKEND_UNAVAILABLE`.

## Privacy

- `log` events carry codes and redacted messages only.
- Diagnostics JSON/CSV: timings and counts, no text.
- Transcript exports only on explicit user action.
- Core dumps off by default.

## Validation commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p ll-eval -- --scenario mock --out /tmp/ll-eval-out
```
