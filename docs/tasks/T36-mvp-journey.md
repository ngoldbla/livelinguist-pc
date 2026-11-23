# T36 — End-to-end journey + soak, then STOP for the human pass (S7) (PR)

**Needs:** T35 done.

## Steps

1. Journey on this PC with the release build, English YouTube video:
   install-free run → onboarding (models present) → System audio → live
   captions → switch to overlay over a fullscreen browser → click a caption
   in the overlay → simplified line appears; the browser keeps focus → drag
   the overlay → pause/resume → expanded mode → export Markdown, plain text,
   diagnostics CSV → clear. Screenshot each step into
   `docs/screenshots/m2-windows/journey/`.
2. Soak: 30 minutes, loopback + real ASR + a click every ~30 s (use the
   `click-bench` path with `--source loopback --seconds 1800`, or a timer in
   `ll-eval`), under the tier C profile. Pass: 0 capture overruns, no lost
   finalized segments (segment ids continuous), peak RSS ≤ 3.5 GB, no crash.
3. Write `docs/qa/2026-11-23-v1-integration.md`: journey table (step, pass/fail,
   screenshot), soak numbers, privacy result from T35.
4. Open PR `windows: v1 end-to-end session (MVP)` from branch
   `stage-w5-integration` (T34–T36); merge when CI is green; set S7 to
   `merged` in the progress board.
5. **STOP.** End the session with `Result: done` and in `Next:` write:
   "MVP ready — owner: do the human pass (real Zoom/Teams/Meet call +
   Narrator listening) and record it in docs/qa/<date>-mvp-human-pass.md
   before T37."

Commit message: `qa: v1 end-to-end journey and soak`
