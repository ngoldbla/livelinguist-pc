# Live Linguist PC — UI

GPUIX 0.11 + React 19.2 + TypeScript. One native window that morphs between the
console, the compact caption overlay, and the expanded transcript.

## Commands

```bash
bun install --frozen-lockfile
bun run typecheck
bun test
bun run check           # Biome
bun run start           # real window
bun run screenshots     # real X11 captures into docs/screenshots/m2
bun run screenshots:windows  # GPUIX DirectX test renderer (desktop session)
```

On a Wayland desktop, force X11 for the dev overlay:

```bash
env -u WAYLAND_DISPLAY bun src/main.tsx
```

## Structure

- `src/main.tsx` — entry; environment bootstrap, `render()` with screen-specific
  window size.
- `src/App.tsx` — screen/router, session lifecycle, transport, settings.
- `src/screens/` — Onboarding (3 beats), Console, Live/Expanded, Overlay,
  Settings (captions/models/developer/about), Summary, Licenses.
- `src/aurora/` — AuroraField, Surface/GlassCard, Badge, Button, Tabs, Toggle,
  Slider, ProgressBar, CaptionRow, LanguagePicker.
- `src/store/` — `useSyncExternalStore` session store with `lastSeq` gating,
  partial coalescing, snapshot replacement.
- `src/i18n/` — 29 locale catalogs converted from the Mac
  `Localizable.xcstrings` plus desktop keys; `t()` with positional placeholders,
  locale resolution, RTL helper.
- `src/bridge/` — generated ts-rs protocol types and the native/Demo transport.
- `src/dev/DevPanel.tsx` — scenario presets, runtime health, fault surface.

## Rules that shaped this UI

- No DOM libraries, no `<button>`, no Tailwind/shadcn/Radix/Framer/react-icons.
- Styles are GPUIX `StyleDesc`: numeric spacing, two-stop linear gradients,
  structured `boxShadow`, explicit `<text color>`.
- Single window; overlay/expanded are layouts + native window policy, not new
  windows.
- GPUIX 0.11 has no Linux PNG test renderer; screenshots use the real X11
  window (`import -window`). On Windows the native DirectX test renderer is
  available and captures offscreen; it needs a logged-on desktop session, not
  session 0. See `scripts/screenshots-windows.tsx` and
  `scripts/run-in-interactive-session.ps1`.
- Caption announcements: no live-region API in 0.11; the expanded view exposes
  a "Read latest" action and Q-013 tracks the upstream addition.

## Screenshot scenarios

`scripts/screenshots.ts` launches one process per screen with
`LL_SCREEN=<screen>`, waits for the window, and captures it. On Windows,
`scripts/screenshots-windows.tsx` mounts the same app in GPUIX's GPU test
renderer and captures each screen offscreen; `scripts/screenshots-windows.ps1`
also captures the real 780x250 overlay window. The X11 overlay policy is applied
by the native bridge (`setWindowMode`), which sets `_NET_WM_STATE_ABOVE`,
skip-taskbar/pager, sticky desktop, utility window type, input=false, and Motif
borderless.

## Known limits

- Real ASR/GGUF models are not installed; the demo conversation and typed input
  exercise the real pipeline/engine. The UI labels this instead of faking a
  listening state.
- Windows/WASAPI acceptance is pending the Windows side of the dual boot.
- RTL text renders (Arabic catalog and Noto Sans Arabic); full row mirroring is
  tracked as a follow-up for GPUIX's limited direction support.
