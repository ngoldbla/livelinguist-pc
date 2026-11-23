# T04 — `ll-eval` loopback source + 30-minute soak

**Needs:** T03 done.

**Goal:** measure that 30 minutes of system audio produces zero overruns.

**Files:**
- `core/crates/ll-eval/Cargo.toml` (feature `capture = ["ll-audio/capture"]`)
- `core/crates/ll-eval/src/lib.rs`

## Steps

1. Add two CLI flags next to the existing `--wav` flag: `--source loopback`
   and `--seconds <n>` (default 60). Update the usage text.
2. With `--source loopback` (and the `capture` feature), read buffers from
   `LoopbackCapture` for `n` seconds and feed the pipeline exactly like the
   `--wav` path does. Without the feature, exit with a clear error.
3. In the run summary that `ll-eval` already writes, add:
   `capture_seconds`, `capture_overrun_events`, `device_rate`. No text content.
4. Add a unit test that `--source loopback --seconds 5` parses, and that an
   unknown `--source` is rejected.
5. Run the soak with a long video or the looped fixture playing:

   ```powershell
   cargo run --release -p ll-eval --features capture -- --scenario mock --language en-US --source loopback --seconds 1800 --out target\soak
   ```

## Done when

- Unit tests and clippy pass.
- The soak finishes and the summary shows `capture_overrun_events = 0`.
  If it is not 0, do not change the ring size more than once; record the
  number and stop.

Write the soak numbers into `docs/qa/2026-11-23-loopback-probe.md` under a new
`## 30-minute soak` heading (rename the file to `2026-11-23-audio-loopback.md`).

Commit message: `eval: loopback source and soak`
