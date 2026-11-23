# Live Linguist PC — Windows v1.0 implementation plan

- Date: 2026-11-23
- Status: **active.** This is the single execution plan for shipping v1.0 on
  Windows 11 x64. It replaces the 2026-11-22 v1 execution plan and the
  2026-11-21 Windows resume plan/prompt (both deleted; see git history).
- Scope (what 1.0 is): `docs/plans/2026-11-22-v1-scope.md` + ADR 0018/0019.
- Architecture/spec reference: `docs/plans/2026-11-21-live-linguist-pc-master-plan.md`.
- Live status: `docs/qa/implementation-progress.md`.
- Agent prompt for the Windows host: `docs/plans/2026-11-23-windows-agent-prompt.md`.

If this plan and the scope doc disagree, the scope doc wins on *what* ships and
this plan wins on *order, files, and tests*. If an ADR disagrees with either,
the newer accepted ADR wins; update this plan in the same PR.

---

## 1. Where we are (2026-11-23, `main` at `6f7961a`)

| Area | State | Evidence |
|---|---|---|
| Deterministic core (29 packs, prompts, segmenter, validators, reversion, metrics, exports) | done, golden-tested | `tests/golden/INVENTORY.md` |
| Protocol / config / pipeline / `ll-eval` | done with mock/demo backends | `docs/qa/2026-11-21-pipeline.md` |
| napi bridge, `{v,seq,ts}` envelopes, 100 ev/s soak | done | `docs/qa/2026-11-21-event-delivery.md` |
| GPUIX UI (onboarding, console, live/expanded, overlay, settings, summary, licenses, dev panel) | done on demo data | `docs/screenshots/m2-windows/` |
| W-0 Windows toolchain + `scripts/verify-windows.ps1` | **merged #10** | `docs/qa/2026-11-22-windows-vm-smoke.md` |
| Renderer/accelerator probe (measured, not declared) | **merged #11** | `docs/qa/2026-11-22-gpu-renderer.md` |
| W-1 single-screen Win32 overlay (topmost, no-activate, drag, 100/125/150%) | **merged #12** | `docs/qa/2026-11-22-overlay-matrix.md` |
| v1.0 scope freeze | **merged #13/#14** | `docs/plans/2026-11-22-v1-scope.md`, ADR 0018 |
| Model pins: Nemotron 3.5 560 ms int8 mirror + Qwen3 0.6B EasyLanguage Q4_K_M | **merged #15** | `docs/qa/2026-11-22-model-pins.md`, ADR 0019 |
| Pinned downloader (Range resume, retries, SHA-256) | done | `ll-models::download_file` |
| **WASAPI loopback capture** | **not started** — `ll-audio` `capture` feature is declared (cpal) but empty; bridge lists `monitor` as `available: false` | `core/crates/ll-audio`, `ll-bridge::list_audio_sources` |
| **sherpa-onnx streaming ASR** | **placeholder** — `SherpaAsr` returns "wiring pending" | `core/crates/ll-asr/src/lib.rs` |
| **llama.cpp CPU simplifier** | **not started** — no `llama` feature, no `LlamaBackend` | `core/crates/ll-llm` |
| **EasyLanguage prompt contract** | **wrong for the GGUF** — click path passes 2 previous segments as rolling context | `ll-pipeline/src/lib.rs` (`simplify`), `ll-core/src/prompt.rs` |
| **150% layout** | console/expanded clip on a 1080-px-tall display (fixed 820/900 logical heights) | `ui/src/main.tsx`, overlay-matrix §150% |
| Accessibility | labels/Tab-order counts pass; full Narrator pass not run | `docs/qa/2026-11-22-gpu-renderer.md` |
| Packaging | NSIS + portable config exist; never validated on a clean machine | `docs/qa/2026-11-21-packaging.md` |

