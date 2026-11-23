# Bridge tests

The addon is a napi-rs cdylib loaded as `livelinguist-core.node`.

```bash
cargo build -p ll-bridge --release
cp target/release/liblivelinguist_core.so tests/bridge/livelinguist-core.node
node tests/bridge/bridge-smoke.mjs
node tests/bridge/hardware-probe.mjs
node tests/bridge/event-delivery.mjs 10
bun  tests/bridge/event-delivery.mjs 10
```

`LL_BRIDGE_ADDON=/path/to/stable.node` runs against a stable copy; this is
required for long soaks so `cargo build` cannot replace a mapped addon (SIGBUS).

- `bridge-smoke.mjs`: start, feed demo, 9 segments, seq monotonic, on-demand
  simplify, summary.
- `hardware-probe.mjs`: measured `RuntimeStatus.ui`, DXGI/DRM adapters,
  Vulkan/CUDA devices, and the `probeBackends` mapping; content-free.
- `event-delivery.mjs`: 100 events/s TSFN drift, p50/p99, dropped count.
