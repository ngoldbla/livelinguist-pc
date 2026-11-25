# Task cards

Small, self-contained work orders for a coding agent. **One card per session.**
Each card says exactly which files to touch, which commands to run, and how to
know it is done. You do not need to read the big plan to do a card; the card
already contains what matters from it.

The big plan (`docs/plans/2026-11-23-windows-v1-plan.md`) is for humans and for
writing new cards. Agents work from cards.

## Order

| Card | Title | Branch | Status |
|---|---|---|---|
| [T01](T01-loopback-probe.md) | Loopback probe example (cpal) | `stage-w2-loopback` | done |
| [T02](T02-loopback-capture.md) | `LoopbackCapture` in `ll-audio` | `stage-w2-loopback` | done |
| [T03](T03-loopback-bridge.md) | Bridge: "System audio" source | `stage-w2-loopback` | done |
| [T04](T04-loopback-soak.md) | `ll-eval` loopback source + 30-min soak | `stage-w2-loopback` | done |
| [T05](T05-loopback-pr.md) | ADR 0003, evidence, PR for S1 | `stage-w2-loopback` | done |
| [T06](T06-loopback-resampler.md) | Streaming `rubato` resampler for loopback | `stage-w2-loopback-fixes` | done |
| [T07](T07-loopback-device-errors.md) | Report device loss / follow default output (PR) | `stage-w2-loopback-fixes` | done |
| [T08](T08-prompt-contract-source.md) | Copy the prompt contract from the model card | `stage-w4-prompt-contract` | done |
| [T09](T09-prompt-contract-renderer.md) | `render_easylanguage_prompt` + 29 goldens | `stage-w4-prompt-contract` | done |
| [T10](T10-drop-rolling-context.md) | No rolling context on the click path (PR) | `stage-w4-prompt-contract` | done |
| [T11](T11-model-fetch-cli.md) | `fetch_models` example; download the pinned set | `stage-w3-asr-runtime` | done |
| [T12](T12-sherpa-probe.md) | sherpa-onnx probe on a TTS speech clip | `stage-w3-asr-runtime` | done |
| [T13](T13-sherpa-engine.md) | Real streaming `SherpaAsr` | `stage-w3-asr-runtime` | done |
| [T14](T14-sherpa-in-app.md) | Real ASR in `ll-eval` and the app (PR) | `stage-w3-asr-runtime` | done |
| [T15](T15-model-download-bridge.md) | Bridge: model status, download, cancel | `stage-w3-model-download` | linux-done |
| [T16](T16-model-download-ui.md) | Onboarding download screen (PR) | `stage-w3-model-download` | done |
| [T17](T17-llama-probe.md) | llama.cpp CPU probe | `stage-w4-llama-cpu` | done |
| [T18](T18-llama-backend.md) | `LlamaBackend` | `stage-w4-llama-cpu` | done |
| [T19](T19-click-queue.md) | Click-only queue rules | `stage-w4-llama-cpu` | done |
| [T20](T20-llama-in-app.md) | Real simplifier in `ll-eval` and the app | `stage-w4-llama-cpu` | done |
| [T21](T21-tier-c-latency.md) | Tier C profile, click p95, RSS | `stage-w4-llama-cpu` | done |
| [T22](T22-simplify-matrix.md) | 29-pack simplifier matrix (PR) | `stage-w4-llama-cpu` | linux-done |
| [T23](T23-work-area.md) | Bridge `workArea()` | `stage-w5-layout` | done |
| [T24](T24-window-size-clamp.md) | Fit windows to the work area | `stage-w5-layout` | done |
| [T25](T25-layout-min-size.md) | Screens fit the minimum size + screenshots (PR) | `stage-w5-layout` | todo |
| [T26](T26-a11y-checks.md) | Automated a11y checks on every screen | `stage-w5-a11y` | todo |
| [T27](T27-keyboard-captions.md) | Keyboard simplify + Read latest | `stage-w5-a11y` | linux-done |
| [T28](T28-a11y-checklist.md) | Contrast, reduced effects, a11y checklist (PR) | `stage-w5-a11y` | linux-done |
| [T29](T29-fleurs-manifest.md) | FLEURS manifest + fetch script | `stage-w3-asr-coverage` | done |
| [T30](T30-wer-metrics.md) | Normalizer, WER, CER | `stage-w3-asr-coverage` | done |
| [T31](T31-asr-wer-scenario.md) | `ll-eval` asr-wer scenario | `stage-w3-asr-coverage` | done |
| [T32](T32-asr-minimum-set.md) | Measure en/fr/de/es/ar | `stage-w3-asr-coverage` | linux-done |
| [T33](T33-asr-coverage-matrix.md) | All 29 locales + coverage statuses (PR) | `stage-w3-asr-coverage` | done |
| [T34](T34-release-build-integration.md) | Real-backend build, measured health | `stage-w5-integration` | todo |
| [T35](T35-privacy-checks.md) | Privacy check script | `stage-w5-integration` | done |
| [T36](T36-mvp-journey.md) | Journey + soak, then STOP for human pass (PR) | `stage-w5-integration` | todo |
| [T37](T37-packaging-build.md) | Installer + portable ZIP build | `stage-w6-packaging` | todo |
| [T38](T38-clean-profile-install.md) | Clean-profile install test (needs admin once) | `stage-w6-packaging` | todo |
| [T39](T39-release-workflow.md) | Release workflow dry run + install notes (PR) | `stage-w6-packaging` | todo |
| [T40](T40-release-candidate.md) | Release gates and `v1.0.0` | `release-v1.0.0` | todo |

Cards marked **(PR)** end their branch: open the PR, merge when CI is green.
Two cards stop for a human on purpose: **T36** (MVP: real call + Narrator
listening pass) and **T38** (needs admin once to create a test user). T37 will
not start until the human-pass file exists. After T40 the list is finished.

## Rules for every card

1. Work on the branch in the card's row. If it doesn't exist, create it from
   the latest `main`; otherwise switch to it. Commit after each card with the
   message the card gives.
2. Do only the steps on the card. Do not refactor, rename, or "improve" other
   code. Do not edit other cards.
3. Run the card's check commands. Paste their last lines into your report.
4. **Anti-loop rule:** if the same command fails twice after two different
   fixes, stop. Write what you tried and the exact error into
   `docs/tasks/BLOCKED.md`, commit it, and end the session.
5. Never: translate text, add network calls, log transcript text, commit audio
   or model files, or disable/weaken a test.
6. When the card is done, change its status in the table above to `done`
   (on the card's branch; it reaches `main` with the PR), commit, and end the
   session with:

```
Card: Txx
Result: done | blocked
Checks: <last lines of each check command>
Next: <next card id>
```
