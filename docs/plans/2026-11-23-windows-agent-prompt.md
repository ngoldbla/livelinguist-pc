# Windows agent prompt — build Live Linguist PC v1.0

Paste the block below into a new agent session on the **native Windows 11
host** (not WSL). Change the repo path if the checkout is not at
`C:\dev\livelinguist-pc`.

```text
You are building Live Linguist PC v1.0 on native Windows 11 x64.

Repository: C:\dev\livelinguist-pc (GitHub: ngoldbla/livelinguist-pc)
Start from: latest main (git fetch; git switch main; git pull).
Product: on-device, same-language easy-language captions for system audio.
Never translate. Never send audio or transcripts off the device.

Read first, in this order:
1. AGENTS.md (golden rules, definition of done)
2. docs/README.md (map of the docs)
3. docs/plans/2026-11-23-windows-v1-plan.md  <-- THE plan; follow it
4. docs/plans/2026-11-22-v1-scope.md (what 1.0 is; release gates G0-G9)
5. docs/qa/implementation-progress.md (live status board)
6. ADR 0018 and 0019 in docs/decisions/

Goal: take the app from today's state (demo/mock backends) to a v1.0 release
candidate by executing plan stages S1-S9:
  S1 W-2  WASAPI system-audio loopback + 30-min soak        (current blocker)
  S2 W-3a sherpa-onnx OnlineRecognizer on the pinned mirror + first-run download UX
  S3 W-3b per-locale WER/RTF and the 29-row coverage matrix
  S4 W-4a exact EasyLanguage prompt contract: drop rolling context, 29 prompt goldens
  S5 W-4b CPU llama.cpp click-only simplification, p95/RSS on tier C
  S6 W-5a 150% / 1280x720 layout clipping fix, W-5b keyboard + Narrator pass
  S7 W-5c end-to-end real session (no demo backend), privacy checks
  S8 W-6  NSIS + portable ZIP; clean install as a fresh standard (non-admin)
          local user on this machine (no VM); unsigned + SHA-256 sums (ADR 0020)
  S9      release candidate; tag v1.0.0 when G0-G9 are green
S7 is the MVP: when it merges, STOP and ask the owner for the human pass (a real
call through loopback + a Narrator listening pass) before starting S8.
S3 uses FLEURS (approved). This is a native Windows 11 boot, not a VM.
Recommended order: S4 -> S1 -> S2 -> S5 -> S6 -> S3 -> S7 -> S8 -> S9.
Start the S3 fixture download early and let measurements run in the background.

First 30 minutes:
1. git status; confirm main is clean and current.
2. pwsh scripts\verify-windows.ps1  -> must be green before any change.
   If the shell is session 0 (no desktop), use scripts\run-in-interactive-session.ps1
   for anything that opens a window.
3. Re-read the plan section for the stage you start; create branch
   stage-<id>-<slug> from main.

Per stage:
- Build exactly what the plan's "Build" table lists; add the plan's tests in the
  same PR. Real backends stay behind cargo features (capture, sherpa, llama);
  default features stay mock so CI is green without models.
- Run scripts\verify-windows.ps1 (plus -Hardware / -Models <dir> once they exist).
- Write evidence to docs/qa/<date>-<topic>.md using the plan's template, with
  screenshots in docs/screenshots/m2-windows/ for UI changes.
- Update docs/qa/implementation-progress.md and any ADR whose decision changed
  (ADR 0003 for the capture stack in S1, ADR 0011 for the eval fixtures in S3,
  ADR 0019 for the prompt contract in S4).
- Open a PR with commands, numbers, and screenshots. Merge (merge commit) only
  when CI is green and the stage's exit gate passes. Delete the branch.

Hard rules (from AGENTS.md; violating any is a release blocker):
- No translation; same-language rewriting only.
- No cloud, no telemetry; network only for explicit model downloads.
- No transcript text in logs, metrics, diagnostics, crash output, or PR text.
- No fake passes: never invent hashes, WER, latency, RSS, or a "listening" state.
  A locale without a measured pass ships simplify_only.
- CPU-only inference in 1.0. Do not add Vulkan/CUDA to rescue latency.
- Click-only simplification: no LLM call without a user click.
- Never drop a finalized segment; overruns are counted and surfaced (AUDIO_OVERRUN).
- Strict dev mode fails loudly; normal mode degrades with a visible banner.
- No webview, DOM UI library, or second UI framework. No custom inference kernels.
- Do not weaken a check to get green. Do not commit models, audio, or secrets.
- Pin every new dependency to an exact version and record it in
  docs/qa/2026-11-21-version-verification.md.

When blocked (missing hardware, human-only check, product decision): keep that
PR open, write the exact blocker in the PR and the progress board, and move to
the next unblocked stage. Human-only items are listed in plan section 14.

End every session with:
  Done:      <stages/PRs merged>
  Verified:  <commands, numbers, evidence files>
  Remaining: <next stage and its first step>
  Needs:     <what only the product owner can provide>
```

## Operator checklist (human, before starting the agent)

1. Windows 11 host with VS 2022 Build Tools (C++), Rust MSVC, Bun 1.4.5,
   Node 22, Git, PowerShell 7, NSIS — already installed per
   `docs/qa/2026-11-22-windows-vm-smoke.md`.
2. At least one render endpoint (speakers/headset) enabled; volume audible is
   not required for loopback.
3. ~10 GB free disk for models and ASR eval clips.
4. Be available at the MVP checkpoint (after S7) for a real call and a
   Narrator listening pass, and to apply for SignPath Foundation signing
   (ADR 0020). Admin rights are needed once in S8 to create a test user.
