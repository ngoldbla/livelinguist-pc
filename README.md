# Live Linguist PC

Native, on-device **same-language simplification captions** for Windows 11 x64,
built with a Rust core and a GPUIX + React UI. Linux x86-64 is the
development/evaluation environment. No translation, no cloud, no audio leaving
the machine.

This repo is the PC spin-out of [live-linguist](https://github.com/ngoldbla/live-linguist)
(the macOS/iOS app). The product conceit is unchanged: a learner hears authentic
native-speed speech and reads the same content rendered into the official
easy-language register for that language — **FALC**, **Leichte Sprache**,
**Easy English**, **Lectura Fácil**, and 25 further locales.

## Source of truth

- **Docs map:** [`docs/README.md`](docs/README.md) — start here
- **Execution plan (v1.0, Windows):** [`docs/plans/2026-11-23-windows-v1-plan.md`](docs/plans/2026-11-23-windows-v1-plan.md)
  and the copy-paste [agent prompt](docs/plans/2026-11-23-windows-agent-prompt.md)
- **v1.0 scope:** [`docs/plans/2026-11-22-v1-scope.md`](docs/plans/2026-11-22-v1-scope.md)
- **Progress board:** [`docs/qa/implementation-progress.md`](docs/qa/implementation-progress.md)
- **Decisions:** [`docs/decisions/`](docs/decisions/) (canonical ADRs)
- **Architecture reference:** [`docs/plans/2026-11-21-live-linguist-pc-master-plan.md`](docs/plans/2026-11-21-live-linguist-pc-master-plan.md)
- **Agent rules:** [`AGENTS.md`](AGENTS.md)

## Scope (v1.0)

A single-screen Windows 11 x64 (23H2+) user installs per-user without admin,
plays a call or video, sees live verbatim captions from **system audio**
(WASAPI loopback) where the locale passes the ASR gate, and **clicks** any
sentence to get a same-language easy-language rewrite in any of the **29**
packs. Inference is CPU-only (sherpa-onnx Nemotron 3.5 ASR + llama.cpp Qwen3
0.6B EasyLanguage), models are downloaded on first run and hash-verified.

Not in 1.0: microphone, per-app capture, multi-monitor/mixed-DPI, GPU
inference, 1.7B, automatic simplification, Windows 10/ARM64, Linux/macOS
packages, translation, cloud. Full list: [`docs/plans/2026-11-22-v1-scope.md`](docs/plans/2026-11-22-v1-scope.md).

## Repository layout

```
core/crates/    Rust workspace: ll-core, ll-config, ll-audio, ll-asr, ll-llm,
                ll-models, ll-pipeline, ll-protocol, ll-bridge, ll-eval, ll-window
ui/             GPUIX 0.11 + React 19.2 + TypeScript
assets/packs/   29 canonical FrameworkPacks (ported verbatim from the Mac repo)
assets/locales/ 29 UI locale JSON files
tests/          golden inventory, fixtures, bridge harness
packaging/      NSIS + portable ZIP
docs/           plan, ADRs, QA evidence
```

## Build

Linux dev prerequisites: Rust 1.98 (pinned by `rust-toolchain.toml`), Bun 1.4.5,
Node 22+, Vulkan (lavapipe works for headless CI; a real GPU is better for the
window), X11 for the dev overlay, and the XCB development headers:
`sudo apt install libxcb1-dev libxcb-shape0-dev libxcb-xfixes0-dev`.

```bash
cargo test --workspace
cargo run -p ll-eval -- --scenario mock --out /tmp/ll-eval-out

cd ui
bun install --frozen-lockfile
bun run start
```

On a Wayland desktop, force X11 for overlay development:

```bash
env -u WAYLAND_DISPLAY bun ui/src/main.tsx
```

## Status

Done: deterministic core and 29 packs, pipeline and `ll-eval`, napi bridge,
GPUIX UI on demo data, Windows toolchain and verify script, measured renderer,
single-screen Win32 overlay, pinned v1.0 models.

Next (see the plan): WASAPI loopback capture, sherpa-onnx streaming ASR and
the 29-locale coverage matrix, the exact EasyLanguage prompt contract and CPU
llama.cpp, 150% layout and accessibility, end-to-end integration, packaging,
release candidate.

UI portfolio: [`docs/screenshots/m2/`](docs/screenshots/m2/README.md).

![Console](docs/screenshots/m2/console.png)

![Live transcript](docs/screenshots/m2/live.png)

![Caption overlay](docs/screenshots/m2/overlay.png)

## Design invariants (inherited)

- Never translate; same-language rewriting only.
- Never drop a finalized segment: a late or unsimplified caption beats no caption.
- Validators are confidence signals, not gates; fall back to verbatim with "≡".
- Degradation order: batch -> step down model size -> skip simplification
  (verbatim) -> never drop audio.
- Latency instrumentation is built in, and exports are content-free.
- Pinned model revisions and checksums; no silent model drift.
- No audio or transcript leaves the device.

## Reference

- Mac/iOS implementation: <https://github.com/ngoldbla/live-linguist>
- Product spec: `PRD.md` in the reference repo

## License

MIT (see [`LICENSE`](LICENSE)). Third-party components keep their own
licenses: [`packaging/licenses/THIRD_PARTY.md`](packaging/licenses/THIRD_PARTY.md).
Models are downloaded on first run and are not part of this repository.
