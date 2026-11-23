# T05 — ADR 0003, evidence, PR for S1

**Needs:** T01–T04 done.

## Steps

1. `docs/decisions/0003.md`: change `Status: baseline` to `Status: accepted`
   and add a section:

   ```
   ## Addendum 2026-11-23 — Windows capture stack
   cpal 0.18 WASAPI loopback on the default render endpoint (input stream on the
   output device). Evidence: docs/qa/2026-11-23-audio-loopback.md.
   ```

2. In `docs/qa/implementation-progress.md`, v1.0 stage board: S1 → `in review`.
3. Run `pwsh scripts\verify-windows.ps1` — must pass.
4. Push `stage-w2-loopback` and open a PR titled
   `windows: WASAPI loopback capture` with: what changed, the check output,
   the soak numbers, and the screenshot.
5. Do not merge it yourself unless CI is green; if green, merge with a merge
   commit and set S1 to `merged` in the progress board on `main`.

Commit message: `docs: S1 loopback evidence and ADR 0003`
