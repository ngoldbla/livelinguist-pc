# T35 — Privacy checks (S7 integration)

Scripts written in PR #31 (`scripts/privacy-check.sh`, `.ps1`); still todo: run the Linux script and record `## Privacy`; Windows pending: run the `.ps1` with real loopback.

**Needs:** T34 done.

**Goal:** prove no transcript text reaches logs/diagnostics/exports-meant-to-be-content-free,
and no network traffic happens except model downloads.

**Files:** create `scripts/privacy-check.ps1`

## Steps

1. The script:
   - plays the T12 TTS clip (`target\asr-smoke\en.wav`, text known) through
     the speakers while running
     `ll-eval --source loopback --asr sherpa --llm llama --seconds 20 --out target\privacy`
     and clicks all segments (reuse `click-bench` logic or a flag);
   - searches **every** file under `target\privacy`, the app log directory
     (search the code for where logs are written), and the diagnostics CSV
     export for the words `meeting`, `notes`, `pen` (case-insensitive). Any hit
     outside an explicitly content-bearing export (Markdown/plain transcript)
     = FAIL;
   - network: while that run is going, sample
     `Get-NetTCPConnection -OwningProcess <pid>` every 500 ms; any remote
     address = FAIL (models are already downloaded, so no traffic is allowed).
2. Print `privacy: PASS` or the failing file/connection (never print the
   matched text itself, just file name and word).
3. Fix any leak found (that's a release blocker; fix in this card).

## Done when

`pwsh scripts\privacy-check.ps1` prints `privacy: PASS`. Record the run in
`docs/qa/2026-11-23-v1-integration.md` under `## Privacy`.

Commit message: `qa: privacy check script`