Reference host (D-051): Windows 11 Pro 10.0.26300, Ryzen 9 3900X (12C), RTX
5060 Ti, one 2560x1080 display, two render endpoints, no microphone. Inference
is CPU-only for 1.0, so the dGPU only matters for the UI.

## 2. Path to 1.0

```
            ┌─ S1  W-2  loopback capture ─────────────┐
            │                                         │
 main ──────┼─ S2  W-3a sherpa runtime + model UX ──┐ │
            │        └─ S3 W-3b 29-locale WER/RTF   │ │
            │                                       ├─┼─> S7 W-5c end-to-end integration
            ├─ S4  W-4a prompt contract (no model) ─┤ │        │
            │        └─ S5 W-4b llama.cpp CPU       ┘ │        v
            │                                         │   S8 W-6 packaging
            └─ S6  W-5a 150% layout + W-5b a11y ──────┘        │
                                                               v
                                                         S9 release candidate → v1.0.0
```

- **Parallel now:** S1, S2, S4, S6. S4 needs no model and no Windows; do it
  first if you want a quick green PR.
- **S3 needs S2.** **S5 needs S4.**
- **S7 needs** S1 + S2 + S5 and at least one live locale from S3 (en).
  S6 should land before S7 so integration screenshots are final.
- **S8 needs S7. S9 needs S8.**
- Recommended single-agent order: **S4 → S1 → S2 → S5 → S6 → S3 → S7 → S8 → S9.**
  (S3 is long-running measurement; start its fixture download early and let it
  run while you do S5/S6.)

## 3. Working rules for every stage

1. Branch `stage-<id>-<slug>` from fresh `main`; one stage per PR; merge
   commit; delete the branch after merge.
2. Before the PR: `scripts/verify-windows.ps1` green (fmt, clippy `-D
   warnings`, tests, UI typecheck/test/Biome, bridge build, mock eval).
3. New behavior has tests in the same PR. Golden fixtures change only on
   purpose and the PR says why.
4. Evidence goes in `docs/qa/2026-11-2x-<topic>.md` using the template in §13;
   screenshots in `docs/screenshots/m2-windows/`.
5. Update `docs/qa/implementation-progress.md` (stage board + next run) and any
   ADR whose decision changed, in the same PR.
6. Feature flags: real backends live behind cargo features (`capture`,
   `sherpa`, `llama`). Default features stay mock/demo so Linux CI and
   `cargo test --workspace` stay green without models. The Windows release
   build enables all three.
7. Never fake a pass: no invented hashes, WER, latency, or "listening" state.
   If blocked, keep the PR open, write the exact blocker, move to an unblocked
   stage.

---

## 4. S1 — W-2 WASAPI system-audio loopback  *(the current blocker)*

**Goal:** audio playing on the default render endpoint (Zoom/Teams/Meet/Skype/
YouTube) reaches the existing VAD → chunker → ring buffer as 16 kHz mono f32.
No microphone, no virtual driver.

**Branch / PR:** `stage-w2-loopback` — `windows: WASAPI loopback capture`.

### Decision to make first (record in ADR 0003, set it to `accepted`)

`cpal 0.18.2` is already the optional `capture` dependency. cpal builds a WASAPI
loopback stream when you call `build_input_stream` on an **output** device.

- **Use cpal** if a 30-minute spike shows: loopback opens on the default output,
  callbacks arrive at steady cadence, silence (nothing playing) does not stall
  the pipeline, and a default-device change is detectable (stream error →
  rebuild).
- **Otherwise use the `windows` crate directly** (`IMMDeviceEnumerator` →
  default `eRender`/`eConsole` endpoint, `IAudioClient::Initialize` shared mode
  with `AUDCLNT_STREAMFLAGS_LOOPBACK | AUDCLNT_STREAMFLAGS_EVENTCALLBACK`,
  `IAudioCaptureClient`, `IMMNotificationClient` for default-device changes),
  and drop cpal from the Windows build. Add the needed `Win32_Media_Audio`,
  `Win32_System_Com` features to the workspace `windows` pin.
