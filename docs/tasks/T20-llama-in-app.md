# T20 — Real simplifier in `ll-eval` and the app (S5)

Linux part done in PR #31; Windows pending: bridge with `--features capture,sherpa,llama`, click a live caption, screenshot `console-click-simplified.png`.

**Needs:** T19 done.

**Files:** `core/crates/ll-eval/{Cargo.toml,src/lib.rs}`,
`core/crates/ll-bridge/{Cargo.toml,src/lib.rs}`

## Steps

1. Both crates: feature `llama = ["ll-llm/llama"]` (not default).
2. `ll-eval`: flag `--llm llama` (default stays demo). Model path =
   catalog LLM artifact file in `model_dir(cache_root(), repo, revision)`.
   Flag `--threads <n>` (default 3).
3. Bridge: when built with `llama` and the scenario is not demo/mock, load
   `LlamaBackend` lazily on the **first click** (not at session start). Missing
   model → `LLM_MODEL_MISSING` event; the clicked caption stays verbatim with
   the `≡` flag and a banner. In a **strict** dev scenario, a missing or failed
   model stops the session with the error instead (golden rule 7).
4. Tests (default features): `ll-eval --llm llama` without the feature errors
   clearly; bridge strict-scenario + missing model returns an error, normal
   scenario returns verbatim + flag (use the existing faulty/mock backends to
   simulate).

## Check

```powershell
cargo run --release -p ll-eval --features llama -- --scenario simplify-only --language en-US --llm llama --text "Notwithstanding the aforementioned considerations, the committee has resolved to postpone the deliberations indefinitely." --out target\eval-llama
```

Then build the bridge with `--features capture,sherpa,llama`, run the UI,
caption an English video, click a sentence: a simplified line appears.
Screenshot → `docs/screenshots/m2-windows/console-click-simplified.png`.

## Done when

Checks pass, plus `cargo test --workspace` and clippy with and without
`--features llama`.

Commit message: `llm: real simplifier in eval and bridge`
