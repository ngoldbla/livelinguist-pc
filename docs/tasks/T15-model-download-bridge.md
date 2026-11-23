# T15 — Model download commands in the bridge (first-run UX)

**Needs:** T11, T14 merged.

**Goal:** the UI can ask which models are present and download them with
progress, without blocking the UI thread.

**Files:** `core/crates/ll-bridge/src/lib.rs`, `core/crates/ll-bridge/Cargo.toml`,
`ui/src/bridge/generated/*` (regenerate if the project generates TS types)

## Steps

1. Add `#[napi] pub fn model_status(&self) -> Result<String>` returning JSON:
   `[{ id, title, bytes, present: bool, verified: bool }]` for every artifact
   in `ModelCatalog::plan_default()`. `present` = all files exist in
   `model_dir(cache_root(), repo, revision)`; `verified` = sizes match (do not
   hash 1 GB on every call; hashing happens during download).
2. Add `#[napi] pub fn download_models(&self, on_event: ThreadsafeFunction<String>) -> Result<()>`:
   - spawns a thread; returns immediately;
   - runs `disk_preflight` then `download_file` for each file (same as
     `fetch_models.rs`; move the shared loop into a function in `ll-models`
     and call it from both);
   - sends envelopes (same `{ v, seq, ts }` helper as session events) of kind
     `model_progress { id, file, done_bytes, total_bytes }` at most 4 per second
     per file, then `model_ready { id }` or `model_error { id, code, reason }`
     with codes `DISK_FULL`, `HASH_MISMATCH`, `NETWORK`, `CANCELLED`;
   - only one download at a time: a second call while running returns an error.
3. Add `#[napi] pub fn cancel_download(&self) -> Result<bool>` using an
   `AtomicBool` the download loop checks between chunks (add a cancel check
   to the shared loop; partial files stay for resume).
4. Tests (no network): `model_status` on an empty temp root reports
   `present: false` for both; a second `download_models` while one is running
   is rejected (use a fake/blocked downloader or a flag; do not hit the
   network in tests).

## Done when

```powershell
cargo test -p ll-bridge
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

Commit message: `bridge: model status, download, and cancel`
