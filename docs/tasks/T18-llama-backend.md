# T18 — `LlamaBackend` (S5)

**Needs:** T17 done.

**Goal:** a real `SimplifierBackend` that turns one sentence into its
easy-language version with the pinned GGUF on CPU.

**Files:** create `core/crates/ll-llm/src/llama.rs`, edit `core/crates/ll-llm/src/lib.rs`
(`#[cfg(feature = "llama")] mod llama; pub use llama::LlamaBackend;`)

## Steps

1. `LlamaBackend::load(gguf: &Path, threads: i32) -> Result<Self, BackendError>`:
   file missing → a model-missing error (add `BackendError::model_missing` if
   there is no such constructor); load failure → backend error with a short
   reason. Keep backend, model, and one context inside the struct (if lifetimes
   fight you, keep `LlamaModel` in a `Box`/`Arc` and create the context per
   call; measure the cost in the T21 card).
2. `impl SimplifierBackend for LlamaBackend` → `simplify(&mut self, request)`:
   - build the prompt with `render_easylanguage_prompt(pack, &request.sentence)`
     — ignore any context fields in `request`;
   - clear the KV cache before each call;
   - greedy decode, stop on end-of-generation, max new tokens =
     `min(256, 3 * sentence_tokens + 32)`;
   - timeout: stop and return `BackendError::timeout()` after 15 s;
   - return `strip_think_block(output)`, trimmed.
   Validators, reversion and the `≡` fallback already run after the backend in
   the pipeline; do not duplicate them here.
3. The trait requires `Send`. If llama types aren't `Send`, run the backend on
   its own worker thread and talk to it with a channel (`LlamaBackend` holds
   the sender). Do not use `unsafe impl Send`.
4. Tests (`llama` feature): `missing_file_is_model_missing` always runs;
   `simplifies_en_sentence` runs only when env `LL_LLM_GGUF` is set: output
   non-empty, no `<think>`, passes `ll_core` language guard for English.

## Done when

```powershell
cargo test -p ll-llm --features llama
$env:LL_LLM_GGUF="<path to Qwen3-0.6B-EasyLanguage-Q4_K_M.gguf>"; cargo test -p ll-llm --features llama simplifies_en_sentence -- --nocapture
cargo test --workspace
cargo clippy -p ll-llm --features llama --all-targets -- -D warnings
```

Commit message: `llm: LlamaBackend on CPU`
