# Loopback probe (cpal) — 2026-11-23

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
