# T17 — llama.cpp CPU probe (S5)

**Needs:** T10 (prompt renderer) and T11 (models on disk) merged.

**Goal:** prove the pinned GGUF loads and generates on CPU through
`llama-cpp-2` with MSVC, using the exact prompt from T09.

**Files:** workspace `Cargo.toml`, `core/crates/ll-llm/Cargo.toml`,
create `core/crates/ll-llm/examples/llama_probe.rs`,
`docs/qa/2026-11-21-version-verification.md`

## Steps

1. Workspace: `llama-cpp-2 = "=0.1.161"`. In `ll-llm/Cargo.toml`:
   `llama-cpp-2 = { workspace = true, optional = true }`,
   `llama = ["dep:llama-cpp-2"]`, `[[example]] name = "llama_probe"`,
   `required-features = ["llama"]`. CPU only: do **not** enable `cuda`,
   `vulkan`, or `dynamic-backends`. If the default features fail to build on
   MSVC, try `default-features = false, features = ["openmp"]` once.
2. `llama_probe.rs` (args: gguf path, locale such as `en-US`):

   ```rust
   use llama_cpp_2::{llama_backend::LlamaBackend, model::{LlamaModel, params::LlamaModelParams},
       context::params::LlamaContextParams, llama_batch::LlamaBatch, sampling::LlamaSampler};
   let backend = LlamaBackend::init()?;
   let model = LlamaModel::load_from_file(&backend, path,
       &LlamaModelParams::default().with_use_mmap(true).with_use_mlock(false))?;
   let ctx_params = LlamaContextParams::default()
       .with_n_ctx(std::num::NonZeroU32::new(1024))
       .with_n_threads(3).with_n_threads_batch(3);
   let mut ctx = model.new_context(&backend, ctx_params)?;
   let prompt = ll_core::render_easylanguage_prompt(&pack, sentence);
   let tokens = model.vocab().tokenize(prompt.as_bytes(), false, true);
   // batch: add every token at pos i, logits only for the last one; ctx.decode(&mut batch)
   // loop up to 160 tokens: let t = sampler.sample(&ctx, last_idx); stop if model.vocab().is_eog(t);
   //   append model.vocab().token_to_piece(t, false, None) bytes; batch.clear(); batch.add(t, pos, &[0], true); ctx.decode
   ```

   Use `LlamaSampler::greedy()`. If an exact name differs in 0.1.161, read
   that crate's source for the right one (it's in your cargo registry);
   do not guess more than twice.
3. Run with the fixed sentence
   `"Notwithstanding the aforementioned considerations, the committee has resolved to postpone the deliberations indefinitely."`
   (en-US) and print: load ms, prompt tokens, TTFT ms, generated tokens,
   tokens/s, total ms, and the output. Then pass the output through
   `ll_core::strip_think_block` and print that too.

## Done when

- Output is plain English, simpler than the input, no `<think>` text left after
  stripping, total < 8 s on this machine.
- `cargo clippy -p ll-llm --features llama --all-targets -- -D warnings` passes.
- New file `docs/qa/2026-11-23-llm-cpu.md` with `## Probe`: crate version,
  build flags, load ms, TTFT, tok/s, total ms, "same language: yes/no".
- Version-verification row for `llama-cpp-2 0.1.161`.

Commit message: `llm: llama.cpp CPU probe`
