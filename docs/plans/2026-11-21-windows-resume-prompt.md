# Resume prompt — Windows 11 takeover

Copy the block below into a new agent session running on the **native Windows
11 side** of the dual boot (not WSL). Adjust `<REPO_ROOT>` and `<MODEL_DIR>` if
the checkout is not at `C:\dev\livelinguist-pc`.

```text
You are resuming Live Linguist PC on native Windows 11 x64.

Repository: <REPO_ROOT> (default C:\dev\livelinguist-pc)
Branch: main, at or after PR #7.
Product: on-device, same-language easy-language captions. Never translate.
Master plan: docs/plans/2026-11-21-live-linguist-pc-master-plan.md (v3).
Windows takeover plan: docs/plans/2026-11-21-windows-resume-plan.md.
Agent rules: AGENTS.md.
Current status: docs/qa/implementation-progress.md.
Final Linux QA: docs/qa/2026-11-21-final-qa.md.

The Linux development baseline is already implemented and merged:
- Rust core, 29 packs, golden tests, protocol/config/audio/ASR/LLM/models/
  pipeline/ll-eval, napi bridge, TSFN soak, GPUIX UI and screenshots.
- X11 overlay verified; Win32 code compiles but has never run on hardware.
- Real ASR/GGUF models and WASAPI capture are NOT implemented end-to-end.

Your objective: execute the Windows takeover plan to close the Windows release
gates S-6W (Win32 overlay), S-5W (WASAPI mic/loopback + 30-min soak), S-3A
(real streaming ASR + 29-locale coverage), S-4 (real llama.cpp CPU/Vulkan/CUDA
+ calibration), M2 (D3D UI, accessibility, Windows screenshots), and M3
(NSIS/portable install and release). Work autonomously through the whole plan.

Start now:
1. Read AGENTS.md, the master plan sections 0-3/5/18/20, the Windows takeover
   plan, and docs/qa/implementation-progress.md.
2. Record the exact hardware first: CPU, GPU, RAM, Windows build, GPU driver,
   display scaling, audio devices -> docs/qa/2026-11-21-windows-hardware.md.
3. Run the W-0 baseline:
   rustup show
   cargo test --workspace
   cd ui; bun install --frozen-lockfile; bun run typecheck; bun test; bun run check; cd ..
   cargo build -p ll-bridge --release
   Copy-Item target\release\livelinguist_core.dll ui\livelinguist-core.node
   cd ui; bun src/main.tsx   # second terminal; leave it running for the smoke
   cargo run -p ll-eval -- --scenario mock --language fr-FR --out target\eval-mock
   Confirm a real D3D-backed window and 9 demo segments before changing code.
4. Open a baseline PR with the hardware record, a PowerShell verify script, and
   the first Windows screenshots. Merge it only after green CI.
5. Work the plan in order: W-1 Win32 overlay, W-2 WASAPI, W-3 real ASR,
   W-4 llama.cpp, W-5 UI/a11y, W-6 installer/release. If a stage is blocked by
   missing models or hardware, record the exact blocker, keep the stage open,
   and move to an unblocked stage.

Operating rules:
- One validated stage per PR; small focused commits; merge after CI is green.
- Every UI PR includes real Windows screenshots in docs/screenshots/m2-windows/.
- Update docs/qa/implementation-progress.md and the relevant ADR/QA file at every
  merged stage.
- Run the plan's acceptance matrix on real hardware; never claim a pass from a
  VM or a compile-only check.
- Do not fake models, hashes, WER, TTFT, D3D, WASAPI, or installer results. If
  the pinned artifact is unavailable, stop at that gate and ask the product
  owner for it or for permission to publish hashes; use a licensed documented
  fallback only if it is an explicit decision.
- Preserve every existing test and golden fixture. Strict mode fails loudly;
  normal mode surfaces degradation. No transcript text in logs/diagnostics.
- Keep the privacy contract: no cloud inference, no background network, model
  downloads only on explicit user action.
- Use the existing native bridge and GPUIX UI; do not add a webview, DOM UI
  library, or a second UI stack.
- If a build or test fails, diagnose and fix it; do not weaken the check to get
  green.

Deliverables:
- W-0: hardware record, Windows verify script, Windows baseline screenshots.
- W-1: Win32 overlay matrix pass + screenshots, updated ADR 0004/0008.
- W-2: real mic + WASAPI loopback, 30-min xrun=0 soak, UI source selection.
- W-3: sherpa-onnx streaming recognizer, WER matrix for >=5 locales, full
  29-row coverage statuses, pinned model hashes and missing-model UX.
- W-4: real Qwen3 0.6B/1.7B GGUF, dynamic CPU/Vulkan/CUDA backends, calibration,
  TTFT/tokens/s tiers A/B/C, 2-D ladder and prefix cache evidence.
- W-5: D3D UI screenshots, keyboard + Narrator/NVDA pass, DPI/multi-monitor,
  RTL status, updated UI QA.
- W-6: NSIS + portable ZIP, clean install/upgrade/uninstall, release artifacts.

When you finish a session, report: what is done, how it was verified (commands,
files, screenshots), what remains, and what only the user can provide. If the
whole Windows takeover is complete, update the progress board and ADRs, tag the
release only if asked, and end with the same four-line status block.
```

## Windows operator checklist (human)

1. Boot the Windows 11 side of the dual boot; install the toolchain from the
   takeover plan §2.
2. Put the checkout and models at paths without spaces if possible; avoid WSL
   for acceptance.
3. Connect the real microphone/headset and the display(s) you will test at
   100/125/150% scaling.
4. Have a call/video app ready for WASAPI loopback and full-screen overlay
   tests, and an empty folder for the portable ZIP.
5. Provide the pinned model artifacts/repos and licenses, or approve a
   documented fallback. Do not paste tokens into the repo or chat; use the
   downloader's tokenless public flow.
6. Review each stage PR; the agent should attach screenshots and measured
   numbers, not just a build result.

## What to do if a model or hardware is missing

- **ASR artifact missing:** keep W-3 open; use the fallback zipformer only after
  recording the license and the WER gap; do not mark locales `ok`.
- **GGUF missing:** keep W-4 open; demo/mock stays the default; do not invent a
  model hash or claim TTFT.
- **No dGPU:** run tier B/C, record the numbers, and label them; do not claim
  Vulkan/CUDA performance.
- **No call app/audio device:** block W-2; record the device/app combination and
  what failed; do not substitute a file replay for the soak.
- **NSIS unavailable:** build the portable ZIP first and record the installer
  gate as pending; do not claim a clean-install pass.
