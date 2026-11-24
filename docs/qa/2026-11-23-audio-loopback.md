# Audio loopback (cpal) — 2026-11-23

Host: Windows 11, default output device `LG HDR WFHD (NVIDIA High Definition Audio)`.
Probe: `cargo run -p ll-audio --features capture --example loopback_probe`
(config `rate=48000 channels=2 format=F32`, 10 s per run).

## Run 1 — with sound (`tests/fixtures/sample.wav` looping)

```
frames=480000 rate=48000 channels=2 peak=0.299928 callbacks=1000
```

Frames and peak both nonzero; peak well above the 0.01 threshold. cpal reported
one `A buffer underrun or overrun occurred` stream error during the run; it did
not stop capture, and the totals above are complete.

## Run 2 — no sound playing

```
frames=0 rate=48000 channels=2 peak=0.000000 callbacks=0
```

The program exits normally after 10 s with no sound (frames may be 0).

## Result

cpal WASAPI loopback works on this machine. No `windows`-crate fallback needed.

## 30-minute soak

`ll-eval` with the loopback source for 1800 s, `tests/fixtures/sample.wav`
looping on the same output device:

```powershell
cargo run --release -p ll-eval --features capture -- --scenario mock --language en-US --source loopback --seconds 1800 --out target\soak
```

```json
{
  "captureSeconds": 1800.0046542,
  "captureOverrunEvents": 0,
  "overrunCount": 0,
  "deviceRate": 48000,
  "segments": 8,
  "eventCount": 890,
  "contentFreeDiagnostics": true
}
```

Zero capture overruns across 30 minutes: the 2 s ring at 48 kHz stereo was
never full. Summary is content-free (counts and rates only).

## Device change

Two playback endpoints exist here: `LG HDR WFHD (NVIDIA High Definition Audio)`
(default) and `Realtek Digital Output (Realtek(R) Audio)`.

A `CoreClient` session was started on `{"language":"en-US","source":"monitor",
"mode":"live"}` against the addon built with `--features capture`, with
`tests/fixtures/sample.wav` looping, and the default output was switched with
`Set-AudioDevice` (AudioDeviceCmdlets, installed for this check only) and
restored 12 s later. Session events:

| t (ms) | seq | type | code | message |
|---|---|---|---|---|
| 4704 | 58 | error | `AUDIO_DEVICE_LOST` | output device lost |
| 5726 | 59 | log | `AUDIO_DEVICE_CHANGED` | capturing Realtek Digital Output (Realtek(R) Audio) |
| 16789 | 60 | error | `AUDIO_DEVICE_LOST` | output device lost |
| 17800 | 61 | log | `AUDIO_DEVICE_CHANGED` | capturing LG HDR WFHD (NVIDIA High Definition Audio) |

`seq` continues the session sequence, so the UI does not drop these. The session
produced 4 segments and 175 partials across the two switches: captions resume
about a second after the change, and there is no fallback to demo audio.

cpal 0.18 also reports routine buffer underruns as its `Xrun` error kind (see
Run 1 above). Those do not end the capture; only `DeviceNotAvailable`,
`StreamInvalidated`, and `HostUnavailable` are treated as device loss.
