# ASR chunk confirmation on a second voice

Date: 2026-11-25. Content-free: no transcript, caption text, audio, video, UIA
tree, or screenshot is recorded here.

## Scope

Confirm the caption-latency candidate on a second voice before changing a
shipping default (ADR 0024 Stage D), then run the click-during-live gate.

Candidate under test: **560 ms input chunk, 2 threads, shipping endpoint**
(rule 1 2400 ms, rule 2 1200 ms, rule 3 20000 ms). Controls: 1100 ms shipping
default, 160 ms, 80 ms.

## Fixtures

| Fixture | Speaker / style | License | Media | Captions |
|---|---|---|---|---|
| `commons-welcome-to-class` | Male lecture, continuous | CC BY 3.0 | 36.8 MB, 1194 s, `1e59af49…dbba` | `8a810404…84e9` |
| `nasa-elena-serova` | Female astronaut profile, narration + interview | Public domain | 11.3 MB, 119.7 s, `c6ffaaae…1561c` | `a78a3d62…1faf` |
| `nsf-rita-colwell` (rejected) | Female scientist interview | Public domain | 22.9 MB, 350.8 s, `b35ba159…0e944` | `326993de…5e528` |

Manifests: `autoresearch/fixtures/*.json`; media and SRT stay under gitignored
`target/autoresearch/media/`. Rita Colwell was rejected as a confirmation
fixture: the 1100 ms baseline measured 77.8% coverage on a 45 s window, below
the 0.80 gate.

## Results (partial tier, 90 s windows unless noted)

| Fixture | Config | p50 | p95 | Coverage | Similarity | p95 per trial |
|---|---|---:|---:|---:|---:|---|
| Lecture | 560 ms | 1710 ms | **2685 ms** | 93.8% | 0.996 | 2684 / 2699 / 2672 |
| Lecture | 80 ms (2026-11-24) | 1323 ms | 2278 ms | 93.8% | 0.996 | — |
| Elena | 1100 ms | 1937 ms | 5671 ms | 84.2% | 0.980 | 5657 / 5672 / 5682 |
| Elena | 560 ms | 1607 ms | 5208 ms | 94.7% | 0.987 | 4418 / 4381 / 6817 / 4376 / 4399 / 6858 |
| Elena | 160 ms | 1254 ms | 11103 ms | 89.5% | 0.996 | 11103 |
| Elena | 80 ms | 1248 ms | 9544 ms | 93.0% | 0.993 | 11093 / 11102 / 6436 |

The lecture 560 ms run is within 18% of the 80 ms confirmation while using the
model's own window. On the second voice, 560 ms improves the median p95 by 22%
over 1100 ms and raises coverage by ~10 points; 80 and 160 ms regress p95 badly.

## The 80/160 ms tail

One Elena cue (start 52.9 s) separates the configurations:

| Config | Delay for that cue |
|---|---|
| 1100 ms | 2255 / 2271 / 2270 ms |
| 560 ms | missed (coverage 94.7%) or 6817 ms when matched |
| 160 ms | 11103 ms |
| 80 ms | 11093 / 11102 / 486 ms |

No media stalls: 911–912 playback ticks and 90.0 s media time in every run.
A 5 ms UIA poll changed nothing. The pattern is a transient partial containing
the cue's tokens being replaced before it is painted at high update rates; the
metric then sees a later occurrence or misses it. 560 ms emits one stable
update per model window and avoids the 11 s tail.

## Accuracy at 560 ms

| Locale | 560 ms score | Recorded 1100 ms | p95 chunk (560) | RTF |
|---|---:|---:|---:|---:|
| en-US | 0.1519 | 0.149 (paired today 0.149) | 229 ms | 0.358 |
| fr-FR | 0.1427 | 0.1427 | 229 ms | 0.374 |
| de-DE | 0.157 | 0.157 | 229 ms | 0.366 |
| es-ES | 0.0459 | 0.0459 | 229 ms | 0.364 |
| ar-SA | 0.1211 | 0.1211 | 229 ms | 0.368 |

Four of the five measured scores are identical to the 1100 ms measurement; en-US
moves by 0.003. p95 chunk latency halves (456.6 → 229.2 ms on en-US) with no RTF
increase in the paired run. de-DE and en-US sit on the same pre-existing score
edge recorded by the owner on 2026-11-24.

Tier C (4 logical CPUs, BelowNormal, en-US): RTF **0.241**, p95 chunk **144.4
ms**, peak RSS **777 MB**, score 0.1519. Gate (score ≤ 0.15, RTF ≤ 0.5, p95 ≤
500) passes except the pre-existing en/de score edge.

## Click-during-live gate: FAIL (pre-existing)

The probe can click the latest final mid-run (`--click-at-ms`). Clicking at
20 s or 45 s stops every caption observation at the click:

| Run | Click | Last caption observation | Media end |
|---|---:|---:|---:|
| Source UI, 560 ms | 20 s | 20.0 s | 45 s |
| Source UI, 560 ms | 45 s | 45.6 s | 90 s |
| Installed rc.1 | 20 s | 20.0 s | 45 s |
| LLM alone (`click-bench`, 3 clicks) | — | — | 1277 / 1331 ms |

The installed rc.1 reproduces it, so this is not introduced by the autoresearch
branch. Cause: `onSimplify` calls the synchronous bridge `simplify`, which holds
the session mutex while running the LLM. The UI thread and the capture thread
both stop until generation ends; long captions take tens of seconds. The gate
requires no caption gap beyond budget and click-to-simplified p95 ≤ 8 s.

## Decision and next step

- Shipping candidate: **560 ms input chunk, 2 threads, shipping endpoint**.
  80 ms is rejected for the default because it fails the second-voice p95.
- The default is **not flipped yet**: the click gate fails. Fix the click path
  first (run simplification off the bridge/session mutex and make the UI path
  asynchronous), then flip `ll-config`, rebuild, and rerun 3 × 90 s with no
  `LL_ASR_*` overrides on both fixtures.
- `v1.0.0-rc.2` was published from the integrated line before this evidence; it
  carries the click freeze (pre-existing in rc.1). A follow-up RC should fix it
  before the 560 ms default ships.

## Harness changes

- `benchmark.ts --fixture <path>` selects a fixture manifest.
- `benchmark.ts --click-at-ms <ms>` asks the UIA probe to click the latest final;
  `result.json` gains `click { requested, app, ok, maxGapMs, mediaMs }`.
- New manifests under `autoresearch/fixtures/`.
