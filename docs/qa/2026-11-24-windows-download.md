# Windows first-run model download (T16)

Status: **pass** on the reference Windows 11 PC, 2026-11-24. The Linux half of
T16 was PR #27; this records the Windows manual run the card asks for.

Machine: Windows 11 Pro 26H2, build 26300.9974, Ryzen 9 3900X (12C/24T),
127.9 GB RAM. Addon built by `scripts/verify-windows.ps1` with
`--features capture,sherpa,llama`; UI launched with `cd ui; bun src/main.tsx`
and `LL_SCREEN=onboarding LL_ONBOARDING_BEAT=2 LL_LANGUAGE=en-US`.

The run used a **fresh temporary `LOCALAPPDATA`** (the card's "point
`LOCALAPPDATA` to a temp dir" option), so the real model cache was never
deleted or touched. Nothing from that folder is committed.

## What was checked

| Step | Observed |
|---|---|
| Fresh cache | Both cards `missing`: Speech recognition 682 MB, Simplifier 397 MB (1.08 GB total). |
| Start download | One progress bar per model, percent and MB: after 15 s `24% · 167 MB of 682 MB` on speech, `0%` on the simplifier (it starts after the ASR files). |
| Cancel | At 30% the bar stops in a rose "stopped at 30%" state and a toast says `Download cancelled. Retry resumes where it stopped.`; the button becomes **Retry download**. |
| Retry | Resumes from the stopped percent (it does not start over) and finishes in about 3 minutes. |
| Finish | Both bars `ready`, the **Models ready** badge appears and the primary button becomes **Continue**. |
| App state agrees | A fresh process on the same cache reports `present: true, verified: true` for both models; **Continue** opens the Console with both model cards **Ready**. |
| On disk | 4 Nemotron files (`encoder.int8.onnx`, `decoder.int8.onnx`, `joiner.int8.onnx`, `tokens.txt`) and the Qwen3 GGUF under `LiveLinguist\models\…`. |

Screenshots (real GPUIX window, PrintWindow, native addon):

- `docs/screenshots/m2-windows/onboarding-download-progress.png`
- `docs/screenshots/m2-windows/onboarding-models-ready.png`

No model files, audio, or transcript text are in this record.

## Fixed on the way

The models beat rendered the language line as three stacked text nodes
(`English (US) ·`, a lone `·`, then `live captioning`) because it passed three
separate children to one `<text>`. It is now a single interpolated string; the
screenshots above show the fixed line.

Not changed (recorded, not in this card): the Console's `DEMO READY` badge is
still static (already noted in `docs/qa/2026-11-24-windows-retest.md`).
