# T42 — German video-evidence run

- Branch: `stage-german-baseline`
- Harness: `autoresearch/benchmark.ts`, `autoresearch/uia-captions.ps1`
- Evidence: `docs/qa/2026-11-25-german-live-baseline.md`
- PR: #43

## Goal

Produce a repeatable one-minute German live-caption run with a real app, real
models, and every other finalized segment simplified, and record it as an MP4
with audio. Use it whenever a change needs visible-speed evidence, not just
JSON.

## Files

- `autoresearch/fixtures/german-fastening.json` — pinned Terra X clip (CC BY
  4.0, checked German SRT), SHA-256 for media and captions.
- `autoresearch/benchmark.ts` — fixture language, `--click-every-n N`,
  `--record`, `clickSeries`.
- `autoresearch/uia-captions.ps1` — alternating clicks, scroll-to-newest,
  click→simplified observation, window placement for the crop.
- `docs/qa/2026-11-25-german-live-baseline.md` — measured baseline.
- `target/tools/ffmpeg` — optional local ffmpeg; never committed.
- `target/autoresearch/german-60s/` — run JSON and the MP4; never committed.

## Steps

1. Build the bridge with `capture,sherpa,llama` and copy
   `target/release/livelinguist_core.dll` to `ui/livelinguist-core.node`.
2. Ensure ffmpeg is available at `--ffmpeg`, `FFMPEG`,
   `target/tools/ffmpeg`, or on `PATH`.
3. Run two 60 s dry trials (no recording) and check partial/final p50/p95,
   coverage, click count, and captured caption gaps.
4. Run once with `--record`; the app is placed at x=1300, y=60 and the player at
   0,0, so the crop contains only the app.
5. Verify the MP4 with ffprobe: exactly the requested duration, H.264 + AAC,
   48 kHz stereo, non-silent audio; extract a few frames to confirm captions,
   spinners, and simplified text.
6. Record the numbers and the video path in a QA note. The MP4 contains public
   fixture captions/audio only. The README publishes a copy under
   `docs/media/` (`german-60s.mp4`, with `readme-german.gif` as the preview);
   the working capture stays under `target/`.

```powershell
bun autoresearch\benchmark.ts `
  --fixture autoresearch\fixtures\german-fastening.json `
  --source-ui --mode live --seconds 60 --start-seconds 3 `
  --click-every-n 2 `
  --record target\autoresearch\german-60s\german-60s.mp4 `
  --out target\autoresearch\german-60s
```

## Checks

```powershell
bun build autoresearch\benchmark.ts --target=bun --outfile=target\benchmark-check.js
bun test autoresearch\align.test.ts
pwsh -NoProfile -Command "$e=$null; $t=$null; [System.Management.Automation.Language.Parser]::ParseFile((Resolve-Path 'autoresearch/uia-captions.ps1').Path,[ref]$t,[ref]$e) | Out-Null; if ($e.Count) { $e | ForEach-Object Message; exit 1 } else { 'ps parse ok' }"
ffprobe -v error -show_entries format=duration:stream=codec_type,codec_name `
  -of default=noprint_wrappers=1 target\autoresearch\german-60s\german-60s.mp4
```

## Done

- Two dry trials are stable and the recorded run has audio.
- Click series reports clicked/applied and click p50/p95; the video shows the
  clicks and the German simplified lines.
- The README recording is published for `v1.0.0-rc.4`:
  `docs/media/german-60s.mp4` and the 30 s preview `docs/media/readme-german.gif`.
- Numbers and caveats are in `docs/qa/2026-11-25-german-live-baseline.md`.
- Known limitation: the recording muxes the fixture's own audio because Realtek
  Stereo Mix is not exposed to DirectShow on the reference PC. A true loopback
  capture can replace the mux without changing the click path.
