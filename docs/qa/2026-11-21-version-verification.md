# S-0 / T-001 — Version verification

- Date: 2026-11-21
- Environment: Linux x86-64 dev host (Dylan PC Linux side), Rust 1.98.0,
  Node 22.23.1, npm 10.9.8, Bun 1.4.5
- Status: **PASS** — every pin in the master plan resolves and exposes the
  feature the plan relies on. No plan bugs found.

Commands used (run 2026-11-21):

```bash
npm view @gpuix/react version          # 0.11.0
npm view @gpuix/native version         # 0.11.0
npm view react version                 # 19.3.1 (we pin 19.2.3)
curl -s https://crates.io/api/v1/crates/<crate> | jq .crate.max_stable_version
bun --version                          # 1.4.5
rustc --version                        # 1.98.0
node --version                         # v22.23.1
```

## Resolved pins

| Pin | Registry | Declared | Resolved | Date | Notes |
|---|---|---|---|---|---|
| `@gpuix/react` | npm | 0.11.0 | 0.11.0 | 2026-11-21 | Exact pin; adapter and native upgraded together |
| `@gpuix/native` | npm | 0.11.0 | 0.11.0 | 2026-11-21 | Installed `@gpuix/native-linux-x64-gnu@0.11.0`; Windows x64 package published |
| `react` | npm | 19.2.x | 19.2.3 | 2026-11-21 | Inside GPUIX peer range `^19.2.0`; 19.3.1 is newer but unpinned |
| `typescript` | npm | 5.9.x | 5.9.3 | 2026-11-21 | Strict mode; pinned by `ui/bun.lock` |
| `@napi-rs/cli` | npm | 3.10.x | 3.10.12 | 2026-11-21 | Build tool for `ll-bridge` |
| `napi` | crates.io | 3.16.x | 3.16.2 | 2026-11-21 | N-API ABI |
| `napi-derive` | crates.io | 3.7.x | 3.7.1 | 2026-11-21 | Derive macros |
| `llama-cpp-2` | crates.io | =0.1.161 | 0.1.161 | 2026-11-24 | Feature-gated `llama` in `ll-llm`/`ll-eval`/`ll-bridge`; CPU only, `default-features = false, features = ["openmp"]` (no `common`, `cuda`, `vulkan`, `dynamic-backends`; `common` clashed with sherpa-onnx static libs); llama.cpp built from the crate sources with cmake; CI clippy+tests on Linux and Windows (T17) |
| `sherpa-onnx` | crates.io | =2.0.1 | 2.0.1 | 2026-11-24 | Feature-gated `sherpa` in `ll-asr`/`ll-eval`/`ll-bridge`; static CPU libs `sherpa-onnx-v2.0.1-<os>-static-lib` downloaded by the `-sys` build script from k2-fsa GitHub releases (Linux archive sha256 `55cfadf6…70871`). Pinned per T12 (was listed as 2.0.2 before it was wired) |
| `cpal` | crates.io | 0.18.x | 0.18.2 | 2026-11-21 | `pipewire` feature exists |
| `rubato` | crates.io | 5.1.x | 5.1.0 | 2026-11-21 | Resampling |
| `ringbuf` | crates.io | 0.6.x | 0.6.0 | 2026-11-21 | Lock-free ring buffer |
| `reqwest` | crates.io | 0.12.x | 0.12.28 | 2026-11-21 | rustls, no OpenSSL |
| `sha2` | crates.io | 0.10.x | 0.10.9 | 2026-11-21 | SHA-256 |
| `serde` | crates.io | 1.0.x | 1.0.229 | 2026-11-21 | Derive |
| `serde_json` | crates.io | 1.0.x | 1.0.154 | 2026-11-21 | |
| `ts-rs` | crates.io | 12.x | 12.0.1 | 2026-11-21 | TS generation |
| `rfd` | crates.io | 0.18.x | 0.18.0 | 2026-11-21 | Portal backend on Linux |
| `x11rb` | crates.io | 0.14.x | 0.14.0 | 2026-11-21 | X11/XWayland dev policy |
| `windows` | crates.io | 0.62.x | 0.62.2 | 2026-11-21 | Win32 overlay policy |
| `unicode-general-category` | crates.io | =1.1.0 | 1.1.0 | 2026-11-24 | `ll-eval` WER/CER normalizer: Unicode `P*` punctuation (T30) |
| Bun | runtime | pinned | 1.4.5 | 2026-11-21 | Linux dev; Windows install recorded in T-005 |
| Rust | toolchain | stable | 1.98.0 | 2026-11-21 | `rust-toolchain.toml` pins the channel |

## Feature checks

- `@gpuix/native` exposes `render`, `automation/launch`, `registerFont`,
  `promptForPaths`, `windowBackground`, and `virtual-list`.
- GPUIX automation `screenshot()` is **not available on Linux** (test-support
  build exists for macOS/Windows only). The Linux dev screenshot path is a real
  X11 window captured with `import -window <id>`; this is documented in
  `docs/qa/2026-11-21-ui-visual-checks.md` when the UI lands.
- `cpal` 0.18.2 exposes the `pipewire` feature.
- `llama-cpp-2` 0.1.161 exposes `dynamic-backends`, `vulkan`, and `cuda`.
- `sherpa-onnx` 2.0.1 exposes `OnlineRecognizer`; CPU static libs are fetched at
  build time (vendoring is X-011).
- `rfd` 0.18.0 exposes the `xdg-portal` backend.

## SQLite/ICU

- Node 22 ships full ICU (`process.versions.icu` non-empty) — checked during the
  UI stage. Bun 1.4.5 ships full ICU on Linux. Arabic number/date formatting
  therefore uses `Intl`; fallback formatter remains in `ui/src/i18n/format.ts`.

## Conclusion

S-0 exit criteria met: every pin resolves, the table is committed, and no plan
bug required a decision change. Versions are locked in `Cargo.toml` /
`ui/package.json` + lockfiles when those land.
