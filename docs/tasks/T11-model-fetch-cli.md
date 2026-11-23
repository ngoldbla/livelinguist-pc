# T11 — Model fetch command (S2)

**Goal:** one command that downloads the pinned ASR + LLM files with the
existing downloader, so later cards and humans can get the models.

**Files:** create `core/crates/ll-models/examples/fetch_models.rs`,
`core/crates/ll-models/Cargo.toml` (only if an `[[example]]` entry is needed)

## Steps

1. Write `fetch_models.rs`:
   - read `ModelCatalog::plan_default()`;
   - destination root = first CLI arg if given, else `cache_root()`;
   - for each artifact and each file, call `download_file(...)` with
     `allow_unverified = false` into
     `model_dir(root, &artifact.repo, &artifact.revision)`;
   - print one line per file: `ok <artifact id> <file> <bytes>` or
     `fail <artifact id> <file> <error>`; exit code 1 if any file failed;
   - call `disk_preflight` first with the catalog total.
   Use a `reqwest::blocking::Client` built the same way any existing code in
   `ll-models` builds it (search for `Client::builder`).
2. Run it: `cargo run --release -p ll-models --example fetch_models`
   (about 1.08 GB). Run it a second time: every file should say `ok` quickly
   (already present, hash verified).
3. Print the final model directory paths and record them in a new file
   `docs/qa/2026-11-23-asr-runtime.md` under `## Model files` (paths and sizes
   only).

## Done when

- Both runs exit 0; second run does not re-download.
- `cargo clippy -p ll-models --all-targets -- -D warnings` passes.

Commit message: `models: fetch_models example for the pinned set`
