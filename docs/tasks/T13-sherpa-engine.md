# T13 — Real `SherpaAsr` engine (S2)

**Needs:** T12 done.

**Goal:** replace the placeholder `SherpaAsr` with a working streaming engine.

**Files:** `core/crates/ll-asr/src/lib.rs` (the `#[cfg(feature = "sherpa")]` part)

## Steps

1. `SherpaAsr::new(model_dir, language) -> Result<Self, AsrError>`:
   - if any of the 4 files is missing → `AsrError::ModelMissing { .. }`
     (keep the existing fields; set `size_mb: 682`);
   - build the recognizer exactly as in `sherpa_probe.rs` (move the config
     building into a shared `fn recognizer_config(model_dir) -> OnlineRecognizerConfig`
     and use it from both); `None` → `AsrError::Unavailable("sherpa recognizer failed to load")`;
   - create one stream; apply the language option found in T12 using
     `language.base_code()` (skip if T12 found none).
   This changes the constructor's signature; update callers.
2. `process(chunk, audio_time)`: `accept_waveform(16000, chunk)`, decode while
   ready, read `get_result`. Return `Some(AsrEvent { hypothesis: text,
   audio_time, detected_language: None })` when the text is non-empty, else
   `None`. If `is_endpoint` is true **after** building the event, call
   `recognizer.reset(&stream)` so the next utterance starts fresh.
3. `reset()`: `recognizer.reset(&stream)`.
4. The engine must be `Send` (the trait requires it). If the sherpa types are
   not `Send`, wrap them in a struct with `unsafe impl Send` **only if** the
   crate docs say they are safe to move between threads; otherwise stop with
   `blocked`.
5. Tests (in `ll-asr`, feature `sherpa`):
   - `missing_model_dir_is_model_missing` — always runs;
   - `transcribes_tts_clip` — runs only when env `LL_ASR_MODEL_DIR` and
     `LL_ASR_SMOKE_WAV` are set (return early with a printed "skipped"
     otherwise); feeds the T12 clip in 17 600-sample chunks and asserts the
     final hypothesis contains "meeting" (case-insensitive).

## Done when

```powershell
cargo test -p ll-asr --features sherpa
$env:LL_ASR_MODEL_DIR="<asr model dir>"; $env:LL_ASR_SMOKE_WAV="target\asr-smoke\en.wav"; cargo test -p ll-asr --features sherpa transcribes_tts_clip -- --nocapture
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy -p ll-asr --features sherpa --all-targets -- -D warnings
```

Commit message: `asr: streaming SherpaAsr engine`
