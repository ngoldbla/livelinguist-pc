# Windows caption-latency baseline and autoresearch screen

Date: 2026-11-24/25. Content-free: no transcript, caption text, audio, video, UIA
tree, or screenshot is recorded here.

## Machine and binaries

| Item | Value |
|---|---|
| OS | Windows 11 Pro 26H2, build 26300.9974 |
| CPU | AMD Ryzen 9 3900X, 12C/24T |
| RAM | 127.9 GB |
| GPU | NVIDIA GeForce RTX 5060 Ti, 16 GB, driver 32.0.16.2124 |
| Inference | CPU-only sherpa-onnx 2.0.1; GPU is not used for ASR |
| Display | 2560×1080, 100% scale |
| Power plan | AMD Ryzen Balanced (`9897998c-92de-4669-853f-b7cd3ecb2790`) |
| Installed RC | `LiveLinguist-1.0.0-rc.1` (`%LOCALAPPDATA%\Programs\LiveLinguist\livelinguist.exe`) |
| RC SHA-256 | `1591668cb4b71bae37875eed4c5a3eafba4a6f27e51e77425529ed444cf06fa6` |
| Repo baseline | `d8cb017b6eeaa283da2b6cc13b54af585e262286` |
| Toolchain | Rust 1.98.0, Bun 1.4.5, PowerShell 7.7.0 |
| Audio endpoints present | LG HDR WFHD (NVIDIA High Definition Audio), Realtek Digital Output |

## Fixture

“Welcome to Class - Python for Informatics” — Charles Severance, University of
Michigan. Source: Wikimedia Commons, originally a YouTube lecture.
License: CC BY 3.0.

- Video SHA-256: `1e59af49d4799484020ccfc93235df6b9fb0a5fdbc4fd5c70b0ad3dc64d9dbba`
- English SRT SHA-256: `8a810404bb516fd8db98361cc43f952df8f58cc911d0ede899bd1d67a01784e9`
- Benchmark window: first 60–90 seconds at 1× playback speed.
- Media and captions stay under gitignored `target/autoresearch/media/`.

## Method

The benchmark plays the local video through the default render endpoint, starts
Live Linguist on **System audio**, and polls the real app's Windows accessibility
text tree every 20 ms. Browser playback beacons map observation wall time to
media time, including stalls. Observed captions are aligned to SRT cue starts.

- Partial delay: cue start → first visible partial containing the cue's first
  three normalized tokens.
- Final delay: cue start → visible finalized caption matching the cue.
- UIA is a visibility proxy; p95 polling/scan quantization is reported with each
  run. It is not a native present timestamp.

## Installed RC baseline

Repeated 3 × 60 s runs with the final harness:

| Surface / tier | p50 | p95 | Coverage | Mean similarity |
|---|---:|---:|---:|---:|
| Live view / partial | 1973 ms | 3657 ms | 82.6% | 0.994 |
| Live view / final | 7282 ms | 12671 ms | 78.3% | 0.970 |
| Overlay / final only | 7263 ms | 12658 ms | 78.3% | 0.970 |

Poll/scan quantum p95: 40–45 ms in live view, 26–28 ms in the overlay. The
overlay's final coverage was stable at 78.3%, just below the 80% validity gate;
its latency numbers are reported as a diagnostic baseline, not an eligible
candidate.

An initial 60 s live-view run measured partial p50/p95 2527/3362 ms and final
p50/p95 7292/12670 ms. The earlier owner retest measured first partial 1.15 s
after playback and first final 4.42 s, but it did not align to timed reference
captions, so it is not directly comparable to the cue-level p50/p95.

## Code and parameter candidates

The current branch adds explicit experiment knobs without changing shipping
defaults:

- `LL_ASR_CHUNK_MS` (default 1100)
- `LL_ASR_THREADS` (default 2)
- `LL_ASR_RULE1_MS` (default 2400)
- `LL_ASR_RULE2_MS` (default 1200)
- `LL_ASR_RULE3_MS` (default 20000)

It also shows the existing verbatim partial in the overlay above the latest
final. No model weights or inference provider changed.

Smoke results (20–30 s, not confirmation evidence):

