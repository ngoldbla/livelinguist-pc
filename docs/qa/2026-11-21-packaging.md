# Packaging evidence (M3)

- Date: 2026-11-21
- Status: **configured, not Windows-validated.** The Linux dev host cannot run
  NSIS, cargo-packager's Windows target, or a clean Windows install.

## Artifacts configured

- `packaging/nsis/livelinguist.nsi` — per-user NSIS installer (no admin), Start
  Menu/Desktop shortcuts, uninstall registry entries, payload from
  `dist/windows`.
- `packaging/packager.json` — cargo-packager alternative (`formats: ["nsis"]`,
  currentUser install mode).
- `scripts/build-windows.ps1` — Rust core, Bun-compiled UI, assets, NSIS,
  portable ZIP.
- `scripts/package-portable.sh` — Linux-side ZIP assembly from an existing
  `dist/windows` payload (inspection only).
- `.github/workflows/release.yml` — Windows runner builds and attaches the
  installer/ZIP to a tag release.

## Checks performed

```bash
python3 -m json.tool packaging/packager.json >/dev/null   # pass
bash -n scripts/package-portable.sh                       # pass
```

## Required Windows acceptance (pending)

- Clean Windows 11 23H2+ install, launch, microphone permission (no admin),
  upgrade, uninstall.
- Portable ZIP unpack/run.
- WASAPI loopback over a real call/video, 30-min soak with xrun = 0.
- Win32 overlay over full-screen video, focus retention, drag, 100/125/150% DPI,
  multi-monitor.
- No models bundled; first-run download prompt contacts Hugging Face.

These are **not claimed** on Linux. Run `scripts/build-windows.ps1` on the
dual-boot Windows side, then record the results here.
