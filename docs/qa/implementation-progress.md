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
| 8 | W-0 Windows baseline, toolchain, dev-VM smoke | pending | branch | `docs/qa/2026-11-22-windows-vm-smoke.md` |
| 9 | W-5a measured renderer/accelerator probe + GPUIX GPU test-renderer portfolio | pending | branch | `docs/qa/2026-11-22-gpu-renderer.md`, `docs/screenshots/m2-windows/` |
| 10 | W-1a Win32 overlay matrix: policy, focus, drag, 100/125/150% | pending | branch | `docs/qa/2026-11-22-overlay-matrix.md`, `docs/screenshots/m2-windows/overlay/` |

## Milestone mapping

| Milestone | Plan scope | Repo status |
|---|---|---|
| M0 | Windows-first feasibility spikes | S-0 done; S-2/S-5L/S-6L done on Linux dev; S-4 device probe finds an RTX 5060 Ti + CUDA 13.5; S-3A/S-5W/S-6W need models/audio/interactive matrix |
| M1 | Headless parity | `ll-core` + `ll-eval` implemented and tested on Linux; Windows CI job defined |
| M2 | Windows desktop app | UI implemented; Linux portfolio done; Windows GPU test-renderer portfolio + a11y counts captured; overlay/DPI/audio acceptance pending |
| M3 | Windows packaging/release | NSIS + portable ZIP config + release workflow; clean-install test pending Windows hardware |
| M4 | macOS | out of scope (separate decision) |

## Spike ledger

| ID | Result | Date | Evidence |
|---|---|---|---|
| S-0 | PASS — all pins resolve | 2026-11-21 | `docs/qa/2026-11-21-version-verification.md` |
| S-2 | PASS — 100 ev/s x 10 min under Node and Bun; 60k/60k, p99 1.07/1.94 ms, 0 drops | 2026-11-21 | `docs/qa/2026-11-21-event-delivery.md` |
| S-3A | Partial: mock/eval path + 29-row matrix; real export/WER blocked on model access | 2026-11-21 | `docs/qa/2026-11-21-asr-coverage.md` |
| S-4 | Partial: calibration/ladder/queue implemented; mock measured; real GPU pending | 2026-11-21 | `docs/qa/2026-11-21-llm-backends.md` |
| S-5L | Partial: file/demo/silence + ring/VAD/chunker tested; device capture pending | 2026-11-21 | `docs/qa/2026-11-21-audio.md` |
| S-6L | PASS — X11 EWMH overlay verified on Linux dev | 2026-11-21 | `docs/qa/2026-11-21-overlay.md` |
| S-7 | Dev settings + runtime health implemented | — | UI settings |
| S-8 | UI screens captured on Linux dev | 2026-11-21 | `docs/screenshots/m2/` |

## Decisions

All 16 ADRs are present in `docs/decisions/`. New structural changes need a
decision entry and an ADR update.

## Merged PR sequence

1. #2 Stage 1 — S-0 pins, ADRs, agent rules, workspace skeleton
2. #3 Stage 2 — deterministic core parity + 29 packs + golden fixtures
3. #4 Stage 3 — protocol, config, audio, engines, pipeline, eval
4. #5 Stage 4 — napi bridge, native window policy, GPUIX UI, screenshots
5. #6 Stage 5 — final QA, packaging, refreshed UI portfolio

## Next run

The Windows W-0 baseline is verified on the Windows 11 host
(`docs/qa/2026-11-22-windows-vm-smoke.md`): native Rust, UI, bridge, and eval
pass. A DXGI/CUDA probe found an RTX 5060 Ti and CUDA 13.5 (no Vulkan ICD, no
microphone endpoint); the GPU test renderer produced the Windows portfolio and
the dev panel now reports measured renderer values
(`docs/qa/2026-11-22-gpu-renderer.md`).

The agent shell is now in the interactive desktop (session 1), so GUI work runs
directly; `scripts/run-in-interactive-session.ps1` remains for session-0
agents. W-1a passed on the real overlay at 100/125/150% (policy, focus
retention, unfocused click, drag, mode table). Mixed-DPI and multi-monitor are
blocked by the single 2560x1080 display; the harness moves/captures per monitor
when a second one is attached. Next: attach a second display for the mixed-DPI
row, run W-2 audio, and W-4 with a pinned GGUF on the CUDA device. Do not repeat
the W-0 build work.

## Known environment limitations

- **One display.** Mixed-DPI and multi-monitor overlay results cannot be
  produced without a second display (or a virtual/indirect display driver).
  `scripts/overlay-matrix-windows.ps1` records the blocked row and is ready.
- **No microphone endpoint.** Two render endpoints exist; W-2 capture/soak is
  blocked on hardware or a virtual-device decision.
- The agent shell can be in session 0 (services) or the interactive desktop.
  DXGI/CUDA/Rust/UI tests work in either; GPUIX window creation needs the
  desktop, so session-0 agents use `scripts/run-in-interactive-session.ps1`.
- Vulkan has no ICD even with the NVIDIA display driver; CUDA works. Record the
  probe, never the target OS.
- GPUIX 0.11 has no Linux PNG test renderer. UI verification uses a real X11
  window plus `import -window` on Linux, and the native DirectX test renderer on
  Windows.
- Real ASR/LLM model runs require downloads and native builds; the default
  build ships deterministic mock/demo backends so CI and UI remain runnable.
