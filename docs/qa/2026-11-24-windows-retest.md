# Windows retest, 2026-11-24

Runbook for the owner's Windows 11 PC after PR #31 (sherpa + llama, Windows
static C runtime) and the Windows-feedback fixes (system audio through the UI,
export to Documents, addon built with `capture,sherpa,llama`) are on `main`.
Fill in **Results** on the Windows machine and commit from there (last section).

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
| 0 | `verify-windows.ps1` | | |
| 1 | System audio captions in the UI | | |
| 1 | Microphone refused, no demo | | |
| 2 | Click loads real GGUF | | first-click s / later clicks s |
| 3 | Export md / txt / csv to Documents | | |
| 4 | Tier C click bench (3 threads) | | p50 / p95 / peak_rss_mb / CPU / RAM |
| 5 | Privacy check | | |
| 6 | ADR 0021 decision | | |

Machine: Windows build `winver` = …, CPU = …, RAM = …, commit = `git rev-parse --short HEAD`.

## Commit from the Windows machine

```powershell
git add docs/qa/2026-11-24-windows-retest.md
git status            # only this file (and screenshots under docs/qa/ if any) — no models, audio, exports
git commit -m "qa: Windows retest results 2026-11-24"
git push -u origin windows-retest-2026-11-24
```

Then open a PR from `windows-retest-2026-11-24` to `main` on GitHub (or tell
the cloud session the branch name; it can open and merge the PR once CI is
green and update the task cards from your results).
