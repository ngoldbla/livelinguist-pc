# T03 — Bridge: "System audio" source

**Needs:** T02 done.

**Goal:** the app can start a session on system audio, and only says it is
available when it really is.

**Files:**
- `core/crates/ll-bridge/Cargo.toml`
- `core/crates/ll-bridge/src/lib.rs`
- `ui/src/screens/Console.tsx` (only if the source tile needs a label change)

## Steps

1. In `ll-bridge/Cargo.toml` add a feature `capture = ["ll-audio/capture"]`.
   Do **not** add it to `default`.
2. In `list_audio_sources` (search for `fn list_audio_sources`): for the
   `monitor` entry, when built with `capture`, set `available` from a quick
   check that `cpal::default_host().default_output_device()` is `Some`
   (expose a tiny `ll_audio::capture::default_output_available() -> bool`
   for this). Without the feature keep today's `available: false`.
   Keep `mic` as `available: false` with `note: "post-1.0"`.
3. In `start` (search for `match request.source.as_str()`): add `"monitor"`.
   With `capture`, build `LoopbackCapture::start_default()` and feed its
   buffers into the pipeline the same way the demo source is fed (look at how
   `SyntheticSource` is used further down in the same function). If start
   fails, send an error event with the short reason and do **not** fall back
   to demo. Without the feature, `"monitor"` returns an error
   `"system audio capture not built"`.
4. Add a unit test: without the `capture` feature, `list_audio_sources`
   reports `monitor` as unavailable and `start` with `source: "monitor"`
   returns an error (not a demo session).

## Done when

```powershell
cargo test -p ll-bridge
cargo build -p ll-bridge --release --features capture
cargo clippy --workspace --all-targets -- -D warnings
```

pass, and a manual run works: build with `--features capture`, copy
`target\release\livelinguist_core.dll` to `ui\livelinguist-core.node`, run
`cd ui; bun src/main.tsx`, choose **System audio**, play a YouTube video:
captions (demo ASR text is fine for now) start appearing. Save a screenshot as
`docs/screenshots/m2-windows/console-system-audio.png`.

Commit message: `bridge: system audio source via loopback`
