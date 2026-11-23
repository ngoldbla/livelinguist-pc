# T32 — Measure the minimum live set en/fr/de/es/ar (S3)

**Needs:** T31 done.

**Gate (per locale):** score ≤ 0.15, RTF ≤ 0.5, p95 chunk ≤ 500 ms on the
**tier C profile** (`scripts\tier-c-profile.ps1`, from T21).

## Steps

1. Fetch the 5 locales, run `asr-wer` for each under the tier C profile, and
   once without it (tier A reference).
2. If a locale fails only on latency: re-run once with `num_threads` 3 vs 2 in
   the recognizer config and keep the better default.
3. Chunk check (ADR 0019): if latency fails at 560 ms on tier C, record it and
   stop — switching to the 160 ms model is a human decision (catalog bump).
4. Write `docs/qa/2026-11-23-asr-coverage-v1.md` with a table:
   locale | FLEURS code | metric | score | RTF | p95 ms | tier | status.
   Status `ok` only if the tier C row passes.

## Done when

5 rows measured. If any of en/fr/de/es/ar fails, write `BLOCKED.md` with the
numbers and stop (plan stop rule: the minimum live set must pass).

Commit message: `eval: ASR minimum live set measured`
