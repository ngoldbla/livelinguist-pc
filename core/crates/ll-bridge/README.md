# ll-bridge

Thin napi-rs shell over `ll-pipeline`/`ll-protocol`. No business logic lives
here; audio never crosses the FFI boundary.

## Build

```bash
cargo build -p ll-bridge --release
cp target/release/liblivelinguist_core.so ../../../ui/livelinguist-core.node
```

The package name is `livelinguist_core`; the cdylib is loaded by Node/Bun as a
`.node` addon. Events are delivered through a `ThreadsafeFunction<String>`;
`deliveryTest` is the dev-only S-2 harness. `setWindowMode` applies the native
window policy (Win32 primary, X11 dev).

## API

`CoreClient`: `start`, `feedDemo`, `stop`, `pause`, `resume`, `simplify`,
`simplifyText`, `cancelSimplify`, `snapshot`, `runtimeStatus`, `drainEvents`,
`export`, `setWindowMode`, `listAudioSources`, `applyRuntimeConfig`,
`applyScenario`, `probeBackends`, `calibrate`, `deliveryTest`,
`stopDeliveryTest`, `lastError`.
