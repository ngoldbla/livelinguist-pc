# Blocked cards

## T33 — ASR coverage, Windows tier C re-check (2026-11-24)

Not a tooling error: a measured gate result that the plan says needs an owner
decision before the matrix can be called done.

**What ran** (reference Windows 11 PC, Ryzen 9 3900X, `scripts\tier-c-profile.ps1`,
affinity 0xF, BelowNormal):

```powershell
pwsh scripts\tier-c-profile.ps1 -- target\release\ll-eval.exe --scenario asr-wer `
  --asr sherpa --locale de-DE --cache target\fleurs --asr-threads 2 --out target\asr-wer-win\de-DE
```

All 12 `ok` locales plus ja-JP were re-scored and re-timed; the full table is in
`docs/qa/2026-11-23-asr-coverage-v1.md` under `## Windows tier C re-check`.

**Exact numbers for the failing row:**

```
"locale": "de-DE", "metric": "wer", "edits": 76, "reference_units": 484,
"score": 0.157, "rtf": 0.246, "p95_chunk_ms": 288.5, "peak_rss_mb": 778, "exit_code": 0
```

Three runs, all 0.157 (idle host; while downloading; `--asr-threads 3`). Linux
measured 0.141 with the same model, clips and threads. The gate is score ≤ 0.15,
so German misses by 0.007 (about 8 word errors in 484) while every other
minimum-set locale passes and all timing passes with ≥1.5× headroom. en-US sits
exactly on the gate (0.149). ja-JP stays `simplify_only` (0.155 → 0.1597).

**Why stop:** the T32/plan stop rule says the minimum live set must pass, and
`CoverageMatrix::v1()` currently marks de-DE `ok` from the Linux measurement.
Changing that is a product decision, not an agent one.

**Owner options**

1. Keep de-DE `ok` and record the Windows miss in this doc (captions work;
   0.157 is 0.007 over on the pessimistic tier C profile).
2. Demote de-DE to `simplify_only` (update `coverage.rs`, the UI table and the
   test that keeps them equal).
3. Re-pin a German model or add German-specific decoding — a model-card change,
   not a QA card.

**Tried already:** `--asr-threads 3` (same score, p95 501.7 ms — worse), two
repeat runs (same 0.157), idle host (same 0.157). Nothing else is in scope for
this card.
