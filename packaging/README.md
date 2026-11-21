# Packaging (Windows 11 x64 only)

The v1 product ships **NSIS installer + portable ZIP**. No Linux package is in
this plan (D-052).

## Layout

`dist/windows/` is the portable payload:

- `livelinguist.exe` — Bun-compiled GPUIX UI
- `livelinguist-core.node` — napi core addon
- `assets/packs`, `assets/locales` — framework packs and UI catalogs
- `backends/` — llama.cpp dynamic backend modules (CPU/Vulkan/CUDA) when built
- `licenses/` — model/data/framework attributions

Models are **not bundled**; they download to `%LOCALAPPDATA%\LiveLinguist\models`.

## Build (on Windows)

```powershell
pwsh scripts/build-windows.ps1
```

Produces `packaging/nsis/LiveLinguist-0.1.0-setup.exe` (NSIS) and
`dist/LiveLinguist-0.1.0-windows-x64-portable.zip`.

`packaging/packager.json` is the cargo-packager alternative:

```powershell
cargo install cargo-packager --locked
cargo packager --release --config packaging/packager.json
```

## Acceptance

Run on a clean Windows 11 23H2+ install and on the dual-boot Windows side:

- install, launch, microphone permission (no admin), upgrade, uninstall;
- WASAPI loopback with a real call/video;
- Win32 topmost/no-activate overlay over full-screen video;
- 100/125/150% DPI and multi-monitor.

These checks are pending the Windows hardware and are not claimed on Linux.
