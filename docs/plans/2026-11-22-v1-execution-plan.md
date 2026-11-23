# Live Linguist PC — v1.0 execution plan

- Date: 2026-11-22
- Status: execution plan; assumes the scope in
  `docs/plans/2026-11-22-v1-scope.md` and ADR 0018.
- Target: Windows 11 x64, one screen, low-end/integrated GPU, system audio,
  CPU-only inference, 29 simplifier languages, click-only simplification.
- Rule: one validated stage per PR, evidence in `docs/qa/` and
  `docs/screenshots/`; do not start a stage whose lower PR is unmerged.

## 0. Current state

| Area | State |
|---|---|
| W-0 baseline | committed; PR #10 open |
| GPU/renderer evidence | committed; PR #11 open (stacked on #10) |
| W-1 single-screen overlay | committed; PR #12 open (stacked on #11) |
| v1.0 scope | committed; PR #13 open (stacked on #12) |
| Audio capture | none; file/demo/silence only |
| ASR | mock/scripted + `ASR_MODEL_MISSING` seam |
| LLM | demo/mock; no llama.cpp, no GGUF |
| UI | implemented; demo/simplify-only paths; single-screen a11y counts pass |
| Packaging | configuration exists; not validated |

## 1. Sequencing

```
Phase 0  land the stack + align docs
   |
   +--> Phase 1 (W-2) loopback capture --------+
   |                                          |
   +--> Phase 2 (W-3) ASR models/coverage ----+--> Phase 4 integration/UI/a11y
   |                                          |         |
   +--> Phase 3 (W-4) CPU 0.6B click-simplify +         v
   |                                                Phase 5 packaging
   |                                                     |
   +--> Phase 4a (W-5) layout/a11y fixes (can start now) v
                                                    Phase 6 release candidate
```

- Phases 1, 2, 3, and the layout half of W-5 can run in parallel.
- Integration waits for loopback + one live ASR language + click simplification.
- Packaging waits for integration; the release candidate waits for packaging.
- ASR model acquisition is the longest lead time; start it first.

## 2. Phase 0 — land the stack and align the docs

**Goal:** the repo has one scope, no contradictory plans, and a green base.

Steps:

1. Merge #10 → #11 → #12 → #13 in order; retarget each to `main` after the
   lower PR lands. Do not squash away the stage boundaries.
2. Update `docs/plans/2026-11-21-live-linguist-pc-master-plan.md` §3 and the
   decision table to match the v1.0 scope doc: CPU-only inference, 29
   simplifier languages, click-only, loopback-first, single-screen.
3. Update `docs/qa/implementation-progress.md` with the v1.0 gates and the
   post-1.0 deferred list.
4. Confirm CI and `scripts/verify-windows.ps1` are green on the merged base.

**Acceptance:** master plan, scope doc, ADRs, and progress board agree; no open
PR carries a superseded requirement.

**Effort:** S. **PR:** none (merge/update only).

## 3. Phase 1 — W-2 WASAPI system-audio loopback

**Goal:** real system audio from the default render endpoint reaches the
existing VAD/chunker pipeline; no microphone required.

Steps:

1. Implement `core/crates/ll-audio/src/capture.rs` for Windows using WASAPI
   loopback (the `windows` crate is already pinned; use the shared-mode default
   render endpoint). Do not add a second audio stack.
2. Negotiate the mix format, convert to 16 kHz mono f32 with `rubato`, and push
   into the existing ring buffer. Keep the callback allocation-free, lock-free,
   log-free, and JS-free.
3. Count overruns and surface `AUDIO_OVERRUN`; never drop a finalized segment
   (INV-006/D-035).
4. Handle device loss, default-device changes, pause/resume, and a missing
   endpoint with a clear unsupported-device state.
5. Wire `listAudioSources`/`selectAudioSource` through `ll-bridge`; show
   **System audio** as the default source in the console; no fake "listening".
6. Test with a non-PII WAV played to the default output (system loopback) and
   with the existing file/demo sources.
7. Run the 30-minute soak with the demo ASR: 0 overruns, no finalized-segment
   loss, p99 callback latency within budget.

**Acceptance:** `docs/qa/2026-11-22-audio-loopback.md` records device, format,
soak numbers, and a screenshot of the source tile; the UI never claims live
audio when capture is unavailable.

**Effort:** M. **PR:** `windows: WASAPI loopback capture`.

## 4. Phase 2 — W-3 streaming ASR and 29-locale coverage

**Goal:** live captions for every locale with a passing model; all 29 locales
remain useful through the simplifier.

This is the longest phase and the main schedule risk. Start model acquisition
before the code lands.

Steps:

