# Windows retest, 2026-11-24

Runbook for the owner's Windows 11 PC after PR #31 (sherpa + llama, Windows
static C runtime) and the Windows-feedback fixes (system audio through the UI,
export to Documents, addon built with `capture,sherpa,llama`) are on `main`.
Results were recorded on the Windows machine on 2026-11-24.

Rules while recording: no transcript text, no screenshots that show captions
of real calls, no model files, no audio. Numbers come from the tools, never
typed from memory.

## 0. Update and build

```powershell
cd <your clone of livelinguist-pc>
git checkout main
git pull
git switch -c windows-retest-2026-11-24
pwsh scripts\verify-windows.ps1
```

`verify-windows.ps1` runs fmt/clippy/test, UI typecheck and tests, and builds
the bridge with `--features capture,sherpa,llama` (it copies the addon to
`ui\livelinguist-core.node`). The first build compiles llama.cpp and takes
several minutes. Record pass/fail of the script.

## 1. System audio through the UI (was: swapped for demo)

1. Start a YouTube/Teams/any English speech in the browser.
2. `cd ui; bun run start`
3. Onboarding: language **English (US)**, source **System audio**, start.
4. Expect: captions follow the playing audio within ~1 s; no scripted demo
   lines appear. Stop the audio: no new captions.
5. Pick **Microphone**: expect the "not available in this version" toast, no
   session start.

Record: works / does not work, and any error code shown.

## 2. Click-to-simplify loads the real GGUF (was: mock backend)

In the same session click a finished caption.

- First click: up to a few seconds (model load), then a simplified English
  line. Later clicks: about 1-3 s.
- A demo-looking canned rewrite on every click means the mock is still active:
  record that.
- Optional evidence: Task Manager shows the app's memory growing by roughly
  0.5-1 GB after the first click (GGUF loaded).

## 3. Export (was: hard-coded /tmp)

Live screen or Summary: export Markdown, Plain text, CSV.

- Expect files in `%USERPROFILE%\Documents\Live Linguist\`, named
  `livelinguist-YYYY-MM-DD-HH-MM-SS.<md|txt|csv>`, and a toast with the path.
- Record: which formats worked. Do **not** commit the exported files.

## 4. Tier C click bench (Windows numbers for T18/T21)

```powershell
cargo build --release -p ll-eval --features sherpa,llama
pwsh scripts\tier-c-profile.ps1 -- target\release\ll-eval.exe --scenario click-bench --clicks 50 --language en-US --llm llama --threads 3 --out target\bench-c3
```

Record from `target\bench-c3\bench.json` (`click_p50_ms`, `click_p95_ms`) and the script output:
p50 / p95 click latency, `peak_rss_mb`, `exit_code`, plus CPU model and RAM.

## 5. Privacy check (T35, Windows part)

Needs `target\asr-smoke\en.wav` and `en.ref.txt` (FLEURS en_us test clip,
CC-BY-4.0; see `tests/asr-eval/README.md`; keep it in `target\`, never commit).
Speakers must be on, other audio off.

```powershell
pwsh scripts\privacy-check.ps1
```

Record the final line only (`privacy: PASS` or the failing file/connection).

## 6. Packaging (ADR 0021, proposed)

Not retested here: the single exe still does not start until ADR 0021 is
decided and T37 implements it. Write your decision (option 1/2/3 or other) in
the Results.

## Results

| # | Check | Result | Notes (no transcript text) |
|---|---|---|---|
| 0 | `verify-windows.ps1` | pass | Needs LLVM (`libclang` for `llama-cpp-sys` bindgen): installed LLVM 23.1.6 first, without it the bridge step fails with `Unable to find libclang`. Close the app before running: copying `ui\livelinguist-core.node` fails while it is loaded. Re-run after the fixes below passed. |
| 1 | System audio captions in the UI | pass | First live partial 1.15 s after playback, first finalized sentence 4.42 s; 31 captions, none matching the built-in demo lines; no new captions after stop (31 after 4 s; second run 7 after 3 s). |
| 1 | Microphone refused, no demo | pass (after fix) | Before the fix the disabled tile swallowed clicks and showed nothing. Now: "Microphone input is not available in this version." and no session start. |
| 2 | Click loads real GGUF | pass | First click 1.98 s / later 1.31 s; app RSS 1011 MB → 1639 MB after the first click; rewrites differ from the canned demo text. |
| 3 | Export md / txt / csv to Documents | pass (after fix) | Before the fix every Summary export failed: `stop()` dropped the pipeline, so `export` returned `session not started`. After: 1015 B `.md`, 1053 B `.txt`, 1491 B `.csv` under `%USERPROFILE%\Documents\Live Linguist\`, each with a full-path toast. |
| 4 | Tier C click bench (3 threads) | pass | p50 1802 ms, p95 2013 ms, max 2051 ms, 50/50 simplified, `peak_rss_mb=886`, `exit_code=0` (gate p95 ≤ 8 s, RSS ≤ 3.5 GB); wall 94.6 s. |
| 5 | Privacy check | pass | Final line `privacy: PASS` (T12 TTS clip, loopback). |
| 6 | ADR 0021 decision | option 1 | Compiled exe + both `.node` addons beside it; recorded in ADR 0021. |

Machine: Windows 11 Pro 26H2, build 26300.9974 (`winver`), CPU = AMD Ryzen 9 3900X (12C/24T), RAM = 127.9 GB, commit = `edab553` plus the fixes above on `windows-retest-2026-11-24`.

## Findings fixed in this retest

All were found by these steps, fixed on the same branch, and re-checked on this
machine:

1. **Summary export never worked** (`ll-bridge`): `stop()` took the pipeline, so
   `export` always answered `session not started`. The stopped session is now
   kept for exports and snapshots; `tests/bridge/bridge-smoke.mjs` exports all
   three formats after `stop` and checks the files are non-empty.
2. **Model cards always said "Not downloaded"** (`Console.tsx`): the panel
   hard-coded the label instead of reading `modelStore`. It now shows Ready,
   a download percentage, or Not downloaded from the bridge status.
3. **Microphone tile was silent** (`Console.tsx`): the disabled tile discarded
   clicks. It now shows the `planSession("mic")` message, stays focusable, and
   exposes `aria-disabled`.
4. **Onboarding rendered twice** (`Onboarding.tsx`): the language and models
   beats existed twice, the second copy carrying the old "downloads arrive in a
   preview build" stub. Removed the duplicate.
5. **`tier-c-profile.ps1` with `--` flags** (`scripts`): the documented
   `pwsh scripts\tier-c-profile.ps1 -- <exe> --scenario ...` form bound
   `--scenario` as a script parameter and failed. The script now takes `$args`.

## Open after this retest

- `CoverageMatrix::v1()` still marks every locale `tbd`, so the console shows
  "Live listening is not available for this language yet" for en-US even though
  it captions. That is T33 (set the matrix from
  `docs/qa/2026-11-23-asr-coverage-v1.md`); it is cosmetic until then.
- The console's "DEMO READY" badge is still static; it should follow the source
  and backend like the model cards now do.

## Branch and PR

Committed on `windows-retest-2026-11-24` and pushed: the results here plus the
fixes they found (the runbook expected a docs-only commit, but steps 1–3 could
not pass without them, so they are on the same branch and land in its PR).

Before committing, `git status` showed only source, script and docs changes: no
models, audio, screenshots of captions, or exported transcripts. The three
exported files stay in `%USERPROFILE%\Documents\Live Linguist\` and are not
tracked.
