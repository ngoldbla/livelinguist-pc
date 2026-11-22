---
title: "Live Linguist PC - Master Plan"
subtitle: "Rust core + GPUIX TypeScript UI, on-device Easy-Language captions for Windows 11 (Linux is a dev/eval environment)"
status: draft
plan_version: 3
supersedes: 2
target: "Windows 11 x64 only"
date: 2026-11-21
repo: livelinguist-pc
branch: main
audience:
  - AI coding agents
  - human maintainers
source_of_truth: true
review_input: "Review: Live Linguist PC - Master Plan (2026-11-21, plan_version 1)"
---

# Live Linguist PC - Master Plan (v3)

> **Status:** working baseline. Decisions marked `accepted` are confirmed by the
> product owner. Decisions marked `baseline` are the working default and may be
> overturned with a new decision entry. As of v3 there are no `proposed`
> decisions; remaining open questions are in section 20.
>
> **v3 platform decision:** **Windows 11 x64 is the only v1 product and release
> target.** Linux x86-64 is the development/evaluation environment; Linux
> packaging is explicitly **not in this plan** (future track only). Windows 10
> and Windows ARM64 are out of scope. Development happens on the Linux side of a
> dual-boot box; Windows acceptance happens on the Windows side.
>
> **v2 incorporated the 2026-11-21 review.** Section 26 maps every review item
> to the section or decision that resolves it. The review's blocking items A1-A5
> and contradictions B1-B3 are resolved before M0 starts.
>
> **Resolved decisions:** D-007 (Win32 primary overlay; XWayland dev-only),
> D-023 (Windows-first), D-024 (Windows D3D supported floor), D-031
> (simplify-only locales), and D-051 (Dylan PC reference hardware) are accepted.
>
> **Source-of-truth rule:** this file is the integrated plan and cross-cutting
> index. Individual `docs/decisions/00NN-*.md` ADRs are canonical for the
> rationale and evidence of that specific decision. If an ADR and this file
> disagree, the ADR wins for that decision; update this file's decision index.
> If two ADRs disagree, the newer accepted ADR wins.

---

## 0. Agent quick start

Read in this order:

1. Sections 0-3 (rules, invariants, scope).
2. Section 5 (decision log) and the ADRs it links.
3. Section 4 (architecture) and the relevant crate/UI section.
4. The milestone in section 18 that you are working on.
5. Section 20 (open questions) before making any structural change.
6. Section 26 if you need to know how a review finding was resolved.

### 0.1 Keywords

- **MUST** - required for correctness or acceptance.
- **SHOULD** - expected unless a documented reason exists.
- **MAY** - optional.
- **MUST NOT** - forbidden; a change that violates this needs a new decision.

### 0.2 Stable IDs

| Prefix | Meaning |
|---|---|
| `INV-xxx` | Product invariant |
| `D-xxx` | Decision |
| `R-xxx` | Requirement |
| `M-x` | Milestone |
| `S-x` | M0 spike |
| `T-x` | Task |
| `X-xxx` | Risk |
| `Q-xxx` | Open question |

### 0.3 Golden rules for agents

1. **Never translate.** Same-language rewriting only.
2. **Never send audio or transcripts off the device.** The network is used only
   for model downloads and **user-initiated, opt-in** update checks.
3. **Never put transcript text in logs, metrics, crash reports, or exports
   unless the user explicitly exports it.** Diagnostics are content-free.
4. **Never replace llama.cpp or sherpa-onnx with custom inference kernels.**
5. **Never add DOM/web UI libraries.** GPUIX is not a browser; Tailwind, shadcn,
   Radix, Framer Motion, `react-icons`, and React Testing Library do not apply.
6. **Never claim a no-GPU GUI works.** D-024 accepted: the Windows UI needs
   D3D11/12; the Linux dev UI needs Vulkan; macOS later needs Metal. CPU-only
   applies to models, not the UI.
7. **Never silently fall back in strict dev mode.** Strict scenarios fail loudly.
8. **Never drop a finalized transcript.** Audio overflow is a measured, surfaced
   event that triggers degradation; it is not silent loss. See INV-006.
9. **Never ship secrets.** No Hugging Face tokens, no embedded credentials.
10. **Every event carries `{ v, seq, ts }`.** The UI drops events older than its
    last applied `seq`.
11. **Update the decision log and ADR** when changing a structural choice; add
    tests with code; keep diagnostics and exports content-free.
12. **Pin and review third-party skills.** No floating `npx skills add` in CI;
    vendor reviewed files or pin a commit SHA.

### 0.4 What changed in v2 and v3

**v3 (platform decision):**

- Windows 11 x64 is the only v1 product/release target.
- Linux is development/evaluation only; no Linux package in this plan.
- Windows 10 and Windows ARM64 are explicitly out of scope.
- D-007, D-023, D-024, D-031, and D-051 are accepted; milestone order is
  Windows-first (M2 = Windows app, M3 = Windows release).
- Go-to-market/test environment is the Windows side of a dual boot on Dylan PC.

**v2 (review fixes):**

- **A1:** Added a per-locale ASR matrix, an unsupported-locale policy, and
  ASR-coverage risk `X-017`.
- **A2:** Made the Nemotron -> sherpa-onnx export the first S-3 task, with an
  explicit fallback and license check.
- **A3:** Added the 100 events/s x 10 min delivery test (Bun and Node), the
  frame-pump fallback, and the Hermes N-API caveat.
- **A4:** Moved `WAYLAND_DISPLAY` control to the launcher/`AppRun`/`.desktop`;
  JS bootstrap is only a fallback.
- **A5:** Restated the audio invariant; added ring-buffer seconds, overflow
  counter, `AUDIO_OVERRUN`, and the definition of "batch".
- **B:** Resolved update-check/privacy, core-dump/log redaction, event envelope,
  ADR/plan authority, provisional overlay decisions, and the 2-D degradation
  ladder.
- **C:** Added X11 focus/window-type/ClientMessage details, Windows
  `WM_MOUSEACTIVATE` subclassing, unfocused-click testing, PipeWire monitor
  decision, VAD/ASR roles, xrun soak, on-demand queue policy,
  `cancelSimplify`, per-locale prefix cache, `enable_thinking=false`,
  publish-time hashes, `lastSeq`/`streamId`, `ts-rs` types, portal file dialogs,
  caption announcements, Linux dev packaging notes, and the RAM budget.
- **D:** Reordered M0 by risk, added S-0 and the dependency graph, seeded T-xxx
  tasks, made exit criteria measurable, proposed reference hardware, shrank the
  skills list, and added the golden-fixture inventory and EN reversion rule.
- **E:** Applied the small wording/consistency fixes.

---

## 1. Executive summary

Live Linguist PC is the x86-64 desktop port of the macOS/iOS Live Linguist app.
It listens to speech (microphone or system audio), transcribes it locally, and
lets the user click a sentence to rewrite it - in the **same language** - into
that language's official easy-to-read register (FALC, Leichte Sprache, Easy
English, Lectura Facil, and 25 further locales). It is not a translator.

The port uses:

- **Rust** for the deterministic pipeline, audio capture, ASR, LLM
  orchestration, model management, metrics, exports, and window policy.
- **GPUIX 0.11 + React 19.2 + TypeScript** for the native GPU-rendered UI, via
  Zed's GPUI. No Electron, no web view.
- **llama.cpp** for the simplifier (Qwen3 0.6B/1.7B EasyLanguage GGUF), with
  dynamic CPU/Vulkan/CUDA backends and first-run calibration.
- **sherpa-onnx** for streaming ASR on CPU.
- **One GPUIX window** that morphs between console, compact caption overlay, and
  expanded transcript.
- **Native window policy** for topmost/no-activate behavior, without forking
  GPUIX. **Win32 is the primary overlay path**; Linux XWayland is dev-only;
  native Wayland layer-shell is a future track.

### 1.1 Goals

- Full feature parity with the Mac app's shipping on-demand UX.
- **Windows 11 x64 is the v1 product**; Linux is the developer/eval environment.
- On-device, offline after model download, no cloud inference.
- CPU-only machines work; GPU acceleration is automatic when it helps.
- **All 29 locales have an explicit ASR status.** Locales without usable
  streaming ASR are marked simplify-only and never pretend to transcribe.
- Clear developer controls to force CPU/GPU/mock/fault scenarios.
- Low-end machines degrade honestly, never silently or catastrophically.
- One maintainable codebase for Windows and Linux dev; macOS later if the
  Rust/TS app is meant to replace the Swift app.

### 1.2 Non-goals

- Translation of any kind.
- Cloud or hosted inference.
- Mobile, ARM Linux, Windows ARM64, or browser targets in v1.
- **Linux packaging/support in this plan** (future track only; the code stays
  portable so it can be added later).
- **Windows 10** (Windows 11 x64 only).
- Speaker diarization, meeting-bot integrations, per-app capture in v1.
- macOS/iOS parity in v1; those remain in the reference repo until a separate
  decision.

---

## 2. Product invariants

| ID | Invariant | Enforcement |
|---|---|---|
| INV-001 | Same-language only; never translate | Prompt packs, validators, review, eval fixtures |
| INV-002 | Two caption tiers: verbatim partial + simplified final | Pipeline state machine; UI renders both |
| INV-003 | On-demand simplification is the shipping default | UI calls `simplify(segmentId)`; live is advanced/dev |
| INV-004 | Validators are confidence signals, not gates | Low confidence shows amber + flags; never hides text |
| INV-005 | Fallback to verbatim with the `=` marker | Decoder/validator fallback path is tested |
| INV-006 | **Never drop a finalized transcript.** Audio overflow is a measured, surfaced event that triggers degradation, not silent loss. | Ring-buffer overflow counter; `AUDIO_OVERRUN`; degradation ladder; summary banner |
| INV-007 | On-device; network only for model downloads and **user-initiated opt-in** update checks | No background network; update checks off by default |
| INV-008 | Pinned model revisions and publish-time SHA-256 | Catalog + downloader; hashes generated at publish, not TOFU |
| INV-009 | No secrets in app or repo | No token fields; tokenless downloads |
| INV-010 | Transcripts are memory-only until explicit export | Session store; no background persistence |
| INV-011 | Exports and diagnostics are content-free; logs/crash reports never contain segment text | Redaction tests; no transcript in log events; core dumps disabled |
| INV-012 | Accessibility and cognitive clarity are release gates | AX tree tests; keyboard paths; caption announcement path |
| INV-013 | Latency instrumentation is built in | Per-stage timestamps; local diagnostics only |
| INV-014 | One window, one process (v1) | GPUIX constraints; no multi-window daemon |
| INV-015 | Every locale has an explicit ASR status (`ok`/`degraded`/`simplify_only`); unsupported locales never show a fake listening state | Per-locale ASR matrix; UI badges |
| INV-016 | Every core event is enveloped with `{ v, seq, ts }`; the UI applies events monotonically | Bridge envelope; `lastSeq` snapshot rule |
| INV-017 | Audio overflow is visible to the user and to diagnostics | `AUDIO_OVERRUN` counter + banner |

### 2.1 Definitions

- **Batch** (in INV-006): process already-captured audio in larger offline
  chunks instead of streaming, trading word-to-glass latency for throughput.
  It is the first degradation step, not a reason to discard audio.
- **Finalized transcript:** a segment the ASR/segmenter has committed to the
  session. Partials may be replaced; finalized segments may not be dropped.
- **Audio overflow:** the bounded capture ring buffer is full and the consumer
  cannot keep up. This increments `AUDIO_OVERRUN`, is surfaced in the UI, and
  triggers the degradation ladder.

---

## 3. Scope

### 3.1 In scope (v1)

- **x86-64 Windows 11 (23H2+) is the only v1 product and release target.**
- **Linux x86-64 is the development/evaluation environment**; it is not a
  packaged release target in this plan. The code stays portable for a future
  Linux track.
- Windows UI uses D3D11/12; model acceleration uses Vulkan and optional CUDA.
- Full pipeline: audio -> VAD -> ASR -> endpointing -> segmentation ->
  simplification -> validation -> two-tier captions.
- On-demand simplification default; automatic/live mode behind advanced settings.
- 29 locales with explicit ASR coverage; simplify-only mode for locales without
  usable streaming ASR.
