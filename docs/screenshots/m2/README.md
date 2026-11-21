# M2 UI screenshots (Linux dev)

Captured 2026-11-21 from real GPUIX windows on `DISPLAY=:0` with the native
bridge, demo conversation, and English UI locale (except the language-specific
view). Regenerate with `cd ui && bun run screenshots`.

| File | Screen | What it shows |
|---|---|---|
| `onboarding-welcome.png` | Onboarding beat 1 | Privacy, same-language promise, sample caption, demo escape hatch |
| `onboarding-language.png` | Onboarding beat 2 | 29-locale picker with explicit `simplify only` status |
| `onboarding-models.png` | Onboarding beat 3 | Model sizes, Hugging Face disclosure, no fake download |
| `console.png` | Console | Language, demo source, on-demand/live, model status, start |
| `console-arabic.png` | Console | Arabic locale selected, translated labels, RTL-ready catalog |
| `live.png` | Live / expanded | Tier 1 + Tier 2 rows, click-to-simplify, session stats |
| `expanded.png` | Expanded | Full transcript, confidence badges, simplify-only input |
| `overlay.png` | Caption overlay | 780×250 borderless bar, topmost/no-activate X11 policy |
| `settings.png` | Settings / Captions | Caption size, tier toggles, preview |
| `settings-developer.png` | Settings / Developer | Scenario presets, runtime health, strict mode, content-free diagnostics |
| `summary.png` | Summary | Duration, sentences, 100% target, reversions, export actions |
| `licenses.png` | Licenses | Models, frameworks, packs, attribution |

## Overlay policy evidence (S-6L, dev-only)

`xprop -id <window>` after `setWindowMode('overlay')`:

```
_NET_WM_STATE = _NET_WM_STATE_SKIP_PAGER, _NET_WM_STATE_SKIP_TASKBAR,
                _NET_WM_STATE_ABOVE, _NET_WM_STATE_STICKY, _NET_WM_STATE_FOCUSED
_NET_WM_WINDOW_TYPE = _NET_WM_WINDOW_TYPE_UTILITY
_NET_WM_DESKTOP = 4294967295
WM_HINTS: Client accepts input or input focus: False
_MOTIF_WM_HINTS = 0x2, 0x0, 0x1, 0x0, 0x0
```

`cargo test -p ll-window` also creates a real X11 window and asserts the
property application (`applies_ewmh_overlay_hints_to_a_real_x11_window`).

Windows Win32 acceptance (topmost/no-activate/drag/DPI/multi-monitor) is pending
the Windows side of the dual boot; the raw-FFI policy compiles under
`cargo check --target x86_64-pc-windows-msvc`.
