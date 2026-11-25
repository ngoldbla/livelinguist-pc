# UI clarity pass - 2026-11-25

Scope: the whole desktop UI (`ui/src`), after a design review of the shipped
build. Plan: `docs/plans/2026-11-22-v1-scope.md` section 4. Decision:
`docs/decisions/0026.md`. Screenshots: `docs/screenshots/m2-windows/`.

## Why

The build worked but the UI did not read like a product for people who need
easier language:

- 30-40% of most windows was empty, and the console was a three-column settings
  dashboard with the primary action in a side rail.
- Metadata was 9-11px and named after the implementation: "Tier 1", "ASR
  status", "English reversion", "simplify-only", "content-free diagnostics".
- Console toggles were inert, language status was repeated three times, and
  "Summary" silently did nothing at zero sentences.
- The overlay ignored the caption-size and two-tier settings - the one surface
  where size matters most.
- Preferences reset on every launch; the window-mode buttons showed no active
  mode; "Settings ->" links were 11px and the arrow wrapped to its own line.

The impeccable critique scored the build **20/40** (Acceptable); it is saved in
`.impeccable/critique/` (local, not committed).

## What changed

| Area | Before | After |
|---|---|---|
| Surface | Aurora glass, gradients, 18px radii, glow shadows | Flat near-black fields, one violet accent, 8-12px radii, no shadows |
| Type | 14 sizes from 9 to 38; metadata 9-11px | Six sizes 12-34; 12px floor; section 20 < page 28 |
| Console | Language card + sound card + captioning rail + models rail + session rail + status bar | Language -> sound -> Start -> two options; footer has model state, privacy and licenses |
| Onboarding | Left step rail + privacy card + full-height card | One card, three beats, progress segments, one primary action per beat |
| Live | Right rail with stats, simplify-only input, privacy; duplicate counts | Transcript spine; one stats line; typed input only for text-only languages |
| Overlay | Ignored caption size/tier; 22px unlabeled glyphs | Reads `captionSize`/`showTier1`; visible 28px labels; centered sparse captions |
| Settings | Two-column, preview void, uppercase/mixed tabs | One form with inline preview; consistent tab names; slider thumb and keys |
| Summary | Ghost "Back to console"; boolean reversion count | Primary "Start another session"; real English-sentence count; `m:ss` |
| State | In-memory settings; inert console toggles | `ui/src/settings.ts` persists to `%LOCALAPPDATA%\LiveLinguist\ui-settings.json`; console writes the same settings |
| Words | "Live ASR", "Simplify only", "Tier 1", "ASR status" | "Live captions", "Simplified text only", "Show the original words", "Speech recognition" |
| Windows | Console 1180x820, settings 1180x880, onboarding 1180x820 | Console 1100x700, settings 1000x860, onboarding 980x680 |

Also fixed: the live header logo/subtitle alignment, badge stretching to full
width, "clean" badges on every caption row, raw validator codes
(`passive_voice` -> "Passive voice"), the 11px arrow links, missing `aria-checked`
on switches, missing `tablist` role, and slider min/max/now attributes.
Opening Settings or Licenses from the overlay now restores a console window
first instead of rendering the page into a 780x250 overlay.

## Evidence

```text
cd ui
bun run typecheck                          # pass
bun test                                   # 114 pass, 0 fail, 318 expect() calls
bun run check                              # pass (pre-existing biome config deprecation warning)
bun scripts/screenshots-windows.tsx         # 13 scenarios, 0 failures
```

A11y counts from the capture run (unlabeled controls = 0 in every scenario):

| Scenario | Nodes | Roles |
|---|---|---|
| onboarding-welcome | 19 | Label, Button |
| onboarding-language | 122 | Label 88, Button 32, TextInput |
| onboarding-models | 27 | Label 20, Button 4, ProgressIndicator 2 |
| console | 38 | Label 25, Button 7, Tab 2, TabList, Switch 2 |
| live | 64 | Label 43, Button 13, Tab 5, TabList 2 |
| overlay | 20 | Label 12, Button 6, Group |
| settings | 40 | Label 26, Button 3, Tab 4, TabList, Slider, Switch 4 |
| settings-developer | 90 | Label 65, Button 17, Tab 4, TabList, Switch 2 |
| summary | 66 | Label 51, Button 14 |
| licenses | 53 | Label 51, Button |

`console-start-captions.png` still proves the console -> live click path; the
keyboard walk on the console reaches 4 distinct stops.

Contrast: `ui/src/aurora/contrast.test.ts` passes for every body token and badge
tone against every surface with the new palette.

## Remaining gaps (not regressions)

- **Translations:** 18 new keys were added to all 29 catalogs with the English
  value as a placeholder; the rewritten English for ten jargon keys is English
  only until each catalog is updated. Count parity is enforced by
  `i18n/locales.test.ts`.
- **UI locale picker:** the "app language follows learning language" toggle is
  still the only control; there is no independent 29-locale picker yet.
- **Fonts:** Windows uses Segoe UI/Consolas from the system. Bundling Noto Sans
  remains a packaging task (`docs/plans/2026-11-23-windows-v1-plan.md`).
- **Capture canvas:** the GPUIX test renderer is fixed at 1536x1061 physical
  pixels, so the PNGs do not show the production window sizes; those are
  asserted by `windowSize.test.ts` and set in `main.tsx`.
- **Light theme:** not included; the pass is dark-only by design.
- **Human Windows pass:** Narrator and a real 100/125/150% walk remain for the
  release gate; this pass covers the renderer, a11y counts and keyboard paths.