- Console, caption overlay, transcript, settings, summary, onboarding, licenses.
- Export: Markdown, plain text, diagnostics CSV.
- Developer settings for CPU/GPU/mock/fault scenarios.
- Packaging: NSIS installer and portable ZIP for Windows 11 x64 only.

### 3.2 Out of scope (v1)

- Translation.
- Cloud inference or hosted APIs.
- Mobile, ARM Linux, Windows ARM64, browser/wasm shipping.
- **Linux packaging/support in this plan** (future track only).
- **Windows 10** (Windows 11 x64 only).
- Speaker diarization.
- Per-app audio capture (stretch after system loopback).
- macOS/iOS shipping (later decision).
- A second UI stack for no-GPU machines.
- Automatic (unprompted) update checks.

### 3.3 Hardware tiers

| Tier | Target hardware | RAM | Product behavior |
|---|---|---|---|
| A | Windows 11 dual boot on Dylan PC: dGPU + 8+ cores | 16 GB+ | Live ASR + live/on-demand simplify; LLM on GPU |
| B | Dylan PC with dGPU disabled / iGPU, or a GPU-less profile | 8-16 GB | Live ASR; on-demand simplify; GPU only if calibration wins |
| C | 4-core/8 GB Windows machine, or a CPU-limited Dylan PC approximation | 8 GB | On-demand only; 0.6B Q4; 2-3 threads; verbatim fallback |

Tier C RSS budget: **<= 3.5 GB total** for UI + core + ASR + 0.6B Q4, leaving
headroom for the OS and the call/video app. This is a hypothesis to measure.

**Caveat:** a high-end CPU with the GPU disabled is not a true tier C. It tests
the no-dGPU code path, not low-end CPU throughput. Use CPU affinity, 2-3
threads, and an 8 GB memory cgroup, and label it an approximation. True tier C
numbers require a low-end 4-core/8 GB machine or a 2-4 vCPU VM for headless
`ll-eval`.

### 3.4 Reference hardware (D-051; exact specs to fill)

| Tier | Machine | Specs | Purpose |
|---|---|---|---|
| A | **Dylan PC**, Windows 11 x64 (dual boot) | CPU/GPU/RAM: TBD | Full GPU path, D3D UI, WASAPI, overlay |
| B | Dylan PC with dGPU disabled / iGPU | TBD | iGPU vs CPU calibration, no-dGPU path |
| C | 4-core/8 GB Windows laptop or CPU-limited profile | TBD | Tier C model/UI budget; approximation if needed |
| Linux dev | Dylan PC, Linux side | Distro/kernel/GPU: TBD | Core/UI development, ASR/LLM eval |

Record exact CPU, GPU, RAM, Windows build, driver, and display scaling in every
measurement. No performance claim is valid until this table is filled.

---

## 4. Architecture

### 4.1 System diagram

```
+--------------------------------------------------------------+
|  GPUIX 0.11 + React 19.2 + TypeScript                        |
|  Aurora screens | session store | i18n | accessibility       |
+------------------------------+-------------------------------+
                               | napi commands + enveloped events
+------------------------------v-------------------------------+
|  livelinguist-core.node  (Rust, napi-rs)                    |
|                                                              |
|  ll-bridge -> ll-pipeline -> ll-asr / ll-llm                 |
|       |            |              |                          |
|       |            |              +-- llama.cpp (CPU/Vulkan)  |
|       |            +-- sherpa-onnx (CPU)                     |
|       +-- ll-audio (PipeWire/WASAPI)                         |
|       +-- ll-core (packs/validators/segmenter/prompt)        |
|       +-- ll-models (HF download/verify)                     |
|       +-- ll-window (topmost/no-activate policy)             |
+------------------------------+-------------------------------+
                               |
              PipeWire / WASAPI | sherpa-onnx | llama.cpp
```

### 4.2 Process model

- **v1: one process.** The GPUIX native addon and `livelinguist-core.node` are
  both loaded by the Bun/Node/Hermes runtime. The core is a normal Rust library
  behind a transport-neutral protocol (`ll-protocol`), exposed through a thin
  napi shell (`ll-bridge`).
- **Future:** if crash isolation or multi-window becomes mandatory, the same
  protocol can run over a sidecar daemon without changing the UI contract.
- The audio callback thread MUST NOT allocate, lock, block on JS, or log.
- The core MUST NOT panic across the FFI boundary.
- The **Windows launcher** owns Windows app identity/DPI settings. On Linux dev,
  a shell wrapper may force X11. `bootstrap.ts` is only a fallback.

### 4.3 Repository layout

```
livelinguist-pc/
├── core/                          # Rust workspace
│   ├── Cargo.toml
│   ├── crates/
│   │   ├── ll-core/               # pure logic, no audio/ML/UI
│   │   ├── ll-config/             # RuntimeConfig, DevOverrides, resolver
│   │   ├── ll-accel/              # measured DXGI/DRM/Vulkan/CUDA inventory
│   │   ├── ll-audio/              # capture, resample, VAD, ring buffer
│   │   ├── ll-asr/                # sherpa-onnx, per-locale, mock, off
│   │   ├── ll-llm/                # llama.cpp, calibration, model manager
│   │   ├── ll-pipeline/           # orchestration, queues, degradation
│   │   ├── ll-models/             # HF resolve, resume, SHA-256
│   │   ├── ll-window/             # X11/Win32/macOS window policy
│   │   ├── ll-protocol/           # command/event types + ts-rs
│   │   ├── ll-bridge/             # napi-rs cdylib
│   │   └── ll-eval/               # headless replay/eval CLI
│   └── vendor/                    # optional vendored native libs
├── ui/                            # GPUIX + React
│   ├── src/
│   │   ├── bootstrap.ts           # fallback env setup
│   │   ├── screens/
│   │   ├── aurora/                # design system/components
│   │   ├── store/                 # useSyncExternalStore session store
│   │   ├── i18n/
│   │   ├── bridge/                # generated types + core client
│   │   └── dev/                   # developer settings panel
│   └── assets/                    # aurora, fonts, icons
├── assets/
│   ├── packs/                     # 29 FrameworkPacks JSON
│   ├── locales/                   # 29 UI locale JSON files
│   └── licenses/
├── tests/
│   ├── golden/                    # ported Swift fixtures + INVENTORY.md
│   ├── scenarios/                 # dev scenario matrix
│   └── fixtures/                  # non-PII WAVs, prompt goldens
├── packaging/
│   ├── nsis/
│   └── packager.json
├── docs/
│   ├── decisions/                 # canonical ADRs
│   ├── qa/
│   └── budgets.md
├── scripts/
└── AGENTS.md
```

### 4.4 Crate responsibilities

| Crate | Owns | MUST NOT |
|---|---|---|
| `ll-core` | Models, packs, validators, segmenter, prompt builder, decoding, reversion, metrics, exporters | Touch audio, ML libs, OS windows, or UI |
| `ll-config` | `RuntimeConfig`, `DevOverrides`, precedence resolver, `RuntimeStatus` (including the measured `ui` status) | Contain platform-specific inference code |
| `ll-accel` | Measured UI renderer and accelerator inventory: DXGI/DRM adapters, Vulkan ICDs, CUDA driver devices | Declare GPU acceleration from `cfg!(target_os)` or own model policy |
| `ll-audio` | Device enumeration, capture, resampling, VAD, ring buffer, demo/file/silence sources | Know about transcripts or UI |
| `ll-asr` | Streaming ASR sessions, per-locale model selection, partial/final results, mock/off | Own segmentation or simplification |
| `ll-llm` | llama.cpp loading, backend devices, calibration, queue policy, model lifecycle, mock | Own prompt/validator policy |
| `ll-pipeline` | Stage orchestration, queues, timeouts, degradation, fault injection | Render UI or own model files |
| `ll-models` | Catalog, HF resolve, Range resume, SHA-256, disk preflight, cache paths | Embed secrets or auto-update silently |
| `ll-window` | Win32/X11/macOS window policies (topmost, no-activate, skip-taskbar, drag) | Render content or own app state |
| `ll-protocol` | Command/event structs, event envelope, `ts-rs` annotations | Depend on GPUIX |
| `ll-bridge` | napi exports, **TSFN** event delivery, panic boundary | Contain business logic |
| `ll-eval` | WAV replay, golden runs, latency CSV/JSON | Require the UI |

### 4.5 Threading model

```
[PipeWire/WASAPI callback] --lock-free ring buffer--> [audio worker]
                                                        |
                                                  [VAD gate]
                                                        |
                                          [streaming ASR + endpointing]
                                                        |
                                            partials + finalized segments
                                                        |
                                                  [pipeline worker]
                                                        |
                                          [LLM worker: on-demand / live]
                                                        |
                                            [event queue, bounded]
                                                        |
                                   [napi ThreadsafeFunction -> JS]
```

Rules:

- Audio callback: no allocation, locks, logging, or JS.
- VAD gates feeding audio to ASR to save CPU during silence.
- sherpa-onnx endpoint rules decide when an utterance ends; the segmenter splits
  sentences. The 1.1 s/10 s numbers are pipeline UI cadence, not ASR chunking.
- ASR worker: 2 threads (configurable).
- LLM worker: `n_threads = max(2, physical_cores / 2)` on CPU.
- Event queue: bounded; coalesce stale partials; never drop finalized segments,
  errors, or state changes.
- Every stage emits latency stamps.
- Event delivery is verified under GPUIX; if TSFN stalls, pump from the GPUIX
  frame/tick callback (S-2 fallback).

---

## 5. Decision log

Status legend: `accepted` = confirmed by product owner; `baseline` = working
default; `proposed` = needs sign-off; `open` = unresolved. ADRs in
`docs/decisions/` are canonical for evidence and rationale; this table is the
index. `docs/decisions/README.md` maintains the D-ID <-> ADR mapping; each ADR
links back to its D-ID. Until an ADR exists, this table is the only record.

