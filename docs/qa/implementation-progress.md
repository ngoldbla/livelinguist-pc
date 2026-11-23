# Implementation progress

Master plan: `docs/plans/2026-11-21-live-linguist-pc-master-plan.md` (v3).
This file is the live status board. Each merged PR is a validated stage.

## Stage board

| Stage | Scope | PR | Status | Evidence |
|---|---|---|---|---|
| 1 | S-0 pins, ADRs, AGENTS, skills, Rust workspace skeleton, CI | #2 | merged | `docs/qa/2026-11-21-version-verification.md` |
| 2 | `ll-core` deterministic parity + 29 packs + golden inventory | #3 | merged | `tests/golden/INVENTORY.md`, 149 tests |
| 3 | `ll-protocol`, `ll-config`, `ll-audio`, `ll-asr`, `ll-llm`, `ll-models`, `ll-pipeline`, `ll-eval` | #4 | merged | `docs/qa/2026-11-21-pipeline.md` |
| 4 | `ll-bridge` napi + envelope + event-delivery harness + X11 window policy + UI screens | #5 | merged | `docs/qa/2026-11-21-ui.md`, `2026-11-21-overlay.md`, `2026-11-21-event-delivery.md` |
| 5 | UI shell: Aurora, onboarding, console, store, i18n | #5 | merged | `docs/screenshots/m2/` |
| 6 | UI overlay, transcript, settings, summary, licenses, dev panel | #5 | merged | `docs/screenshots/m2/` |
| 7 | Packaging, CI release workflow, final QA | #6 | merged | `docs/qa/2026-11-21-packaging.md`, `2026-11-21-final-qa.md` |
| 8 | W-0 Windows baseline, toolchain, dev-VM smoke | #10 | merged | `docs/qa/2026-11-22-windows-vm-smoke.md` |
| 9 | W-5a measured renderer/accelerator probe + GPUIX GPU test-renderer portfolio | #11 | merged | `docs/qa/2026-11-22-gpu-renderer.md`, `docs/screenshots/m2-windows/` |
| 10 | W-1a single-screen Win32 overlay matrix: policy, focus, drag, 100/125/150% | #12 | merged | `docs/qa/2026-11-22-overlay-matrix.md`, `docs/screenshots/m2-windows/overlay/` |
| 11 | v1.0 scope freeze + execution plan | #13 | merged | `docs/plans/2026-11-22-v1-scope.md`, `2026-11-22-v1-execution-plan.md`, ADR 0018 |

## Milestone mapping

| Milestone | Plan scope | Repo status |
|---|---|---|
| M0 | Windows-first feasibility spikes | S-0/S-2/S-5L/S-6L done; DXGI/CUDA probe finds an RTX 5060 Ti (CPU-only for 1.0); S-3A real models and S-5W loopback are next |
| M1 | Headless parity | `ll-core` + `ll-eval` implemented and tested on Linux; Windows CI job defined |
| M2 | Windows desktop app | UI implemented; single-screen overlay + a11y counts pass; Windows portfolio captured; loopback/ASR/LLM integration is the remaining path |
| M3 | Windows packaging/release | NSIS + portable ZIP config + release workflow; clean-install test pending |
| M4 | macOS | out of scope (separate decision) |

## Spike ledger

