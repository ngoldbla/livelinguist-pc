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

## Milestone mapping

| Milestone | Plan scope | Repo status |
|---|---|---|
| M0 | Windows-first feasibility spikes | S-0 done; S-2/S-5L/S-6L and UI spikes done on Linux dev; S-3A/S-4/S-5W/S-6W need Windows hardware/models |
| M1 | Headless parity | `ll-core` + `ll-eval` implemented and tested on Linux; Windows CI job defined |
| M2 | Windows desktop app | UI implemented and captured on Linux dev; Windows acceptance pending hardware |
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

The Windows W-0 baseline is verified on the QEMU dev VM
(`docs/qa/2026-11-22-windows-vm-smoke.md`): native Rust, UI, bridge, and eval
pass, and the GPUIX window renders on the basic display adapter. That VM has no
GPU and no audio devices, so the hardware gates remain open.

Resume on the real Windows 11 host with
`docs/plans/2026-11-21-windows-resume-plan.md` and the copy-paste
`docs/plans/2026-11-21-windows-resume-prompt.md`. Start with W-1 on hardware,
then W-2, and do not repeat the W-0 build work.

## Known environment limitations

- The current Windows session is a QEMU dev VM with the Microsoft Basic Display
  Adapter and no audio devices. Build and logic gates pass; W-1/W-2/W-5
  hardware acceptance still requires the real D-051 host.
- No Windows 11 release host is reachable from the original Linux session.
  Win32/WASAPI code is target-gated and compiled by the Windows CI job; the
  real-hardware acceptance matrix (S-6W/S-5W) is recorded as pending, not
  claimed.
- GPUIX 0.11 has no Linux PNG test renderer. UI verification uses a real X11
  window plus `import -window`; logic uses `bun test`.
- Real ASR/LLM model runs require downloads and native builds; the default
  build ships deterministic mock/demo backends so CI and UI remain runnable.