| ID | Decision | Status | Rationale | Trade-off / revisit trigger |
|---|---|---|---|---|
| D-001 | UI is GPUIX 0.11 + React 19.2, exact-pinned | baseline | GPUIX chosen; React 19.2 is the recommended adapter | Pre-1.0 churn; Solid remains an alternative |
| D-002 | Rust core + TypeScript UI split | accepted | Rust for audio/ML, TS for UI velocity | Two languages; typed bridge required |
| D-003 | One window for v1 | accepted | GPUIX has no multi-window | No simultaneous overlay+sidebar |
| D-004 | Window modes: console, overlay, expanded | baseline | Fits single-window constraint | Mode switch complexity |
| D-005 | Core is an in-process napi addon | baseline | Lowest latency, typed calls, simple packaging | Core crash takes down UI; sidecar if needed |
| D-006 | Native window policy, no GPUIX fork | baseline | Keeps GPUIX upgradable | Per-platform native code |
| D-007 | Overlay platform: **Win32 primary** (topmost/no-activate); Linux XWayland dev-only; layer-shell future | accepted | Windows is the product; Win32 gives real topmost/focus control | Linux overlay is best-effort, not a release gate |
| D-008 | Overlay no-activate/pointer-first; console allows focus | baseline | Clicking captions must not steal focus | No overlay keyboard shortcuts |
| D-009 | Overlay is a compact bar, not full-screen | baseline | Avoids click-through complexity | Less cinematic |
| D-010 | Honest fallback if topmost/no-activate fails | baseline | Predictable support | User friction |
| D-011 | LLM runtime is llama.cpp (`llama-cpp-2`) | accepted | One GGUF across CPU/Vulkan/CUDA | C++ build |
| D-012 | ASR runtime is sherpa-onnx on CPU | accepted | Streaming transducer, int8 | No GPU ASR in v1 |
| D-013 | Dynamic backends + first-run calibration | baseline | Works without GPU; picks fastest | Calibration time |
| D-014 | CPU baseline; GPU accelerator | baseline | Widest hardware reach | Slower on CPU-only |
| D-015 | Vulkan universal; CUDA optional | baseline | Cross-vendor; no huge default runtime | CUDA faster on NVIDIA |
| D-016 | Models downloaded, pinned, verified | baseline | Small installer; no secrets | First-run network |
| D-017 | Deterministic layer pure Rust; no UniFFI | baseline | Clean ownership; tests are spec | Duplicates logic |
| D-018 | Developer settings with strict scenarios | accepted | Makes failures testable | More surface |
| D-019 | UI tests on Windows (primary) + Linux dev smoke; macOS later | baseline | Windows is the release target; GPUIX has a Windows test renderer | Linux dev CI is weaker; acceptable |
| D-020 | Custom JSON i18n; no i18next | baseline | 129 keys; small footprint | Hand-rolled RTL |
| D-021 | `useSyncExternalStore`; no Zustand/Redux | baseline | Simple event stream | Hand-rolled selectors |
| D-022 | Aurora pre-baked backgrounds + 2-stop gradients | baseline | GPUI has no radial gradients | Less dynamic |
| D-023 | **Windows 11 x64 is the v1 product; Linux is dev/eval only; Linux packaging is a future track; macOS later** | accepted | Windows is the user market; dual boot provides real acceptance testing | Windows-only v1; Linux package not in this plan |
| D-024 | **Windows supported GUI floor: D3D11/12; Linux dev floor: Vulkan; software fallback best-effort; no no-GPU GUI** | accepted | Windows is the product; D3D is broad; GPUIX Linux needs Vulkan | Software/WARP is degraded, not a release target |
| D-025 | Bun first; Hermes later | baseline | Documented hot-reload path | Hermes N-API caveat (D-034) |
| D-026 | No custom inference kernels | accepted | llama.cpp/sherpa-onnx solve it | C++ dependency |
| D-027 | No DOM/web UI libraries | baseline | GPUIX is not a browser | Build Aurora components |
| D-028 | Skills: 2 custom + max 3 external, pinned/reviewed | baseline | Supply-chain/prompt-injection reduction | Less breadth |
| D-029 | Accessibility + cognitive clarity are release gates | baseline | Product audience | GPUIX a11y gaps |
| D-030 | Qwen3 0.6B Q4_K_M primary; 1.7B escalation; Nemotron ASR | baseline | Matches Mac fine-tunes | Quality gate may force Q8_0 |
| **D-031** | **Per-locale ASR policy:** matrix in section 12; locales without usable streaming ASR are `simplify_only` (paste/type + simplification, no fake listening) | accepted | Prevents a false "listening" promise; preserves the simplification value | Some locales have no live captions; UI must be explicit |
| **D-032** | **ASR export is a kill-risk gate:** first S-3 task is export -> `sherpa-onnx` `OnlineRecognizer` -> streaming endpointing. Fallback: per-language streaming zipformer. | baseline | The custom Nemotron ONNX export may fail | If it fails, more per-language models and downloads |
| **D-033** | **Event delivery test:** 100 events/s for 10 min under Bun **and** Node; p99 delivery into React state; fallback is a GPUIX frame/tick pump. | baseline | GPUI main loop may starve TSFN callbacks | If stalled, bridge redesign |
| **D-034** | **Launcher sets `WAYLAND_DISPLAY`**, not JS; assert GPUI chose X11 and record it in `RuntimeStatus`. | baseline | `process.env` may not reach native `getenv()` reliably | If X11 unavailable, native Wayland mode |
| **D-035** | **Audio overflow invariant:** never drop a finalized transcript; overflow is counted, surfaced, and degrades. | baseline | A bounded buffer cannot literally never drop | User sees a degradation banner |
| **D-036** | **Update checks are opt-in and off by default.** | baseline | Keeps INV-007 honest | Users may miss updates |
| **D-037** | **Core dumps disabled by default; crash reports contain stack/codes only, never transcript text.** | baseline | Privacy | Harder debugging; opt-in debug builds |
| **D-038** | **Event envelope `{ v, seq, ts, event }`; `getSnapshot()` returns `lastSeq`; UI drops `seq <= lastSeq`.** | baseline | Snapshot/event race | Slightly larger payloads |
| **D-039** | **ADRs canonical for rationale/evidence; this plan is the integrated index.** | baseline | Avoids two sources of truth | Must keep index current |
| **D-040** | **2-D degradation ladder:** model {1.7B, 0.6B} x backend {GPU full, GPU partial, CPU}, then verbatim. | baseline | The old ladder repeated 0.6B | More state to test |
| **D-041** | **On-demand queue policy:** one active + one queued; newest click replaces the queued item; live mode yields to on-demand; `cancelSimplify` exists. | baseline | Predictable UX; avoids unbounded queue | User clicks can replace queued work |
| **D-042** | **Per-locale prefix KV cache** with save/restore; measure prefill on tier C. | baseline | 0.6B prefill is the tier C bottleneck | Memory per locale; evict LRU |
| **D-043** | **Generate TS types from `ll-protocol` with `ts-rs`; napi is a thin pass-through.** | baseline | napi `--dts` handles unions poorly | Extra codegen step |
| **D-044** | **`rfd` uses native Windows dialogs; Linux dev uses the xdg-desktop-portal backend.** | baseline | No GTK main-loop conflict under Flatpak; Windows native is primary | Portal availability on Linux dev |
| **D-045** | **Caption announcements are a release gate**; if GPUIX lacks a live-region API, upstream it. | baseline | Screen-reader users need new captions | Upstream latency; fallback "read latest" action |
| **D-046** | **Keyboard access is guaranteed in console/expanded mode; overlay is pointer-first.** | baseline | Single-window + no-activate trade-off | Keyboard users must switch modes |
| **D-047** | **Skills are vendored/reviewed/pinned; 2 custom + max 3 external.** | baseline | Supply-chain safety | Less skill breadth |
| **D-048** | **S-0 version verification is the first task**; any pin that does not resolve is a plan bug. | baseline | Avoids building on fictional versions | Small upfront cost |
| **D-049** | **Golden fixture inventory is a file** (`tests/golden/INVENTORY.md`) with count/paths/provenance. | baseline | "Port verbatim" must be verifiable | Upfront inventory work |
| **D-050** | **Reversion detection is disabled for `en` learning sessions.** | baseline | "EN vs target" is meaningless when target is EN | UI hides EN pill for EN |
| **D-051** | **Reference hardware: Dylan PC (Windows 11 dual boot) is tier A; GPU-less/limited Dylan PC is tier B/C approximation; exact specs recorded per measurement.** | accepted | Tier hypotheses need machines; one box covers GPU/no-GPU paths | True tier C still needs a low-end machine/VM |
| **D-052** | **Linux packaging is a future track, not in this plan.** | accepted | User base is Windows; avoids v1 Linux packaging scope | Linux package deferred |
| **D-053** | **Windows 11 x64 only; Windows 10 and Windows ARM64 are out of scope.** | accepted | Reduces test surface; GPUIX is fine on 11 | Windows 10 users excluded |

---

## 6. Tech stack and pinning

| Component | Choice | Version policy |
|---|---|---|
| UI renderer | `@gpuix/react` + `@gpuix/native` | **0.11.0 exact**, both upgraded together |
| React | `react` / `react-dom` | 19.2.x, compatible with GPUIX peer range |
| TypeScript | `typescript` | Strict mode; pinned by lockfile |
| JS runtime | Bun | Pinned by `bun.lock`; Node 20+ alternative |
| Core bridge | `napi` / `napi-derive` | 3.16.x / 3.7.x; `@napi-rs/cli` 3.10.x |
| LLM | `llama-cpp-2` | 0.1.161; `dynamic-backends`, optional `vulkan`, `cuda` |
| ASR | `sherpa-onnx` | 2.0.2; official crate; vendor libs for offline CI |
| Audio | `cpal` | 0.18.x with `pipewire` feature on Linux |
| Resampling | `rubato` | 5.1.x |
| Ring buffer | `ringbuf` | 0.6.x |
| HTTP | `reqwest` + `rustls` | Pinned; no OpenSSL |
| Hashing | `sha2` | Pinned |
| Serialization | `serde` / `serde_json` | Pinned |
| TS types | `ts-rs` | Pinned; generated from `ll-protocol` |
| Native dialogs | `rfd` | 0.18.x; xdg-desktop-portal backend |
| Window policy | `x11rb`, `windows`, `objc2`/`objc2-app-kit` | Pinned per target |
| Rust toolchain | `rust-toolchain.toml` | Pin stable; verify MSRV |
| Packaging | `cargo-packager` | Pinned CLI |
| QA | `cargo-nextest`, `insta`, `proptest`, `criterion` | Dev-only |
| TS QA | `bun test`/`vitest`; Biome | Dev-only |

### 6.1 S-0 version verification

Before any code depends on a pin, verify it exists and record the resolved
version in `docs/qa/2026-11-21-version-verification.md`:

| Pin | Registry | Declared | Resolved | Date | Notes |
|---|---|---|---|---|---|
| `@gpuix/react` | npm | 0.11.0 | TBD | TBD | Must match `@gpuix/native` |
| `@gpuix/native` | npm | 0.11.0 | TBD | TBD | Prebuilt targets: Linux x64, Win x64, macOS arm64 |
| `react` | npm | 19.2.x | TBD | TBD | GPUIX peer range |
| `napi` | crates.io | 3.16.x | TBD | TBD | N-API ABI |
| `napi-derive` | crates.io | 3.7.x | TBD | TBD | |
| `llama-cpp-2` | crates.io | 0.1.161 | TBD | TBD | Features exist |
| `sherpa-onnx` | crates.io | 2.0.2 | TBD | TBD | CPU static libs |
| `cpal` | crates.io | 0.18.x | TBD | TBD | `pipewire` feature |
| `rfd` | crates.io | 0.18.x | TBD | TBD | Portal backend |
| Bun | runtime | pinned | TBD | TBD | Record exact version |

Any pin that does not exist or does not expose the required feature is a **plan
bug**; update this table and the decision log before coding.

---

## 7. Rust core specification

### 7.1 `ll-core` (pure logic)

Owns domain models, 29 framework packs, prompt builder, decoding, segmenter,
validators (`ValidatorSuite`, `FidelityGuard`, `NumberGuard`, `LanguageGuard`,
`TextAnalysis`), reversion detection, metrics, and exporters.

MUST:

- Have zero dependencies on audio, ML, OS windows, or UI.
- Compile/test on Windows (primary), Linux dev, and macOS later.
- Port Swift fixtures verbatim before changing behavior.
- Disable EN-reversion detection when the learning language is `en` (D-050);
  hide the EN pill in that case.
- Keep exports byte-compatible with the Mac reference once fixtures are in.

### 7.2 `ll-config`

Owns `RuntimeConfig`, `DevOverrides`, `RuntimeStatus`, precedence resolution, and
validation. `RuntimeStatus` includes:

- Requested vs effective backend/device/layers.
- Loaded backend modules and CPU variant.
- Measured UI renderer status (`ui.backend`, adapter, hardware flag, degraded
  reason) from `ll-accel`, plus the window policy result. Never declared from
  the target OS.
- Model/quant/hash; TTFT/tokens per second.
- Last calibration, fallback reason, strict state.
- Event-delivery health (`seq`, last event age, dropped count).

Precedence: CLI > env > dev overrides > user settings > calibration > defaults.

`ll-accel` owns the measurement: DXGI adapters on Windows, DRM render nodes and
Vulkan ICDs on Linux, and the CUDA driver API for accelerator devices. A
software-only adapter is degraded, not a GPU claim (ADR 0017).

### 7.3 `ll-audio`

| Source | Windows (primary) | Linux dev | Use |
|---|---|---|---|
| Microphone | WASAPI capture | PipeWire/Pulse/ALSA via cpal | Room/person |
| System audio | WASAPI loopback | PipeWire monitor source | Calls/videos |
| Virtual sink | n/a | `pw-loopback`/remap source | Stable routing |
| File | WAV replay | WAV replay | Eval/QA |
| Demo | Scripted audio/text | Same | UI tests |
| Silence | Synthetic zeros | Same | Pipeline timing |

Rules:

- Output 16 kHz mono f32; resample with `rubato`.
- **VAD role:** gate feeding ASR to save CPU during silence.
- **ASR endpointing:** sherpa-onnx decides utterance ends.
- **Segmenter:** splits sentences for the UI; it is not the ASR chunker.
- Ring buffer: **10 seconds** at 16 kHz mono f32 (~640 KB); overflow increments
  `AUDIO_OVERRUN` and triggers degradation.
- The launch/stop logs record xrun/overrun counts; no audio content.
- **WASAPI loopback is the primary system-audio path.** S-5W decides between
  `cpal` and the `windows`/`wasapi` crate.
- **Linux dev only:** S-5L decides between the `pipewire` crate and
  `libpulse-simple` over pipewire-pulse if cpal cannot enumerate monitor
  sources. Record the result in section 13.3.

