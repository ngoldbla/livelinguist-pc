# T06 — Streaming resampler for loopback (S1 follow-up)

**Goal:** replace the per-buffer linear resampler in `LoopbackCapture` with a
stateful `rubato` resampler, so there is no click or aliasing at every buffer
boundary. Speech recognition quality depends on this.

**Files:** `core/crates/ll-audio/Cargo.toml`, `core/crates/ll-audio/src/capture.rs`

## Steps

1. In `ll-audio/Cargo.toml` add `rubato = { workspace = true, optional = true }`
   and change the feature to `capture = ["dep:cpal", "dep:rubato"]`.
   (`rubato = "5.1.0"` is already pinned in the workspace `Cargo.toml`.)
2. In `capture.rs`, add to `LoopbackCapture`:
   - `resampler: rubato::Fft<f32>` created in `start_default` with
     `rubato::Fft::<f32>::new(device_rate as usize, 16_000, 1024, 1, rubato::FixedSync::Input)`;
   - `pending: Vec<f32>` (mono samples waiting for a full input chunk).
3. In `next_buffer`, after `downmix_to_mono`, append to `pending`. Then, while
   `pending.len() >= resampler.input_frames_next()`:

   ```rust
   use rubato::Resampler;
   use rubato::audioadapter_buffers::direct::SequentialSliceOfVecs;
   let n = self.resampler.input_frames_next();
   let chunk = vec![self.pending[..n].to_vec()];
   let input = SequentialSliceOfVecs::new(&chunk, 1, n).expect("1-channel adapter");
   let out = self.resampler.process(&input, None); // Result<InterleavedOwned<f32>, _>
   // on Err: stop this loop and return what you have; never panic in capture
   // append the produced mono frames to the output Vec
   self.pending.drain(..n);
   ```

   To read samples out of the returned `InterleavedOwned<f32>`, use the
   `audioadapter::Adapter` trait (`out.frames()` and
   `out.read_sample(0, frame)`), or `out.take_data()` if it exists in this
   version. If neither compiles after two tries, apply the anti-loop rule.
   Leave samples smaller than one chunk in `pending` for the next call.
4. Remove the `TODO(T02)` comment and the call to `resample_linear` in
   `capture.rs` (keep `resample_linear` itself; other code uses it).
5. Add a unit test that doesn't need a device: create the same resampler for
   48 kHz, feed 1 s of a 440 Hz sine in 7 uneven pieces through the same
   pending/chunk logic (move that logic into a small `fn` you can call from
   the test), and assert output length is 16 000 ± 1 024 samples and RMS is
   within 10% of the input RMS.

## Done when

```powershell
cargo test -p ll-audio --features capture
cargo test -p ll-audio --features capture -- --ignored loopback_captures_playing_audio
cargo clippy -p ll-audio --features capture --all-targets -- -D warnings
cargo fmt --all -- --check
```

Commit message: `audio: stateful rubato resampling for loopback`
