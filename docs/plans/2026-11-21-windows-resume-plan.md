# Windows 11 resume plan (handoff)

- Date: 2026-11-21
- Status: handoff / execution plan
- Baseline: `main` after PR #7, commits #2–#7 merged
- Master plan: `docs/plans/2026-11-21-live-linguist-pc-master-plan.md` (v3)
- Resume prompt: `docs/plans/2026-11-21-windows-resume-prompt.md`

This plan takes over on the **Windows 11 x64 side of the dual boot**. Linux dev
work is finished for the current baseline; do not repeat it. The goal is to
close the Windows release gates that need real hardware, real audio, real
D3D, and real model artifacts.

## 0. Current baseline (do not rebuild)

| Area | State on `main` | Evidence |
|---|---|---|
| Core deterministic layer | 29 packs, prompts, segmenter, validators, reversion, metrics, exports | `tests/golden/INVENTORY.md`; `cargo test -p ll-core` |
| Headless pipeline + eval | protocol/config/audio/ASR/LLM/models/pipeline/`ll-eval` | `docs/qa/2026-11-21-pipeline.md` |
| napi bridge + events | `CoreClient`, TSFN, `{v,seq,ts}`, 100 ev/s x 10 min pass | `docs/qa/2026-11-21-event-delivery.md` |
| UI | onboarding, console, live/expanded, overlay, settings, dev panel, summary, licenses | `docs/screenshots/m2/` |
| X11 overlay | verified EWMH policy on Linux dev | `docs/qa/2026-11-21-overlay.md` |
| Win32 overlay | raw-FFI policy compiles for MSVC; **not run on hardware** | `docs/qa/2026-11-21-windows.md` |
| Packaging | NSIS script, cargo-packager config, build workflow; **not validated** | `docs/qa/2026-11-21-packaging.md` |
| Real ASR/LLM | **not implemented end-to-end**; feature seams return model-missing/unavailable | `docs/qa/2026-11-21-asr-coverage.md`, `2026-11-21-llm-backends.md` |
| Device audio | file/demo/silence only; **WASAPI and PipeWire capture not implemented** | `docs/qa/2026-11-21-audio.md` |

The Linux baseline is green: `./scripts/verify.sh` passes, including the
10-minute TSFN soak. Preserve every existing test and golden fixture.

## 1. Definition of done

The Windows takeover is done when all of these are true and evidenced:

1. **S-6W** Win32 overlay matrix passes on real Windows 11 23H2+ hardware
   (topmost over full-screen video/call, focus retention, unfocused
   click/hover, drag, mode switch, 100/125/150% DPI, multi-monitor, virtual
   desktops).
2. **S-5W** microphone + WASAPI loopback capture real call/video audio; a
   30-minute soak at the tier C thread budget has xrun/overrun = 0 and no
   finalized-segment loss.
3. **S-3A** a real streaming ASR model loads through `sherpa-onnx`, streams
   with endpointing, and a WER matrix records at least de/fr/es/en/ar; the
   29-row coverage matrix is filled with measured statuses (no remaining `tbd`
   claimed as `ok`).
4. **S-4** a real Qwen3 EasyLanguage GGUF runs through `llama-cpp-2` on CPU and
   Vulkan (CUDA if available), calibration picks a backend, and TTFT/tokens/s
   are recorded on tiers A/B/C.
5. **M2** a real on-demand session runs through the Windows UI using D3D11/12,
   with keyboard and Narrator/NVDA passes and fresh Windows screenshots.
6. **M3** NSIS install, upgrade, uninstall, and portable ZIP pass on a clean
   Windows 11 machine and on the dual-boot Windows side; release artifacts are
   attached to a tag.
7. Progress board, ADRs, QA evidence, and this plan are updated in the repo;
   every validated stage is merged as its own PR.

## 2. Environment setup (W-0)

### Hardware

- Windows 11 x64, 23H2 or newer, on Dylan PC (D-051).
- Prefer the discrete GPU and a real microphone + speakers/headset.
- Record exact CPU, GPU, RAM, Windows build, GPU driver, display scaling,
  audio devices, and whether the dGPU is enabled. Fill
  `docs/qa/2026-11-21-windows-hardware.md` (create it).

### Toolchain

Install natively (not WSL):

- Visual Studio Build Tools 2022: **Desktop development with C++**, Windows 11
  SDK, MSVC x64 tools, CMake, LLVM/Clang if the GPU backend build asks for it.
