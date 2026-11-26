# T43 — Microphone capture and Windows endpoint selection

**Branch:** `stage-audio-devices`
**Depends on:** T02/T03 (loopback capture), T14 (real ASR in the app)
**Decision:** ADR 0028

## Goal

Let the user choose **system audio** or **microphone**, and choose the Windows
output or input endpoint, without silent device swaps.

## Files

- `core/crates/ll-audio/src/capture.rs` — one `DeviceCapture` for both
  directions, endpoint listing, f32 conversion, explicit-id errors.
- `core/crates/ll-bridge/src/lib.rs` — `mic` source, `deviceId` start field,
  endpoint JSON, capture feeder by kind, real ASR for `mic`.
- `ui/src/screens/Console.tsx` — enabled Microphone entry + Device selector.
- `ui/src/App.tsx`, `ui/src/settings.ts`, `ui/src/sessionPlan.ts`,
  `ui/src/bridge/transport.ts` — device id plumbing and saved preference.
- `assets/locales/*.json` — keys for the new English strings (placeholders).
- `docs/decisions/0028.md`, `docs/qa/2026-11-25-microphone-device-selection.md`.

## Steps

1. In `ll-audio`, generalize capture to `CaptureKind::{Loopback, Microphone}`.
   `list_devices` returns `{id, name, is_default}`. `DeviceCapture::start(kind,
   device_id)` opens the endpoint or the Windows default; a bad explicit id is
   an error. Convert the endpoint sample format to f32.
2. In the bridge, map `"mic"` to `AudioSourceKind::Microphone`, start the right
   capture kind, return `devices` + `defaultDeviceId` from `listAudioSources`,
   accept `deviceId` in the start request, and treat `mic` as real audio for ASR.
3. Keep the default-following rule: only an unspecified selection follows
   Windows' default; an explicit endpoint is reopened by id and then stops the
   session honestly.
4. In the UI, enable Microphone, add the Device selector under the SOUND tile,
   refresh endpoints when the SOUND menu opens, and persist
   `outputDeviceId` / `inputDeviceId`.
5. At start, if a saved id is gone, drop to the default and show
   "The selected device is unavailable; using the system default."

## Commands

```powershell
cargo fmt --all -- --check
cargo clippy -p ll-audio -p ll-bridge --features capture --all-targets -- -D warnings
cargo test -p ll-audio --features capture
cargo test -p ll-bridge --features capture
cargo test -p ll-audio --features capture print_audio_endpoints -- --ignored --nocapture
cargo test -p ll-audio --features capture every_listed_endpoint_opens_by_id -- --ignored
cargo test -p ll-audio --features capture microphone_opens_and_reports_its_endpoint -- --ignored

cd ui
bun run typecheck
bun test
bun run check
```

## Done

- Both endpoint lists and the default id appear in `listAudioSources`.
- Every listed output and input endpoint opens by id; a malformed id errors
  instead of opening the default.
- A microphone session starts and stops with no overruns on the reference host.
- The real console shows the microphone entry, the Device selector, the default
  label, and a saved non-default output, and starts on that output.
- No transcript text in logs, events, settings, or evidence.
