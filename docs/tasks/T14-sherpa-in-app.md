# T14 — Use real ASR in `ll-eval` and the app (S2)

**Needs:** T13 done.

**Goal:** real captions from system audio in the app when the models are
downloaded; an honest "model missing" state when they are not.

**Files:** `core/crates/ll-eval/Cargo.toml`, `core/crates/ll-eval/src/lib.rs`,
`core/crates/ll-bridge/Cargo.toml`, `core/crates/ll-bridge/src/lib.rs`

## Steps

1. Both crates: add feature `sherpa = ["ll-asr/sherpa"]` (not default).
2. `ll-eval`: add flag `--asr sherpa` (default stays the current mock/scripted
   engine). With it, build `SherpaAsr::new(model_dir(cache_root(), <asr repo>,
   <asr revision>), language)` from the catalog; on error exit with the error
   code text. Works with `--wav` and `--source loopback`.
3. Bridge `start`: when built with `sherpa` and the request's scenario is not
   a demo/mock scenario, use `SherpaAsr` the same way. On `ModelMissing` emit
   an `ASR_MODEL_MISSING` error event and do **not** fall back to scripted
   text. Demo/mock scenarios keep the scripted engine.
4. Test (default features): `ll-eval` rejects `--asr sherpa` with a clear
   "not built with sherpa" error.

## Check

```powershell
cargo run --release -p ll-eval --features sherpa -- --scenario mock --language en-US --asr sherpa --wav target\asr-smoke\en.wav --out target\eval-sherpa
```

shows ≥ 1 segment. Then build the bridge with
`--features capture,sherpa`, copy the DLL to `ui\livelinguist-core.node`, run
the UI, pick **System audio**, play an English YouTube video: real captions
appear. Screenshot → `docs/screenshots/m2-windows/console-live-asr.png`.
Record segment count and RTF (no caption text) in
`docs/qa/2026-11-23-asr-runtime.md` under `## In app`.

## Done when

Checks above pass, plus `cargo test --workspace` and clippy with and without
`--features sherpa`. Then open PR `windows: sherpa-onnx streaming ASR` from
branch `stage-w3-asr-runtime` (T11–T14), merge when CI is green, and set S2 to
`merged` (first-run download UI is a later card) in the progress board.

Commit message: `asr: real sherpa engine in eval and bridge`