### 7.4 `ll-asr`

- Per-locale model selection from the section 12 matrix.
- Primary candidate: Nemotron 3.5 streaming 0.6B transducer, int8 encoder.
- Fallback: per-language streaming zipformer/Parakeet models.
- Modes: `SherpaOnnx`, `Mock`, `Off`.
- CPU only in v1; no GPU provider in the official Rust crate.
- Emits partials with `streamId`/`utteranceId`, and finalized segments.
- `setLearningLanguage(code)`:
  - If the model is installed, swap the recognizer and reset the stream.
  - If not, emit `ASR_MODEL_MISSING` with the model size and a download action;
    show "model for this language not installed".
  - Never show "listening" for `simplify_only` locales.
- Tests: multilingual WER matrix, RTF, p95 chunk latency, endpoint behavior,
  model-missing path.

### 7.5 `ll-llm`

- Wrapper around `llama-cpp-2`; loads GGUF with `mmap` on, `mlock` off.
- Dynamic backend loading from `backends/cpu`, `backends/cpu-vulkan`,
  `backends/cpu-cuda`; `load_backends_from_path()` before backend init.
- Device enumeration: `list_llama_ggml_backend_devices()`; classify `Gpu`,
  `IntegratedGpu`, `Cpu`; read free/total memory.
- Model params: `with_devices`, `with_n_gpu_layers`.
- Calibration: fixed prompt, 32 generated tokens, temperature 0; CPU vs GPU;
  cache by hardware/driver/model hash.
- **2-D degradation ladder (D-040):**
  `1.7B x GPU-full -> 1.7B x GPU-partial -> 1.7B x CPU -> 0.6B x GPU-full ->
  0.6B x CPU -> verbatim`.
- **Queue policy (D-041):** one active + one queued; newest click replaces the
  queued item; live mode yields to on-demand; `cancelSimplify(segmentId)`
  cancels queued or active work for that segment.
- **Prefix cache (D-042):** cache the stable system/few-shot prefix per locale
  with sequence save/restore; measure prefill on tier C.
- **Qwen3:** `enable_thinking=false` in the chat template; think-block stripping
  remains a safety net only.
- **Hashes:** generated at publish time and committed; never trust-on-first-use.
- Modes: `LlamaCpp`, `Mock`, `Off`.
- Tests: TTFT/gen tok/s, prefill, p50/p95, OOM/fallback, queue ordering,
  calibration accuracy.

### 7.6 `ll-pipeline`

- State machine: `setup -> listening <-> paused -> summary`.
- Channels: audio -> VAD -> ASR -> segmenter -> transcript -> simplification.
- On-demand priority over live; queue policy from D-041.
- Emits enveloped events with stage latency.
- On audio overflow: increment counter, emit `AUDIO_OVERRUN`, switch to batch
  ASR, then smaller model, then pause with a banner; never delete finalized
  segments.
- Fault injection at stage boundaries (drop/error/timeout/OOM/delay).
- MUST preserve a final verbatim transcript even when simplification fails.

### 7.7 `ll-models`

- Catalog with repo, revision, file list, size, sha256, license.
- HF resolve URLs; tokenless public downloads.
- Range resume; retry with backoff; SHA-256 verification.
- Disk preflight 6 GB.
- Cache paths:
  - Linux: `$XDG_DATA_HOME/livelinguist/models/<repo>/<revision>/`
  - Windows: `%LOCALAPPDATA%\LiveLinguist\models\<repo>\<revision>\`
  - macOS (later): `~/Library/Application Support/LiveLinguist/models/`
- Progress events with bytes/sec and ETA.
- No silent drift; no embedded tokens; hashes published, not TOFU.

### 7.8 `ll-window`

Windows is the release-gated overlay platform. Linux X11 is a developer
convenience, not a v1 release gate.

#### Linux X11/XWayland (dev-only)

- Topmost: `_NET_WM_STATE_ABOVE` via **root-window ClientMessage** after map.
- Window type: default `_NET_WM_WINDOW_TYPE_UTILITY` for overlay; test
  `_NOTIFICATION` and `_DOCK` on Mutter/KWin; `NORMAL` for console.
- Focus:
  - Remove `WM_TAKE_FOCUS` from `WM_PROTOCOLS` in overlay mode.
  - Set `WM_HINTS.input = False`.
  - Set `_NET_WM_USER_TIME = 0`.
  - Do not call `XSetInputFocus` from the app in overlay mode.
- Skip taskbar/pager: `_NET_WM_STATE_SKIP_TASKBAR`,
  `_NET_WM_STATE_SKIP_PAGER`.
- All workspaces: `_NET_WM_DESKTOP = 0xFFFFFFFF` / `_NET_WM_STATE_STICKY`.
- Borderless: `_MOTIF_WM_HINTS`.
- Drag: `_NET_WM_MOVERESIZE`.
- Window discovery: `_NET_WM_PID` + a unique `WM_CLASS`/title nonce the app sets;
  never rely on title alone.
- Mode switch: test unmap -> set hints -> remap vs live ClientMessage; record
  flicker result.

#### Windows (primary, release-gated)

- `SetWindowPos(HWND_TOPMOST)`; `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW`.
- Subclass the window with `SetWindowSubclass`; return `MA_NOACTIVATE` from
  `WM_MOUSEACTIVATE` so a click cannot activate it.
- `DwmSetWindowAttribute(DWMWA_WINDOW_CORNER_PREFERENCE)` on Win11.
- Drag: `ReleaseCapture` + `WM_NCLBUTTONDOWN`/`HTCAPTION`.
- Find HWND by process ID + title/class nonce.
- Virtual desktops: verify; document the Task View toggle if pinning is
  unreliable.

#### macOS (later)

- `NSWindow.level = .floating`/`.statusBar`,
  `collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]`.
- Non-activating `NSPanel`; `hidesOnDeactivate = false`.
- Prebuilt GPUIX is arm64-only.

#### Wayland layer-shell (later, opt-in)

- `render(..., { layerShell: { layer: 'top', anchor,
  keyboardInteractivity: 'none' } })`.
- GNOME/Mutter lacks `wlr-layer-shell`; XWayland remains the fallback.
- Single window means the surface is always a layer-shell panel; console becomes
  a panel.
- Runtime keyboard-interactivity change requires an upstream GPUIX addition.

### 7.9 `ll-bridge` and `ll-protocol`

- `ll-protocol` defines commands/events and the envelope; `ts-rs` generates TS.
- `ll-bridge` exposes napi classes/functions; events go through
  **ThreadsafeFunction (TSFN)**.
- `catch_unwind` at every FFI boundary; structured errors.
- `getSnapshot()` returns `{ state, lastSeq }`.
- Event delivery fallback: pump from a GPUIX frame/tick callback if TSFN stalls
  (tested in S-2).
- Hermes: does not natively speak N-API; the Hermes path likely needs a different
  binding layer (D-025 caveat). Keep `ll-protocol` transport-neutral.

### 7.10 `ll-eval`

- `ll-eval --wav <file> --scenario <preset> --out <dir>`.
- Runs the real pipeline without UI.
- Outputs per-segment CSV, latency JSON, summary JSON.
- Uses golden fixtures and non-PII audio only.
- CI gate for M0/M1.

---

## 8. Bridge protocol

### 8.1 Commands (TS -> Rust)

| Command | Purpose | Notes |
|---|---|---|
| `start(config)` | Start a session | Returns when listening |
| `stop()` | End session, produce summary | Flushes final transcript |
| `pause()` / `resume()` | Pause/resume processing | No data loss |
| `simplify(segmentId)` | On-demand simplification | Queue policy D-041 |
| `cancelSimplify(segmentId)` | Cancel queued/active work | Required by D-041 |
| `setLearningLanguage(code)` | Change language | May swap ASR model |
| `listAudioSources()` | Enumerate devices | Includes monitor sources |
| `selectAudioSource(id)` | Select source | Live where possible |
| `downloadModels(manifest)` | Install models | Progress events |
| `cancelDownload()` | Stop download | Keep partials for resume |
| `export(format, path)` | Export | Explicit user action only |
| `getSnapshot()` | Late-attach state | Returns `{ state, lastSeq }` |
| `getRuntimeStatus()` | Requested vs effective | Dev panel + health |
| `applyRuntimeConfig(config)` | Apply dev overrides | Validates; may reload |
| `probeBackends()` | Enumerate backends/devices | Dev panel |
| `calibrate()` | Run CPU/GPU calibration | Returns chosen backend |
| `reloadModel()` | Unload/reload | For backend/model changes |
| `setWindowMode(mode)` | console/overlay/expanded | Applies window policy |

### 8.2 Event envelope

```ts
type Envelope<T> = {
  v: number        // protocol version
  seq: number      // monotonic per process
  ts: number       // monotonic milliseconds
  event: T
}

type CoreEvent =
  | { type: 'state'; state: 'setup'|'listening'|'paused'|'summary'; sessionId: string }
  | { type: 'partial'; streamId: string; utteranceId: string; text: string; confidence?: number }
  | { type: 'segment'; id: string; text: string; startMs: number; endMs: number }
  | { type: 'simplified'; id: string; text: string;
      status: 'clean'|'flagged'|'verbatim'; flags: string[]; latencyMs: number }
  | { type: 'reversion'; active: boolean; count: number }
  | { type: 'download'; repo: string; file: string; received: number; total: number;
      bytesPerSec: number; etaMs: number }
  | { type: 'runtimeStatus'; status: RuntimeStatus }
  | { type: 'calibration'; backend: string; device: string; ttftMs: number; tokensPerSec: number }
  | { type: 'latency'; stage: string; ms: number }
  | { type: 'audioOverrun'; count: number; droppedMs: number; recovery: string }
  | { type: 'error'; code: string; message: string; recoverable: boolean }
  | { type: 'log'; level: 'info'|'warn'|'error'|'debug'; code: string; message: string }
