---
name: gpuix
description: Use when writing, testing, packaging, or debugging the Live Linguist PC UI built with GPUIX 0.11 + React 19.2. Covers exact pins, JSX/runtime rules, style subset, a11y, single-window constraints, Linux dev capture, and forbidden web libraries.
---

# GPUIX 0.11 for Live Linguist PC

## Pins

- `@gpuix/react` **0.11.0 exact**
- `@gpuix/native` **0.11.0 exact**
- `react` **19.2.3 exact** (peer `^19.2.0`)
- `typescript` 5.9.3, strict
- Upgrade adapter + native together; never let the range float.

`tsconfig.json` must set `"jsx": "react-jsx"`, `"jsxImportSource": "@gpuix/react"`,
`"moduleResolution": "bundler"`, `"strict": true`.

## Runtime

- Entry file ends with `render(<App />, options)`; use `bun --hot app.tsx` for
  same-window hot remount. Never call `init()` in the entry.
- Single window. Modes (console/overlay/expanded) are React state + window
  policy, not multiple windows.
- Linux selects Wayland when `WAYLAND_DISPLAY` is set, otherwise X11. To force
  X11 dev: `env -u WAYLAND_DISPLAY bun app.tsx`. `bootstrap.ts` only clears it
  before loading the native module.
- `focus: false` and `show: false` are ignored on Linux; automation still works
  through the live input pipeline.
- GPUIX automation `screenshot()` needs a test-support build on macOS/Windows;
  on Linux dev capture the real X11 window with `import -window <id>`.
- `startFrameLoop` polls on Windows/Linux; do not build a `setImmediate` ticker.

## Styling

- `div` is block; set `display: "flex"` before `flexDirection`/`gap`/`alignItems`.
- No CSS shorthands/strings: numbers only for padding/margin/borderWidth.
- `boxShadow` is a structured object, not a string.
- `background` supports only a **two-stop linear-gradient**; no radial/conic.
- Every `<text>` needs an explicit `color` (no inheritance from a parent unless
  the parent sets `color` on its style; prefer explicit).
- `<text>` siblings merge into one line; do not nest `<text>` in `<text>`.
- `<input>` has no default inner padding; style the input or its wrapper.
- Hover/active/focusVisible are nested one level deep.
- No `<button>`: use `<div role="button" tabIndex={0} onClick>`.
- Nested scrolling is unsupported (one parent scroller; virtual-list counts).
- Use `<virtual-list>` for transcripts, with bounded height/flex and stable
  keyed rows.

## A11y and keyboard

- Every interactive element: `role`, `aria-label`, state, keyboard path.
- `role="button"` + `tabIndex={0}` + Enter/Space for caption rows.
- Sliders: `role="slider"`, `aria-valuetext`, arrow keys.
- Visible focus via `focusVisible` outline; GPUIX dims other focusables by
  default (turn off with `keyboardFocusDim: false` if the screen needs it).
- Caption announcements: no live-region API was found in GPUIX 0.11. Provide the
  "read latest" action and record Q-010/Q-013 upstream.

## Forbidden

Tailwind, shadcn, Radix, Framer Motion, `react-icons`, React Testing Library,
Playwright DOM tests, Electron/Tauri/webview, a second UI framework, CSS
frameworks, DOM `window`/`document` assumptions, `<button>`.

## Testing

- Logic: `bun test` (store, i18n, formatting).
- Component behavior: GPUIX `createTestRoot` + `connectTest` — **no PNG capture
  on Linux**, but text/bounds/events/locators work.
- Real-window smoke: launch under X11, `xdotool search --name <title>`, then
  `import -window <id> out.png`.
- Jobs must set `GPUIX_BACKGROUND=1` and the entry must map it to `focus: false`
  on macOS/Windows.
