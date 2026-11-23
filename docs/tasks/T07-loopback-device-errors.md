# T07 — Surface loopback device errors (S1 follow-up)

**Goal:** if the output device disappears or Windows switches the default
output, the app says so instead of silently stalling.

**Files:** `core/crates/ll-audio/src/capture.rs`, `core/crates/ll-bridge/src/lib.rs`

## Steps

1. In `capture.rs`, add `device_lost: Arc<AtomicBool>`. In `build_stream`, the
   error callback (today `|_error| {}`) sets it to `true`. Nothing else in that
   callback.
2. Add `pub fn is_device_lost(&self) -> bool` and
   `pub fn default_device_name() -> Option<String>` (name of
   `cpal::default_host().default_output_device()`). Store the name of the
   device you opened in `start_default` and expose `pub fn device_name(&self)`.
3. In `next_buffer`: if `device_lost` is true, return `None` (the source has
   ended). Do not reopen inside `next_buffer`.
4. In the bridge, where loopback buffers are read (search `LoopbackCapture`):
   - when `next_buffer` returns `None`, or every ~2 s when
     `default_device_name()` differs from `device_name()`, drop the capture,
     emit an error/status event with code `AUDIO_DEVICE_LOST` (reason text
     only, no audio data), then try `LoopbackCapture::start_default()` once per
     second for up to 10 s;
   - on success emit a status event `AUDIO_DEVICE_CHANGED`; on failure stop the
     session with `AUDIO_DEVICE_LOST`. Never switch to demo audio.
   Use the same event helper the bridge already uses for errors (search for
   `error` events in `lib.rs`); keep the `{ v, seq, ts }` envelope.
5. Unit test (no device): a fake flag set to `true` makes `next_buffer`
   return `None`. If `LoopbackCapture` can't be built in tests, test the small
   helper you extract for this decision instead.

## Manual check

Start a session on **System audio** with a video playing. In Windows sound
settings switch the default output to the other device. Captions keep coming
after a short gap, and the UI shows a device-changed message. Record what you
saw in `docs/qa/2026-11-23-audio-loopback.md` under `## Device change`.

## Done when

```powershell
cargo test -p ll-audio --features capture
cargo test -p ll-bridge
cargo clippy --workspace --all-targets -- -D warnings
```

pass and the manual check is recorded. Then open PR
`audio: loopback resampling and device changes` from branch
`stage-w2-loopback-fixes` (T06–T07) and merge it when CI is green.

Commit message: `audio: report device loss and follow default output`
