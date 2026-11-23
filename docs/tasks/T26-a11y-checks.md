# T26 — Automated a11y checks on every screen (S6 a11y)

**Goal:** a failing check if any core screen has an unlabeled control or a
broken Tab order.

**Files:** `ui/scripts/screenshots-windows.tsx` (it already counts unlabeled
controls and console Tab stops), `scripts/verify-windows.ps1`

## Steps

1. In `screenshots-windows.tsx`, the console scenario already presses Tab and
   counts stops. Do the same for onboarding (each beat), live/expanded,
   settings, summary, licenses: press Tab up to 40 times and record the focused
   node label sequence (labels only, never caption text — skip nodes whose
   role is text).
2. Fail (non-zero exit) when: any screen has `unlabeledControls > 0`; Tab
   does not move; or focus gets stuck on one node for 3 presses.
3. Print one line per screen: `screen unlabeled=<n> tabStops=<n>`.
4. Add `-A11y` to `scripts/verify-windows.ps1` that runs this script (it needs
   the interactive desktop; document `run-in-interactive-session.ps1` for
   session 0).

## Done when

`pwsh scripts\verify-windows.ps1 -A11y` passes after fixing any missing labels
it finds (add `aria`-style labels the same way existing components do; search
`label` in `ui/src/aurora/components.tsx`).

Commit message: `ui: a11y checks for every core screen`
