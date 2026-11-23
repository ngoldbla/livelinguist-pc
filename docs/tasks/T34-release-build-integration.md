# T34 — One real build, honest health card (S7 integration)

**Needs:** S1, S2 (T14), S4, S5 (T22) merged and at least en-US `Ok`.

**Files:** `scripts/build-windows.ps1`, `scripts/verify-windows.ps1`,
`core/crates/ll-bridge/src/lib.rs` (runtime status), UI health card
(search `RuntimeStatus` / `HealthCard` in `ui/src`).

## Steps

1. `build-windows.ps1` builds the bridge with
   `--release --features capture,sherpa,llama` and copies the DLL to the UI.
   `verify-windows.ps1` gains `-Release` that does the same and runs the bridge
   smoke.
2. Runtime status reports **measured** states: capture (`capturing`, `no
   device`, `device lost`), ASR model (`ready`, `missing`, `load failed`), LLM
   (`not loaded yet`, `ready`, `missing`, `failed`), renderer (already
   measured). No hardcoded "ready".
3. Demo/mock scenarios stay available only under developer settings; the
   default start uses **System audio** + real ASR + real simplifier.
4. Language picker: locales with `SimplifyOnly` open the typed/pasted text
   box instead of starting capture, with the existing honest message.
5. Test: bridge unit test that runtime status with no models reports
   `missing` for both (temp `LOCALAPPDATA` or injected root).

## Done when

`verify-windows.ps1 -Release` passes; the health card screenshot
(`docs/screenshots/m2-windows/health-real.png`) shows real states.

Commit message: `app: real-backend build and measured health`