1. **Model selection and pins.**
   - Prefer per-language streaming zipformer int8 models from the sherpa-onnx
     model zoo; record exact repo, revision, license, size, and SHA-256.
   - Minimum live set: en, fr, de, es, ar. Evaluate the remaining 24 as
     candidates, not promises.
   - Do not depend on the unreachable `ndgold/nemotron-3.5-asr-streaming-0.6b-onnx`
     artifact; it stays an optional experiment.
   - If a locale has no model that passes the gate, it ships `simplify_only`.
2. **Runtime.** Complete the `sherpa` feature with `OnlineRecognizer`:
   streaming partials, endpointing, finalized segments, cancellation, and an
   explicit `ASR_MODEL_MISSING` state. No fake listening, ever.
3. **Model UX.** Model cards with size, language, license, hash, download,
   resume, disk preflight, and a clear retry on a bad hash. No trust on first
   use.
4. **Evaluation.** Five-minute non-PII fixture per language; report WER, RTF,
   p95 chunk latency, and endpoint behavior. Gate: WER ≤ 15%, RTF ≤ 0.5,
   p95 chunk ≤ 500 ms on a 4-core.
5. **Coverage.** Fill the 29-row matrix; promote a row to `ok` only after its
   measurement. The UI shows live vs simplify-only honestly.
6. **Tests.** Model missing, bad hash, endpointing, multilingual WER, RTF,
   p95 chunk, and a mid-session model switch failure.

**Acceptance:** `docs/qa/2026-11-22-asr-v1.md` with the matrix and hashes;
real live captions in the minimum set; all other rows `simplify_only` with a
measured status.

**Effort:** L. **PRs:** `windows: ASR model catalog and download`, then
`windows: streaming ASR and coverage matrix`.

## 5. Phase 3 — W-4 CPU llama.cpp and click-only simplification

**Goal:** a real Qwen3 0.6B Q4 model simplifies exactly the phrase the user
clicks; no background work.

Steps:

1. Add the `llama` feature with `llama-cpp-2` on CPU; pin the version; build on
   MSVC; no GPU backend modules in the 1.0 installer.
2. Pin the Qwen3 0.6B Q4_K_M GGUF (EasyLanguage fine-tune if licensed, otherwise
   a documented stock fallback), SHA-256, license, and download/resume UX.
3. Implement `LlamaBackend` against `SimplifierBackend`: chat template,
   `enable_thinking=false`, think-block stripping as a safety net, mmap on,
   mlock off.
4. Wire the click-only queue: one active + one queued, newest click replaces
   the queued item, cancel works on active or queued, live/partial text never
   triggers a simplification.
5. Keep the prompt/validator/reversion policy: same-language only; no
   translation; flags on invalid output; verbatim fallback.
6. Measure click-to-simplified p95 ≤ 8 s on an 8 GB / 4-core profile, RSS
   ≤ 3.5 GB, with 2-3 threads.
7. Tests: model missing, bad hash, OOM, timeout, cancel, strict-mode failure,
   and no-background-work assertion.
8. Run a typed **29-pack simplifier matrix** with the real model: every pack
   loads, rewrites in the same language, never translates, and is deterministic
   for the eval fixtures.

**Acceptance:** `docs/qa/2026-11-22-llm-cpu.md` with model hash, p95, RSS, and
thread count; a real click path in the UI; no LLM activity without a click; the
29-pack typed matrix passes.

**Effort:** L. **PR:** `windows: CPU llama.cpp and click-to-simplify`.

## 6. Phase 4 — UI integration, layout, and accessibility (W-5)

**Goal:** the real pipeline works end-to-end in console, overlay, and expanded
mode on one screen, and the UI is usable with keyboard and a screen reader.

Steps:

1. Integrate loopback → ASR → verbatim captions → click → simplification →
   validation → rendered simplified text. Keep the overlay pointer-first.
2. Fix the 150% console/expanded clipping on 1080p: size windows to the monitor
   work area, not a fixed logical height; verify 1280x720 and 1920x1080.
3. Keyboard/Narrator pass on onboarding, console, live, settings, summary,
   licenses; focus order, names, roles, state, and "Read latest". Record the
   GPUIX live-region gap if it remains.
4. RTL: Arabic text renders; full chrome mirroring stays a documented follow-up.
5. Rebuild the screenshot portfolio at 100/125/150 on the single display; no
   multi-monitor claim.
6. Privacy/content-free checks: no transcript in logs, metrics, diagnostics, or
   crash output; no network except the explicit model download.

**Acceptance:** `docs/qa/2026-11-22-v1-ui.md` + screenshots + a11y checklist;
end-to-end session works without the demo backend.

**Effort:** M. **PR:** `windows: v1 UI integration and accessibility`.

## 7. Phase 5 — W-6 packaging and release

**Goal:** a clean Windows 11 machine installs, runs, upgrades, and uninstalls
without admin; models are not bundled.

Steps:

1. Build the per-user NSIS installer and portable ZIP; bundle the addon and
   native DLLs; no models, no GPU backends.
