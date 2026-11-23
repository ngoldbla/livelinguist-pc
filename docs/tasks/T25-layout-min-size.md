# T25 — Screens fit the minimum size + screenshots (S6 layout) (PR)

**Needs:** T24 done.

**Goal:** every screen is fully usable at the smallest window T24 can produce.

**Files:** `ui/src/screens/*.tsx` (only layout styles), `ui/src/App.tsx`

## Steps

1. Launch each screen at the minimum size:
   `$env:LL_WIDTH=960; $env:LL_HEIGHT=620; $env:LL_SCREEN="<screen>"; cd ui; bun src/main.tsx`
   for `console`, `expanded`, `settings`, `summary`, `licenses`, and onboarding.
2. Wherever content is cut off: replace fixed heights with `flexGrow: 1` +
   `minHeight: 0`, and put long lists/columns in a scroll container (use the
   same scroll style other screens already use; search `overflow`). Buttons
   and the caption list must stay reachable. Do not change colors, fonts, or
   copy.
3. Screenshots with the existing script
   (`scripts/screenshots-windows.ps1` / `bun run screenshots:windows`) at
   display scale 100%, 125%, 150%, for resolutions 1920x1080 and 1280x720 (change
   the resolution in Windows display settings, restore it afterwards). Save to
   `docs/screenshots/m2-windows/layout/<screen>-<res>-<scale>.png`.
4. Look at every screenshot: nothing clipped, no overlapping text.

## Done when

UI checks pass (`typecheck`, `test`, `check`), and screenshots exist with no
clipping. Write `docs/qa/2026-11-23-v1-ui.md` `## Layout` with the screenshot
list and any non-claims (for example "full RTL mirroring deferred").

Open PR `ui: fit 100/125/150% and 1280x720` from branch `stage-w5-layout`
(T23–T25), merge when CI is green.

Commit message: `ui: screens fit the minimum window`