| ID | Result | Date | Evidence |
|---|---|---|---|
| S-0 | PASS — all pins resolve | 2026-11-21 | `docs/qa/2026-11-21-version-verification.md` |
| S-2 | PASS — 100 ev/s x 10 min under Node and Bun; 60k/60k, p99 1.07/1.94 ms, 0 drops | 2026-11-21 | `docs/qa/2026-11-21-event-delivery.md` |
| S-3A | Partial: model pinned (Nemotron 3.5 560 ms int8, ADR 0019); mock/eval path + 29-row matrix done; runtime and WER blocked on the mirror + `sherpa` wiring | 2026-11-22 | `docs/qa/2026-11-22-model-pins.md` |
| S-4 | Partial: calibration/ladder/queue implemented; **CPU-only 1.0**; mock measured; GPU inference deferred | 2026-11-21 | `docs/qa/2026-11-21-llm-backends.md` |
| S-5L | Partial: file/demo/silence + ring/VAD/chunker tested; WASAPI loopback capture pending | 2026-11-21 | `docs/qa/2026-11-21-audio.md` |
| S-6L | PASS — X11 EWMH overlay verified on Linux dev | 2026-11-21 | `docs/qa/2026-11-21-overlay.md` |
| S-6W | Partial — single-screen Win32 overlay passes (policy, focus, drag, 100/125/150%); multi-monitor/mixed-DPI descoped for 1.0 | 2026-11-22 | `docs/qa/2026-11-22-overlay-matrix.md` |
| S-7 | Dev settings + runtime health implemented | — | UI settings |
| S-8 | UI screens captured on Linux dev | 2026-11-21 | `docs/screenshots/m2/` |

## Decisions

All 19 ADRs are present in `docs/decisions/`. New structural changes need a
decision entry and an ADR update. **For 1.0, D-055/D-056 and
`docs/plans/2026-11-22-v1-scope.md` are the source of truth.**

## Merged PR sequence

1. #2 Stage 1 — S-0 pins, ADRs, agent rules, workspace skeleton
2. #3 Stage 2 — deterministic core parity + 29 packs + golden fixtures
3. #4 Stage 3 — protocol, config, audio, engines, pipeline, eval
4. #5 Stage 4 — napi bridge, native window policy, GPUIX UI, screenshots
5. #6 Stage 5 — final QA, packaging, refreshed UI portfolio
6. #10 W-0 — Windows baseline, toolchain, smoke, Windows CI
7. #11 W-5a — measured renderer/accelerator probe + GPUIX GPU portfolio
8. #12 W-1a — single-screen Win32 overlay matrix (policy, focus, drag, DPI)
9. #13 — v1.0 scope freeze + execution plan

## Next run

The v1.0 model set is now pinned (ADR 0019,
`docs/qa/2026-11-22-model-pins.md`): Nemotron 3.5 streaming 0.6B int8 at 560 ms
for all 29 locales, and `Qwen3-0.6B-EasyLanguage-Q4_K_M.gguf`; both mirrored per
file and downloaded on first run. Start the parallel tracks:

1. **W-2** WASAPI system-audio loopback capture + 30-minute soak.
2. **W-3** wire `sherpa-onnx` `OnlineRecognizer` against the published mirror,
   then WER/RTF per locale and the 29-locale coverage matrix.
3. **W-4** CPU llama.cpp, the exact EasyLanguage prompt contract, click-only
   simplification, p95/RSS.
4. **W-5** layout/a11y fixes (150% clipping, 1280x720, keyboard/Narrator) can
   start independently.

Integration waits for loopback + one live ASR language + click simplification.
Packaging waits for integration. Execution order and gates:
`docs/plans/2026-11-22-v1-execution-plan.md`.

## Known environment limitations

- **One display; multi-monitor/mixed-DPI are descoped for 1.0.** The single-screen
  overlay matrix passes at 100/125/150%. The harness is ready if a second display
  is attached post-1.0.
- **No microphone endpoint; loopback first.** Two render endpoints exist, so
  WASAPI system-audio loopback (the initial call/video use case) is testable.
  Microphone capture is deferred.
- The agent shell can be in session 0 (services) or the interactive desktop.
  DXGI/CUDA/Rust/UI tests work in either; GPUIX window creation needs the
  desktop, so session-0 agents use `scripts/run-in-interactive-session.ps1`.
- Vulkan has no ICD even with the NVIDIA display driver; **1.0 inference is
  CPU-only**, so this is not a 1.0 blocker. Record the probe, never the target OS.
- GPUIX 0.11 has no Linux PNG test renderer. UI verification uses a real X11
  window plus `import -window` on Linux, and the native DirectX test renderer on
  Windows.
- Real ASR/LLM model runs require downloads and native builds; the default
  build ships deterministic mock/demo backends so CI and UI remain runnable.
