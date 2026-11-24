# Windows feedback, 2026-11-24 (owner run on the Windows 11 PC)

| Area | State | Cause / next step |
|---|---|---|
| Startup, native core, models | works | — |
| Model download (start / progress / cancel / resume) | works | T16 Windows check effectively passed; screenshots still to file |
| Real ASR from system audio (bridge level) | works — 283 partials + final segment, 0 errors | T14 bridge path verified on Windows |
| System audio through the UI | **does not work** — UI swaps it for demo | `ui/src/App.tsx:130` maps `mic`/`monitor` to `demo` before `transport.start`. Fix: pass `monitor` through (keep `mic` blocked: post-1.0), show `AUDIO_SOURCE_UNAVAILABLE`/`ASR_MODEL_MISSING` errors instead of falling back |
| Click-to-simplify | works, but mock backend (GGUF never loaded) | Follows from the swap above: with source `demo` + scenario `mock`, `use_real_asr` is false, so the bridge never builds `lazy_llama`. Also the addon must be built with `--features capture,sherpa,llama`. Fix both, and surface which simplifier is active in runtime status |
| All screens, overlay policy | render; overlay is topmost + no-activate | — |
| Export from the UI | **fails** — hard-coded `/tmp` path | `ui/src/App.tsx:269`. Fix: use the GPUIX save prompt (`promptForPaths`/rfd) or default to the user's Documents folder; bridge export already takes a path |
| Single .exe / installer | **does not start** (GPUIX addon cannot be bundled) | The GPUIX native `.node` addon cannot be embedded in a single-file exe. Packaging must ship Bun/Node + `@gpuix/native` `.node` + `livelinguist-core.node` side by side (portable folder / NSIS installing a folder). Needs a packaging decision (ADR) before T37 |
