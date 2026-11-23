# T01 — Loopback probe example (cpal)

**Goal:** prove that cpal can capture what Windows is playing (WASAPI loopback)
on this machine. This is a throwaway-style example, not product code.

**Files:**
- create `core/crates/ll-audio/examples/loopback_probe.rs`
- edit `core/crates/ll-audio/Cargo.toml` (add the example, see step 1)

## Steps

1. In `core/crates/ll-audio/Cargo.toml` add:

   ```toml
   [[example]]
   name = "loopback_probe"
   required-features = ["capture"]
   ```

2. Write `loopback_probe.rs` that:
   - gets `cpal::default_host()` and `host.default_output_device()`
     (the **output** device — cpal turns an input stream on an output device
     into WASAPI loopback);
   - reads `device.default_output_config()` and prints sample rate, channels,
     sample format;
   - builds an input stream on that device with that config (f32 samples;
     if the format is not f32, print it and exit with code 2);
   - in the callback, only add to two atomics: total frames received and the
     max absolute sample value (store as `f32::to_bits` in an `AtomicU32`).
     No allocation, no printing, no locks inside the callback;
   - runs for 10 seconds, then prints:
     `frames=<n> rate=<hz> channels=<c> peak=<p> callbacks=<k>`.
3. Build: `cargo build -p ll-audio --features capture --example loopback_probe`
4. Run it **while sound plays**. In a second PowerShell window:

   ```powershell
   $p = New-Object System.Media.SoundPlayer "tests\fixtures\sample.wav"; $p.PlayLooping()
   ```

   then in the first window:
   `cargo run -p ll-audio --features capture --example loopback_probe`
   Stop the sound afterwards with `$p.Stop()`.
5. Run it again **with no sound playing** and note the numbers.

## Done when

- With sound: `frames` > 0 and `peak` > 0.01.
- Without sound: the program still exits normally after 10 s (frames may be 0).
- `cargo clippy -p ll-audio --features capture --all-targets -- -D warnings` passes.

## Report

Write both result lines into a new file `docs/qa/2026-11-23-loopback-probe.md`
with the date, the output device name, and the two runs. If frames stay 0
**with** sound playing, write that in the file and stop (that triggers the
`windows`-crate fallback; a human will decide).

Commit message: `audio: cpal loopback probe example`
