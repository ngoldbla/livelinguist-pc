# Windows caption-latency autoresearch plan

- Date: 2026-11-24
- Status: active
- Decision: `docs/decisions/0023.md` (D-060)
- Scope: post-v1 optimization; does not change the v1 CPU-only/click-to-simplify
  release gates without a separate accepted decision.

## Goal

Minimize the time between a reference caption becoming visible in the playing
media and the same words becoming visible in Live Linguist. Start with the
shipping CPU path on the Windows reference PC. Add GPU only after the CPU
baseline and harness are stable.

## Canonical metrics

| Metric | Definition | Use |
|---|---|---|
| Partial p50/p95 | Reference cue start → first visible partial containing the cue's first three normalized tokens | Primary live-latency objective |
| Final p50/p95 | Reference cue start → visible finalized caption matching the cue | Secondary; overlay baseline before partials |
| Coverage | Matched cues / cues in the benchmark window | Quality gate ≥ 80% |
| Similarity | Mean token edit-distance similarity of matched text | Quality gate ≥ 0.85 |
| Poll quantum | UIA scan time + poll sleep, p95 | Measurement uncertainty |

Simplification clicks are excluded. They are a separate Tier 2 objective.

## Harness

`autoresearch/benchmark.ts`:

1. Downloads and hash-verifies the pinned Wikimedia Commons video and English
   SRT into `target/autoresearch/media/`.
2. Serves them locally with a browser player and WebVTT captions.
3. Launches the installed RC or current source UI, selects **System audio**, and
   starts captions through Windows UIA.
4. Plays the media through the default render endpoint so WASAPI loopback hears
   exactly what a user hears.
5. Polls the app's UIA text tree every 20 ms and records only caption changes.
6. Maps UIA wall time to media time using browser playback beacons, including
   stalls.
7. Aligns observed captions to SRT cue starts and writes content-free JSON.

Raw observations and transcript text stay in process memory. No audio, video,
caption text, UIA tree, or screenshot is committed or exported.

## Experiment order

### Stage A — baseline (required)

1. Installed RC, live view, 90 s × 3.
2. Installed RC, overlay, 90 s × 3.
3. Record machine, commit, binary SHA-256, model revisions, display/audio
   endpoint, and poll quantum.

### Stage B — cheap parameter screen

Run `autoresearch/optimize.ts` with 45–60 s windows:

1. Overlay partial + original 1100 ms pipeline chunk (control).
2. 560 ms input chunk (matches the model package name).
3. 160 ms input chunk.
4. 80 ms input chunk.
5. Fast endpoint candidate: rule1 1200 ms, rule2 400 ms, rule3 10 s.
6. Thread counts 1/2/3/4/6 on the reference host.

Do not change two variables at once after the initial combined candidate.

### Stage C — code candidates

Only after Stage B:

1. Partial coalescing/frame pump if delivery stalls are measurable.
2. Move simplification off the bridge mutex if click tests show caption gaps.
3. Native capture/present timestamps if UIA quantization is too large.

### Stage D — confirmation

For the winner:

1. 90 s × 3 on the reference host.
2. Repeat with a second speech style/speaker.
3. Run the existing WER/CER matrix or at least en/fr/de/es/ar.
4. Run a click-during-live test and verify zero finalized-segment loss, surfaced
   overruns, and no caption gap beyond the accepted budget.
5. Update ADR 0019/0023, budgets, and the progress tracker before changing a
   shipping default.

### Stage E — GPU track (later)

Keep the same fixture, metrics, and quality gates. Add an explicit ONNX Runtime
provider/model record for the RTX 5060 Ti. Compare CPU and GPU as separate rows;
never describe a UI-renderer GPU probe as GPU inference.

## Agent loop

Follow `autoresearch/PROGRAM.md`. One hypothesis per iteration. Keep a change
only when p95 improves by at least 5% and all quality/privacy gates hold. Revert
or flag a candidate that wins only within measurement noise.

## Exit gate

A trustworthy baseline, a content-free journal, and at least one confirmed CPU
candidate that materially reduces overlay partial p95 without quality loss.
GPU work does not start until this gate is met.