```

### 8.3 Rules

- Audio buffers MUST NOT cross the FFI boundary.
- Only transcript/status/download events cross into JS.
- Event queue bounded; coalesce partials; never drop finals/errors/state.
- `getSnapshot()` returns `lastSeq`; the UI drops `seq <= lastSeq`.
- `partial` events carry `streamId`/`utteranceId`; a later `segment` replaces the
  right partial.
- `log` events MUST NOT contain segment text; only codes and redacted messages.
- Errors use stable codes (`ASR_INIT_FAILED`, `ASR_MODEL_MISSING`,
  `LLM_OOM`, `MODEL_HASH_MISMATCH`, `AUDIO_DEVICE_LOST`, `AUDIO_OVERRUN`,
  `BACKEND_UNAVAILABLE`, etc.).
- No `any` in TS; `ll-protocol` + `ts-rs` is the contract. CI checks generated
  types are committed and current.
- The UI MUST handle `getSnapshot()` so a remount does not lose the session.

---

## 9. UI specification

### 9.1 Screens

| Screen | Purpose | Notes |
|---|---|---|
| Onboarding | 3 beats: welcome/privacy -> language -> model download | Demo at every beat; discloses Hugging Face download |
| Console | Language, sound source, model status, Start/Stop/Pause | Aurora tiles |
| Caption overlay | Compact bar, Tier 1 + transcript rows | Click to simplify; drag; pin |
| Expanded transcript | Full transcript, confidence ticks, reversion ring | Same window |
| Settings | Caption size, UI language, models, licenses | Developer section gated |
| Summary | Duration, language, sentences, target %, reversions, export | Memory-only notice |
| Licenses | Model/data attributions | Required |

Onboarding MUST say that model downloads contact Hugging Face, and that audio and
transcripts stay local. Privacy claims must remain literally true.

### 9.2 Aurora component library (`ui/src/aurora`)

- `AuroraField`: pre-rendered idle/captioning PNGs; crossfade.
- `BrandMark`: PNG at 1x/2x.
- `Surface`/`GlassCard`: translucent fill, 1 px border, `boxShadow`; opaque
  fallback.
- `StatusBadge`, `SectionHeader`, `CaptionRow`, `TranscriptList`
  (`<virtual-list>`), `LanguagePicker`, `Slider`, `Switch`, `Tabs`,
  `SummaryTile`, `ModelCard`, `ProgressBar`, `Toast`.
- `useI18n`, `useDirection`.

### 9.3 State

- One `SessionStore` with `getSnapshot`/`subscribe`.
- `useSyncExternalStore`; row-level selectors.
- Partials coalesced to <= 1 update per ~50 ms/frame.
- Finals/simplified applied immediately.
- `lastSeq` gating from D-038.
- No Zustand/Redux/Jotai.
- `collectGarbageWhenIdle()` after bursts.

### 9.4 i18n and RTL

- 129 keys x 29 locales; converted from `Localizable.xcstrings`.
- Resolution: explicit learning language -> device locale -> `en`; base-code
  fallback.
- Modes: `learningLanguage` (immersive), `deviceLanguage`.
- RTL: manual direction helper; mirror rows, text alignment, icons, padding;
  test Arabic.
- Fonts: UI family + Noto Sans Arabic; register every weight/style.
- **Intl/ICU check:** confirm the chosen Bun/Hermes build includes full ICU data
  before relying on `Intl` for Arabic numbers/dates. If not, bundle a small
  formatter or polyfill.

### 9.5 Accessibility

- Use GPUIX `role`, `aria-label`, `aria-description`, `aria-id`, `aria-expanded`,
  `aria-selected`, `aria-valuetext`, `aria-level`.
- Every interactive element has role, name, state, keyboard path.
- Visible focus via `focusVisible`/outline.
- Caption rows: `role="button"`, Enter/Space, label includes sentence + action.
- Sliders: `role="slider"`, `aria-valuetext`, arrow keys.
- **Caption announcements (D-045):** new finalized/simplified captions must reach
  screen readers. Check GPUIX for a live-region/announcement API; if absent, add
  it to the upstream list (Q-010) and provide a "read latest" action in expanded
  mode until it lands.
- **Keyboard access (D-046):** everything must be doable from console/expanded
  mode; overlay is pointer-first. If a global hotkey exists, it switches modes.
- Release gate: keyboard-only pass, screen-reader pass, contrast, text scaling.

### 9.6 GPUIX constraints (do not fight them)

- Single window; `[ ] Multiple windows` is still a GPUIX roadmap item.
- Only two-stop linear gradients; no radial/conic/repeating.
- No Linux headless test renderer; use real-window smoke.
- `focus`/`show` window options are ignored on Linux.
- `windowBackground: 'blurred'` is macOS-only.
- No DOM: no Tailwind, shadcn, Radix, Framer Motion, `react-icons`, RTL CSS.
- `<text>` needs explicit color; no color inheritance.
- Events do not bubble like the DOM.
- A11y props are limited; upstream missing mappings if they block.

---

## 10. Window and overlay specification

**Windows 11 is the release-gated overlay platform.** Linux X11/XWayland is a
developer convenience and is not a v1 release gate (D-007). Windows acceptance
runs on the Windows side of the dual boot, not a VM.

### 10.1 Window modes

| Mode | Appearance | Focus | Policy |
|---|---|---|---|
| `console` | Normal decorated resizable window | Keyboard allowed | Topmost off, taskbar on |
| `overlay` | Compact borderless bar, bottom-center | No-activate, pointer-only | Topmost, skip taskbar, all workspaces |
| `expanded` | Larger transcript view | Keyboard allowed | Topmost optional |

Mode changes apply policy atomically; session state survives.

### 10.2 Windows (primary, release-gated)

- `SetWindowPos(HWND_TOPMOST)` for overlay; `HWND_NOTOPMOST` for console.
- `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW` in overlay; remove in console.
- Subclass with `SetWindowSubclass`; return `MA_NOACTIVATE` from
  `WM_MOUSEACTIVATE` so a click cannot activate the overlay.
- `DwmSetWindowAttribute(DWMWA_WINDOW_CORNER_PREFERENCE)` on Win11.
- Drag via `ReleaseCapture` + `WM_NCLBUTTONDOWN`/`HTCAPTION`.
- Find HWND by process ID + a unique title/class nonce.
- Virtual desktops: verify; document the Task View toggle if pinning is
  unreliable.
- Test: topmost over full-screen video/call, focus retention while typing
  elsewhere, unfocused click/hover, drag, 100/125/150% DPI, multi-monitor.
- Acceptance: Windows 11 23H2+ on Dylan PC (real GPU, real audio).

### 10.3 Linux dev (best effort)

- Launcher/shell wrapper may force X11; `bootstrap.ts` is a fallback only.
- Assert GPUI chose X11; record in `RuntimeStatus` (dev diagnostics).
- EWMH hints: `_NET_WM_STATE_ABOVE`, skip taskbar/pager, sticky desktop,
  `WM_HINTS.input=False` + remove `WM_TAKE_FOCUS` + `_NET_WM_USER_TIME=0`,
  `_MOTIF_WM_HINTS` borderless, `_NET_WM_MOVERESIZE` drag.
- Window type `_NET_WM_WINDOW_TYPE_UTILITY` by default; test alternatives.
- Window discovery: `_NET_WM_PID` + unique `WM_CLASS`/title nonce.
- Test 100/125/150% scaling; blurry text is acceptable for dev only, not a
  release gate.
- No GNOME/KDE/Sway parity requirement in v1.

### 10.4 macOS (later)

- `NSWindow.level`, `collectionBehavior`, non-activating `NSPanel`,
  `hidesOnDeactivate = false`.
- Prebuilt GPUIX arm64 only.

### 10.5 Wayland layer-shell (future track)

- `layerShell` top layer, anchor, `keyboardInteractivity: 'none'`.
- GNOME/Mutter lacks the protocol; XWayland fallback.
- Console becomes a panel; runtime keyboard-interactivity change needs upstream.
- Not part of this plan's release scope.

### 10.6 Fallback behavior

- If topmost/no-activate is unavailable on Windows: normal window + in-app
  banner; never silent.
- Linux dev: if topmost is unavailable, use a normal window and note it.
- Console mode always restores focus/keyboard.

### 10.7 Acceptance matrix (M0)

| Environment | Topmost | Focus retention | Click/hover while unfocused | Drag | Mode switch |
|---|---|---|---|---|---|
| Windows 11 23H2+ / Dylan PC | required | required | required | required | required |
| Windows full-screen video/call | required | required | required | required | required |
| Windows multi-monitor/mixed DPI | required | required | required | required | required |
| Linux dev (X11/XWayland) | optional | optional | optional | optional | optional |
| macOS 15 arm64 (later) | required | required | required | required | required |

Also test: virtual desktops, screen recording (best-effort), and a clean Windows
install for packaging.

---

## 11. Inference specification

### 11.1 Runtime split

| Stage | Runtime | Backend | Rationale |
|---|---|---|---|
| VAD | sherpa-onnx Silero | CPU | Tiny; gates ASR |
| ASR | sherpa-onnx | CPU int8 | Streaming, small chunks |
| Simplifier | llama.cpp | CPU or GPU | One GGUF; GPU when it wins |
| Segmentation/validation | Rust | CPU | Pure logic |

### 11.2 LLM backend matrix

| Backend | Ship | When used | Notes |
|---|---|---|---|
| CPU | Always | Baseline; no GPU; calibration wins | AVX2/AVX-512/NEON; runtime dispatch |
| Vulkan | Always (Linux/Windows) | Universal GPU | NVIDIA/AMD/Intel; below CUDA on NVIDIA |
| CUDA | Optional pack | NVIDIA with pack | Faster; large runtime |
| Metal | macOS later | Apple Silicon | Only if macOS target |
| ROCm/SYCL/OpenCL | No | - | Vulkan covers most users |

### 11.3 Backend loading

- `dynamic-backends` (`GGML_BACKEND_DL=ON`, `GGML_CPU_ALL_VARIANTS=ON`).
- Scenario directories: `backends/cpu`, `backends/cpu-vulkan`,
  `backends/cpu-cuda`.
- `load_backends_from_path()` before `LlamaBackend::init()`.
- No hard link to Vulkan/CUDA.
- No `-march=native`; portable CPU variants.

### 11.4 Device selection and calibration

1. Enumerate with `list_llama_ggml_backend_devices()`.
2. Classify `Gpu`, `IntegratedGpu`, `Cpu`; read free VRAM.
3. dGPU with enough memory -> full-offload candidate.
4. iGPU -> calibrate against CPU; do not assume GPU wins.
5. Calibration: fixed prompt, 32 tokens, temperature 0; CPU vs GPU; measure
   TTFT + generation.
6. Cache by CPU, GPU, driver, model hash, backend version.
7. Prefer GPU when it frees CPU for ASR/UI even if the LLM alone is close.
8. On OOM/error, fall back immediately and update the cache.

### 11.5 Queue policy and prefill

- On-demand has priority over live.
- One active + one queued; newest click replaces the queued item (D-041).
- `cancelSimplify(segmentId)` cancels queued/active work for that segment.
- Live mode yields while an on-demand request is active/queued.
- **Prefill is the tier C bottleneck:** system + few-shots can be 600-1000
  tokens. Cache prefix KV per locale with sequence save/restore; measure prefill
  tok/s in S-4.
- Qwen3: `enable_thinking=false`; think-block stripping is a safety net only.

### 11.6 Performance levers

- Prefix KV cache per locale.
- `n_threads = max(2, physical_cores / 2)` on CPU.
- `n_batch=512`, `n_ubatch=128` (tune).
- Context 2048 by default.
- Q4_K_M default; Q8_0/IQ4_XS optional.
- Warm-up after load; keep model resident; unload 1.7B after idle.
- `mmap` on; `mlock` off on low-RAM.
- Flash attention on GPU; measure on CPU.
- No speculative decoding at 0.6B.

### 11.7 Rejected alternatives

| Option | Why not |
|---|---|
| ONNX Runtime GenAI | ONNX conversion; 4-bit LLM maturity/ecosystem weaker |
| Candle | No Vulkan; CPU/quantization maturity trails llama.cpp |
| mistral.rs | Candle-based; no Vulkan; less mature |
| vLLM/ExLlama/TensorRT-LLM | GPU-only, server-class |
| whisper.cpp | Pseudo-streaming; use for batch only |
| Custom kernels | Months of work, no quality advantage |
| GPU ASR in v1 | Official Rust crate lacks providers; CPU sufficient |

---

## 12. Models, ASR coverage, and downloads

### 12.1 Per-locale ASR matrix (blocking A1)

Status values:

- `ok` - streaming ASR passes the WER gate on the locale fixture.
- `degraded` - streaming works but WER/RTF is worse; UI shows a subtle badge.
- `simplify_only` - no usable streaming ASR; paste/type + simplification only;
  no listening state.
- `tbd` - not yet measured; cannot ship as `ok`.

The matrix is filled by S-3 and T-003. Any locale not `ok`/`degraded` is
`simplify_only` for v1.

| Locale | Candidate ASR | Streaming | WER fixture | Status | UI behavior |
|---|---|---|---|---|---|
| de | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| fr | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| es | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| en | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| ar | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| da | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| et | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| fi | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| hi | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| it | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| ja | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| ko | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| nl | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| pt-BR | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| pt-PT | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| ru | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| sk | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| sv | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| tr | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| vi | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| zh-CN | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| uk | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| ro | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| bg | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| pl | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| hr | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| nb | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| hu | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |
| cs | Nemotron / zipformer TBD | TBD | TBD | tbd | TBD |

Rules:

- `setLearningLanguage` swaps the recognizer only for `ok`/`degraded` locales.
- For `simplify_only`, show "Live listening not available for this language;
  paste or type text to simplify." Provide copy/paste and typed input.
- Onboarding skips ASR-dependent steps for `simplify_only` locales but still
  installs the simplifier.
- The language picker labels each locale with its ASR status.
- Any locale can be promoted to `ok` only by passing the WER gate and adding the
  model to the catalog.

### 12.2 Model artifacts

| Role | Artifact | Repo (target) | Size | License |
|---|---|---|---|---|
| ASR | Nemotron 3.5 streaming 0.6B ONNX int8 (`encoder.int8.onnx`, `decoder.onnx`, `joiner.onnx`, `tokens.txt`) | `ndgold/nemotron-3.5-asr-streaming-0.6b-onnx` | ~600 MB | Resolve from upstream before publishing |
| ASR fallback | Per-language streaming zipformer/Parakeet | TBD per S-3 | 100-300 MB each | Per model |
| LLM live | Qwen3 0.6B EasyLanguage `Q4_K_M.gguf` | `ndgold/Qwen3-0.6B-EasyLanguage-GGUF` | ~400 MB | Apache-2.0 |
| LLM quality | Qwen3 1.7B EasyLanguage `Q4_K_M.gguf` | `ndgold/Qwen3-1.7B-EasyLanguage-GGUF` | ~1 GB | Apache-2.0 |
| VAD | `silero_vad.onnx` | sherpa-onnx bundle | ~2 MB | MIT (verify) |

### 12.3 Export and publish gates

- **First S-3 task:** export upstream Nemotron to ONNX, load it in the Rust
  `sherpa-onnx` `OnlineRecognizer`, and run streaming with endpointing.
- If it fails: use per-language streaming zipformer/Parakeet; record the
  trade-off and download sizes.
- License check before publishing the derived ONNX export (X-010).
- WER gate: int8 delta <= 2 pp vs fp32 on locale fixtures.
- Hashes generated at publish time and committed; no TOFU.
- Conversion is idempotent: skip if repo/files/hashes already exist and verify.

### 12.4 Download rules

- Public, tokenless; pin repo + revision + file list + SHA-256.
- Range resume; retry; disk preflight 6 GB.
- Progress events with bytes/sec and ETA.
- No bundled models by default.
- Cache paths as in section 7.7.

---

## 13. Audio capture

### 13.1 Sources

- Microphone via WASAPI capture (Windows primary); cpal on Linux dev.
- System audio: **WASAPI loopback (Windows primary)**; PipeWire monitor source
  (Linux dev only).
- Virtual sink (`pw-loopback`/remap) as an optional Linux dev route.
- File, demo, silence for eval/QA.

### 13.2 Pipeline roles

- Capture at device rate; resample to 16 kHz mono f32.
- VAD gates feeding ASR (CPU saver).
- Streaming ASR performs endpointing.
- Segmenter splits sentences for the UI.
- Chunker/commit-window numbers are UI cadence, not an offline ASR design.
- Ring buffer 10 s; overflow counter; `AUDIO_OVERRUN`; degradation.

### 13.3 Open questions

- **WASAPI loopback:** decide between `cpal`, the `wasapi` crate, or raw
  `windows` COM in S-5W. Verify loopback works with a real call/video on the
  Windows side of the dual boot.
- **Linux dev only:** cpal's ALSA host usually cannot see PipeWire/Pulse monitor
  sources. S-5L decides between the `pipewire` crate and `libpulse-simple` over
  pipewire-pulse.
- Virtual sink creation without root as a user service (Linux dev only).
- Windows per-app loopback (stretch).

### 13.4 Soak test

- Windows primary: 30 minutes of WASAPI loopback with a test call/video, at the
  tier C thread budget, with the LLM simplifying concurrently.
- Linux dev: same on PipeWire if the dev path is used.
- Pass: xrun/overrun counter = 0, no finalized segment loss, p99 capture
  callback latency within budget.
- Failure: record the counter and the degradation event.

---

## 14. Developer settings and test scenarios

### 14.1 Location

- **Settings -> Developer**, gated by a master switch or `LIVELINGUIST_DEV=1`.
- Visible by default in dev builds.
- Persist in `dev-overrides.json` (separate from user settings) with schema
  version and Reset.

### 14.2 Scenario presets

| Preset | Forces |
|---|---|
| Auto | Calibration chooses CPU/GPU; normal fallback |
| CPU only (strict) | No GPU module loaded; `n_gpu_layers=0`; fail if not CPU |
| No GPU (strict) | Alias of CPU-only with GPU modules removed; used by CI |
| Vulkan only (strict) | CPU+Vulkan; Vulkan device; fail if unavailable |
| CUDA only (strict) | CPU+CUDA; fail if pack missing |
| iGPU only / dGPU only | Device-type filter; fail if absent |
| Mock pipeline | Mock ASR + demo simplifier + demo audio |
| Tier C | CPU, 2-3 threads, 0.6B Q4, on-demand |
| Stress | Real models, live mode, 1.7B, validators on |
| ASR only / LLM only | Disable the other stage |
| Simplify-only | ASR off; paste/type input; tests the unsupported-locale path |

### 14.3 Advanced toggles

| Group | Toggles |
|---|---|
| LLM | backend, device, GPU layers, threads, model, quant, context, max tokens, temperature, seed, prefix cache, flash attention |
| ASR | backend (CPU/Mock/Off), locale model, threads, VAD |
| Audio | source, device, sample rate |
| Pipeline | on-demand/live, validators, reversion, chunk/commit/timeout, queue policy |
| Fault injection | ASR drop/error, LLM timeout/error/OOM, audio overrun, delays |
| UI | renderer info; software rendering requires restart/env |
| Diagnostics | log level, latency CSV, event-delivery stats |
| Safety | strict backend on/off |

### 14.4 Runtime health card

Shows requested vs effective backend/device/layers, loaded modules, CPU variant,
Windows D3D11/12 (or Linux dev X11/Wayland), window policy result,
device/VRAM, threads, model/quant/hash,
TTFT/tokens/s, last calibration/fallback, event `seq`/lag/dropped, strict state.

Actions: Probe backends, Run calibration, Reload model, Clear KV cache, Copy
diagnostics JSON, Reset dev overrides.

### 14.5 Precedence and env

```
CLI flags > environment > dev overrides > user settings > calibration > defaults
```

```bash
LL_BACKEND=auto|cpu|vulkan|cuda|metal
LL_DEVICE=auto|discrete|integrated|0
LL_GPU_LAYERS=auto|0|all|24
LL_THREADS=6
LL_LLM_MODEL=0.6b|1.7b
LL_QUANT=q4_k_m|q8_0|iq4_xs
LL_ASR=cpu|mock|off
LL_ASR_MODEL=nemotron|zipformer|parakeet
LL_AUDIO=demo|file|mic|monitor|silence
LL_PIPELINE=on_demand|live
LL_STRICT_BACKEND=1
LL_CALIBRATION=auto|skip|force_cpu|force_gpu
LL_FAULT_LLM=none|timeout|error|oom
LL_FAULT_AUDIO=none|overrun
```

### 14.6 Apply semantics

- Live: audio source, pipeline mode, validators, fault injection, log level.
- Reload model: backend, device, layers, threads, model, quant, context, cache.
- Restart: UI renderer/software rendering.
- Validate first, reload, then emit `runtimeStatus`; never half-apply.

### 14.7 Scenario matrix (CI)

```bash
for s in auto cpu-strict no-gpu vulkan-strict mock tier-c simplify-only \
         asr-only llm-only; do
  LL_SCENARIO=$s ll-eval --wav tests/fixtures/sample.wav \
    --diagnostics out/$s.json
