# Third-party notices

Live Linguist PC is built on the following projects. Exact versions are pinned in
`Cargo.lock` and `ui/bun.lock`.

## UI and runtime

- GPUIX (`@gpuix/react`, `@gpuix/native`) 0.11.0 - Apache-2.0
- GPUI (Zed) - Apache-2.0
- React / React DOM 19.2.3 - MIT
- Bun 1.4.5 - MIT
- Lucide icons - ISC (SVG paths vendored in `ui/src/aurora/iconData.ts`)
- M PLUS Rounded 1c - SIL Open Font License 1.1 (`ui/assets/fonts`)

## Inference

- llama.cpp / `llama-cpp-2` - MIT
- Qwen3 0.6B EasyLanguage `Q4_K_M.gguf` - Apache-2.0 (LoRA fine-tune of
  `Qwen/Qwen3-0.6B`); pinned at
  `ndgold/Qwen3-0.6B-EasyLanguage-GGUF@658db5ad`
- sherpa-onnx - Apache-2.0 (runtime and int8 conversion tooling)
- NVIDIA Nemotron 3.5 ASR 0.6B - NVIDIA Open Model License (OpenMDW v1.1);
  int8 files mirrored from the sherpa-onnx `asr-models` release with provenance
  in `docs/qa/2026-11-22-model-pins.md`

No VAD model is shipped; the energy VAD is code.

## Rust crates

- serde / serde_json — MIT OR Apache-2.0
- reqwest / rustls — MIT OR Apache-2.0
- sha2 — MIT OR Apache-2.0
- ringbuf — MIT OR Apache-2.0
- x11rb — MIT OR Apache-2.0
- ts-rs — MIT
- napi / napi-derive — MIT
- fs2 — MIT OR Apache-2.0

## Fonts and data

- Noto Sans / Noto Sans Arabic — SIL Open Font License 1.1
- Framework packs — product data; national easy-language standards are cited in
  each pack's prompt. Distribution requires the same rights as the Mac app.

No models are bundled. The app downloads public, tokenless artifacts only after
the user starts a download; both models are pinned by SHA-256 before first use.
