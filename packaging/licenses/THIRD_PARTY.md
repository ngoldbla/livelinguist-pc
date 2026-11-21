# Third-party notices

Live Linguist PC is built on the following projects. Exact versions are pinned in
`Cargo.lock` and `ui/bun.lock`.

## UI and runtime

- GPUIX (`@gpuix/react`, `@gpuix/native`) 0.11.0 — Apache-2.0
- GPUI (Zed) — Apache-2.0
- React / React DOM 19.2.3 — MIT
- Bun 1.4.5 — MIT

## Inference

- llama.cpp / `llama-cpp-2` — MIT
- Qwen3 0.6B / 1.7B and the EasyLanguage fine-tunes — Apache-2.0 (model cards
  must be checked at publish time)
- sherpa-onnx — Apache-2.0
- Silero VAD — MIT (verify upstream)
- Nemotron 3.5 ASR — NVIDIA model license; resolve before publishing the ONNX
  export

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
the user starts a download; hashes are required at publish time.