- Either way there is **one** capture stack per OS. Do not ship both on Windows.

### Build

| Step | Where |
|---|---|
| `capture.rs`: `LoopbackCapture::start(default_endpoint) -> Result<Handle, CaptureError>`, `stop`, `pause`, `resume` | `core/crates/ll-audio/src/capture.rs` (new), `lib.rs` re-export behind `capture` |
| Callback thread: copy frames into an SPSC `ringbuf` only — no alloc, no lock, no log, no JS | same |
| Consumer thread: downmix to mono, resample mix rate (44.1/48/96 kHz) → 16 kHz with `rubato` (already pinned), feed existing VAD/chunker | `ll-audio` |
| Handle `AUDCLNT_BUFFERFLAGS_SILENT` (write zeros) and data discontinuity (count it) | `ll-audio` |
| Overrun counter → `AUDIO_OVERRUN` event + degradation; never drop a finalized segment | `ll-audio`, `ll-pipeline::feed_audio` |
| States: `Capturing`, `Paused`, `DeviceLost`, `NoRenderEndpoint`, `Unsupported` with content-free reasons | `ll-audio`, `ll-protocol` |
| Default-device change → stop, re-open new default, emit `audio_source_changed`; never silently capture a different device type | `ll-audio` |
| Bridge: `listAudioSources` reports `monitor` with real availability; `selectAudioSource("monitor")` starts capture; `mic` stays `available:false, note: "post-1.0"` | `core/crates/ll-bridge/src/lib.rs` |
| UI: **System audio** is the default source tile when available; honest unavailable state otherwise; no "listening" animation without frames | `ui/src/screens/Console*`, store |
| `ll-eval --scenario loopback --seconds N` for headless soak | `core/crates/ll-eval` |

### Test

Automated (run in CI where possible):

- Unit: resampler 48k/44.1k/96k stereo → 16k mono length and RMS within
  tolerance; silent-flag path yields zeros; overrun counter increments when the
  consumer is starved (inject a slow consumer); state machine transitions.
- Windows-only integration test (`#[cfg(windows)]`, `#[ignore]` by default, run
  by `verify-windows.ps1 -Hardware`): open loopback on the default endpoint,
  play `tests/fixtures/sample.wav` through `System.Media.SoundPlayer`, assert
  non-zero RMS frames arrive within 2 s and the chunker emits ≥ 1 chunk.

Manual on the reference host:

1. Play the fixture → frames arrive; stop playback → pipeline keeps running on
   silence; resume → frames resume.
2. Switch default output between the two render endpoints mid-session →
   capture follows, event emitted, no crash.
3. Disable the active endpoint → `DeviceLost` banner; re-enable → recover.
4. YouTube in a browser (borderless fullscreen) → frames arrive.
5. **Soak:** `ll-eval --scenario loopback --seconds 1800` with demo ASR while
   a looped non-PII WAV/video plays: **0 overruns, 0 finalized-segment loss,
   callback p99 recorded.** Repeat under the tier C profile (§12).

**Evidence:** `docs/qa/2026-11-2x-audio-loopback.md` (device, mix format,
chosen stack, soak numbers), screenshot of the source tile in both states.

**Exit gate (G3):** soak passes; UI never claims live audio without frames.

---

## 5. S2 — W-3a sherpa-onnx streaming runtime + first-run model UX

**Goal:** the pinned Nemotron 3.5 560 ms int8 model streams real partials and
finals through `ll-asr` on CPU, downloaded and hash-verified on first run.

**Branch / PR:** `stage-w3-asr-runtime` — `windows: sherpa-onnx streaming ASR`.

### Build