| Candidate | Partial p50 | Partial p95 | Partial coverage | Final p50 | Final p95 |
|---|---:|---:|---:|---:|---:|
| 1100 ms, default endpoint, 2 threads | 1971 ms | 3161 ms | 92.3% | 4853 ms | 9163 ms |
| 80 ms, default endpoint, 2 threads | 1433 ms | 2417 ms | 92.3% | 3533 ms | 8437 ms |
| 80 ms, fast endpoint, 2 threads | 1028 ms | 1682 ms | 100% | 2805 ms | 4405 ms |
| 80 ms, fast endpoint, 4 threads | 972 ms | 1630 ms | 100% | 2738 ms | 4338 ms |

Fast endpoint = rule1 1200 ms, rule2 400 ms, rule3 10 s. A more aggressive
300 ms rule2 reduced partial coverage to 55.6% and was rejected.

## Stable 60 s screen

| Candidate | Partial p50 | Partial p95 | Partial coverage | Final p50 | Final p95 | Eligible |
|---|---:|---:|---:|---:|---:|---|
| Installed RC, live | 2011 ms | 3648 ms | 82.6% | 7295 ms | 12645 ms | yes |
| Installed RC, overlay (final-only) | — | — | 0% | 7254 ms | 12648 ms | no (78.3% final coverage) |
| Overlay partial + 1100 ms | 1983 ms | 3617 ms | 82.6% | 7249 ms | 12685 ms | yes |
| Overlay partial + 560 ms | 1712 ms | 2632 ms | 91.3% | 6017 ms | 10134 ms | yes |
| Overlay partial + 80 ms | 1339 ms | 2240 ms | 91.3% | 5586 ms | 12344 ms | yes |
| 80 ms + fast endpoint + 4 threads | 1092 ms | 3515 ms | 82.6% | 3138 ms | 6285 ms | yes, but worse p95 |

The screen selected **80 ms + default endpoint + 2 threads**. It reduced
partial p95 by 38.1% versus the overlay-partial/1100 ms control and by 82.3%
versus the installed overlay's final p95. The aggressive endpoint candidate
improved p50 but regressed partial p95, so it was not selected.

## Confirmation

Winner: overlay partial + 80 ms input chunk + 2 threads + default endpoint
settings. Three 90 s runs:

| Tier | p50 | p95 | Coverage | Mean similarity |
|---|---:|---:|---:|---:|
| Partial | 1323 ms | 2278 ms | 93.8% | 0.996 |
| Final | 5340 ms | 12523 ms | 82.3% | 0.984 |

Poll/scan quantum p95: 27 ms. All three trials passed the validity gate.

Against the repeated RC baseline:

- Partial p95 improved **37.7%** versus the RC live view (3657 → 2278 ms).
- Partial p95 improved **82.0%** versus the RC overlay's final-only p95
  (12658 → 2278 ms).
- Partial p50 improved **32.9%** versus the RC live view (1973 → 1323 ms).
- Cue coverage increased from 82.6% to 93.8%; mean similarity remained high.

The finalized-caption tail remains about 12.5 s. The large product improvement
comes from showing the already-available partial in the overlay; the 80 ms
input chunk then reduces the partial delay further.

No shipping ASR default has changed. The confirmed candidate still needs a
second voice/style, the WER/CER matrix, tier C profiling, and a
click-during-live continuity test before a default change.

## Commands

```powershell
bun autoresearch\benchmark.ts --app "$env:LOCALAPPDATA\Programs\LiveLinguist\livelinguist.exe" `
  --mode live --seconds 60 --repeat 3 --out target/autoresearch\baseline-live-final

bun autoresearch\optimize.ts --seconds 60 --repeat 1 `
  --out target\autoresearch\optimization-final

bun autoresearch\benchmark.ts --app "$env:LOCALAPPDATA\Programs\LiveLinguist\livelinguist.exe" `
  --mode overlay --seconds 60 --repeat 3 --out target\autoresearch\baseline-overlay-final

bun autoresearch\benchmark.ts --source-ui --mode overlay `
  --seconds 90 --repeat 3 --asr-chunk-ms 80 --asr-threads 2 `
  --out target\autoresearch\candidate-80-confirm
```

## Privacy check

All 9 repeated-run `result.json` files parsed as JSON and contained no
`text`, `hypothesis`, `transcript`, or `uia` fields:
`content-free benchmark outputs: PASS (9 files)`.

## Limitations and next gate

- UIA polling quantization is tens of milliseconds.
- The fixture is one speaker/style; confirmation needs a second voice and the
  existing WER/CER matrix.
- Click-during-live contention is not part of this baseline yet.
- GPU inference is not implemented or measured. The RTX 5060 Ti only renders
  the UI in the current RC.
