# Implementation progress

Execution plan: `docs/plans/2026-11-23-windows-v1-plan.md` (stages S1–S9).
Scope: `docs/plans/2026-11-22-v1-scope.md`. Docs map: `docs/README.md`.
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
| 7 | Packaging, CI release workflow, final QA | #6 | merged | `docs/qa/2026-11-21-packaging.md` |
| 8 | W-0 Windows baseline, toolchain, dev-VM smoke | #10 | merged | `docs/qa/2026-11-22-windows-vm-smoke.md` |
| 9 | W-5a measured renderer/accelerator probe + GPUIX GPU test-renderer portfolio | #11 | merged | `docs/qa/2026-11-22-gpu-renderer.md`, `docs/screenshots/m2-windows/` |
| 10 | W-1a single-screen Win32 overlay matrix: policy, focus, drag, 100/125/150% | #12 | merged | `docs/qa/2026-11-22-overlay-matrix.md`, `docs/screenshots/m2-windows/overlay/` |
| 11 | v1.0 scope freeze + execution plan | #13 | merged | `docs/plans/2026-11-22-v1-scope.md`, ADR 0018 |
| 12 | Master plan, budgets, progress aligned with v1.0 | #14 | merged | `docs/budgets.md` |
| 13 | v1.0 model pins + public ASR mirror | #15 | merged | `docs/qa/2026-11-22-model-pins.md`, ADR 0019 |
| 14 | Docs consolidation + Windows v1 implementation plan | #16 | merged | `docs/README.md`, `docs/plans/2026-11-23-windows-v1-plan.md` |

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
10. #14 — master plan/budgets/progress aligned with v1.0
11. #15 — v1.0 model pins + public ASR mirror
12. #16 — docs consolidation + Windows v1 implementation plan

## v1.0 stage board (plan S1–S9)

| Stage | Plan | Branch | Status |
|---|---|---|---|
| S1 | W-2 WASAPI loopback + 30-min soak | `stage-w2-loopback` | merged (#19) |
| S2 | W-3a sherpa-onnx runtime + first-run model UX | `stage-w3-asr-runtime` | ready |
| S3 | W-3b per-locale WER/RTF, 29-row matrix | `stage-w3-asr-coverage` | needs S2 |
| S4 | W-4a EasyLanguage prompt contract (drop rolling context, 29 goldens) | `stage-w4-prompt-contract` | ready (no model needed) |
| S5 | W-4b CPU llama.cpp click-only simplify, p95/RSS | `stage-w4-llama-cpu` | needs S4 |
| S6 | W-5a 150%/1280x720 layout, W-5b keyboard + Narrator | `stage-w5-layout`, `stage-w5-a11y` | ready |
| S7 | W-5c end-to-end real session + privacy checks (**MVP**; then human pass) | `stage-w5-integration` | needs S1, S2, S5, en live |
| S8 | W-6 NSIS + portable, clean-profile install, unsigned + sums | `stage-w6-packaging` | needs S7 + MVP pass |
| S9 | Release candidate → `v1.0.0` | `release-v1.0.0` | needs S8 + owner go-ahead |

## Next run

Agents: work from `docs/tasks/README.md` — cards T06–T40 cover everything to
`v1.0.0`. T01–T05 (S1) are done.

| Cards | Stage |
|---|---|
| T06–T07 | S1 follow-ups (stateful resampling, device loss) |
| T08–T10 | S4 prompt contract |
| T11–T14 | S2 real ASR |
| T15–T16 | first-run model download |
| T17–T22 | S5 llama.cpp click-to-simplify |
| T23–T28 | S6 layout + a11y |
| T29–T33 | S3 coverage matrix (FLEURS) |
| T34–T36 | S7 integration → **MVP, human pass** |
| T37–T39 | S8 packaging |
| T40 | S9 release `v1.0.0` |

Owner inputs resolved 2026-11-23: FLEURS approved; native Windows 11 boot (no
VM, clean install = fresh standard user); `v1.0.0` tag approved; unsigned
v1.0.0 + SignPath application (ADR 0020). Pending: the MVP human pass (real
call + Narrator) after S7 (plan §14).

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
