# T09 — `EasyLanguagePrompt` renderer + 29 goldens (S4)

**Needs:** T08 done.

**Goal:** one function that renders the exact prompt from T08, with tests that
pin its output for every language.

**Files:**
- `core/crates/ll-core/src/prompt.rs`
- `core/crates/ll-core/src/lib.rs` (re-export)
- create `tests/golden/easylanguage_prompt_goldens.json`
- `tests/golden/INVENTORY.md`

## Steps

1. In `prompt.rs` add:

   ```rust
   /// Renders the exact prompt contract of the Qwen3 EasyLanguage GGUF.
   /// No few-shot examples, no previous segments.
   pub fn render_easylanguage_prompt(pack: &FrameworkPack, sentence: &str) -> String
   ```

   Build the string by filling the template copied in
   `docs/qa/2026-11-23-easylanguage-prompt-contract.md`. Use the pack's
   system prompt field (the one loaded from `systemPrompt` in the JSON). Do
   not trim or reformat the system prompt. Do not touch `PromptBuilder` or its
   existing goldens.
2. Write a tiny test helper that renders the prompt for every language with
   the fixed sentence `"This is a test sentence."` and collects
   `{ "locale": ..., "prompt": ... }` objects. Run it once with an env var
   (for example `LL_WRITE_GOLDENS=1`) to write
   `tests/golden/easylanguage_prompt_goldens.json`; without the env var the
   test compares against the file. Check the generated file by eye for 2–3
   languages against the T08 template.
3. Tests:
   - `easylanguage_prompt_matches_goldens` (all 29);
   - `easylanguage_prompt_has_no_context`: render for a sentence and assert the
     output contains the sentence exactly once and does not contain the
     pack's few-shot sources (if the pack has any);
   - if the template has `/no_think` or an empty think block, assert each
     appears exactly once.
4. Add a line for the new golden file to `tests/golden/INVENTORY.md`.

## Done when

```powershell
cargo test -p ll-core
cargo clippy -p ll-core --all-targets -- -D warnings
cargo fmt --all -- --check
```

Commit message: `core: EasyLanguage prompt renderer and goldens`
