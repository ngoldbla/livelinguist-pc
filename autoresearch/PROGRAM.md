# Autoresearch program — caption latency

You are optimizing Live Linguist's real Windows live-caption latency. Work on a
branch from fresh `main`. Read `docs/decisions/0023.md` and
`docs/plans/2026-11-24-autoresearch-plan.md` before changing code.

## Objective

Minimize overlay **partial p95**: reference caption cue start → first visible
Live Linguist partial containing the cue's first three normalized tokens.

Do not optimize final p50 alone. Do not include click-to-simplify time.

## Non-negotiable gates

A candidate is eligible only when all of these hold:

1. `valid === true` in every confirmation trial.
2. Optimized-tier cue coverage ≥ 0.80.
3. Mean token similarity ≥ 0.85.
4. p95 improves by at least 5% over the current accepted candidate.
5. No invariant regression: same-language only, on-device, no transcript text in
   logs/metrics/exports, envelope sequence intact, no silent strict-mode
   fallback, no dropped finalized transcript.
6. The improvement survives 3 × 90 s confirmation runs. A single smoke run is
   not evidence.

## Iteration protocol

1. **Read the journal.** Open the newest `target/autoresearch/optimization-*/journal.json`.
2. **Pick one hypothesis.** Examples: smaller input chunk, lower endpoint silence,
   fewer/more threads, partial coalescing, overlay presentation, bridge-lock
   contention. Change one variable at a time after the initial screen.
3. **Record the hypothesis before running.** Add a short content-free entry to
   the run's `hypothesis.md`: candidate ID, change, expected mechanism, gates.
4. **Build.** For core changes:
   ```powershell
   cargo fmt --all
   cargo test -p ll-config -p ll-audio -p ll-pipeline
   cargo test -p ll-eval --features sherpa
   cargo build -p ll-bridge --release --features capture,sherpa,llama
   Copy-Item target\release\livelinguist_core.dll ui\livelinguist-core.node -Force
   ```
5. **Screen.** Run 45–60 s with `autoresearch/benchmark.ts` or `optimize.ts`.
6. **Decide.** Keep, revert, or mark inconclusive. Never keep a change that only
   wins inside `pollQuantumP95Ms` or run-to-run noise.
7. **Confirm.** Run the winner 3 × 90 s, then a second voice/style and the
   relevant WER/CER rows.
8. **Document.** Update `docs/qa/2026-11-24-caption-latency-baseline.md`, the
   journal, and an ADR before changing a shipping default.

## Candidate backlog

- Input chunk: 1100 → 560 → 160 → 80 ms.
- Endpoint rule2: 1200 → 500 → 400 ms; stop before fragmentation reduces coverage.
- Threads: 1/2/3/4/6 on the reference host; re-check tier C separately.
- Overlay: show verbatim partial above the latest final.
- Delivery: measure TSFN/React/paint stalls; coalesce stale partials only if the
  UI is the bottleneck.
- Contention: move click simplification off the bridge mutex if live captions
  gap during a click.
- Instrumentation: add native capture/present timestamps if UIA is too coarse.

## Stop rules

- Stop if the same benchmark command fails twice after two different fixes.
- Stop if media playback, loopback, or UIA cannot be trusted; fix the harness
  before optimizing the product.
- Stop if a candidate improves latency but loses text, fragments captions, or
  violates privacy/content-free output.
- Do not start GPU work until the CPU baseline and one confirmed CPU candidate
  exist. GPU must be a separate provider/model row, never a UI-renderer claim.
