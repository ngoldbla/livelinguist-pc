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
| [T06](T06-loopback-resampler.md) | Streaming `rubato` resampler for loopback | `stage-w2-loopback-fixes` | todo |
| [T07](T07-loopback-device-errors.md) | Report device loss / follow default output (PR) | `stage-w2-loopback-fixes` | todo |
| [T08](T08-prompt-contract-source.md) | Copy the prompt contract from the model card | `stage-w4-prompt-contract` | todo |
| [T09](T09-prompt-contract-renderer.md) | `render_easylanguage_prompt` + 29 goldens | `stage-w4-prompt-contract` | todo |
| [T10](T10-drop-rolling-context.md) | No rolling context on the click path (PR) | `stage-w4-prompt-contract` | todo |
| [T11](T11-model-fetch-cli.md) | `fetch_models` example; download the pinned set | `stage-w3-asr-runtime` | todo |
| [T12](T12-sherpa-probe.md) | sherpa-onnx probe on a TTS speech clip | `stage-w3-asr-runtime` | todo |
| [T13](T13-sherpa-engine.md) | Real streaming `SherpaAsr` | `stage-w3-asr-runtime` | todo |
| [T14](T14-sherpa-in-app.md) | Real ASR in `ll-eval` and the app (PR) | `stage-w3-asr-runtime` | todo |

Cards marked **(PR)** end their branch: open the PR, merge when CI is green.
More cards (first-run download screen, llama.cpp, 150% layout, coverage
matrix) are written after T14. If you reach the end of this list, stop and
report.

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
