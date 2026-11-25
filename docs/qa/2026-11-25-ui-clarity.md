# UI clarity pass - 2026-11-25

Scope: the whole desktop UI (`ui/src`). Flow reference: the macOS build
`ngoldbla/live-linguist` @ `8eecdc4` (`App/ControlView.swift`,
`MacOnboardingView.swift`, `CaptionOverlayView.swift`, `SettingsView.swift`,
`SummarySheetView.swift`, `TranscriptSidebarView.swift`,
`Sources/LiveLinguistKit/Models/AuroraLayoutMetrics.swift`). Plan:
`docs/plans/2026-11-22-v1-scope.md` section 4. Decision:
`docs/decisions/0026.md`. Screenshots: `docs/screenshots/m2-windows/`.

## Why

The build worked but the UI did not read like the Mac product or like a tool for
people who need easier language:

- 30-40% of most windows was empty; the console was a three-column settings
  dashboard with the primary action in a side rail.
- Metadata was 9-11px and named after the implementation: "Tier 1", "ASR
  status", "English reversion", "simplify-only", "content-free diagnostics".
- Console toggles were inert, language status was repeated three times, and
  "Summary" silently did nothing at zero sentences.
- The overlay ignored the caption-size and two-tier settings.
- Preferences reset on every launch; the window-mode buttons showed no active
  mode; "Settings ->" links were 11px and the arrow wrapped.

The impeccable design review scored the build **20/40** (Acceptable); it is
saved in `.impeccable/critique/` (local, not committed).

## Mac flow (what was matched)

| Mac | PC |
|---|---|
| Control window: lockup, state headline, two decision tiles, bottom Start cluster | Same, with the tiles opening inline pickers instead of menus |
| Captions in floating overlay / transcript sidebar | Transcript screen after Start; overlay panel keeps one small control row because the PC has one window |
| Onboarding sheet: welcome/privacy, language (pick advances), one aggregate download | Same three beats in the window; no step rail or numbered progress |
| Summary sheet: stats, Export, Done | Same compact page; no transcript list |
| Settings: Captions, Models, Licenses | Same, plus the gated Developer tab; Licenses is a tab now |
| Metrics: 640 content, 820 reading, 44 headline, 24 caption | 640 / 820 / 40 / 24 (16-40 slider) |

Not copied: the Mac's glass/aurora world. The owner chose the quieter,
higher-contrast utility surface, so the PC keeps flat surfaces, one accent and
the 12px type floor.

## What changed

| Area | Before | After |
|---|---|---|
| Surface | Glass, gradients, 18px radii, glow shadows | Flat near-black fields, one violet accent, 8-12px radii, no shadows |
| Type | 14 sizes from 9 to 38; metadata 9-11px | Six sizes 12-34; 12px floor; section 20 < page 28 |
| Console | Language card + 3 source cards + captioning rail + models rail + session rail | Two decision tiles + Start cluster + model note |
| Onboarding | Left step rail, sample caption, progress segments, Continue after picking | Welcome/privacy, pick-to-advance language, aggregate download |
| Live | Right rail with stats, typed input, privacy; timestamped rows with badges | Headline + cadence line + transcript rows + centered Pause/Stop |
| Overlay | Top bar + footer, ignored caption size | Caption panel sized from Settings; one 26px control row |
| Settings | Captions/Models/Developer/About, two-column preview | Captions/Models/Licenses/Developer, one form, no preview rail |
| Summary | Transcript list + export rail + Back to console | Compact stat sheet + exports + Done |
| State | In-memory settings; inert console toggles | `ui/src/settings.ts` persists to `%LOCALAPPDATA%\LiveLinguist\ui-settings.json` |
| Windows | 1180x820 everywhere | Console 960x660, onboarding 960x680, summary 960x700, settings 1000x860 |

Also fixed: live header alignment, badge stretching, raw validator codes (now
one "~ simplified — may not be exact" line), the 11px arrow links, missing
`aria-checked` on switches, missing `tablist` role, slider min/max/now, and
opening Settings from the overlay (it restores a console window first).

Craft pass: no kicker above a heading, no unicode glyph icons, caption rows cap
at 820px, and the transcript has no second pane.

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
| onboarding-welcome | 9 | Label 6, Button 2 |
| onboarding-language | 111 | Label 79, Button 30, TextInput |
| onboarding-models | 10 | Label 6, Button 3 |
| console | 17 | Label 12, Button 4 |
| live | 49 | Label 31, Button 13, Tab 3, TabList |
| overlay | 18 | Label 10, Button 6, Group |
| settings | 30 | Label 18, Button 3, Tab 4, TabList, Slider, Switch 2 |
| settings-developer | 89 | Label 64, Button 17, Tab 4, TabList, Switch 2 |
| summary | 20 | Label 15, Button 4 |
| licenses (Settings tab) | 62 | Label 55, Button, Tab 4, TabList |

`console-start-captions.png` proves the console -> live click path; the keyboard
walk on the console reaches 4 distinct stops.

Contrast: `ui/src/aurora/contrast.test.ts` passes for every body token and badge
tone against every surface with the palette.

## Remaining gaps (not regressions)

- **Translations:** 19 keys were added to all 29 catalogs with English
  placeholders; the rewritten English for ten jargon keys is English only until
  each catalog is updated. Count parity is enforced by
  `i18n/locales.test.ts`.
- **UI locale:** the follow-learning toggle is gone; the app language follows
  the learning language. The Mac has a toolbar UI-language toggle; the PC has no
  equivalent yet.
- **Finished transcript:** the summary no longer lists it (Mac parity). Export
  is the only way to keep a session.
- **Fonts:** Windows uses Segoe UI/Consolas from the system; bundling Noto Sans
  remains a packaging task.
- **Capture canvas:** the GPUIX test renderer is fixed at 1536x1061 physical
  pixels, so the PNGs do not show the production window sizes; those are
  asserted by `windowSize.test.ts` and set in `main.tsx`.
- **Human Windows pass:** Narrator and a real 100/125/150% walk remain for the
  release gate.
