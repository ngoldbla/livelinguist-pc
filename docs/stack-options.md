# PC Stack Options — working matrix

Everything here is a hypothesis until an M0 spike produces numbers. The goal is
to keep one primary stack per layer and record the alternative that would replace
it. Any choice must be shippable from a package manager or bundle, work
offline after model download, and be licensable for redistribution.

## Audio capture

| Option | Mic | System audio | Notes |
|---|---|---|---|
| PipeWire (native API) | ✅ | ✅ via monitor source | Default on modern Ubuntu/Fedora. Monitor capture is a *source*, not a permissioned API — the app can list `*Monitor` sources, but the user may need to pick the right output device. |
| PulseAudio (compat) | ✅ | ✅ via `.monitor` | Still what many apps link against; PipeWire provides the compatibility server. |
| Virtual sink + loopback | ✅ | ✅ deterministic | Most product-like: route target apps (Zoom, browser) into a LiveLinguist sink so capture is stable regardless of the user's output device. Requires a one-time setup step. |
| ALSA direct | ✅ | ⚠️ | Bypasses the sound server; fragile with multiple apps. Fallback only. |

**Open questions:** Can we create the virtual sink without root? (PipeWire
`pw-loopback` / `module-remap-source` as user services — yes, but distro
differences matter.) What happens under Flatpak sandboxing? (PipeWire access is
permitted; PulseAudio socket may be blocked.)

## ASR

| Option | Model shape | Streaming | Notes |
|---|---|---|---|
| sherpa-onnx | Transducer/CTC zipformer, NeMo exports, Parakeet | ✅ | C++ core, ONNX Runtime, small footprint, permissive licensing. The Windows PRD already chose it. Best default candidate. |
| ONNX Runtime directly | Anything exported | Depends on model | More control, more glue. Needed if the Nemotron streaming export is only available as raw ONNX. |
| whisper.cpp | Whisper GGML | ⚠️ pseudo-streaming (windowed) | Mature and easy, but not cache-aware streaming; word-to-glass worse for live captions. Good fallback and a useful quality baseline. |

**Open questions:** Is there an ONNX/CoreML-free export of NVIDIA Nemotron 3.5
streaming 0.6B (NeMo RNN-T) that sherpa-onnx or ORT can run? If not, does
Parakeet/Canary streaming reach comparable French/German quality? The Mac
measured ~19 ms/chunk on ANE; CPU RTF is the M0 number that matters.

## Simplification LLM

| Option | Runtime | x86 acceleration | Notes |
|---|---|---|---|
| llama.cpp | GGUF, Q4_K_M etc. | AVX2/AVX-512, Vulkan, CUDA, ROCm, SYCL | De facto standard for on-device GGUF; the Windows and Android PRDs both assume it. Chat templates render from `chat_template.jinja`, `enable_thinking=false`. |
| ONNX Runtime GenAI | ONNX | CPU, DirectML, CUDA | Viable, less mature ecosystem for 4-bit LLMs on Linux. |
| ExLlamaV2 / vLLM | GPU only | CUDA/ROCm | Not suitable for a consumer desktop app. |

**Open questions:** Do the fine-tuned EasyLanguage checkpoints have GGUF
conversions, or must we convert from the fused 4-bit safetensors? (`mlx_lm.convert`
produced the Mac artifacts; `llama.cpp` needs its own quantize path.) If only
safetensors exist, the port needs a one-time conversion pipeline in CI, and the
catalog must pin the resulting GGUF hashes.

## UI and overlay

| Option | Stack | Overlay feasibility | Notes |
|---|---|---|---|
| Qt 6 / QML | C++ | X11: easy. Wayland: wlr-layer-shell on wlroots; KWin rules on KDE; GNOME needs an extension | Best cross-distro control and mature packaging. |
| GTK 4 | C | Same Wayland problem | Native on GNOME; weaker on KDE. |
| Tauri | Rust + web UI | Same Wayland problem | Small binaries, fast UI iteration, but WebKitGTK dependency. |
| Avalonia / .NET | C# | Same Wayland problem | Good Windows symmetry if Windows is a real target. |

**Open questions:** Is a "join all workspaces, always on top" caption window
achievable on stock GNOME Wayland without an extension? If not, the honest
options are (a) require X11 for overlay mode, (b) ship a GNOME extension, or
(c) fall back to a normal window the user keeps beside the call.

## Language for the core pipeline

The deterministic layer (packs, validators, segmentation, reversion, metrics) is
pure logic and portable. The ASR/LLM bindings and UI push toward C++ or Rust.

| Option | Pros | Cons |
|---|---|---|
| C++ | Direct llama.cpp/ONNX/sherpa APIs | Slower to test, weaker safety story |
| Rust | Safe FFI to the same C APIs, strong testing | More binding glue (llama.cpp, sherpa-onnx) |
| Swift on Linux | Reuse of parser/validator code shape from the reference | No MLX, thin C++ interop, weaker desktop UI story |

**Recommendation to validate:** Rust (or C++) core with a thin UI shell, and port
the JSON packs/validators as a shared, test-first module. The packs are data;
the validators are the part worth reproducing exactly.

## Packaging and model delivery

- Flatpak (recommended primary) + AppImage (portable) + distro `.deb`.
- Models to `$XDG_DATA_HOME/livelinguist/models/<repo>/<revision>/`, resumable
  downloads, SHA-256 verification, disk preflight.
- Same pinned-revision discipline as the reference catalog; conversion artifacts
  (GGUF/ONNX) are pinned by hash per release.
