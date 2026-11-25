# ASR runtime evidence (S2: T11–T14)

Linux dev container (4 cores, 15 GB RAM), 2026-11-24. **Every RTF/latency
number here is a Linux 4-core approximation, not the Windows tier C gate.**
No transcript or caption text is recorded here.

## Model files

`cargo run --release -p ll-models --example fetch_models -- <root>` (shared
`download_catalog` loop: disk preflight, resume, SHA-256). First run into an
empty root: exit 0 in 32 s. Second run: exit 0 in 0.9 s; nothing was
re-downloaded and every hash was re-checked.

| Artifact | Directory (relative to the model root) | File | Bytes |
|---|---|---|---|
| asr-nemotron-3.5-0.6b-560ms | `ndgold/nemotron-3.5-asr-streaming-0.6b-onnx/f73e3bc81614fa69e24b091b892e8672983fbb9c` | encoder.int8.onnx | 657,601,403 |
| | | decoder.int8.onnx | 14,978,075 |
| | | joiner.int8.onnx | 9,504,438 |
| | | tokens.txt | 131,440 |
| llm-qwen3-0.6b-easylanguage | `ndgold/Qwen3-0.6B-EasyLanguage-GGUF/658db5ad6a05e3f549990b10e8d8c59a78866695` | Qwen3-0.6B-EasyLanguage-Q4_K_M.gguf | 396,704,608 |

Root = `cache_root()` (`%LOCALAPPDATA%\LiveLinguist\models` on Windows,
`$XDG_DATA_HOME/livelinguist/models` on Linux, or `LL_MODELS_DIR`).

## Probe

- Crate: `sherpa-onnx` =2.0.1, static CPU libs. Behind the proxy, the `-sys`
  build script's own TLS does not trust the proxy CA. The archive was fetched
  with curl from the same k2-fsa release URL and passed via
  `SHERPA_ONNX_ARCHIVE_DIR`. GitHub CI runners download it directly.
- Config: transducer encoder/decoder/joiner/tokens, `num_threads = 2`,
  provider `cpu`, endpointing on, all else default. No `model_type` needed.
- **Speech clip:** Windows TTS is unavailable here. The clip is one FLEURS
  `en_us` **test** clip (`google/fleurs@70bb2e84`, `1003119935936341070.wav`,
  10.56 s, CC-BY-4.0, Google). It was downloaded into `target/asr-smoke/` and
  never committed.
- Results: load 1,864 ms; audio 10.56 s; decode 2.41 s; **RTF 0.228**;
  1 endpoint. Text matched the reference: **yes** (WER 0.211 on 19 reference
  words; the differences are number formatting and hyphenation).
- **Language conditioning: not found.** No language option for streaming
  transducers in the sherpa-onnx 2.0.1 C/C++ API strings (only FunASR-nano and
  Qwen3-ASR have one). The engine sets no option.

## Engine (T13)

`SherpaAsr::new(model_dir, language)` returns `ModelMissing { size_mb: 682 }`
when a file is missing and `Unavailable` when the model fails to load. Tests:
`missing_model_dir_is_model_missing` (always runs) and `transcribes_tts_clip`.
With `LL_ASR_MODEL_DIR`, `LL_ASR_SMOKE_WAV` (the FLEURS clip) and
`LL_ASR_SMOKE_WORD` (a word from its reference), the clip was fed in
17,600-sample chunks, and the test passed (20 words recognized).

## In app

- `ll-eval --features sherpa --scenario mock --language en-US --asr sherpa
  --wav <FLEURS clip>`: **1 segment**, WER 0.211 (same as the probe).
  - Before the endpoint fix (see ADR 0001 addendum 2026-11-24): 8 overlapping
    segments, WER 1.05, because the energy VAD gated quiet speech and split it
    per chunk.
- Bridge: with `--features sherpa`, system audio (and any non-mock scenario)
  uses `SherpaAsr`. A missing model emits `ASR_MODEL_MISSING` and the start
  fails. There is no scripted fallback, and a build without sherpa gives real
  audio no ASR instead of fake captions.
- **Windows (2026-11-24, run through the UI):** bridge built with
  `--features capture,sherpa,llama`; Console → **System audio** → **Start
  captions**; a 16 kHz TTS clip (`target/asr-smoke/t20.wav`, project-authored
  sentence, played through the default output) was captured as loopback and
  decoded by `SherpaAsr`.
  - **1 finalized segment** (stamp `00:02`) from the 10.2 s clip, plus the live
    partial of the next clip updating while it played.
  - The UI shows no RTF; the same engine on this host measures RTF 0.255
    (p95 chunk 313.8 ms) in the Windows tier C row of
    `2026-11-23-asr-coverage-v1.md`, and RTF ≈ 0.21 on the Linux
    approximation.
  - Clicking the finalized sentence ran the real GGUF (T20) — see
    `2026-11-23-llm-cpu.md`.
  - Screenshots: `docs/screenshots/m2-windows/console-live-asr.png`,
    `console-click-simplified.png`. No call audio or private content: the clips
    are project-generated TTS of test sentences.
  - Found and fixed here: the live screen's ASR panel hard-coded the amber
    badge and "Live listening is not available for this language yet" even for
    a `ok` language, contradicting the session badge. Both now follow
    `language.asrStatus`.
