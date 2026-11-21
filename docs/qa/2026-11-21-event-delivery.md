# Event delivery (S-2 / T-004)

- Date: 2026-11-21
- Harness: `tests/bridge/event-delivery.mjs` (napi TSFN -> JS callback)
- Requirement: 100 events/s for 10 minutes under Bun and Node; p99 delivery
  drift <= 50 ms, no drops; fallback is a GPUIX frame/tick pump.

## Method

The addon's `deliveryTest(100, seconds)` spawns a Rust thread that emits one
content-free `log` envelope every 10 ms through the ThreadsafeFunction. The JS
callback records arrival time and drift against the envelope's monotonic `ts`.
Each event carries `{v, seq, ts, event}`; no transcript text is involved.

```bash
cargo build -p ll-bridge --release
cp target/release/liblivelinguist_core.so tests/bridge/livelinguist-core.node
node tests/bridge/event-delivery.mjs 600 docs/qa/artifacts/event-delivery-node.json
bun  tests/bridge/event-delivery.mjs 600 docs/qa/artifacts/event-delivery-bun.json
```

## 10-second smoke (pass)

| Runtime | Expected | Received | Dropped | p50 | p99 | max |
|---|---|---|---|---|---|---|
| Node v22.23.1 | 1000 | 1000 | 0 | 0.54 ms | 1.04 ms | 1.05 ms |
| Bun 1.4.5 | 1000 | 1000 | 0 | 1.48 ms | 1.97 ms | 2.77 ms |

## 10-minute soak

Status: **in progress** (600 s x 100 events/s = 60,000 events per runtime). The
JSON results are written to `docs/qa/artifacts/event-delivery-{node,bun}.json`.
The first attempt crashed with SIGBUS because the addon file was rebuilt while
the processes had it mapped; the rerun uses a stable copy via
`LL_BRIDGE_ADDON`.

## Conclusion

The 10-second smoke is far below the 50 ms p99 threshold with zero drops on both
runtimes. The 10-minute numbers are the release-gate evidence and replace this
paragraph when complete. `deliveryTest` is dev-only and content-free.
