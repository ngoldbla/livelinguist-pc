# T28 — Contrast, reduced effects, a11y checklist (S6 a11y) (PR)

Linux part done in PR #26; Windows pending: Narrator listening pass (human), Arabic screenshot `console-arabic-v1.png`, live checklist columns (names/roles, Tab order) after T26.

**Needs:** T27 done.

**Files:** `ui/src/aurora/theme.ts` (only if contrast fails),
`docs/qa/2026-11-23-v1-ui.md`

## Steps

1. Contrast: write a small `bun test` that computes WCAG contrast for each
   text color token against its background token in `theme.ts` and asserts
   ≥ 4.5:1 for body text and ≥ 3:1 for large text (≥ 18 px). Fix tokens that
   fail with the smallest change.
2. Reduced effects: confirm the setting disables motion/glow and targets 30
   fps (search `reduced` in `ui/src`). If it doesn't, make it so.
3. Write `## A11y checklist` in `docs/qa/2026-11-23-v1-ui.md`: one row per
   screen × (names/roles, Tab order, keyboard actions, focus ring, contrast,
   Arabic text renders) with pass/fail from T26–T28. Add a row
   **"Narrator listening pass — pending human (MVP checkpoint)"**. Do not mark
   it passed.
4. Arabic: run onboarding/console with locale `ar-SA`, screenshot
   `docs/screenshots/m2-windows/console-arabic-v1.png`, confirm text shapes
   correctly. Full chrome mirroring stays deferred (say so).

## Done when

UI checks pass; checklist written. Open PR `ui: keyboard and a11y pass` from
branch `stage-w5-a11y` (T26–T28), merge when CI is green. Set S6 to `merged`
(Narrator item open) in the progress board.

Commit message: `ui: contrast test and a11y checklist`
