# Implementation progress

Master plan: `docs/plans/2026-11-21-live-linguist-pc-master-plan.md` (v3).
This file is the live status board. Each merged PR is a validated stage.

## Stage board

| Stage | Scope | PR | Status | Evidence |
|---|---|---|---|---|
| 1 | S-0 pins, ADRs, AGENTS, skills, Rust workspace skeleton, CI | #2 | in progress | `docs/qa/2026-11-21-version-verification.md` |
| 2 | `ll-core` deterministic parity + 29 packs + golden inventory | — | pending | — |
| 3 | `ll-protocol`, `ll-config`, `ll-audio`, `ll-asr`, `ll-llm`, `ll-models`, `ll-pipeline`, `ll-eval` | — | pending | — |
| 4 | `ll-bridge` napi + envelope + event-delivery harness + X11 window policy | — | pending | — |
| 5 | UI shell: Aurora, onboarding, console, store, i18n | — | pending | — |
| 6 | UI overlay, transcript, settings, summary, licenses, dev panel | — | pending | — |
| 7 | Packaging, CI release workflow, final QA | — | pending | — |

## Milestone mapping

| Milestone | Plan scope | Repo status |
|---|---|---|
| M0 | Windows-first feasibility spikes | S-0 done; S-2/S-5L/S-6L and UI spikes done on Linux dev; S-3A/S-4/S-5W/S-6W need Windows hardware/models |
| M1 | Headless parity | `ll-core` + `ll-eval` implemented and tested on Linux; Windows CI job defined |
| M2 | Windows desktop app | UI implemented; Linux dev captures; Windows acceptance pending hardware |
| M3 | Windows packaging/release | NSIS + portable ZIP config + release workflow; clean-install test pending Windows hardware |
| M4 | macOS | out of scope (separate decision) |

## Spike ledger

| ID | Result | Date | Evidence |
|---|---|---|---|
| S-0 | PASS — all pins resolve | 2026-11-21 | `docs/qa/2026-11-21-version-verification.md` |
| S-2 | Linux dev harness: 100 ev/s x 10 min, p50/p99 recorded | — | `docs/qa/2026-11-21-event-delivery.md` |
| S-3A | Mock/eval path implemented; real model export blocked on model access/hardware | — | `docs/qa/2026-11-21-asr-coverage.md` |
| S-4 | Calibration ladder + mock measured; real backends need Windows GPU | — | `docs/qa/2026-11-21-llm-backends.md` |
| S-5L | File/demo/silence + PipeWire probe on Linux dev | — | `docs/qa/2026-11-21-audio.md` |
| S-6L | X11 EWMH overlay verified on Linux dev | — | `docs/qa/2026-11-21-overlay.md` |
| S-7 | Dev settings + runtime health implemented | — | UI settings |
| S-8 | UI screens captured on Linux dev | — | `docs/screenshots/` |

## Decisions

All 16 ADRs are present in `docs/decisions/`. New structural changes need a
decision entry and an ADR update.

## Known environment limitations

- No Windows 11 host is reachable from this Linux session. Win32/WASAPI
  code is target-gated and compiled by the Windows CI job; the real-hardware
  acceptance matrix (S-6W/S-5W) is recorded as pending, not claimed.
- GPUIX 0.11 has no Linux PNG test renderer. UI verification uses a real X11
  window plus `import -window`; logic uses `bun test`.
- Real ASR/LLM model runs require downloads and native builds; the default
  build ships deterministic mock/demo backends so CI and UI remain runnable.