- Rust stable with the MSVC toolchain; verify `rustup show` says
  `x86_64-pc-windows-msvc`.
- Bun **1.4.5**, Node **22.x**, Git, PowerShell 7, NSIS (`makensis` on PATH),
  optionally `cargo-packager`.
- Vulkan runtime/driver for the GPU path; CUDA toolkit only if testing the
  optional CUDA pack.

### First build and smoke

Run from a native PowerShell in the repo root:

```powershell
rustup show
cargo test --workspace
cd ui
bun install --frozen-lockfile
bun run typecheck
bun test
bun run check
cd ..
cargo build -p ll-bridge --release
Copy-Item target\release\livelinguist_core.dll ui\livelinguist-core.node
cd ui
bun src/main.tsx
```

Expected: a D3D-backed GPUIX window opens, the console screen is usable, the
native bridge loads, and the demo conversation starts. If the window does not
open, record the exact error and stop this gate before changing policy code.

Then:

```powershell
cargo run -p ll-eval -- --scenario mock --language fr-FR --out target\eval-mock
```

Expected: 9 segments, content-free diagnostics, 0 overruns. This proves the
same pipeline baseline on Windows.

### W-0 deliverables

- `docs/qa/2026-11-21-windows-hardware.md` with the hardware/OS/driver record.
- A new `scripts/verify-windows.ps1` mirroring `scripts/verify.sh` (PowerShell,
  no Bash dependency), or an explicitly documented equivalent command list.
- Windows UI CI: add a `windows-latest` job that installs the UI deps and runs
  `bun run typecheck`, `bun test`, `bun run check` (logic only; no window).
- First Windows screenshots in `docs/screenshots/m2-windows/`:
  console, live, overlay. Do not overwrite the Linux portfolio.
- PR: **`windows: baseline environment and smoke`**.
- Gate: all commands above pass; window screenshot exists; no code behavior
  changes beyond build fixes.

## 3. Workstreams

### W-1 — S-6W Win32 overlay (start here; highest product risk)

**Why first:** the overlay is the Windows release gate and current Win32 code
has never touched a real HWND.

Tasks:

1. Launch a real session in overlay mode; verify the bridge finds the HWND by
   process id + title nonce and applies `HWND_TOPMOST`,
   `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW`, and the `WM_MOUSEACTIVATE -> MA_NOACTIVATE`
   subclass.
2. Test topmost over a full-screen video and a real call app; test focus
   retention while typing in another app; test click/hover while unfocused.
3. Test drag (`ReleaseCapture` + `WM_NCLBUTTONDOWN/HTCAPTION`), mode switches
   (console/overlay/expanded), 100/125/150% DPI, multi-monitor with mixed DPI,
   and virtual desktops.
4. Add the window resize/placement step for mode changes if GPUIX does not
   resize automatically. Record whether GPUIX re-layouts after an external
   `SetWindowPos`.
5. Add an automated Win32 integration test where possible (window creation +
   `GetWindowLongPtrW`/`GetWindowRect` checks); keep the real acceptance manual.
6. If topmost/no-activate cannot work, fall back to a normal window plus an
   in-app banner (D-010); never silently claim overlay success.
7. Update ADR 0004/0008 and `docs/qa/2026-11-21-overlay.md` with the Windows
   matrix and exact evidence.

**Code touchpoints:** `core/crates/ll-window/src/win32.rs`,
`core/crates/ll-bridge/src/lib.rs` (`setWindowMode`),
`ui/src/App.tsx` / `screens/Overlay.tsx`, `ui/scripts/screenshots-windows.ts`.

**Acceptance:** the plan §10.7 Windows rows all pass, with screenshots and a
signed-off matrix. PR: **`windows: Win32 overlay acceptance`**.

### W-2 — S-5W WASAPI microphone + loopback

Current `ll-audio` has no capture implementation (the `capture` feature is
declared but empty). This is new code, not a port.

Tasks:

1. Implement `core/crates/ll-audio/src/capture.rs` with cpal or the
   `windows`/WASAPI crate (S-5W decision). Enumerate input devices and the
   default output's loopback; select mic vs system audio in the UI.
2. Push 16 kHz mono f32 through the existing VAD/chunker/ring buffer; count
   overruns; keep the callback allocation/lock/log/JS-free.
3. Wire `listAudioSources`/`selectAudioSource` through the bridge and the
   console source tiles (currently demo/silence only).
