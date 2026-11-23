# T30 — Text normalizer, WER and CER (S3)

**Goal:** one tested way to score ASR output against references.

**Files:** create `core/crates/ll-eval/src/wer.rs`, edit `core/crates/ll-eval/src/lib.rs`

## Steps

1. `pub fn normalize(text: &str, cjk: bool) -> String`: lowercase (Unicode),
   remove punctuation (Unicode `P*` categories), collapse whitespace, trim.
   Keep digits as-is (FLEURS normalized text already spells numbers its own
   way; do not convert). For `cjk`, also remove all spaces.
2. `pub fn wer(reference: &str, hypothesis: &str) -> (usize, usize)` returns
   (edit distance in words, reference word count) using Levenshtein on word
   tokens of the normalized strings.
   `pub fn cer(...)` the same on Unicode scalar values (chars).
3. Corpus score = sum(edits) / sum(reference length) across clips, not the
   average of per-clip rates.
4. Tests with hand-checked values:
   - `wer("the cat sat", "the cat sat") == (0, 3)`;
   - `wer("the cat sat", "the bat sat on") == (2, 3)`;
   - normalize removes `¿?¡!.,;:«»"` and lowercases `ÉCOLE` → `école`;
   - `cer("今天天气好", "今天天汽好") == (1, 5)`;
   - empty hypothesis → edits = reference length.

## Done when

```powershell
cargo test -p ll-eval
cargo clippy -p ll-eval --all-targets -- -D warnings
```

Commit message: `eval: normalizer and WER/CER`