2. Test clean install, launch, system-audio capture, overlay, export, upgrade,
   uninstall, and portable unpack on a clean Windows 11 machine/VM.
3. Verify no admin prompt, no registry leakage outside the per-user hive, and
   model storage under `%LOCALAPPDATA%`.
4. Validate the release workflow on a tag; attach installer/ZIP and hashes.
5. Record the SmartScreen/code-signing decision (unsigned internal beta vs
   signed public build).

**Acceptance:** `docs/qa/2026-11-22-packaging-v1.md`; artifacts attached; a
clean install passes the smoke journey.

**Effort:** M. **PR:** `windows: installer and release`.

## 8. Phase 6 — release candidate

**Goal:** prove the v1.0 promise on a clean, low-end Windows 11 profile.

Steps:

1. Clean install on an 8 GB / 4-core profile (or a CPU-limited VM labelled as
   an approximation).
2. Download the 0.6B and one ASR model; play a non-PII WAV / YouTube video;
   verify live captions, click simplification, overlay, pause/resume, export.
3. Run the 30-minute loopback + ASR + click soak; 0 overruns, no finalized
   segment loss, RSS within budget.
4. Keyboard + Narrator pass, privacy checklist, content-free diagnostics.
5. Freeze docs, ADRs, progress board; tag `v1.0.0`; attach artifacts.

**Acceptance:** the G0-G9 gates in the scope doc all pass with evidence; no
open P0 issue.

**Effort:** M. **PR:** `release: v1.0.0` (or the tag PR).

## 9. PR map and dependencies

| Order | Branch | Scope | Depends on |
|---|---|---|---|
| 1 | `stage-w0-windows-baseline` | W-0 baseline (#10) | — |
| 2 | `stage-gpu-renderer` | GPU evidence (#11) | #10 |
| 3 | `stage-w5-gpu-evidence` | W-1 overlay (#12) | #11 |
| 4 | `docs/v1-scope` | v1.0 scope (#13) | #12 |
| 5 | `stage-w2-loopback` | loopback capture | scope |
| 6 | `stage-w3-asr-model` | model catalog/download | scope (parallel with 5) |
| 7 | `stage-w3-asr` | streaming ASR + coverage | 6 |
| 8 | `stage-w4-llama-cpu` | CPU 0.6B + click path | scope (parallel with 5-7) |
| 9 | `stage-w5-ui-v1` | integration + a11y | 5, 7, 8 |
| 10 | `stage-w6-packaging` | installer/release | 9 |
| 11 | `release-v1.0.0` | tag + artifacts | 10 |

## 10. v1.0 acceptance checklist

- [ ] #10-#13 merged; master plan and progress board aligned.
- [ ] System-audio loopback captures Zoom/Teams/Meet/Skype/YouTube output.
- [ ] 30-minute soak: 0 overruns, no finalized-segment loss.
- [ ] All 29 packs/catalogs present; every locale measured live or simplify-only.
- [ ] Minimum live set en/fr/de/es/ar passes WER/RTF; no fake listening.
- [ ] Qwen3 0.6B Q4 CPU click-to-simplify p95 ≤ 8 s on 8 GB; RSS ≤ 3.5 GB.
- [ ] 29-pack typed simplifier matrix passes; no translation; deterministic.
- [ ] No simplification without a user click; no background LLM work.
- [ ] Single-screen overlay: topmost, no-activate, focus retention, drag,
      borderless fullscreen; 100/125/150% without clipping.
- [ ] Keyboard + Narrator pass; RTL text renders; "Read latest" works.
- [ ] Per-user install/uninstall/portable ZIP; no admin; models not bundled.
- [ ] No cloud, no telemetry, no transcript in logs/exports/diagnostics.
- [ ] Tag `v1.0.0` with installer, portable ZIP, and hashes.

## 11. Risks and stop rules

| Risk | Mitigation / stop rule |
|---|---|
| No acceptable ASR model for a locale | Ship `simplify_only`; never fake listening; record status |
| Loopback capture fails on a device | Clear unsupported-device banner; keep file/typed fallback |
| 0.6B misses p95 on 8 GB | Reduce threads; keep click-only; verbatim fallback with a banner; no GPU rescue |
| llama.cpp MSVC build breaks | Pin a known-good version; vendor/build in CI; no custom kernels |
| Installer cannot load the addon | Portable ZIP remains the supported 1.0 path; fix before release |
| GPUIX has no live-region API | Keep "Read latest"; record upstream gap; do not claim announcement support |
| Model download/privacy regression | Gate on no network except explicit download; hashes verified |

## 12. Deferred after 1.0

Microphone, per-app capture, multi-monitor/mixed-DPI, virtual-desktop pinning,
GPU inference (Vulkan/CUDA), 1.7B, automatic/live simplification, full RTL
mirroring, auto-update, Windows 10, ARM64, Linux/macOS packaging.
