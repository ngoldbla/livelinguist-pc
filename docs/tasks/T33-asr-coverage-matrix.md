# T33 — Measure all 29 locales and set coverage statuses (S3) (PR)

Measurements for all 29 locales are in PR #31 (`docs/qa/2026-11-23-asr-coverage-v1.md`); `coverage.rs` + UI statuses, tests and the historical marker are done (2026-11-24, 12 `ok` / 17 `simplify_only`). Windows (2026-11-24): tier C re-check on the 12 `ok` rows + ja-JP recorded in that doc; language picker screenshot `docs/screenshots/m2-windows/language-picker-coverage.png` (the picker shows `Live ASR` and `Simplify only` from the matrix). **Blocked on one owner decision:** de-DE scores 0.157 on Windows (gate 0.15) — see `docs/tasks/BLOCKED.md`.

**Needs:** T32 passed.

**Files:** `core/crates/ll-core/src/coverage.rs`, `docs/qa/2026-11-23-asr-coverage-v1.md`,
`docs/qa/2026-11-21-asr-coverage.md` (status line only)

## Steps

1. Run `asr-wer` under the tier C profile for the remaining 24 locales (one at
   a time; the fetch script deletes tarballs as it goes). Add their rows.
2. In `coverage.rs`, change `CoverageMatrix::v1()` so each locale's status
   comes from the measurement: `Ok` if it passed, `SimplifyOnly` if it failed
   or has `no_fixture`. Add a comment per row: score, RTF, date. **Nothing
   stays `Tbd`.** Update the existing test that expects all `Tbd` to expect the
   measured table instead, and add a test that en/fr/de/es/ar are `Ok`.
3. Mark `docs/qa/2026-11-21-asr-coverage.md` historical in its status line
   ("superseded by 2026-11-23-asr-coverage-v1.md").
4. Check the UI language picker shows "live" vs "simplify-only" from the new
   matrix (screenshot `docs/screenshots/m2-windows/language-picker-coverage.png`).

## Done when

`cargo test --workspace`, clippy, UI checks pass. Open PR
`windows: ASR coverage matrix` from branch `stage-w3-asr-coverage` (T29–T33)
with the 29-row table; merge when CI is green; set S3 to `merged`.

Commit message: `core: measured ASR coverage for 29 locales`
