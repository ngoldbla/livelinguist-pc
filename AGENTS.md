# AGENTS.md — Live Linguist PC

This repository implements the master plan in
`docs/plans/2026-11-21-live-linguist-pc-master-plan.md`. Read it before changing
anything structural. The plan is the integrated index; ADRs in
`docs/decisions/` are canonical for their decision.

## Non-negotiables (golden rules)

1. **Never translate.** Same-language rewriting only. Prompt packs, validators,
   and eval fixtures all enforce this.
2. **Never send audio or transcripts off the device.** Network use is limited to
   model downloads and user-initiated, opt-in update checks (off by default).
3. **Never put transcript text in logs, metrics, crash reports, or diagnostics
   exports.** They are content-free by construction and by test.
4. **Never replace llama.cpp or sherpa-onnx with custom inference kernels.**
5. **Never add DOM/web UI libraries or a webview.** GPUIX is not a browser:
   no Tailwind, shadcn, Radix, Framer Motion, `react-icons`, Playwright DOM
   tests, Electron, Tauri, or a second UI framework.
6. **Never claim a no-GPU GUI works.** Windows UI floor is D3D11/12; Linux dev
   floor is Vulkan; software rendering is best-effort/degraded.
7. **Never silently fall back in strict dev mode.** Strict scenarios fail loudly.
8. **Never drop a finalized transcript.** Audio overflow is measured, surfaced
   (`AUDIO_OVERRUN`), and triggers degradation; it is not silent loss.
9. **Never ship secrets.** No Hugging Face tokens, no embedded credentials.
10. **Every event carries `{ v, seq, ts }`.** The UI drops events at or below its
    last applied `seq`.
11. **Update the decision log/ADR when changing a structural choice.** Add tests
    with code. Keep diagnostics and exports content-free.
12. **Pin and review third-party skills.** No floating `npx skills add` in CI.

## Resuming on Windows

The current baseline is the Linux-validated handoff. The next run happens on
native Windows 11; start with
`docs/plans/2026-11-21-windows-resume-plan.md` and the copy-paste
`docs/plans/2026-11-21-windows-resume-prompt.md`. Do not repeat merged Linux
work; close the Windows gates (W-0..W-6) one PR at a time.

## Repository map

```
core/           Rust workspace (ll-core .. ll-bridge, ll-eval)
ui/             GPUIX 0.11 + React 19.2 + TypeScript
assets/packs/   29 canonical FrameworkPacks (ported verbatim from the Mac repo)
assets/locales/ 29 UI locale JSON files
tests/golden/   Golden fixture inventory and ported fixtures
tests/fixtures/ Non-PII WAV/prompt fixtures
packaging/      NSIS + portable ZIP configuration
docs/           Plan, ADRs, QA evidence, progress tracker
```

## Build and test

Linux dev (all default-feature crates):

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p ll-eval -- --scenario mock --out /tmp/ll-eval-out
```

UI:

```bash
cd ui
bun install --frozen-lockfile
bun run typecheck
bun test
bun run start            # real window; requires DISPLAY or WAYLAND_DISPLAY
bun run screenshots      # X11 real-window capture on Linux dev
```

Bridge:

```bash
cd core/crates/ll-bridge
bun run build            # napi build --platform --release
node ../../../tests/bridge/event-delivery.mjs --seconds 10
```

## Porting order

1. `assets/packs` verbatim, then `ll-core` (packs, prompt, segmenter,
   validators, reversion, decoding, metrics, exporters).
2. Protocol/config, then pipeline with mock backends.
3. Audio, models, bridge, `ll-eval`.
4. UI, window policy, dev settings.
5. Packaging and CI.

Port the Swift tests as the spec before changing behavior. Golden fixtures are
listed in `tests/golden/INVENTORY.md`.

## Definition of done

- `cargo fmt`, `cargo clippy -D warnings`, `cargo test` pass.
- `bun run typecheck`, `bun test`, Biome pass for UI changes.
- New behavior has tests; golden fixtures updated only intentionally.
- No invariant violated; no secret/cloud/translation added.
- Diagnostics/exports/logs content-free; envelope/seq tests pass for bridge
  changes.
- Decision log/ADR and progress tracker updated.
- Windows acceptance is recorded when the change touches Win32/WASAPI; Linux
  dev smoke is not a release gate.

## Git workflow

- Branch per validated stage: `stage-N-<slug>`.
- Commit with focused messages. Open a PR with evidence (commands, test output,
  screenshots for UI stages), merge only after the stage's checks pass.
- Never force-push `main`; never commit generated model files or secrets.
