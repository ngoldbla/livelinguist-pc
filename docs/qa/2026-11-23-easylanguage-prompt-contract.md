# EasyLanguage prompt contract (S4, T08)

Source: model card of `ndgold/Qwen3-0.6B-EasyLanguage-GGUF` at the pinned
revision `658db5ad6a05e3f549990b10e8d8c59a78866695`:
<https://huggingface.co/ndgold/Qwen3-0.6B-EasyLanguage-GGUF/raw/658db5ad6a05e3f549990b10e8d8c59a78866695/README.md>
(fetched 2026-11-24). Everything in the quoted sections below is copied from
the card by script, not retyped.

## Template (copied exactly)

The model was trained against one exact rendering, and deviating from it degrades output
**silently**. There is no error, just worse rewrites.

````text
<|im_start|>system
{SYSTEM_PROMPT}<|im_end|>
<|im_start|>user
Original: {SENTENCE}
Rewritten:<|im_end|>
<|im_start|>assistant
<think>

</think>

````

The block ends with `<think>`, an empty line, `</think>`, and an empty line:
the assistant turn is pre-filled with `<think>\n\n</think>\n\n` and
generation starts right after it.

## Warnings (copied exactly)

Three things people get wrong:

1. **The assistant turn is pre-filled with an EMPTY think block** — literally
   `<think>\n\n</think>\n\n`. This is Qwen3's `enable_thinking=false` form. Omit it and the model
   may emit reasoning into your caption.
2. **`{SYSTEM_PROMPT}` carries a trailing ` /no_think` soft switch.** It belongs to the prompt,
   not the template.
3. **No few-shot examples, and no rolling context.** The fine-tune internalised the register, so it
   was trained on the bare `Original:/Rewritten:` turn. Injecting examples or prior segments is
   out-of-distribution and bleeds context into the rewrite.

## Generation settings (copied exactly)

## Sampling — fixed, and evaluated under these values

| parameter | value |
|---|---|
| temperature | **0** (greedy) |
| repeat_penalty | **1.3** |
| repeat_last_n | **20** |
| n_predict | 128 |

These are not tuning knobs. The gate results below were produced under exactly these settings.

## Placeholders

| Placeholder | Filled with |
|---|---|
| `{SYSTEM_PROMPT}` | The pack's `systemPrompt` from `assets/packs/<pack>.json` (unchanged, not trimmed), followed by the soft switch ` /no_think` (a space, then `/no_think`), as the card's point 2 requires. |
| `{SENTENCE}` | The clicked caption text, exactly as finalized. One sentence/segment only. |

No few-shot examples and no previous segments go anywhere in the prompt
(card point 3; pipeline change T10).

## Cross-check against the training data

The card names the training set `ndgold/live-linguist-easylanguage-sft`
(revision `5d1359ec82495f52a4ed2c2ad6f5416532e8c052`). Its `*/test.jsonl`
files store `messages` (system/user/assistant) before chat-template rendering:

- For all 29 languages the `system` message is byte-identical to our pack's
  `systemPrompt` (checked by script on 2026-11-24; `de`→`leichte_sprache`,
  `en`→`easy_english`, `es`→`lectura_facil`, `fr`→`falc`).
- The `user` message is `Original: <sentence>\nRewritten:`, which matches
  the template.
- The stored `system` messages do **not** contain ` /no_think`. The card says
  the soft switch "belongs to the prompt, not the template". We read that as:
  the trainer appended it when rendering, and the stored messages are the
  pre-render form. The renderer (T09) follows the card and appends
  ` /no_think`.

**Open item for the owner:** confirm that the training renderer appended
` /no_think` to the system prompt. Until confirmed, the 29-pack simplifier
matrix (T22) records an A/B check (with vs without the soft switch) on the
real model, so a wrong guess shows up as a measured difference instead of
silently.
