# Public release plan: open source (MIT) + a foolproof Windows download

- Date: 2026-11-25
- Decision: ADR 0023 (D-060), with ADR 0020 (signing) and ADR 0021 (layout)
- Fits into: S8 (T37–T39) and S9 (T40) of `2026-11-23-windows-v1-plan.md`

## The target experience

A regular person on Windows 11 who has never used GitHub:

1. Opens the project page or a link, clicks **Download for Windows**.
2. Opens `LiveLinguist-Setup-1.0.0.exe`. No admin prompt, no choices to make,
   and the app starts on its own when setup finishes.
3. The app says "Download the language models (1.1 GB)" with one button, a
   progress bar, and resume after a dropped connection. (Done: T16.)
4. They play a video and see captions.

Anything that can break this path gets a plain-language message on screen,
never a missing-DLL dialog, a silent exit, or a console window.

## What already works

- Per-user NSIS installer with no admin rights, Start menu and desktop
  shortcuts, and an uninstaller (`packaging/nsis/livelinguist.nsi`).
- Static CRT for Rust, llama.cpp and sherpa-onnx (`.cargo/config.toml`), so no
  Visual C++ Redistributable is needed.
- Models are not bundled; they download on first run, are hash-checked and
  resume (T15/T16).
- A tag-triggered release workflow builds on `windows-latest`
  (`.github/workflows/release.yml`).
- No secrets in the 109-commit history (scan 2026-11-25, ADR 0023).

## Gaps found (2026-11-25 review)

### Release workflow (`.github/workflows/release.yml`)

- `gh release upload` fails when no release exists for the tag. Create a
  **draft** release in the job and publish it by hand after a smoke test.
- The version is hard-coded `0.1.0` in `livelinguist.nsi` and
  `build-windows.ps1`. Derive it from the tag (`makensis /DVERSION=...`).
- No `SHA256SUMS.txt` and no `actions/attest-build-provenance` step (ADR 0020
  promises published sums).
- The GPUIX `.node` is not copied yet (T37, ADR 0021).
- Pin third-party actions by commit SHA before going public.

### Installer (`packaging/nsis/livelinguist.nsi`)

- There is no "Launch Live Linguist" step on the finish page. Add
  `MUI_FINISHPAGE_RUN` (checked by default).
- Drop the directory page. Fewer choices means fewer ways to get it wrong.
- Upgrading while the app is running fails with "Error opening file for
  writing". Detect the running app and ask the user to close it, or close it.
- On Windows 10, or on ARM, block setup with a friendly message ("Live
  Linguist needs Windows 11 on a 64-bit Intel or AMD PC"); don't let it fail at
  runtime.
- There is no app icon yet (installer, exe, shortcuts, Add/Remove Programs).
  Add `MUI_ICON`, `DisplayIcon`, `EstimatedSize` and `URLInfoAbout`.
- The uninstaller deletes files by name, so it will miss the GPUIX `.node`.
  Remove the program files, and ask "Also delete downloaded language models
  (1.1 GB)?" (default **no**, so a reinstall doesn't download them again).
  Models live in `%LOCALAPPDATA%\LiveLinguist\models`, inside the install
  folder.

### Runs on other people's PCs, not only the reference PC

- **CPU instructions.** llama.cpp is built for baseline x86-64, because no
  `target-cpu` is set. That is safe everywhere but may be slower than it needs
  to be. Never ship `target-cpu=native`: the CI runner's CPU is not the user's.
  If click latency needs AVX2, use llama.cpp's runtime CPU-variant dispatch
  rather than raising the baseline. Some Windows 11 Pentium and Celeron chips
  lack AVX2. This change needs an ADR and a tier C re-measure.
- **Test on a truly clean machine.** T38's clean *user profile* on the dev PC
  still has the drivers, runtimes and redistributables the developer
  installed. Also run the installer in **Windows Sandbox**, or a fresh VM, so a
  missing DLL or font would show up.
- **Packaged load failures** must show an error screen with "Copy diagnostics"
  (ADR 0021), never fall back to the mock silently or exit with nothing shown.
- **Minimum system** in the README and release notes: Windows 11 23H2+ x64,
  8 GB RAM (the app uses about 1.6 GB), about 2.5 GB free disk, a DirectX 11
  GPU (any normal PC), and internet once for the model download.

### Trust and warnings

- **SmartScreen** shows "Windows protected your PC" for an unsigned exe.
  Until signing lands, the release notes and README show one screenshot with
  the two clicks ("More info" → "Run anyway") and the SHA-256 sum.
- **Signing.** Apply to SignPath Foundation the day the repo goes public; it
  requires a public OSI repo built in CI, which this plan provides. Once
  accepted, sign both the installer and `livelinguist.exe`.
- **Defender false positives.** Unsigned compiled-JS executables are sometimes
  flagged. Submit each release's installer to the Microsoft Security
  Intelligence portal before announcing it.

### Project page for non-technical visitors

- The top of `README.md` gets a short **Download for Windows** section with
  the latest-release link (`releases/latest`), the minimum system, and the
  SmartScreen picture. Developer docs move below it.
- `.github/ISSUE_TEMPLATE/problem.yml`: a plain-language "Something didn't
  work" form that asks for Windows version, what they clicked, and the
  content-free diagnostics export. It says never to paste transcripts.
- Add `SECURITY.md` (private vulnerability reporting) and a short
  `CONTRIBUTING.md` that points to `AGENTS.md`.

## Owner steps to go public

1. Confirm you hold the rights to publish `assets/packs/` under MIT (ADR 0023,
   open question).
2. Merge the license PR.
3. GitHub → Settings → General → Change visibility → Public.
4. Settings → Code security: turn on secret scanning with push protection,
   Dependabot alerts, and private vulnerability reporting.
5. Protect `main` (require a PR and green CI).
6. Apply to SignPath Foundation (ADR 0020).

## Task cards to cut (agent work)

| Card | What | Folds into |
|---|---|---|
| R1 | Release workflow: draft release, version from tag, `SHA256SUMS.txt`, attestation, SHA-pinned actions | T39 |
| R2 | Installer polish: launch on finish, no directory page, close running app, Windows 11 x64 check, icon, uninstall cleanup with models prompt | T37 |
| R3 | Packaged load-failure screen with "Copy diagnostics" | T34/T37 |
| R4 | Clean-machine test in Windows Sandbox, alongside T38's clean profile | T38 |
| R5 | README download section, SmartScreen picture, issue form, `SECURITY.md`, `CONTRIBUTING.md` | T39 |
| R6 | SignPath signing step, gated on the action being configured (no secrets in the repo) | after acceptance |
| R7 | CPU-variant dispatch for llama.cpp: ADR, then re-measure tier C | optional, after 1.0 |

The existing gates still hold: packaging (T37) waits for the MVP human pass,
and `v1.0.0` (T40) waits for the owner's go-ahead.
