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

## 10-minute soak (PASS)

600 s x 100 events/s = 60,000 events per runtime, run concurrently against a
stable addon copy (`LL_BRIDGE_ADDON`), JSON in
`docs/qa/artifacts/event-delivery-{node,bun}.json`.

| Runtime | Expected | Received | Dropped | p50 | p99 | max |
|---|---|---|---|---|---|---|
| Node v22.23.1 | 60,000 | 60,000 | 0 | 0.59 ms | 1.07 ms | 2.70 ms |
| Bun 1.4.5 | 60,000 | 60,000 | 0 | 1.45 ms | 1.94 ms | 4.40 ms |

Both p99 values are ~25-45x below the 50 ms threshold, with zero drops. No
frame-pump fallback is needed for the TSFN path in the mock/demo workload; the
GPUIX frame/tick pump remains documented as the fallback if a future runtime
stalls.

Note: the first attempt crashed with SIGBUS because the addon file was rebuilt
while the processes had it mapped. The rerun uses a stable copy; that is now
documented in `tests/bridge/README.md`.

## Conclusion

The 10-minute release-gate evidence passes under both runtimes. `deliveryTest`
is dev-only and emits content-free `log` envelopes.
