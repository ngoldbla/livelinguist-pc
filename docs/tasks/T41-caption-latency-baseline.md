# T41 — Caption-latency baseline and autoresearch harness

- Branch: `stage-autoresearch-baseline`
- Plan: `docs/plans/2026-11-24-autoresearch-plan.md`
- Decision: `docs/decisions/0024.md` (D-061)

## Goal

Measure the installed Windows RC's media-clock → visible-caption delay with a
pinned captioned video, then provide a repeatable CPU-first parameter search.
Do not change shipping ASR defaults or claim GPU inference.

## Files

- `autoresearch/*` — fixture manifest, player, UIA probe, alignment, benchmark,
  optimizer, tests, and agent program.
- `ll-config`, `ll-audio`, `ll-asr`, `ll-pipeline`, `ll-bridge`, `ll-eval` —
  explicit ASR chunk/thread/endpoint experiment knobs; defaults unchanged.
- `ui/src/screens/Overlay.tsx`, `Live.tsx` — show/tag the existing verbatim
  partial.
- Docs: ADR 0024, plan, QA evidence, budgets/progress/decision index.

## Steps

1. Pin a CC-BY video + English SRT by SHA-256; never commit media.
2. Automate local playback, WASAPI loopback session start, and UIA caption
   observation with a shared media clock.
3. Align cue starts to visible partial/final text; write content-free JSON only.
4. Add `LL_ASR_THREADS`, `LL_ASR_CHUNK_MS`, `LL_ASR_RULE1_MS`,
   `LL_ASR_RULE2_MS`, and `LL_ASR_RULE3_MS`; validate bounds and pass them to
   sherpa/pipeline.
5. Add `ll-eval --scenario asr-wer --asr-chunk-ms` for quality follow-ups.
6. Show the verbatim partial in the overlay.
7. Run installed-RC live/overlay baselines and a one-variable parameter screen.
8. Record commands, hardware, binary hash, p50/p95, coverage, similarity, and
   polling quantization. No transcript text.
9. Confirm the winner with 3 × 90 s and a second voice/style before proposing a
   shipping-default change.

## Checks

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p ll-eval --features sherpa
bun test autoresearch/align.test.ts
cd ui
bun run typecheck
bun test
bun run check
```

Hardware evidence:

```powershell
bun autoresearch\benchmark.ts --app "$env:LOCALAPPDATA\Programs\LiveLinguist\livelinguist.exe" `
  --mode live --seconds 90 --repeat 3 --out target\autoresearch\baseline-live

bun autoresearch\optimize.ts --seconds 60 --repeat 1 `
  --out target\autoresearch\optimization-stable
```

## Done

- Baseline and screen are reproducible and content-free.
- Quality gates reject fragmenting candidates.
- No shipping default changes without the confirmation gate.
- Evidence and ADR are committed on the branch.
