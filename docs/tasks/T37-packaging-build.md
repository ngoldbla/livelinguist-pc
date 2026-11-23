# T37 — Installer and portable ZIP build (S8)

**Needs:** S7 merged **and** `docs/qa/*-mvp-human-pass.md` exists (the human
pass). If it doesn't exist, stop with `blocked`.

**Files:** `scripts/build-windows.ps1`, `packaging/nsis/livelinguist.nsi`,
`packaging/*` as needed

## Steps

1. `build-windows.ps1 -Package` produces, from a clean `target`:
   `dist\LiveLinguist-<version>-setup.exe` (per-user NSIS, no admin) and
   `dist\LiveLinguist-<version>-portable.zip`, plus `dist\SHA256SUMS.txt`.
   Version comes from the workspace `Cargo.toml`.
2. Payload: Bun/Node runtime per the existing packaging design, the UI, the
   real-feature addon, any DLLs it needs (check with `dumpbin /dependents` on
   the `.node` file; static sherpa/llama should need only system DLLs).
   **No models, no GPU backends, no `.pdb`.**
3. Size gate: installed app ≤ 60 MB (excluding models). Print the size.
4. NSIS: installs to `%LOCALAPPDATA%\Programs\LiveLinguist`, Start Menu
   shortcut, uninstaller registered under HKCU only, `RequestExecutionLevel user`.
   Uninstall asks (checkbox, default **off**) whether to also delete
   `%LOCALAPPDATA%\LiveLinguist\models`.

## Done when

The build produces all three files; `makensis` reports 0 warnings or only
documented ones; size ≤ 60 MB. Record in new `docs/qa/2026-11-23-packaging-v1.md`
`## Build`.

Commit message: `packaging: v1 installer and portable zip`
