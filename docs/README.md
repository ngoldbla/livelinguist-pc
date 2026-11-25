# Docs map

Start here. Everything in `docs/` is one of four kinds: **what to do next**,
**what we decided**, **what we measured**, or **reference**.

## 1. What to do next

| File | Use it for |
|---|---|
| [`plans/2026-11-23-windows-v1-plan.md`](plans/2026-11-23-windows-v1-plan.md) | **The execution plan.** Stages S1–S9 to ship v1.0 on Windows: what to build, which files, which tests, exit gates. |
| [`tasks/README.md`](tasks/README.md) | **Task cards:** small one-session work orders cut from the plan. Agents work from these. |
| [`plans/2026-11-23-windows-agent-prompt.md`](plans/2026-11-23-windows-agent-prompt.md) | Copy-paste prompts for the Windows host agent (short card-based prompt + full prompt). |
| [`plans/2026-11-24-autoresearch-plan.md`](plans/2026-11-24-autoresearch-plan.md) | Post-v1 caption-latency baseline and CPU-first autoresearch loop. |
| [`qa/implementation-progress.md`](qa/implementation-progress.md) | Live status board: merged stages, current stage, next run. |
| [`plans/2026-11-22-v1-scope.md`](plans/2026-11-22-v1-scope.md) | What v1.0 is and is not; supported system; release gates G0–G9. |
| [`budgets.md`](budgets.md) | Latency/RSS/fps budgets per hardware tier. |

## 2. What we decided

| File | Use it for |
|---|---|
| [`decisions/README.md`](decisions/README.md) | ADR index (0001–0019). ADRs are canonical for their decision. |
| [`plans/2026-11-21-live-linguist-pc-master-plan.md`](plans/2026-11-21-live-linguist-pc-master-plan.md) | Architecture and spec reference (crates, protocol, UI, inference, packaging, decision log). Sections superseded for 1.0 carry a banner. Read the relevant section before structural changes; do not execute from it. |

## 3. What we measured (evidence)

Current evidence (Windows host):

| File | Gate |
|---|---|
| [`qa/2026-11-22-windows-vm-smoke.md`](qa/2026-11-22-windows-vm-smoke.md) | W-0 toolchain + verify script |
| [`qa/2026-11-21-windows-hardware.md`](qa/2026-11-21-windows-hardware.md) | Reference hardware record |
| [`qa/2026-11-22-gpu-renderer.md`](qa/2026-11-22-gpu-renderer.md) | Measured renderer/accelerators, DirectX test-renderer portfolio |
| [`qa/2026-11-22-overlay-matrix.md`](qa/2026-11-22-overlay-matrix.md) | W-1 single-screen overlay matrix |
| [`qa/2026-11-22-model-pins.md`](qa/2026-11-22-model-pins.md) | Pinned ASR/LLM artifacts and hashes |
| [`qa/2026-11-21-event-delivery.md`](qa/2026-11-21-event-delivery.md) | S-2 bridge event delivery |
| [`qa/2026-11-21-version-verification.md`](qa/2026-11-21-version-verification.md) | Dependency pins (add new pins here) |
| [`qa/skills-lock.md`](qa/skills-lock.md) | Pinned third-party agent skills |

Baseline evidence from the Linux dev phase (still true, partially superseded by
the plan's pending stages): `qa/2026-11-21-{pipeline,audio,asr-coverage,llm-backends,overlay,ui,packaging}.md`.
Each will be replaced or marked historical by the stage that supersedes it
(S1 audio, S3 ASR coverage, S5 LLM, S6 UI, S8 packaging).

Screenshots: `screenshots/m2-windows/` (Windows, current) and `screenshots/m2/`
(Linux dev portfolio, used by the root README).

## 4. Conventions

- New evidence: `qa/YYYY-MM-DD-<topic>.md`, using the template in plan §13.
- New plans are rare; prefer editing the active plan. If a plan is replaced,
  delete the old file and say so in the new one's header (git keeps history).
- A structural change updates its ADR and the decision log in the master plan
  in the same PR.

## Removed on 2026-11-23

Superseded and deleted (see git history): `plans/2026-11-22-v1-execution-plan.md`,
`plans/2026-11-21-windows-resume-plan.md`, `plans/2026-11-21-windows-resume-prompt.md`,
`stack-options.md` (pre-decision Linux/Qt matrix), `qa/2026-11-21-final-qa.md`
(Linux-phase checklist), `qa/2026-11-21-windows.md` (cross-compile-only check,
superseded by the overlay matrix and W-0 smoke).