done
```

Each run asserts effective backend, no GPU module in CPU-strict, degradation on
faults, latency within tier, and content-free diagnostics.

---

## 15. Performance and low-end budgets

### 15.1 Targets and measurement definitions

| Tier | Live p50 | On-demand p95 | UI |
|---|---|---|---|
| A | <= 1.5 s | <= 8 s | 60 fps |
| B | <= 2.5 s | <= 8 s | 60 fps |
| C | on-demand only | <= 8 s | >= 30 fps, reduced effects |

Definitions (CI asserts from event timestamps):

- **Word-to-glass:** `capture.chunk_start` -> `ui.partial_paint`.
- **Click-to-simplified:** `ui.click` -> `ui.simplified_paint`.
- **ASR chunk latency:** `asr.chunk_in` -> `asr.segment_out` p95.
- **LLM latency:** `llm.prefill_start` -> `llm.first_token` (TTFT) and
  `llm.first_token` -> `llm.last_token`.
- **UI frame:** GPUIX frame stats p95.

### 15.2 Low-end rules

- CPU baseline; GPU optional.
- On-demand default on tier C.
- Thread budget leaves headroom for ASR/audio/UI.
- `mmap`; unload 1.7B after idle.
- Disk preflight 6 GB; model total ~1 GB default.
- Reduced-effects mode: static field, no animations/glows, 30 fps.
- **Windows UI floor: D3D11/12 hardware.** WARP is best-effort/degraded, not a
  release target.
- **Linux dev UI floor: Vulkan.** Lavapipe is best-effort; not a release target.
- Tier C RSS <= 3.5 GB.
- Never claim 60 fps on software rendering/WARP.

---

## 16. Testing strategy

| Layer | Tooling | Gate |
|---|---|---|
| Rust unit/golden | `cargo nextest`, `insta`, `proptest` | All core tests pass |
| Rust perf | `criterion` | Budgets measured |
| Bridge | Bun/Vitest integration against mock core | Types/events/commands |
| Event delivery | 100 events/s x 10 min, Bun and Node | p99 into React state |
| UI logic | Bun test/Vitest | Store/i18n/formatting |
| UI integration | GPUIX `createTestRoot` on Windows (primary) | Component behavior |
| Linux dev UI | Real window Xvfb/Weston + lavapipe + screenshot (optional) | Dev smoke only |
| E2E | GPUIX `launch` + mock core + screenshots on Windows | Scenario matrix |
| Accessibility | Windows UIA snapshots; manual Narrator/NVDA | Release gate |
| Visual | Screenshot diff vs Mac reference on Windows | Layout delta <= 2 px; color delta <= 5%; no text clipping; font AA exempt (S-8) |
| Packaging | Clean Windows install/upgrade/uninstall on the dual-boot Windows side | Release gate |

### 16.1 Event-delivery test (A3)

- Rust thread emits 100 events/s for 10 minutes.
- Measure p50/p99 delivery into React state; count dropped/stalled.
- Run under Bun **and** Node.
- If p99 exceeds the threshold (proposed 50 ms) or stalls, implement the GPUIX
  frame/tick pump and re-test.
- Record results in `docs/qa/2026-11-21-event-delivery.md`.

### 16.2 Golden fixture inventory (D-049)

Create `tests/golden/INVENTORY.md` with:

- Suite name, Swift path, fixture count, provenance, Mac-generated sample path.
- Suites: `PromptGoldenTests`, `ChatTemplateGoldenTests`, `SegmenterTests`,
  `CaptionDecodingTests`, `ValidatorTests`, `FidelityGuardTests`,
  `NumberGuardTests`, `SimplifierProfileTests`, `VoiceActivityDetectorTests`,
  `SilenceGateTests`, `CaptionPipelineTests`, `SessionCSVExporterTests`,
  `SessionStoreTests`, `LanguageTests`, `UILocalizationTests`,
  `ReversionDetectorTests`, `LatencyMetricsTests`, `FrameworkPackTests`.
- No PII media; generated or consented fixtures only.

### 16.3 Reference hardware and CI

- Record exact Windows build, CPU, GPU, RAM, driver, and DPI in every
  measurement.
- CI matrix: **Windows 11 23H2+ primary**; Linux dev smoke optional; macOS arm64
  later. No GNOME/KDE/Sway release gate.
- Windows UI tests run headless via GPUIX test renderer; the dual-boot Windows
  side runs the real-window/audio/overlay acceptance pass.
- Linux dev tests run only as a convenience; they do not gate a release.

---

## 17. Packaging and release

### 17.1 Runtime and layout

- Bun first; Hermes later (N-API binding caveat).
- Ship: `livelinguist.exe`, `gpuix-native.node`, `livelinguist-core.node`,
  `backends/cpu`, `backends/cpu-vulkan`, optional `backends/cpu-cuda`,
  `assets/`, licenses.
- Models download to `%LOCALAPPDATA%\LiveLinguist`; not bundled.
- Linux packaging is a **future track, not in this plan** (D-052). The code
  stays portable, but no Linux installer, Flatpak, AppImage, or `.deb` is
  released in v1.

### 17.2 Formats

- **Windows 11 x64: NSIS installer + portable ZIP only.**
- Linux: future track; not in this plan.
- macOS later: `.app` + `.dmg`, signed/notarized.

### 17.3 Permissions and checks

- Windows: microphone permission; no admin. WASAPI loopback does not require a
  virtual driver.
- Verify the installer on a clean Windows 11 install and on the dual-boot
  Windows side.
- Verify NVIDIA/AMD/Intel Vulkan and CUDA pack detection on real hardware, not
  a VM.
- No Linux/Flatpak/AppImage checks in v1; move them to the future Linux track.

### 17.4 Privacy, updates, licenses

- Update checks opt-in/off by default (D-036).
- Core dumps disabled by default (D-037); debug builds may opt in.
- Crash reports contain stack/codes only; redaction test.
- Logs never contain segment text.
- Licenses screen lists models, data, fonts, and frameworks (GPUIX, GPUI,
  llama.cpp, sherpa-onnx). Do not leave stray toolkit placeholders.
- No telemetry; local diagnostics until explicit export.

---

## 18. Milestones (Windows-first)

M0 is ordered by **risk**. Week 1 covers only Windows kill-risks. Linux dev
smoke is optional and never gates a release.

### 18.1 M0 dependency graph

```
S-0 version verification
  |
  +--> S-3A ASR export + multilingual WER        (kill-risk)
  +--> S-2 event delivery under GPUI (Windows)   (kill-risk)
  +--> S-6W Win32 overlay/focus policy           (kill-risk)
          |
          +--> S-1 GPUIX shell on Windows
          +--> S-4 llama.cpp Windows GPU/calibration
          +--> S-5W WASAPI loopback/soak
                  |
                  +--> S-7 dev settings
                  +--> S-8 UI parity

