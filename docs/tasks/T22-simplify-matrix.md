# T22 — 29-pack simplifier matrix (S5) (PR)

Linux part done in PR #31 (28/29; es-ES language-guard false negative, human decision); Windows pending: none beyond re-run if wanted.

**Needs:** T21 done.

**Goal:** prove the real model rewrites in the same language for all 29 packs,
never translates, and is deterministic.

**Files:** `core/crates/ll-eval/src/lib.rs`

## Steps

1. Scenario `--scenario simplify-matrix`: for every language, take the pack's
   first 2 demo sentences (the same source the demo conversation uses), run
   them through `LlamaBackend` **twice**, then through the existing
   validators/language guard. Per language record: `ran`, `same_language`
   (language guard pass count), `deterministic` (run 1 == run 2), `fallbacks`
   (validator fallbacks to verbatim), `p95_ms`. Write `matrix.json` to `--out`.
   No sentences or outputs in the summary; outputs may go only to
   `--out\outputs\` for a human to inspect locally (never committed).
2. Run it (release, `--threads` = the T21 default).
3. Gate per language: `ran` and `deterministic` must be true; `same_language`
   must be 2/2. If a language fails, look at its outputs file and record why
   (do not change the pack or the prompt in this card).

## Done when

`docs/qa/2026-11-23-llm-cpu.md` has `## 29-pack matrix` (29 rows from
`matrix.json`). Mark `docs/qa/2026-11-21-llm-backends.md` as historical in its
status line ("GPU ladder is post-1.0; see 2026-11-23-llm-cpu.md").
Set S5 to `in review` in the progress board.

Open PR `windows: CPU llama.cpp click-to-simplify` from branch
`stage-w4-llama-cpu` (T17–T22) with the latency table and matrix summary;
merge when CI is green; set S5 to `merged`. If any language fails the gate,
say so in the PR and leave the decision to a human — do not merge in that case.

Commit message: `eval: 29-pack simplifier matrix`
