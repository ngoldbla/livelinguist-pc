# T02 — `LoopbackCapture` in `ll-audio`

**Needs:** T01 done with frames > 0.

**Goal:** a reusable capture type that turns system audio into 16 kHz mono f32
buffers, the same format every other source in `ll-audio` produces.

**Files:**
- create `core/crates/ll-audio/src/capture.rs`
- edit `core/crates/ll-audio/src/lib.rs` (add `#[cfg(feature = "capture")] pub mod capture;`)
- edit `core/crates/ll-audio/Cargo.toml` only if you need `rubato` (see step 4)

## Steps

1. Pure helper (no cpal, always compiled, put it in `lib.rs` next to
   `resample_linear`): `pub fn downmix_to_mono(interleaved: &[f32], channels: usize) -> Vec<f32>`
   averaging channels. Add tests: stereo `[1,0,1,0]` → `[0.5,0.5]`; mono passthrough;
   `channels == 0` returns empty.
2. In `capture.rs` define:

   ```rust
   pub struct LoopbackCapture { /* stream, ring consumer, rate, channels, counters */ }
   impl LoopbackCapture {
       pub fn start_default() -> Result<Self, AudioError>;   // open default output, loopback
       pub fn device_rate(&self) -> u32;
       pub fn overrun_events(&self) -> u64;
   }
   impl AudioSource for LoopbackCapture { /* next_buffer, name = "loopback" */ }
   ```

3. Callback: push raw interleaved f32 into a `ringbuf` SPSC producer (the crate
   already depends on `ringbuf`). If the ring is full, add 1 to an
   `AtomicU64` overrun counter. No allocation, logging, or locks in the
   callback. Size the ring for 2 seconds of audio at the device rate.
4. `next_buffer()`: pop what is available (wait up to 50 ms if empty, then
   return an empty `Vec` rather than `None` — capture never "ends"), downmix
   with step 1, resample to 16 kHz. Use `rubato` (already in the workspace) if
   it works within two attempts; otherwise use `resample_linear` and write
   `TODO(T02): switch to rubato` in a comment.
5. Errors from cpal become `AudioError::Device(<short reason>)`. No audio data
   in error strings.
6. Add a test in `capture.rs` marked `#[ignore]` named
   `loopback_captures_playing_audio` that starts capture, plays
   `tests/fixtures/sample.wav` via `powershell -c` SoundPlayer for 3 s, and
   asserts at least one buffer with RMS > 0.001.

## Done when

```powershell
cargo test -p ll-audio
cargo test -p ll-audio --features capture
cargo test -p ll-audio --features capture -- --ignored loopback_captures_playing_audio
cargo clippy -p ll-audio --features capture --all-targets -- -D warnings
cargo fmt --all -- --check
```

all pass.

Commit message: `audio: WASAPI loopback capture source`
