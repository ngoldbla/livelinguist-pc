# T23 — Bridge `workArea()` (S6 layout)

Linux part done in PR #25; Windows pending: run the `workArea()` node check on the real Windows desktop at 100% and 150% display scale and record the numbers.

**Goal:** the UI can learn how much screen space it really has, so windows
stop clipping at 150%.

**Files:** `core/crates/ll-window/src/win32.rs`, `core/crates/ll-window/src/x11.rs`
(or `lib.rs` fallback), `core/crates/ll-bridge/src/lib.rs`, workspace
`Cargo.toml` (`windows` features)

## Steps

1. `ll-window`: `pub fn work_area() -> Option<WorkArea>` with
   `WorkArea { x, y, width, height, scale: f64 }` in **physical** pixels.
   Windows: `MonitorFromPoint` (cursor position or primary) →
   `GetMonitorInfoW` → `rcWork`; scale from `GetDpiForMonitor` (MDT_EFFECTIVE_DPI)
   / 96. Add the needed `windows` crate features (`Win32_Graphics_Gdi`,
   `Win32_UI_HiDpi`) to the workspace pin. Linux X11: return the root window
   size with scale 1.0, or `None` if that's hard — Linux is dev only.
2. Bridge: `#[napi] pub fn work_area(&self) -> Result<String>` → JSON
   `{ x, y, width, height, scale }` or `null`. It must work **before** a
   session starts (the UI calls it before opening the window).
3. Test: on Windows (`#[cfg(windows)]`), `work_area()` is `Some` with
   width/height > 0 and scale in 1.0..=3.0.

## Done when

```powershell
cargo test -p ll-window
cargo test -p ll-bridge
cargo clippy --workspace --all-targets -- -D warnings
```

and a quick check prints sensible numbers at 100% and 150% display scale:
`node -e "const c=require('./ui/livelinguist-core.node'); console.log(new c.CoreClient().workArea())"`
(adjust to how the addon is constructed in `tests/bridge/bridge-smoke.mjs`).

Commit message: `window: report the monitor work area`
