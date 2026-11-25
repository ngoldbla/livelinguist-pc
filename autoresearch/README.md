# Live Linguist caption-latency autoresearch

This directory contains the Windows end-to-end benchmark and parameter search
for live-caption latency. The canonical decision is
[`docs/decisions/0024.md`](../docs/decisions/0024.md); the execution order is
[`docs/plans/2026-11-24-autoresearch-plan.md`](../docs/plans/2026-11-24-autoresearch-plan.md).

## What it measures

A pinned, captioned, CC-BY video plays locally through the default Windows
render endpoint. Live Linguist captures it with WASAPI loopback. The harness
polls the real app's Windows accessibility tree and aligns observed caption
text to the video's SRT cue starts.

- **Primary:** partial p95, cue start → visible partial.
- **Secondary:** final p95, cue start → visible final.
- **Gates:** cue coverage ≥ 80%, mean token similarity ≥ 0.85, valid media clock.
- **Not measured:** click-to-simplify latency, WER/CER, or GPU inference.

The UIA timestamp is a visibility proxy. `pollQuantumP95Ms` reports its
resolution; it is not a native GPU present timestamp.

## Requirements

- Windows 11 interactive desktop (not session 0).
- Microsoft Edge.
- Audible default output endpoint.
- No other Live Linguist instance running.
- Live Linguist models installed under `%LOCALAPPDATA%\LiveLinguist\models`.
- Bun, PowerShell 7, and Git on `PATH`.
- For source candidates: a bridge addon built with `capture,sherpa,llama` and
  copied to `ui/livelinguist-core.node`.

The benchmark plays audio aloud. Close meetings/media first and run on an idle
machine. It kills only processes it starts.

## Commands

Installed release baseline:

```powershell
bun autoresearch\benchmark.ts `
  --app "$env:LOCALAPPDATA\Programs\LiveLinguist\livelinguist.exe" `
  --mode live --seconds 90 --repeat 3 `
  --out target\autoresearch\baseline-live
```

Current source UI, candidate overlay:

```powershell
cargo build -p ll-bridge --release --features capture,sherpa,llama
Copy-Item target\release\livelinguist_core.dll ui\livelinguist-core.node -Force

bun autoresearch\benchmark.ts --source-ui --mode overlay `
  --seconds 90 --repeat 3 `
  --asr-chunk-ms 80 --asr-threads 4 `
  --asr-rule1-ms 1200 --asr-rule2-ms 400 --asr-rule3-ms 10000 `
  --out target\autoresearch\candidate-overlay
```

Parameter screen:

```powershell
bun autoresearch\optimize.ts --seconds 45 --repeat 1 `
  --out target\autoresearch\optimization
```

Second voice and click probe:

```powershell
bun autoresearch\benchmark.ts --fixture autoresearch\fixtures\elena-serova.json `
  --source-ui --mode overlay --seconds 90 --repeat 3 --asr-chunk-ms 560 `
  --out target\autoresearch\second-voice

bun autoresearch\benchmark.ts --source-ui --mode overlay --seconds 90 `
  --asr-chunk-ms 560 --click-at-ms 45000 --out target\autoresearch\click
```

`--fixture` defaults to `autoresearch/fixture.json`. `--click-at-ms` clicks the
latest final through UIA; `result.json` then carries
`click { requested, app, ok, maxGapMs, mediaMs }`, and the observation times show
whether captions continued after the click.

Rebuild a journal from completed runs without replaying audio:

```powershell
bun autoresearch\optimize.ts --from target\autoresearch\optimization `
  --out target\autoresearch\optimization-journal
```

Tests:

```powershell
bun test autoresearch\align.test.ts
cargo test -p ll-config -p ll-audio -p ll-pipeline
cargo test -p ll-eval --features sherpa
```

## Runtime experiment knobs

| Environment variable | Default | Meaning |
|---|---:|---|
| `LL_ASR_CHUNK_MS` | 1100 | Pipeline input chunk; 80–2000 ms |
| `LL_ASR_THREADS` | 2 | ONNX Runtime CPU threads; 1–64 |
| `LL_ASR_RULE1_MS` | 2400 | Endpoint rule 1 trailing silence |
| `LL_ASR_RULE2_MS` | 1200 | Endpoint rule 2 trailing silence |
| `LL_ASR_RULE3_MS` | 20000 | Endpoint rule 3 utterance length |

These are explicit experiment overrides. They do not change the pinned model
and do not silently fall back.

## Outputs and privacy

Only content-free files are written under the run directory:

- `summary.json`: aggregate trial metrics.
- `trial-N/result.json`: per-cue indices, times, delays, scores, and counts.
- `hypothesis.md`: candidate description written by the optimizer.
- `journal.json`: candidate comparison from `optimize.ts`.

The harness never writes observed caption text, UIA trees, screenshots, audio,
or video. Fixture media lives under gitignored `target/autoresearch/media/`;
only its URL, license, and SHA-256 are committed in `fixture.json`.