S-6L / S-5L Linux dev smoke: optional, no release gate.
```

S-7 and S-8 MUST NOT start until S-2, S-3A, and S-6W pass their gates.

### 18.2 M0 - Windows-first feasibility spikes (3 weeks)

| ID | Spike | Exit criteria |
|---|---|---|
| S-0 | Version verification | Every pin resolves; table committed; plan bugs fixed |
| S-3A | ASR export + coverage | Nemotron export loads in `OnlineRecognizer`, streams with endpointing; multilingual WER matrix (>= 5 locales incl. de/fr/es/en/ar) recorded; fallback chosen |
| S-2 | Event delivery on Windows | 100 events/s x 10 min under Bun and Node; p50/p99 recorded; no stalls; frame-pump fallback documented |
| S-6W | Windows overlay policy | Topmost, focus retention, unfocused click/hover, drag, mode switch, DPI, multi-monitor, full-screen video/call pass on Windows 11 |
| S-1 | GPUIX shell on Windows | Window, Aurora background, fonts, virtual list, a11y roles, D3D path |
| S-4 | LLM Windows backends | CPU/Vulkan/CUDA numbers on tier A/B/C; calibration; 2-D ladder; prefill measured |
| S-5W | WASAPI audio | Mic + loopback with a real call/video; 30-min soak xrun=0 at tier C budget with LLM running |
| S-7 | Dev settings | Presets, strict mode, runtime health, event stats |
| S-8 | UI parity | Console + captioner screenshots vs Mac on Windows; component plan |
| S-6L/S-5L | Linux dev smoke (optional) | XWayland overlay and PipeWire capture usable for development; no release gate |

### 18.3 M0 task seeds

| ID | Task | Depends | Acceptance |
|---|---|---|---|
| T-001 | Verify all pins (S-0) | - | Table + resolved versions |
| T-002 | Export Nemotron ONNX and load in `OnlineRecognizer` | T-001 | Streaming transcript on a fixture |
| T-003 | Build multilingual WER matrix | T-002 | 29 rows; statuses; fallback chosen |
| T-004 | Event-delivery harness on Windows | T-001 | p99 recorded Bun/Node on Windows |
| T-005 | Dual-boot Windows dev environment | T-001 | VS Build Tools, Rust MSVC, Bun/Node, Git, NSIS; app builds |
| T-006 | Win32 overlay matrix | T-005 | Topmost/focus/drag/DPI/multi-monitor pass |
| T-007 | Win32 `WM_MOUSEACTIVATE` subclass | T-006 | Unfocused click does not activate |
| T-008 | llama.cpp Windows backends + calibration | T-005 | CPU/Vulkan/CUDA numbers |
| T-009 | WASAPI loopback probe | T-005 | Real call/video audio captured |
| T-010 | Bridge envelope + `ts-rs` types | T-004 | Generated types committed |
| T-011 | Core-dump/log redaction test | T-001 | No segment text in logs/crash |
| T-012 | Golden fixture inventory | T-001 | `INVENTORY.md` complete |
| T-013 | EN reversion rule test | T-012 | EN sessions hide pill |
| T-014 | Skills vendoring/pinning | - | Reviewed files + SHAs |
| T-015 | Dev settings prototype | T-004, T-008 | Presets + health card |
| T-016 | Linux dev smoke (optional) | T-001 | X11/PipeWire usable for development |

### 18.4 M1 - Headless parity (2-3 weeks)

- Port `ll-core`: packs, validators, segmenter, prompt, decoding, reversion,
  metrics, exports.
- Port Swift tests; `INVENTORY.md`; Mac export samples in `tests/golden/`.
- `ll-eval` CLI with CSV/JSON; runs on Windows and Linux dev.
- Exit: byte-compatible exports; golden tests pass; latency report; all 29 ASR
  rows have a status.

### 18.5 M2 - Windows desktop app (4-6 weeks)

- Onboarding, console, overlay, transcript, settings, summary, licenses.
- Event store; i18n/RTL; accessibility pass; caption announcements.
- Model manager/download UX; simplify-only path.
- Win32 window modes/policies; WASAPI loopback.
- Exit: end-to-end on-demand session on Windows 11; a11y/keyboard pass; Linux
  dev smoke is optional.

### 18.6 M3 - Windows packaging and release (2-3 weeks)

- NSIS installer + portable ZIP; Bun vs Hermes decision.
- Reduced-effects; memory/CPU/RSS budgets.
- Crash handling, diagnostics, licenses, signing.
- Exit: clean Windows 11 install/upgrade/uninstall; budgets pass/fail; release
  candidate on the dual-boot Windows side.

### 18.7 M4 - macOS (optional, separate decision)

- Metal; `NSWindow` policy; signing/notarization; arm64 only.
- Linux packaging remains a **future track**, not a milestone in this plan
  (D-052).

---

## 19. Risks and mitigations

| ID | Risk | Impact | Mitigation | Trigger |
|---|---|---|---|---|
| X-001 | GPUIX pre-1.0 breaking changes | High | Exact pins; upgrade together | Upgrade breaks |
| X-002 | GPUIX single window limits UX | Medium | Morphing window; sidecar later | Sidebar parity demanded |
| X-003 | No radial gradients | Low | Pre-baked PNG fields | Design rejects |
| X-004 | Windows UI test coverage gaps | Medium | GPUIX Windows test renderer + dual-boot acceptance; Linux dev smoke optional | CI blind spots |
| X-005 | XWayland scaling blur | Low | Dev-only; 125/150% check; native Wayland future | Dev text unreadable |
| X-006 | GNOME lacks layer-shell | Low | Future Linux track only | Native Wayland required |
| X-007 | Win32 topmost/no-activate fails | High | Win32 policy + fallback; dual-boot matrix | Matrix fails |
| X-008 | GPU driver variance | Medium | Dynamic backends; CPU fallback | Crashes/OOM |
| X-009 | CPU budget misses | High | 0.6B Q4; on-demand; smaller ASR; verbatim | p95 > 8 s |
| X-010 | Model conversion/license | Medium | Idempotent pipeline; gates; resolve licenses | Gate fails |
| X-011 | sherpa-onnx build downloads libs | Medium | Vendor/pin; offline CI | Build flaky |
| X-012 | npm supply chain | Medium | Lockfile, audit, minimal deps | Vulnerabilities |
| X-013 | Low-end memory | Medium | mmap, unload, RSS budget | OOM |
| X-014 | A11y gaps in GPUIX | High | Roles/aria; AX tests; upstream | Screen reader fails |
| X-015 | Overlay focus steal | Medium | No-activate; console mode | User complaints |
| X-016 | Scope creep | High | Non-goals; gates | Schedule slip |
| **X-017** | **ASR does not cover all 29 locales** | **High** | Per-locale matrix; `simplify_only`; fallback models | Any locale `tbd` at M1 |
| **X-018** | **TSFN event delivery stalls under GPUI** | **High** | S-2 test; frame-pump fallback | p99 stall |
| **X-019** | **XWayland scaling/text quality** | Low | Dev-only; native Wayland future | Dev text unreadable |
| **X-020** | **Transcript leakage via logs/core dumps** | **High** | No text in logs; core dumps off; redaction tests | Leak found |
| **X-021** | **Hermes does not support N-API** | Medium | Keep protocol neutral; Bun/Node first | Hermes spike fails |
| **X-022** | **Skill supply-chain/prompt injection** | Medium | Vendor/review/pin; 2 custom + 3 external | Review finds risk |
| **X-023** | **Windows 11-only excludes Windows 10 users** | Medium | Accepted scope; document requirement; revisit only with evidence | Support requests |
| **X-024** | **Dual-boot cadence slows Windows iteration** | Medium | Windows CI every change; batched dual-boot acceptance; Linux dev for fast loops | Schedule slip |
| **X-025** | **Win32 build/toolchain drift** | Medium | Pin VS Build Tools/SDK; CI image; lockfile | Build breaks |

---

## 20. Open questions and resolved decisions

### 20.1 Resolved in v3

| ID | Resolution |
|---|---|
| Q-001 | **Resolved by D-007:** Win32 primary overlay; XWayland dev-only; layer-shell future. |
| Q-002 | **Resolved by D-023:** Windows 11 x64 is the v1 product; Linux is dev/eval only; no Linux package in this plan. |
| Q-003 | **Resolved by D-024:** Windows D3D11/12 supported; Linux dev Vulkan; software best-effort; no no-GPU GUI. |
| Q-011 | **Resolved by D-031:** unsupported locales are simplify-only with a clear badge. |
| Q-015 | **Resolved by D-053:** Windows 11 x64 only; Windows 10 and ARM64 out of scope. |

### 20.2 Remaining open questions

| ID | Question | Recommendation | Blocks |
|---|---|---|---|
| Q-004 | Is a simultaneous overlay + sidebar required in v1? | No; in-window panel | D-003, UX |
| Q-005 | Exact Dylan PC specs for tiers A/B/C? | Fill section 3.4 before measurements | S-3/S-4/all budgets |
| Q-006 | macOS replacement target? | Later, separate decision | M4 |
| Q-007 | ASR fallback model on tier C? | Test zipformer/Parakeet | S-3A |
| Q-008 | CUDA pack distribution/size? | Optional download; measure | S-4, packaging |
| Q-009 | Virtual sink default (Linux dev only)? | Monitor first; sink optional | S-5L |
| Q-010 | Upstream GPUIX contributions? | `alwaysOnTop`, `noActivate`, `skipTaskbar`, `setLayerShellKeyboardInteractivity`, live-region | D-006/D-007/D-045 |
| Q-012 | Hermes binding layer if N-API is unavailable? | Bun/Node first; defer Hermes | D-025, M3 |
| Q-013 | Caption announcement mechanism if GPUIX lacks live regions? | Upstream + "read latest" fallback | D-045, a11y |
| Q-014 | Exact Dylan PC CPU/GPU/RAM/Windows build? | Fill section 3.4 | D-051, all measurements |

---

## 21. Tooling, skills, and plugins

### 21.1 Skills policy (D-047)

- Start with **two custom skills** and **at most three external skills**.
- Vendor skill files into `.opencode/skills/` at a reviewed commit; do not run a
  floating `npx skills add` in CI.
- Record repo + commit SHA + reviewer + date in `docs/qa/skills-lock.md`.
- Confirm each skill exists and its audit status before install.

### 21.2 External skills (max 3)

| Skill | Why | Pin |
|---|---|---|
| `vercel-labs/agent-skills/vercel-react-best-practices` | React 19 re-render/composition rules (ignore Next.js parts) | Record commit |
| `apollographql/skills/rust-best-practices` | Rust idioms/review checklist | Record commit |
| `po4yka/rust-skills/rust-port` | Behavior-first Swift -> Rust port method | Record commit |

Optional, only if a gap appears: `supercent-io/skills-template/web-accessibility`,
`mattpocock/skills` (`to-issues`, `implement-spec`), `emilkowalski/skills/apple-design`,
`community-access/accessibility-agents/cognitive-accessibility`.

### 21.3 Custom skills

1. `.opencode/skills/gpuix/SKILL.md`
   - Exact pins, JSX import source, no DOM, `<text>` color, style subset,
     two-stop gradients, a11y props, single window, layer-shell, Linux gaps,
     testing, packaging, forbidden libraries.
2. `.opencode/skills/live-linguist-core/SKILL.md`
   - INV-001..017, model pins, budgets, degradation, porting order, no secrets,
     no transcript in logs, event envelope.

### 21.4 Plugins and tools

- `@gpuix/cli`, `@gpuix/react`, `@gpuix/native` exact pins.
- `@napi-rs/cli`, `napi`, `napi-derive`.
- `cargo-packager`.
- `cargo-nextest`, `insta`, `proptest`, `samply`, `cargo-flamegraph`,
  `cargo-deny`, `cargo-audit`, `cargo-mutants`.
- Biome, `bun test`/Vitest.
- `rust-analyzer`.
- Optional OpenCode plugin: `.opencode/plugin/rust-guard.ts` runs
  `cargo fmt --check`/`cargo clippy -D warnings` on edited crates and
  `cargo test -p ll-core` when idle.

### 21.5 Do not install

- `longbridge/gpui-kit` coding skills (wrong framework).
- `terraphim/gpui-components`, random `gpui` skills.
- Tailwind/shadcn/Radix/Framer Motion/web design-system skills.
- DOM/Playwright testing skills.
- Cloud STT, Tauri packaging, Flutter/mobile, video/marketing skills.
- `gpui-fast` (does not apply to GPUIX).

---

## 22. Agent operating instructions

### 22.1 Before changing code

1. Identify the invariant/decision/milestone affected.
2. Read the relevant ADR, spec section, and existing tests.
3. If the change alters a structural choice, add/update a `D-xxx` and ADR first.
4. If the change adds a dependency, justify it against section 6 and update pins.
5. If the change touches inference, verify CPU fallback and strict dev behavior.
6. If the change touches logging/crash handling, verify INV-011/X-020.

### 22.2 Definition of done

- Code compiles; warnings denied where practical.
- Tests cover new behavior; golden fixtures updated if intentional.
- `cargo fmt`, `cargo clippy`, Biome/TS typecheck pass.
- No invariant violated; no secret/cloud/translation added.
- Diagnostics/exports/logs content-free.
- Event envelope/seq tests pass for bridge changes.
- Decision log/ADR and this plan updated if needed.
- Validated on the Windows 11 x64 target; Linux dev smoke is optional.

### 22.3 Forbidden shortcuts

- No webview/Electron/Tauri/second UI framework.
- No DOM UI/CSS framework.
- No custom inference engine.
- No audio through JS.
- No translation "as a demo."
- No hidden GPU fallback in strict mode.
- No bundled models/secrets.
- No transcript text in logs/metrics/crash dumps.
- No macOS support claim before M4.

### 22.4 Task template

```md
### T-xxx - <short title>

