# Windows 11 dev-VM smoke (W-0 baseline)

- Date: 2026-11-22
- Status: toolchain and smoke **pass** on the QEMU dev VM; Windows hardware
  acceptance is **not claimed**.
- Branch: `stage-w0-windows-baseline`
- Plan: `docs/plans/2026-11-21-windows-resume-plan.md` (W-0)

## Machine

| Item | Value |
|---|---|
| OS | Windows 11 Pro, 10.0.26300, x64 |
| Machine | QEMU Standard PC (Q35 + ICH9, 2009) |
| CPU | AMD Ryzen 9 3900X, 6 cores / 6 logical (VM) |
| RAM | 64 GB |
| GPU | Microsoft Basic Display Adapter (driver 10.0.26100.1) |
| Display | 1280 x 800 |
| Audio | no `Win32_SoundDevice` entries |
| Disk | 430.7 GB free on C: |

This is a development VM, not the D-051 reference hardware. Results are useful
for build and logic verification; they do not satisfy the Win32 overlay,
WASAPI, or D3D hardware gates.

## Toolchain

| Tool | Version / path |
|---|---|
| Visual Studio Build Tools 2022 | 17.14.45 (17.14.37906.3), VCTools workload + recommended |
| MSVC + Windows SDK | installed under the Build Tools root |
| Rust | 1.98.0 `x86_64-pc-windows-msvc` (pinned by `rust-toolchain.toml`) |
| rustup | 1.29.3 |
| Bun | 1.4.5 |
| Node | 22.23.5 |
| CMake | 4.4.4 |
| PowerShell | 7.7.0 |
| Git | present |
| NSIS (`makensis`) | **not installed** (packaging only, W-6) |
| `cargo-packager` | **not installed** (packaging only, W-6) |

Bun, Node, CMake, and PowerShell were installed user-scope; no admin is needed
for them. Visual Studio Build Tools is machine-wide and requires an
administrator. The agent could not raise the UAC dialog from its desktop; the
install was performed by the operator in an elevated terminal.

## Results

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `cargo test --workspace` | **237 passed, 0 failed** (incl. 143 `ll-core` and 6 golden) |
| `bun install --frozen-lockfile` | pass |
| `bun run typecheck` | pass |
| `bun test` | 12 passed, 0 failed |
| `bun run check` (Biome) | 23 files checked, no fixes |
| `cargo build -p ll-bridge --release` | pass (`livelinguist_core.dll`) |
| `node tests/bridge/bridge-smoke.mjs` | 223 envelopes, 9 segments, 1 simplified |
| `node tests/bridge/event-delivery.mjs 10` | 1000/1000, 0 dropped, p99 0.67 ms (latest run) |
| `bun tests/bridge/event-delivery.mjs 10` | 1000/1000, 0 dropped, p99 2.06 ms (latest run; 0.55 ms earlier) |
| `ll-eval --scenario mock --language fr-FR` | 9 segments, 0 overruns, content-free |
| `ll-eval --scenario mock --wav tests/fixtures/sample.wav` | 1 segment, 0 overruns |
| `ll-eval --scenario simplify-only --text ...` | 1 segment, 1 simplified |

Biome reports a non-blocking deprecation for `linter.recommended` in
`ui/biome.json`; the config should be migrated in a follow-up.

## GPUIX window

The UI was launched with `LL_SCREEN=console` and the native addon staged at
`ui/livelinguist-core.node`. GPUIX reported:

```text
[gpuix] created native window
[gpuix] mount complete
```

The console screen rendered with the native bridge loaded: language card,
model cards, on-demand/live tabs, caption settings, and the `Simplify only`
badge. See `docs/screenshots/m2-windows/console-dev-vm.png`.

The adapter is the Microsoft Basic Display Adapter, so this is software-class
rendering. It proves the window and native path start on a GPU-less Windows VM;
it is not a D3D11/12 hardware, frame-rate, overlay-over-fullscreen, or
multi-monitor acceptance.

## Known limitations and open gates

- **No GPU.** W-1 (Win32 overlay over full-screen video, mixed DPI,
  multi-monitor) and W-5 (D3D hardware, performance, accessibility captures)
  still need the real Windows 11 host.
- **No audio devices.** W-2 (WASAPI microphone/loopback and the 30-minute soak)
  cannot run on this VM.
- **No real models.** ASR and LLM paths remain mock/demo; W-3/W-4 need pinned
  artifacts and downloads.
- **No packaging tools.** W-6 needs NSIS (or `cargo-packager`) and a clean
  install target.

## Reproduce

From a native PowerShell in the repo root:

```powershell
pwsh -File scripts/verify-windows.ps1
```

Use `-SkipUi`, `-SkipEval`, or `-SkipBridge` for a partial run. The script needs
`cargo`, `rustc`, `bun`, and `node` on `PATH`; open a new terminal after
installing them.

## Line endings

The repository enforces LF via `.gitattributes`. If a Windows clone was created
before that file existed and `core.autocrlf=true`, Biome will report every file
as unformatted. Fix it with:

```powershell
git config core.autocrlf false
git config core.eol lf
git rm --cached -r ui/src ui/scripts
git reset --hard
```

(or simply re-clone after the `.gitattributes` change).
