# German live-caption baseline with alternating clicks (2026-11-25)

One-minute German live run through the real app and models, with every other
finalized segment simplified and recorded. This is a baseline measurement, not a
default change.

## Fixture

| Field | Value |
|---|---|
| Clip | *Wie Fasten den Stoffwechsel verändert* (3sat/nano, Terra X) |
| Source | [Wikimedia Commons](https://commons.wikimedia.org/wiki/File:Wie_Fasten_den_Stoffwechsel_ver%C3%A4ndert_(CC_BY_4.0).webm) |
| License | CC BY 4.0 (attribution in `autoresearch/fixtures/german-fastening.json`) |
| Media | 79.967 s, 1280x720 VP9/Opus, 19.4 MB, SHA-256 `c526c44e…466950` |
| Captions | checked German TimedText SRT, 21 cues |
| Window | media 3–63 s (18 cues) |

The app language is `de-DE` (UI locale `en` so the UIA probe's labels stay
stable). The ASR input chunk is the shipping default (560 ms, 2 threads); no
`LL_ASR_*` override was passed.

## Method

`autoresearch/benchmark.ts --source-ui --mode live --seconds 60
--start-seconds 3 --click-every-n 2` starts the real app, selects System audio,
plays the clip through Edge, and aligns UIA-observed captions to the SRT on the
media clock. Every second finalized row (first, third, fifth, …) is clicked
after the live list is scrolled to the newest row; the probe records the click,
waits for that row's accessible name to become `… Simplified: …`, and reports
the per-click latency. Content-free JSON only.

Two 60 s runs were made without recording; the third used `--record` and
captures the app window at 20 fps (gdigrab) with the fixture's own audio track
muxed at the media clock.

## Results

| Run | Partial p50 / p95 | Partial cov. | Final p50 / p95 | Final cov. |
|---|---:|---:|---:|---:|
| Dry 1 | 1910 / 6330 ms | 94.4% | 5246 / 8898 ms | 77.8% |
| Dry 2 | 1907 / 6302 ms | 94.4% | 5222 / 8888 ms | 77.8% |
| Recorded | 2185 / 6329 ms | 94.4% | 5516 / 10494 ms | 83.3% |

| Run | Clicks | Applied | Click p50 / p95 | Max caption gap |
|---|---:|---:|---:|---:|
| Dry 1 | 7/7 | 7 | 1622 / 2382 ms | 1132 ms |
| Dry 2 | 7/7 | 7 | 1620 / 2240 ms | 1136 ms |
| Recorded | 6/6 | 5 | 2118 / 2813 ms | 1136 ms |

The recording run is ~0.3 s slower at partial p50 than the dry runs; the capture
encoder is the difference. One recorded click (index 6) produced no visible
simplified text within the 20 s probe window, so 5 of 6 clicks are counted as
applied. Captions keep flowing through the clicks (max gap ≈ 1.14 s, the normal
poll/scan interval).

## Recording

- `target/autoresearch/german-60s/german-60s.mp4`
- Exactly 60.000 s, 1196x828, 20 fps, H.264 + AAC 48 kHz stereo, 3.3 MB.
- Audio: the fixture's German track (what playback heard), muxed at the media
  clock; mean volume -26.5 dB, peak -4.3 dB.
- The app window is placed at x=1300, y=60 and the player at 0,0 so the crop
  contains only the app. The trim starts at the media start.
- Contains the fixture's public captions and audio only. No microphone, system
  audio, or transcript leaves the device; the MP4 stays under `target/` and is
  not uploaded.

## Reproduction

```powershell
bun autoresearch\benchmark.ts `
  --fixture autoresearch\fixtures\german-fastening.json `
  --source-ui --mode live --seconds 60 --start-seconds 3 `
  --click-every-n 2 `
  --record target\autoresearch\german-60s\german-60s.mp4 `
  --out target\autoresearch\german-60s
```

`--record` needs ffmpeg (`--ffmpeg`, `FFMPEG`, `target/tools/ffmpeg`, or PATH).
The 20 s and 30 s smoke runs used the same fixture and reported 100%/90% partial
coverage and 2/2 and 3/3 clicks respectively.