4. Test with a real call/video: mic + loopback, device loss, pause/resume.
5. Run the 30-minute soak at the tier C thread budget with the LLM simplifying
   concurrently. Pass: xrun/overrun = 0, no finalized-segment loss, p99 capture
   callback latency within budget.
6. Record the device matrix and soak numbers in
   `docs/qa/2026-11-21-audio.md`; no audio content in logs.

**Code touchpoints:** `ll-audio`, `ll-pipeline::feed_audio`,
`ll-bridge::listAudioSources`, UI Console/Live, `ll-config` audio modes.
**Acceptance:** real mic and loopback sessions end-to-end; soak passes. PR:
**`windows: WASAPI capture and soak`**.

### W-3 — S-3A real ASR export, sherpa-onnx, coverage

Current `SherpaAsr` is a placeholder that returns `ASR_MODEL_MISSING` unless a
model is present and otherwise says wiring is pending. This workstream closes
that.

Tasks:

1. Resolve the planned artifact `ndgold/nemotron-3.5-asr-streaming-0.6b-onnx`
   and its license. If unavailable, choose the fallback per-language streaming
   zipformer/Parakeet and record the decision.
2. Complete the ONNX export and verify it loads in the Rust `sherpa-onnx`
   `OnlineRecognizer`; implement streaming partials + endpointing behind the
   `sherpa` feature.
3. Build/evaluate at least de/fr/es/en/ar; record WER and RTF against non-PII
   fixtures; int8 delta <= 2 pp vs fp32.
4. Fill the 29-row matrix in `docs/qa/2026-11-21-asr-coverage.md` and update
   `CoverageMatrix` statuses. Any locale without a passing model stays
   `simplify_only`; never show a fake listening state.
5. Implement model install/download + pinned SHA-256 + `ASR_MODEL_MISSING`
   download UX in the UI. No trust-on-first-use.
6. Tests: model-missing, bad hash, endpointing, multilingual WER, RTF, p95
   chunk latency.

**Code touchpoints:** `ll-asr`, `ll-models` catalog/hashes, `ll-pipeline`,
bridge download commands, UI language picker/model cards.
**Acceptance:** real streaming transcript on the Windows host + matrix.
PR: **`windows: streaming ASR and coverage matrix`**.

### W-4 — S-4 real llama.cpp backends and calibration

There is no `llama` feature or `LlamaBackend` in `ll-llm` yet; the demo/mock
path is complete.

Tasks:

1. Add `llama-cpp-2 = 0.1.161` with `dynamic-backends`, optional `vulkan`/`cuda`
   features; build `backends/cpu`, `backends/cpu-vulkan`, optional
   `backends/cpu-cuda`; `load_backends_from_path()` before init.
2. Implement `LlamaBackend` against the existing `SimplifierBackend` trait;
   Qwen3 chat template with `enable_thinking=false`; think-block stripping as a
   safety net.
3. Load a pinned Qwen3 0.6B/1.7B EasyLanguage GGUF (or a documented stock
   fallback), verify SHA-256, and wire the model catalog/download UX.
4. Enumerate devices; calibrate with a fixed prompt, 32 tokens, temperature 0;
   cache by hardware/driver/model hash/backend version; implement the 2-D
   ladder and prefix KV cache (already policy-tested).
5. Measure TTFT, prefill, tokens/s, p50/p95 on tiers A/B/C, CPU vs Vulkan vs
   CUDA if present. Record in `docs/qa/2026-11-21-llm-backends.md`.
6. Test OOM/fallback, queue ordering, cancel, strict mode. Strict mode must
   fail loudly, not silently fall back.

**Code touchpoints:** `ll-llm`, `ll-models`, `ll-config`, bridge status,
UI model cards/dev panel.
**Acceptance:** real on-demand simplification in the Windows UI with measured
numbers and a documented backend choice. PR:
**`windows: llama.cpp backends and calibration`**.

### W-5 — M2 UI, D3D, accessibility, screenshots

Tasks:

1. Verify the GPUIX window uses D3D11/12 (record the actual renderer/backend;
   `RuntimeStatus` should not be a hardcoded claim).
2. Port the screenshot script to Windows: GPUIX `launch` + `screenshot` if the
   installed build has the test-support renderer; otherwise use a documented
   Win32 capture path. Capture onboarding, console, live, overlay, settings,
   summary, licenses into `docs/screenshots/m2-windows/`.
3. Run keyboard-only and Narrator/NVDA passes; fix focus order, names, roles,
   state, and the "Read latest" fallback. Check contrast and 100/125/150%
   scaling.
