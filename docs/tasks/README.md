# Task cards

Small, self-contained work orders for a coding agent. **One card per session.**
Each card says exactly which files to touch, which commands to run, and how to
know it is done. You do not need to read the big plan to do a card; the card
already contains what matters from it.

The big plan (`docs/plans/2026-11-23-windows-v1-plan.md`) is for humans and for
writing new cards. Agents work from cards.

## Order

| Card | Title | Status |
|---|---|---|
| [T01](T01-loopback-probe.md) | Loopback probe example (cpal) | done |
| [T02](T02-loopback-capture.md) | `LoopbackCapture` in `ll-audio` | done |
| [T03](T03-loopback-bridge.md) | Bridge: "System audio" source | done |
| [T04](T04-loopback-soak.md) | `ll-eval` loopback source + 30-min soak | done |
| [T05](T05-loopback-pr.md) | ADR 0003, evidence, PR for S1 | todo |

More cards (S2 sherpa ASR, S5 llama.cpp, S6 layout, ...) are written after
S1 merges. If you reach the end of this list, stop and report.

## Rules for every card

1. Work on branch `stage-w2-loopback` (create it from `main` on the first card;
   reuse it afterwards). Commit after each card with the message the card gives.
2. Do only the steps on the card. Do not refactor, rename, or "improve" other
   code. Do not edit other cards.
3. Run the card's check commands. Paste their last lines into your report.
4. **Anti-loop rule:** if the same command fails twice after two different
   fixes, stop. Write what you tried and the exact error into
   `docs/tasks/BLOCKED.md`, commit it, and end the session.
5. Never: translate text, add network calls, log transcript text, commit audio
   or model files, or disable/weaken a test.
6. When the card is done, change its status in the table above to `done`,
   commit, and end the session with:

```
Card: Txx
Result: done | blocked
Checks: <last lines of each check command>
Next: <next card id>
```