| Step | Where |
|---|---|
| Pin the sherpa-onnx Rust binding (prefer the official k2-fsa `sherpa-onnx` crate; else `sherpa-rs`) with the exact version in `Cargo.toml`; record in `docs/qa/2026-11-21-version-verification.md` | workspace `Cargo.toml`, `ll-asr/Cargo.toml` (`sherpa = ["dep:…"]`) |
| Decide static vs DLL linking of onnxruntime; if DLLs, list them for packaging (S8) | `ll-asr`, `packaging/` |
| `SherpaAsr::new(model_dir, language)` builds an `OnlineRecognizer` (transducer encoder/decoder/joiner + tokens) using the config the upstream package's own example uses for this exact asset | `ll-asr/src/lib.rs` |
| Language conditioning: two-letter key from `Language` (ADR 0019); `auto` only in a dev scenario | `ll-asr`, `ll-core::language` |
| Streaming: `accept_waveform(16k)` per chunk, `decode` while ready, partials → `asr_partial`, endpoint → finalized segment; cancellation; reset on language switch | `ll-asr`, `ll-pipeline` |
| States: `ASR_MODEL_MISSING`, `ASR_MODEL_HASH_MISMATCH`, `ASR_LOAD_FAILED` — explicit, never a fake listening state | `ll-asr`, `ll-protocol` |
| First-run UX: onboarding downloads ASR (4 files) + LLM (1 file) = ~1.08 GB via `ll-models::download_file`; disk preflight 6 GB; per-file progress off the UI thread; retry on bad hash; cancel | `ll-models`, `ll-bridge` (`downloadModels`, `modelStatus`), `ui/src/screens/Onboarding*` |
| Storage: `%LOCALAPPDATA%\LiveLinguist\models\<repo>\<revision>\` | `ll-models`, `ll-config` |

### Test

- Unit (default features, no model): model-missing, bad hash, partial file
  resume, disk-preflight refusal, language-key mapping for all 29 locales.
- `sherpa`-feature tests gated on `LL_MODEL_DIR` being set (skip with a clear
  message otherwise): load, stream `tests/fixtures/sample.wav`, ≥ 1 final
  segment, non-empty text; endpointing on a silence gap; mid-session language
  switch; load failure surfaces `ASR_LOAD_FAILED`.
- Manual: delete models → onboarding shows download; corrupt one byte → hash
  failure + retry; kill mid-download → resume from offset.

**Evidence:** `docs/qa/2026-11-2x-asr-runtime.md` (crate version, link mode,
load time, RSS after load, sample transcript **length only**, not text).

**Exit gate:** real English partials/finals in `ll-eval --scenario wav` with
the `sherpa` feature on the Windows host.

---

## 6. S3 — W-3b per-locale WER/RTF and the 29-row coverage matrix

**Goal:** every one of the 29 locales has a measured status: `live` (passes the
gate) or `simplify_only` (measured miss or not measurable). Minimum live set
**en, fr, de, es, ar** must pass.

**Branch / PR:** `stage-w3-asr-coverage` — `windows: ASR coverage matrix`.

### Fixtures (decision — record in ADR 0011 addendum)

- Use a public, redistributable read-speech test set with reference text
  (recommended: **FLEURS** test split, CC-BY-4.0, covers all 29). Build ~5 min
  per locale from listed utterance IDs.
- Commit only `tests/asr-eval/manifest.json` (dataset, revision, utterance IDs,
  SHA-256 per clip, license). **Never commit audio** (CI rejects `.wav` outside
  `tests/fixtures/`). A script fetches clips into a gitignored cache.
- If FLEURS is unacceptable to the product owner, stop and ask; do not use
  scraped or PII audio.

### Build

- `ll-eval --scenario asr-wer --locale xx-XX --manifest … --out …` reporting WER
  (CER for ja/ko/zh), RTF, p95 chunk latency (`asr.chunk_in → asr.segment_out`),
  endpoint count. Normalize text (case, punctuation, numbers) with one shared,
  tested normalizer.
- Update `CoverageMatrix` statuses in `ll-core::coverage` from the measured
  table only; UI language picker reads it.

### Gate per locale

WER (or CER) ≤ 15%, RTF ≤ 0.5, p95 chunk ≤ 500 ms on the **tier C profile**.
Also record tier A numbers for reference.

### Also measure once

160 ms vs 560 ms package on tier C (ADR 0019). 560 stays unless it misses the
latency gate; a change is a catalog version bump + ADR update.

### Test

- Unit: normalizer, WER/CER computation against hand-checked cases, matrix
  serialization; a test that no locale is `live` without a measurement record.
- Run: all 29 locales on tier C; en/fr/de/es/ar also on tier A.

**Evidence:** `docs/qa/2026-11-2x-asr-coverage-v1.md` with the 29-row table
(locale, WER/CER, RTF, p95, status, model hash). Supersedes
`docs/qa/2026-11-21-asr-coverage.md` (mark it historical).

**Exit gate (G4):** 29 rows measured; en/fr/de/es/ar `live`.

---

## 7. S4 — W-4a exact EasyLanguage prompt contract  *(no model, quick win)*

**Goal:** the click path renders exactly the prompt the GGUF was fine-tuned on,
and nothing else.

**Branch / PR:** `stage-w4-prompt-contract` — `core: EasyLanguage prompt contract`.
Can be done on Linux or Windows.

### Build

1. Fetch the model card (`README.md`) for
   `ndgold/Qwen3-0.6B-EasyLanguage-GGUF` at revision `658db5ad…` and copy the
   prompt template verbatim into the ADR 0019 addendum. The card is
   canonical; the summary below is what to expect:
   - ChatML turns (`<|im_start|>role\n…<|im_end|>`);
   - system = the pack's system prompt with trailing `/no_think`;
   - user = `Original:` sentence / `Rewritten:` cue;
   - assistant turn pre-filled with an empty think block.
2. Add an `EasyLanguagePrompt` renderer in `ll-core::prompt` that takes
   `(pack, sentence)` only — **no few-shot, no previous segments**. Keep the
   existing Mac-parity renderer and its goldens untouched (they document the
   Mac app).
3. Drop rolling context from the click path: `ll-pipeline` `simplify()` and the
   live path stop collecting `previous`; the `SimplifierBackend` signature for
   real backends takes no context. Mock/demo backends adapt.
4. Retry prompt (validator-failure retry) also follows the contract or is
   disabled for the llama backend — decide, test, and document.

### Test

- Golden test **per language (29)**: rendered prompt bytes for a fixed
  sentence equal `tests/golden/easylanguage_prompt_goldens.json`.
- Test that the rendered prompt contains no text from other segments even when
  the session has prior segments.
- Test that the trailing `/no_think` and pre-filled empty think block are
  present exactly once.
- `strip_think_block` stays as a safety net (already tested).

**Evidence:** PR description + updated `tests/golden/INVENTORY.md`.

**Exit gate:** 29 goldens pass; `rg previous` on the click path is clean.

---

## 8. S5 — W-4b llama.cpp CPU backend and click-only simplification

**Goal:** the pinned Qwen3 0.6B EasyLanguage Q4_K_M simplifies exactly the
sentence the user clicks, on CPU, within budget.

**Branch / PR:** `stage-w4-llama-cpu` — `windows: CPU llama.cpp click-to-simplify`.

### Build

| Step | Where |
|---|---|
| Add `llama-cpp-2` pinned (exact version that builds on MSVC with VS 2022 Build Tools; record in version verification); `llama = ["dep:llama-cpp-2"]`; **CPU only**, no Vulkan/CUDA, no dynamic backends | workspace `Cargo.toml`, `ll-llm/Cargo.toml` |
| `LlamaBackend: SimplifierBackend` — load GGUF (`mmap` on, `mlock` off), context ≈ 1024, threads 2–3 on tier C (configurable), greedy/temperature 0, stop on `<|im_end|>`, max tokens bounded by input length | `ll-llm/src/lib.rs` (or `llama.rs`) |
| Render with `EasyLanguagePrompt` (S4); decode → `strip_think_block` → existing validators/reversion → verbatim fallback with `≡` flag | `ll-llm`, `ll-core` |
| Click queue: one active + one queued; newer click replaces queued; cancel works on either; partial/live text never triggers simplification | `ll-pipeline`, `ll-bridge` (`simplify`, `cancelSimplify`) |
| Errors: `LLM_MODEL_MISSING`, hash mismatch, load failure, OOM, timeout → verbatim + banner; **strict dev mode fails loudly** | `ll-llm`, `ll-config` |
| Model unload after idle is optional (post-1.0); load lazily on first click and warm in the background only after the user has clicked once | `ll-llm` |

### Test

- Default features: queue ordering, replace-queued, cancel active/queued,
  timeout, strict-mode failure, **no-background-work assertion** (no backend
  call without a click event).
- `llama`-feature tests gated on `LL_MODEL_DIR`: load, deterministic output for
  2 fixed sentences, think-block absent, same-language output.
- **29-pack typed matrix** (`ll-eval --scenario simplify-matrix`): every pack
  loads, output is in the same language (language guard), never translates,
  deterministic across two runs. Content stays in the eval output dir, never
  in logs.
- Performance on tier C profile: click-to-simplified p50/p95 over ≥ 50 clicks,
  TTFT, tokens/s, peak RSS (whole process with ASR loaded).

**Evidence:** `docs/qa/2026-11-2x-llm-cpu.md` (crate version, model hash,
threads, p50/p95, TTFT, tok/s, RSS, 29-pack matrix result). Mark
`docs/qa/2026-11-21-llm-backends.md` historical for the GPU ladder.

**Exit gate (G5):** p95 ≤ 8 s and RSS ≤ 3.5 GB on tier C; 29-pack matrix
passes; no LLM activity without a click.

---

## 9. S6 — W-5a 150% layout and W-5b accessibility

Can start immediately; no model or audio needed. Two PRs if the a11y work is
large.

### W-5a layout — `stage-w5-layout` — `ui: fit windows to the work area`

**Problem:** `ui/src/main.tsx` opens fixed logical sizes (console 1180x820,
expanded 1180x900). At 150% on a 1080-px-tall display that is 1230/1350
physical px, taller than the work area, so the bottom of the console clips.

Build:

1. Bridge exposes `workArea()` → `{ width, height, scale }` of the monitor the
   window will open on (`MonitorFromPoint`/`GetMonitorInfoW` `rcWork` +
   `GetDpiForMonitor`; X11 equivalent for dev).
2. `main.tsx` sizes each mode as `min(default, workArea / scale − margin)`;
   enforce a minimum logical size that fits **1280x720 at 100%** and
   **1920x1080 at 150%** (≈ 1240x660 logical after taskbar).
3. Screens (console, expanded, settings, summary, licenses) must lay out at
   that minimum: scroll regions instead of fixed heights; no control pushed
   off-screen; overlay unchanged (780x250 already fits).
4. On mode switch (`setWindowMode`) re-apply the clamp; record whether GPUIX
   relayouts after the external `SetWindowPos` (overlay-matrix notes it does).

Test:

- Unit: size-clamp function for 100/125/150% × 1280x720, 1920x1080, 2560x1080.
- Screenshots (GPUIX DirectX test renderer + real window) at 100/125/150% for
  1280x720 and 1920x1080 for console, expanded, settings, summary, overlay; no
  clipping. Use `ChangeDisplaySettingsEx` or the display settings UI to change
  resolution on the reference host; restore afterwards.

Exit gate (G2): no clipped control in any capture.

### W-5b accessibility — `stage-w5-a11y` — `ui: keyboard and Narrator pass`

Build/check, on onboarding, console, live/expanded, settings, summary, licenses:

1. Every interactive control has a name, role, and state; Tab order is logical;
   Esc/Enter behave; visible focus ring at all scales.
2. Clickable caption sentences are keyboard-reachable and activatable (this is
   the click-to-simplify path; it must work without a mouse).
3. "Read latest" button/shortcut announces the latest caption; record the
   GPUIX live-region gap if it still exists (do not claim announcements).
4. Overlay stays pointer-first and documented as such.
5. Contrast ≥ 4.5:1 for text in both themes; reduced-effects mode at 30 fps.
6. Arabic text renders correctly (shaping, direction within text); full chrome
   mirroring stays deferred and documented.

Test:

- Automated: the existing label/Tab-stop counters extended to all core screens
  (0 unlabeled controls; expected Tab-stop counts per screen) in `bun test`.
- Manual: keyboard-only walkthrough of the user journey (scope §4); Narrator
  walkthrough (needs a human listener — see §14).

Evidence: `docs/qa/2026-11-2x-v1-ui.md` + a11y checklist + refreshed
`docs/screenshots/m2-windows/`.

Exit gate (G7 UI half): checklist passes; known gaps listed as non-claims.

---

## 10. S7 — W-5c end-to-end integration

**Goal:** a real session with no demo backend: loopback → VAD → Nemotron ASR →
verbatim captions → click → Qwen3 simplify → validate → simplified caption,
in console, overlay, and expanded mode.

**Branch / PR:** `stage-w5-integration` — `windows: v1 end-to-end session`.

Build:

- Release-profile bridge with `capture,sherpa,llama` features; demo stays
  selectable only in dev settings.
- Runtime health card shows real states (capture, ASR model, LLM model,
  renderer) from measurements, not declarations.
- Language picker shows per-locale `live` / `simplify_only` from the S3 matrix;
  `simplify_only` locales offer typed/pasted text.
- Degradation: ASR RTF over budget → chunked mode + banner; LLM failure →
  verbatim + banner; capture lost → banner, keep transcript.
- Export Markdown / plain text / diagnostics CSV from a real session.

Test:

- Journey (scope §4) on the reference host: YouTube video, then a real call app
  if available (human may need to start the call); overlay over borderless
  fullscreen; focus stays in the other app while clicking/dragging the
  overlay; pause/resume; export; clear.
- **Privacy checks:** grep logs, diagnostics CSV, crash output, and eval
  artifacts for transcript text from a known fixture → must be absent; network
  monitor (e.g. `Get-NetTCPConnection` sampling or a proxy) shows no traffic
  except model downloads.
- 30-minute loopback + ASR + periodic clicks soak on tier C: 0 overruns, no
  finalized-segment loss, RSS ≤ 3.5 GB, UI ≥ 30 fps.

Evidence: `docs/qa/2026-11-2x-v1-integration.md` + screenshots.

Exit gate (G3/G5/G6/G8 in the real app).

---

## 11. S8 — W-6 packaging; S9 — release candidate

### S8 `stage-w6-packaging` — `windows: installer and release`

1. `scripts/build-windows.ps1` produces the per-user NSIS installer and
   portable ZIP with the addon + any onnxruntime/sherpa DLLs; **no models, no
   GPU backends**; app ≤ 60 MB.
2. Clean Windows 11 23H2+ VM (no dev tools): install without admin, launch,
   onboarding download, loopback, overlay, export, upgrade over a previous
   build, uninstall (models kept unless the user opts to remove), portable
   unzip-and-run.
3. Verify: no UAC prompt; registry writes only under HKCU; files only under
   the install dir and `%LOCALAPPDATA%\LiveLinguist`.
4. Release workflow on a test tag attaches installer, ZIP, and SHA-256 sums.
5. **Product-owner decision:** unsigned beta (SmartScreen warning documented)
   vs signed build. Record in a new ADR.

Evidence: `docs/qa/2026-11-2x-packaging-v1.md` (replaces the 2026-11-21 file's
status). Exit gate: G1, G9 dry run.

### S9 `release-v1.0.0`

1. Clean install on the tier C machine or the labelled tier C profile.
2. Full journey + 30-min soak + keyboard/Narrator + privacy checklist.
3. All G0–G9 gates (scope §5) green with links to evidence.
4. Freeze docs: progress board, ADR statuses, README status section.
5. Tag `v1.0.0` **only when the product owner says so**; attach artifacts.

---

## 12. Test strategy summary

| Layer | Where | Runs in CI | Needs |
|---|---|---|---|
| Deterministic core + goldens | `cargo test -p ll-core` | Linux + Windows | nothing |
| Pipeline/protocol/bridge (mock) | `cargo test --workspace`, `tests/bridge/*.mjs` | Linux + Windows | nothing |
| UI logic, labels, Tab stops, size clamp | `cd ui && bun test` | Linux + Windows | nothing |
| Capture integration | `verify-windows.ps1 -Hardware` (`#[ignore]` tests) | no (hardware) | render endpoint |
| ASR/LLM real-model tests | feature tests gated on `LL_MODEL_DIR` | no (models) | downloaded models |
| WER/RTF, click p95, RSS | `ll-eval` scenarios | no | models + tier C profile |
| Overlay/DPI/a11y/journey | manual + screenshot scripts | no | interactive desktop |
| Clean install | VM | no | clean Windows 11 VM |

Add `-Hardware` and `-Models <dir>` switches to `scripts/verify-windows.ps1`
in S1/S2 so one command runs everything the host can run.

**Tier C profile** (until a real 4-core/8 GB machine exists): a script
`scripts/tier-c-profile.ps1` that launches the process in a Job Object limited
to 4 logical CPUs and 8 GB committed memory. Label all numbers from it
"tier C approximation".

## 13. Evidence template

```text
Date:
Machine / Windows build:
CPU / GPU / RAM / profile (tier A | tier C approx):
Display / scale:
Audio endpoint + mix format:
Build commit + features:
Model + revision + SHA-256:
Command:
Result (numbers, pass/fail against the gate):
Screenshots / artifacts:
Non-claims / known gaps:
```

## 14. What only the product owner / a human can provide

- Approval of the ASR eval fixture source (FLEURS recommended) — S3.
- A human Narrator listening pass and a real Zoom/Teams/Meet call — S6/S7.
- A clean Windows 11 VM image and, ideally, a real 4-core/8 GB laptop — S8/S9.
- Code-signing decision and certificate (if signed) — S8.
- Go-ahead to tag `v1.0.0` — S9.

## 15. Stop rules

| Situation | Do |
|---|---|
| Loopback fails on a device | Unsupported-device banner; keep typed/file input; record device; never capture another device silently |
| A locale misses the ASR gate | Ship `simplify_only`; never fake listening |
| en/fr/de/es/ar miss the gate | Stop S3; record numbers; ask the product owner (160 ms package, fixture review) |
| 0.6B misses p95 on tier C | Fewer threads, shorter max tokens, re-measure; verbatim + banner; **no GPU rescue** |
| llama-cpp-2 does not build on MSVC | Try the previous release; record the error; no custom kernels |
| sherpa binding lacks a needed API | Switch binding (official ↔ sherpa-rs) before writing FFI; record it |
| Topmost/no-activate fails for a user | Normal window + banner (D-010) |
| NSIS fails | Portable ZIP is the supported 1.0 path; record the gate as pending |
| Any transcript text in logs/diagnostics | Release blocker; fix before merging anything else |

## 16. Deferred after 1.0

Microphone; per-app capture; multi-monitor/mixed-DPI; virtual-desktop pinning;
GPU inference (Vulkan/CUDA) and GPU calibration; Qwen3 1.7B;
automatic/live simplification; full RTL chrome mirroring; auto-update;
Windows 10; ARM64; Linux/macOS packaging.
