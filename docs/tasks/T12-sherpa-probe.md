# T12 — sherpa-onnx streaming probe (S2)

**Needs:** T11 done (models on disk).

**Goal:** prove the pinned Nemotron model loads and transcribes speech through
the official `sherpa-onnx` Rust crate on this machine.

**Files:** workspace `Cargo.toml`, `core/crates/ll-asr/Cargo.toml`,
create `core/crates/ll-asr/examples/sherpa_probe.rs`,
`docs/qa/2026-11-21-version-verification.md`

## Steps

1. Workspace `Cargo.toml` `[workspace.dependencies]`: add
   `sherpa-onnx = "=2.0.1"` (official k2-fsa crate; default feature `static`
   links statically, so no extra DLLs). In `ll-asr/Cargo.toml`:
   `sherpa-onnx = { workspace = true, optional = true }` and
   `sherpa = ["dep:sherpa-onnx"]`. Add `[[example]] name = "sherpa_probe"`
   with `required-features = ["sherpa"]`.
2. Make a speech clip with Windows' built-in voice (local, no network, no PII):

   ```powershell
   Add-Type -AssemblyName System.Speech
   New-Item -ItemType Directory -Force target\asr-smoke | Out-Null
   $s = New-Object System.Speech.Synthesis.SpeechSynthesizer
   $fmt = New-Object System.Speech.AudioFormat.SpeechAudioFormatInfo 16000, ([System.Speech.AudioFormat.AudioBitsPerSample]::Sixteen), ([System.Speech.AudioFormat.AudioChannel]::Mono)
   $s.SetOutputToWaveFile("$PWD\target\asr-smoke\en.wav", $fmt)
   $s.Speak("The meeting starts at nine. Please bring your notes and a pen.")
   $s.Dispose()
   ```

   Do not commit this file (`target\` is ignored).
3. `sherpa_probe.rs` (args: model dir, wav path):
   - build `sherpa_onnx::OnlineRecognizerConfig` with
     `model_config.transducer.encoder/decoder/joiner` =
     `encoder.int8.onnx` / `decoder.int8.onnx` / `joiner.int8.onnx`,
     `model_config.tokens = tokens.txt`, `num_threads = 2`,
     `enable_endpoint = true`; leave other fields at `Default::default()`;
   - `OnlineRecognizer::create(&config)` (returns `Option`; print a clear
     error and exit 2 on `None`);
   - decode the wav with `ll_audio::decode_wav` (+ `mono_16k()`), feed it in
     560 ms pieces with `stream.accept_waveform(16000, piece)`, call
     `recognizer.decode(&stream)` while `recognizer.is_ready(&stream)`;
     at the end `stream.input_finished()` and decode again;
   - print `text=<result text>` and the elapsed time, audio seconds, and
     RTF = elapsed / audio seconds.
   If `model_config.model_type` must be set for this model and the default
   fails, look in the sherpa-onnx GitHub repo for "nemotron" examples and use
   the same value. Do not guess more than twice.
4. Run: `cargo run --release -p ll-asr --features sherpa --example sherpa_probe -- <asr model dir> target\asr-smoke\en.wav`
5. **Language:** search the sherpa-onnx repo/examples for how Nemotron 3.5
   receives the language (a stream option via `stream.set_option(key, value)`
   or a config field). If found, set it to `"en"` and note the key. If not
   found in two searches, write "language conditioning: not found" in the
   evidence and continue without it.

## Done when

- The probe prints text that clearly matches the sentence (small differences
  are fine) and RTF < 1.
- `cargo clippy -p ll-asr --features sherpa --all-targets -- -D warnings` passes.
- `docs/qa/2026-11-23-asr-runtime.md` has a `## Probe` section with crate
  version, load time, RTF, whether the text matched (yes/no, not the text), and
  the language-option finding.
- `docs/qa/2026-11-21-version-verification.md` has a row for
  `sherpa-onnx 2.0.1`.

Commit message: `asr: sherpa-onnx probe for the pinned Nemotron model`