- Milestone: M?
- Depends on: T-xxx / D-xxx
- Invariants: INV-xxx
- Files: <paths>
- Change: <one paragraph>
- Tests: <unit/integration/e2e>
- Acceptance: <measurable>
- Risks: X-xxx
```

### 22.5 Decision template

```md
### D-xxx - <title>

- Status: proposed | accepted | superseded
- Date: YYYY-MM-DD
- Context: <why>
- Options: <A/B/C with evidence>
- Decision: <what>
- Consequences: <easy/hard/impossible>
- Evidence: <numbers, commands, paths>
- Revisit trigger: <condition>
```

---

## 23. Glossary

| Term | Meaning |
|---|---|
| ADR | Architecture Decision Record |
| ASR | Automatic speech recognition |
| VAD | Voice activity detection |
| GGUF | llama.cpp quantized model format |
| GPUI | Zed's GPU-accelerated Rust UI framework |
| GPUIX | React/Solid bindings for GPUI |
| napi / N-API | Node-API; native addon ABI |
| TSFN | ThreadsafeFunction (napi event delivery) |
| EWMH | Extended Window Manager Hints (X11) |
| XWayland | X11 compatibility layer inside a Wayland session |
| layer-shell | `wlr-layer-shell` protocol for panels/overlays |
| TTFT | Time to first token |
| RTF | Real-time factor |
| xrun | Audio buffer underrun/overrun |
| Prefix cache | Reused KV state for a stable prompt prefix |
| Live region | Accessibility mechanism that announces changing content |
| FALC | Facile a Lire et a Comprendre |
| Leichte Sprache | German easy language |
| Lectura Facil | Spanish easy-read standard |
| Q4_K_M | 4-bit k-quant, medium; default GGUF quant |
| Strict mode | Dev mode that fails instead of falling back |
| Simplify-only | Locale without usable ASR; paste/type + simplification |

---

## 24. References

### Product and reference implementation

- Live Linguist Mac/iOS: `https://github.com/ngoldbla/live-linguist` (private)
- Windows PRD: `docs/prd/PRD-windows.md` in the reference repo
- Live budget notes: `docs/qa/2026-06-10-live-budget-levers.md`
- Current PC repo: `README.md`, `docs/stack-options.md`,
  `docs/decisions/README.md`

### Frameworks and runtimes

- GPUIX: `https://gpuix.dev` (docs zip: `https://gpuix.dev/docs.zip`)
- GPUI: `https://github.com/zed-industries/zed/tree/main/crates/gpui`
- llama.cpp: `https://github.com/ggml-org/llama.cpp`
- `llama-cpp-2`: `https://crates.io/crates/llama-cpp-2`
- sherpa-onnx: `https://github.com/k2-fsa/sherpa-onnx`
- `sherpa-onnx` Rust crate: `https://crates.io/crates/sherpa-onnx`
- cpal: `https://crates.io/crates/cpal`
- rubato: `https://crates.io/crates/rubato`
- napi-rs: `https://napi.rs`
- `ts-rs`: `https://crates.io/crates/ts-rs`
- cargo-packager: `https://github.com/crabnebula-dev/cargo-packager`

### Skills

- `https://skills.sh/vercel-labs/agent-skills/vercel-react-best-practices`
- `https://skills.sh/apollographql/skills/rust-best-practices`
- `https://skills.sh/po4yka/rust-skills/rust-port`
- `https://skills.sh/supercent-io/skills-template/web-accessibility`
- `https://skills.sh/mattpocock/skills`

### Decisions

- `docs/decisions/0001` ASR runtime
- `docs/decisions/0002` Simplifier runtime/GGUF
- `docs/decisions/0003` System audio capture
- `docs/decisions/0004` Overlay strategy
- `docs/decisions/0005` Core language
- `docs/decisions/0006` Live budget
- New: `0007` UI framework (GPUIX), `0008` Overlay policy (Win32 primary),
  `0009` Inference backends, `0010` Dev settings, `0011` ASR coverage,
  `0012` Event delivery, `0013` Platform order (Windows-first),
  `0014` GUI floor (D3D/Vulkan), `0015` Windows-only scope,
  `0016` Reference hardware (Dylan PC)

---

## 25. Document maintenance

- Update this file when a decision changes; do not fork a second planning doc.
- ADRs are canonical for evidence/rationale; this file is the integrated index.
- Keep the decision log as the audit trail; superseded entries stay with a link.
- Add measurement results under the relevant decision as dated addenda.
- Keep the plan AI-readable: stable IDs, explicit MUST/SHOULD, measurable
  acceptance, no vague tasks.
- If a section becomes obsolete, mark it `superseded` rather than deleting it.

---

## 26. Review resolution matrix

Every item from the 2026-11-21 review and where it is addressed.

### A. Blocking gaps

| Review item | Resolution |
|---|---|
| A1 ASR language coverage | Section 12.1 per-locale matrix; D-031; INV-015; X-017; S-3A/T-003; `setLearningLanguage` behavior in 7.4; UI behavior in 9.1/12.1 |
| A2 Nemotron export kill-risk | D-032; first S-3A task; fallback per-language zipformer; license gate in 12.3; X-010 |
| A3 event delivery under GPUI | D-033; S-2/T-004; 100 events/s x 10 min Bun+Node; frame-pump fallback in 7.9; Hermes caveat in 7.9/D-025 |
| A4 `WAYLAND_DISPLAY` in launcher | D-034; 4.2; 10.2; S-1/S-6 assertion; `RuntimeStatus` |
| A5 "never drop audio" | INV-006/INV-017; 2.1 definitions; 7.3 ring buffer 10 s/overflow; `AUDIO_OVERRUN` in 8.2; degradation in 7.6 |

### B. Contradictions

| Review item | Resolution |
|---|---|
| Update checks vs network-only | INV-007; D-036; 17.4 |
| Core dumps/crash reports | INV-011; D-037; 17.4; redaction test T-011 |
| `log` can carry transcript | 8.3; INV-011; golden rule 3; log redaction test |
| Event version field missing | D-038; 8.2 envelope; `lastSeq` in 7.9/8.3 |
| ADR vs plan source of truth | Frontmatter; D-039; section 25 |
| D-007/D-023/D-024 stated as fact | Accepted in v3; section 10 is Windows-first and the provisional banner is removed |
| 2-D fallback ladder | D-040; 7.5; 11.5 |

### C. Technical gaps

| Review item | Resolution |
|---|---|
| X11 `WM_TAKE_FOCUS`/`_NET_WM_USER_TIME` | 7.8 Linux rules |
| X11 window type matters | 7.8/10.2 (`UTILITY` default; test `NOTIFICATION`/`DOCK`) |
| Mapped vs unmapped / ClientMessage | 7.8/10.2 |
| Window discovery nonce | 7.8/10.2/10.3 |
| Windows `WM_MOUSEACTIVATE` subclass | 7.8/10.3; T-007 |
| Unfocused click/hover | 10.7 matrix |
| cpal monitor sources | 13.3; S-5/T-009 |
| VAD/chunker vs streaming ASR | 4.5/7.3/13.2 |
| Soak xrun definition | 13.4 |
| On-demand queue policy | D-041; 7.5/7.6/11.5; `cancelSimplify` in 8.1 |
| Prefill/prefix cache | D-042; 11.5; S-4 |
| `enable_thinking=false` | 7.5/11.5 |
| Publish-time hashes | INV-008; 7.7/12.3 |
| Snapshot/event race | D-038; 7.9/8.1/8.3 |
| Partial `streamId` | 7.4/8.2 |
| `ts-rs` instead of napi `--dts` | D-043; 7.9; 8.3; T-010 |
| `rfd` portal | D-044; 6 |
| TSFN typo | 4.4/7.9 |
| Caption announcements | D-045; 9.5; Q-013 |
| Keyboard access to overlay | D-046; 9.5 |
| Flatpak CUDA/PipeWire | Out of scope in v1 (D-052); future Linux track |
| AppImage no libvulkan | Out of scope in v1 (D-052); future Linux track |
| RAM dimension/RSS budget | 3.3/15.2 |

### D. Structure and process

| Review item | Resolution |
|---|---|
| Order M0 by risk | 18.1 Windows-first dependency graph; 18.2 order |
| Seed T-xxx tasks | 18.3 T-001..T-016 |
| Measurable exit criteria | 18.2; 15.1; 16 |
| Reference hardware | 3.4; D-051; Q-014 |
| Shrink/pin skills | D-028/D-047; 21.1-21.5; T-014 |
| Version verification S-0 | D-048; 6.1; S-0/T-001 |
| Golden fixture inventory | D-049; 16.2; T-012 |
| EN reversion behavior | D-050; 7.1; T-013 |

### E. Small fixes

| Review item | Resolution |
|---|---|
| Removed stray toolkit placeholder | 17.4 |
| §3.1 wording | 3.1 |
| Intl/ICU availability | 9.4 |
| Scenario `no-gpu` mismatch | 14.2/14.7 |
| Latency definitions | 15.1 |
| Onboarding HF disclosure | 9.1 |

### F. v3 platform decision (post-review)

| Item | Resolution |
|---|---|
| Windows 11 x64 only | D-023, D-053; §1, §3, §17 |
| Linux dev/eval only; no Linux package | D-023, D-052; §3, §17, §18 |
| Win32 overlay is primary | D-007; §7.8, §10 |
| Windows D3D11/12 / Linux dev Vulkan floor | D-024; §15 |
| Dylan PC tier A; GPU-less approximation | D-051; §3.3, §3.4 |
| Dual-boot Windows acceptance | §10.7, §16.3, §18.2, X-024 |
| Windows 10/ARM64 out of scope | D-053; §1.2, §3.2 |
