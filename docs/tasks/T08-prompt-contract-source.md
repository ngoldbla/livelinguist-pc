# T08 — Copy the EasyLanguage prompt contract into the repo (S4)

**Goal:** get the exact prompt format the simplifier model was trained on, from
its model card, into a repo file the next card can copy from. No code.

**Files:** create `docs/qa/2026-11-23-easylanguage-prompt-contract.md`

## Steps

1. Download the model card at the pinned revision:

   ```powershell
   New-Item -ItemType Directory -Force target\modelcard | Out-Null
   curl.exe -sSL -o target\modelcard\README.md "https://huggingface.co/ndgold/Qwen3-0.6B-EasyLanguage-GGUF/raw/658db5ad6a05e3f549990b10e8d8c59a78866695/README.md"
   ```

2. Open `target\modelcard\README.md`. Find the section that shows the prompt
   or chat template (look for `<|im_start|>`, `/no_think`, `Original:`,
   `Rewritten:`, `<think>`).
3. Create `docs/qa/2026-11-23-easylanguage-prompt-contract.md` containing:
   - the source URL and revision;
   - the template, **copied exactly** inside a fenced code block, including
     every newline and space. Do not retype it; copy-paste it;
   - any generation settings the card gives (temperature, stop tokens, max
     tokens), copied exactly;
   - any warning about few-shot examples or extra context, copied exactly.
4. Under a `## Placeholders` heading, list each placeholder in the template
   (for example the system prompt and the sentence) and what fills it:
   system prompt = the pack's `systemPrompt` from `assets/packs/<lang>.json`;
   sentence = the clicked caption text.
5. If the card has **no** clear template, write that in the file, commit it,
   and stop with `Result: blocked`. Do not invent a template.

## Done when

The file exists with the copied template, and `git diff` shows no other
changes.

Commit message: `docs: EasyLanguage prompt contract from the model card`
