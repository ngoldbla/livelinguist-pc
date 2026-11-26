# README media

| File | What it is |
|---|---|
| `german-60s.mp4` | Real 60 s German run on Windows with `v1.0.0-rc.4`: system audio, live ASR, every second finalized sentence clicked for a simplification. 1196×828, 20 fps, H.264 + AAC 48 kHz stereo. |
| `readme-german.gif` | 30 s preview of the MP4 (00:13–00:43), 960 px wide, 12 fps. |

Both contain only the public fixture clip's captions and audio. The clip is
*Wie Fasten den Stoffwechsel verändert* (3sat/nano, Terra X), CC BY 4.0, from
[Wikimedia Commons](https://commons.wikimedia.org/wiki/File:Wie_Fasten_den_Stoffwechsel_ver%C3%A4ndert_(CC_BY_4.0).webm).
No microphone input, system audio, or local transcript is included.

Regenerate with the T42 harness:

```powershell
bun autoresearch\benchmark.ts `
  --fixture autoresearch\fixtures\german-fastening.json `
  --source-ui --mode live --seconds 60 --start-seconds 3 `
  --click-every-n 2 `
  --record target\autoresearch\readme-60s\german-60s.mp4 `
  --out target\autoresearch\readme-60s
```

Method and measured numbers: `docs/qa/2026-11-25-german-live-baseline.md`.
