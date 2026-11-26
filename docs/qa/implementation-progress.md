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

All 27 ADRs are present in `docs/decisions/`. New structural changes need a
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
| S2 | W-3a sherpa-onnx runtime + first-run model UX | `stage-w3-asr-runtime` | merged on Linux (#27, #29); Windows UI checks pending |
| S3 | W-3b per-locale WER/RTF, 29-row matrix | `stage-w3-asr-coverage` | needs S2 |
| S4 | W-4a EasyLanguage prompt contract (drop rolling context, 29 goldens) | `stage-w4-prompt-contract` | merged (#24, #28) |
| S5 | W-4b CPU llama.cpp click-only simplify, p95/RSS | `stage-w4-llama-cpu` | needs S4 |
| S6 | W-5a 150%/1280x720 layout, W-5b keyboard + Narrator | `stage-w5-layout`, `stage-w5-a11y` | ready |
| S7 | W-5c end-to-end real session + privacy checks (**MVP**; then human pass) | `stage-w5-integration` | needs S1, S2, S5, en live |
| S8 | W-6 NSIS + portable, clean-profile install, unsigned + sums | `stage-w6-packaging` | needs S7 + MVP pass |
| S9 | Release candidate → `v1.0.0` | `release-v1.0.0` | needs S8 + owner go-ahead |

## Next run

**2026-11-25 UI clarity pass (branch `stage-ui-clarity`, merged in #44,
released as `v1.0.0-rc.4`):** the shipped UI was
reviewed with the impeccable design skill (20/40, Acceptable) and rebuilt on the
macOS/iOS flow (`ngoldbla/live-linguist` @ `8eecdc4`, plus the iOS store listing
`ngoldbla/live-linguist-ios`): a console with two decision tiles and one Start
cluster, pick-to-advance onboarding, a captioner with language + timer + cadence
and confidence bars, a caption-only overlay, a compact summary sheet, and
settings with an App-language control. Icons are Lucide (ISC), the mark is the
macOS twin-L on the indigo→azure tile, simplified captions use M PLUS Rounded 1c
(OFL, bundled), and the two console tiles open GPUIX Select menus. The surface is
quiet and high-contrast (12px type floor, flat panels) and UI settings persist to
`%LOCALAPPDATA%\LiveLinguist\ui-settings.json`. ADR 0026; evidence
`docs/qa/2026-11-25-ui-clarity.md` and the rebuilt
`docs/screenshots/m2-windows/` portfolio. UI: typecheck, 114 tests, Biome and
14 capture scenarios pass. Remaining: the 28-catalog translation pass for 19
new strings, the input-level meter (no bridge data), and the human
Narrator/DPI walk. `v1.0.0-rc.4` is published as a pre-release.

**2026-11-25 window chrome fix (same branch):** users could not move, minimize
or maximize the frameless window. ADR 0027 adds a custom 34px title bar (drag,
double-click maximize, minimize/maximize/close) on every screen except the
overlay, a `closeWindow` bridge command that posts `WM_CLOSE`, and changes the
drag helper to `HWND_TOP` so a moved console is not left topmost. Windows human
pass still needed for drag and the three buttons.

**2026-11-25 README media refresh (`v1.0.0-rc.4`):** the top-level README now
shows the current console, language picker, live, overlay, settings and summary
screens from `docs/screenshots/m2-windows/`, and publishes a new 60 s German run
with the redesigned UI (`docs/media/german-60s.mp4`, 30 s GIF preview
`docs/media/readme-german.gif`). The harness probe was updated for the new UI
(SOUND tile menu, caption-row button labels, `Listening — keep talking` partial
group); capture uses `-draw_mouse 0`. Recorded numbers: partial p50/p95
1866/6308 ms at 94.4% coverage, clicks 7/7 applied, click p95 2266 ms, max
caption gap 1235 ms. Details in `docs/qa/2026-11-25-german-live-baseline.md`
(`## README rerun with the redesigned UI`).

Checkpoint 2026-11-24 (Linux cloud session; PRs #24-#31). Card statuses are in
`docs/tasks/README.md` (`linux-done` = only the Windows part remains).

**Post-v1 autoresearch (T41, 2026-11-24):** branch `stage-autoresearch-baseline`
adds the content-free media-clock → UIA caption-latency harness, explicit ASR
chunk/thread/endpoint experiment knobs, and overlay partial presentation. The
installed RC baseline and CPU parameter screen are recorded in
`docs/qa/2026-11-24-caption-latency-baseline.md`: RC live partial p95 3657 ms;
confirmed overlay-partial + 80 ms candidate p95 2278 ms with 93.8% cue coverage.
Shipping ASR defaults remain unchanged pending second-voice/WER/tier-C/click
continuity checks. GPU inference is not implemented or measured.

**2026-11-25 README rerun (same branch):** the four benchmark commands were
executed on the reference PC. RC live 90 s × 3 partial p95 3349 ms (84.4%
coverage); the README's example candidate (80 ms + fast endpoint + 4 threads)
reached p95 2666 ms but only 75.0% coverage, so 0/3 trials were valid; the 45 s
screen again selected 80 ms + default endpoint + 2 threads (p95 2722 ms, 88.9%).
Numbers in `docs/qa/2026-11-24-caption-latency-baseline.md`; no default changed.

**2026-11-25 chunk confirmation (supersedes "defaults unchanged" above):** a
second voice (NASA Elena Serova, public domain) rejects 80 ms for the default:
p95 6436–11102 ms vs 5671 ms at 1100 ms. 560 ms measures 4380–6811 ms on the
second voice and 2685 ms on the lecture; WER is unchanged on four of five
required locales, the ASR chunk p95 halves, and tier C passes (RTF 0.241,
777 MB). **`ll-config` now defaults to 560 ms**, and the click gate's pre-existing
freeze is fixed by ADR 0025 (engine borrowed to a worker; a live click keeps
92.3% coverage). Shipped-path reruns are in
`docs/qa/2026-11-25-asr-chunk-confirmation.md`; the change is released as rc.3.

**2026-11-25 German click baseline:** the autoresearch harness gained
fixture-selected app language, `--click-every-n` (alternating finalized rows,
scroll-to-newest, per-click click→simplified latency) and `--record` (app-window
capture + the fixture audio track). Two 60 s dry runs on the Terra X German clip
measured partial p50/p95 1907–1910 / 6302–6330 ms at 94.4% coverage, 7/7 clicks
applied, click-to-simplified p50/p95 1620–1622 / 2240–2382 ms. The recorded run
is 60.0 s with audio at `target/autoresearch/german-60s/german-60s.mp4`; details
and the recording's small encoder overhead are in
`docs/qa/2026-11-25-german-live-baseline.md`.

**Owner Windows retest 2026-11-24: steps 0–5 pass** —
`docs/qa/2026-11-24-windows-retest.md` (results table, machine, evidence). The
run found and the branch `windows-retest-2026-11-24` fixes: Summary exports
(`stop()` dropped the pipeline), model cards hard-coded "Not downloaded", the
microphone tile swallowed clicks, onboarding rendered twice, and
`tier-c-profile.ps1` mis-parsed `--scenario`. Measured: captions start 1.15 s
after playback, click 1.98 s then 1.31 s (RSS 1,011 → 1,639 MB), Tier C p50
1,802 / p95 2,013 ms / 886 MB (pass), `privacy: PASS`. ADR 0021 is **accepted:
option 1** (compiled exe + both `.node` addons beside it), so T37 is unblocked.

**Done since (Linux, 2026-11-24):**
- T33 (Linux part): `CoverageMatrix::v1()` and `ui/src/i18n/languages.ts` follow
  the measured table, with 12 `ok` and 17 `simplify_only` (none `tbd`). A UI test keeps
  the two equal. System-audio sessions report the language's status, so en-US
  no longer says "live listening is not available".
- T35: Linux `scripts/privacy-check.sh` prints `privacy: PASS` (0 `connect()`
  calls); recorded in `docs/qa/2026-11-23-v1-integration.md`. It first exposed
  a Linux-only crash when sherpa and llama share a binary; fixed by linking
  sherpa-onnx shared on Linux (ADR 0022). Windows is unchanged.

**Windows, 2026-11-24 (owner PC, this run):** T33 tier C re-check of the 12 `ok`
locales + ja-JP: timing passes on every row (RTF 0.24–0.26, p95 283–322 ms,
peak RSS 778-780 MB); scores match Linux within 0.016 except **de-DE, which
scores 0.157 and misses the 0.15 gate** (owner kept it `ok` on 2026-11-24 with
the number recorded in the coverage doc).
ja-JP stays `simplify_only` (0.1597). The language picker shows the measured
statuses (`docs/screenshots/m2-windows/language-picker-coverage.png`, scenario
`language-picker-coverage` in `ui/scripts/screenshots-windows.tsx`).

**Windows, 2026-11-24:** T16 first-run download verified end to end on the
reference PC with a fresh `LOCALAPPDATA`: cancel at 30%, retry resumes, finish →
"Models ready", and the Console then shows both models ready.
`docs/qa/2026-11-24-windows-download.md`, screenshots
`onboarding-download-progress.png` / `onboarding-models-ready.png`.

**Windows, 2026-11-24:** T14 live ASR confirmed in the app (System audio → real
captions, 1 finalized segment plus a live partial; screenshots
`console-live-asr.png`, `console-click-simplified.png`). Found and fixed there:
the live screen's ASR panel said "live listening is not available" for `ok`
languages. T20's click path produced a real GGUF simplification (flagged
`unfaithful` by the validator); evidence in `docs/qa/2026-11-23-llm-cpu.md`.

**Windows, 2026-11-24:** T23 `workArea()` numbers recorded at 100% (2560x1032,
scale 1.0) and 150% (1707x672 logical; 2560x1008 physical, dpi 144) in
`docs/qa/2026-11-23-v1-ui.md` `## Work area`; the per-monitor-aware app fits
the Console inside the work area at 150%, with a 13 px centred-position
overshoot left to T25.

**Release pipeline, 2026-11-25 (owner request; ADR 0023):** MIT only; tag
`vX.Y.Z` → `release.yml` builds, self-tests, test-installs/uninstalls on a
clean runner and publishes an unsigned GitHub release with `SHA256SUMS.txt`
(`packaging/README.md`). This covers the mechanics of T37/T39 ahead of the MVP
human pass; tagging `v1.0.0` still waits for T36 and the owner (T40).
First run: **`v1.0.0-rc.1` published 2026-11-25** as an unsigned pre-release
(Actions run 45534402943, 21 min, Run workflow with "Publish a release").
Setup 56.5 MB, portable ZIP 77.2 MB, installed payload 160.7 MB (over T37's
60 MB target; the Bun runtime dominates). All install-test checks passed:
payload/installed/upgraded/portable self-tests `gpuix ok, core ok`, shortcuts,
Installed apps entry, uninstall cleanup, models kept. Not yet run on a real
PC with a GPU window and the model download.

**Linux work still open (resume here):**
1. Owner decisions: es-ES
   matrix guard false negative (T22); confirm the ` /no_think` soft switch in
   the training renderer (T08).

**Windows-only work left:**
- T25, T26 (not started), T27 keyboard-only live run, T28 Narrator pass
  (human) + Arabic screenshot.
- T34, T36–T40 (T37 has its ADR now; T38 needs admin once).

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