4. RTL: Arabic text/locale is in place; full row mirroring is partial. Either
   implement a direction wrapper across screens or record a scoped follow-up
   with screenshots.
5. Memory-only/privacy claims must remain literally true.

**Acceptance:** real Windows screenshots + a11y checklist; no regressions in
`ui` tests. PR: **`windows: D3D UI and accessibility acceptance`**.

### W-6 — M3 packaging and release

Tasks:

1. Run `scripts/build-windows.ps1` on Windows; produce
   `packaging/nsis/LiveLinguist-0.1.0-setup.exe` and the portable ZIP.
2. Test clean install, launch, microphone permission (no admin), upgrade,
   uninstall, and portable unpack on a clean Windows 11 machine and the
   dual-boot side.
3. Verify WASAPI loopback and overlay from the installed app; verify file
   dialogs/export paths.
4. Validate the release workflow on a tag; attach installer/ZIP; record hashes.
5. Confirm models are not bundled and the download flow contacts only Hugging
   Face on explicit user action.
6. Update `docs/qa/2026-11-21-packaging.md` with results and screenshots.

**Acceptance:** clean-install pass; artifacts attached; no admin requirement.
PR: **`windows: installer and release acceptance`**.

## 4. Suggested merge sequence

| PR | Scope | Blocks | Evidence |
|---|---|---|---|
| 1 | W-0 baseline/hardware/toolchain | all | hardware doc, smoke screenshot |
| 2 | W-1 Win32 overlay | M2/M3 | overlay matrix + screenshots |
| 3 | W-2 WASAPI capture | M2/M3 | device matrix + 30-min soak |
| 4 | W-3 real ASR/coverage | M2 | WER matrix + model hashes |
| 5 | W-4 llama.cpp/calibration | M2/M3 | TTFT/tok/s + ladder |
| 6 | W-5 UI/a11y/screenshots | M2 | Windows gallery + a11y pass |
| 7 | W-6 installer/release | M3 | clean install + artifacts |

Do not merge a stage with a failing gate. If a stage is blocked by missing
artifacts or hardware, leave it open, record the exact blocker, and move to a
stage that is not blocked (for example W-1 while waiting on ASR models).

## 5. Evidence templates

Copy into the matching QA file for every measurement:

```text
Date:
Machine: Dylan PC Windows 11 (build ...)
CPU / GPU / RAM:
GPU driver / Vulkan / CUDA:
Display scaling / monitors:
Audio devices:
Build commit:
Command:
Result (numbers, pass/fail):
Screenshots/artifacts:
Notes/fallbacks:
```

Overlay matrix (plan §10.7): topmost, focus retention, unfocused click/hover,
drag, mode switch, DPI, multi-monitor, full-screen video/call, virtual desktops.

Audio soak: duration, source, thread budget, xrun/overrun count, finalized
segments before/after, callback p99, degradation events.

ASR: locale, model, revision/hash, fixture, WER, RTF, p95 chunk latency,
endpoint behavior.

LLM: model/quant/hash, backend/device/layers, TTFT, prefill, tokens/s, p50/p95,
peak RSS, fallback reason.

## 6. Decision rules and blockers

- **No model artifacts:** do not invent hashes or claim WER/TTFT. Stop at the
  model gate, use the documented fallback if one is licensed, and ask the
  product owner for the pinned artifact or permission to publish it.
- **No discrete GPU / driver issues:** run tier B/C, record honestly, and do not
  claim GPU numbers. Strict GPU scenarios fail loudly.
- **WASAPI loopback fails:** fall back to the documented virtual-device route
  only if it is a product decision; otherwise block and record the app/driver
  combination that failed.
- **D3D/WARP:** WARP is a degraded best-effort, never a 60 fps claim.
- **No transcript leakage:** keep logs/diagnostics/errors content-free. Any
  regression is a release blocker.
- **No silent fallback:** strict mode fails; normal mode shows a banner and
  records a degradation event.

## 7. Handoff rules for the next agent

- Read `AGENTS.md`, the master plan, `docs/qa/implementation-progress.md`, and
  this plan before changing code.
- The Linux baseline is already merged. Do not redo core/UI work; port only
  what the Windows gates require.
- One validated stage per PR, with screenshots for UI stages and evidence in
  `docs/qa/`.
- Update the progress board and ADRs when a structural choice changes.
- End every session with a status block: done, verified, remaining, needs.
- If blocked, say exactly which hardware/model/decision is missing and stop
  rather than faking a pass.
