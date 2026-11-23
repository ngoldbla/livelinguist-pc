# T31 — `ll-eval` ASR WER scenario (S3)

**Needs:** T29, T30 done.

**Files:** `core/crates/ll-eval/src/lib.rs`

## Steps

1. Scenario `--scenario asr-wer --locale <xx-XX> --manifest tests\asr-eval\manifest.json --cache <dir> --asr sherpa`:
   for each clip of that locale: decode the audio to 16 kHz mono f32 (FLEURS
   is 16 kHz WAV; if a clip isn't, use the existing resampler), feed it in
   17 600-sample chunks to `SherpaAsr` (fresh `reset()` per clip), take the
   final hypothesis, score with T30 (CER for ja/ko/zh, WER otherwise).
2. Measure: `rtf` = total processing time / total audio time; per-chunk
   latency (time for one `process` call) → `p95_chunk_ms`; count endpoints.
3. Write `wer.json` to `--out`: `{ locale, fleurs_code, clips, audio_s, metric
   ("wer"|"cer"), score, rtf, p95_chunk_ms, model_sha256 (encoder) }`.
   Hypotheses go only to `--out\hyp.txt` for local inspection (never
   committed, never logged).
4. Flag `--all` runs every locale in the manifest and writes one `wer.json`
   per locale plus `matrix.json`.
5. Unit test: argument parsing; a fake engine that echoes the reference gives
   score 0.

## Check

```powershell
python -I scripts\asr-eval\fetch_fleurs.py --cache target\fleurs --locales en-US
cargo run --release -p ll-eval --features sherpa -- --scenario asr-wer --locale en-US --manifest tests\asr-eval\manifest.json --cache target\fleurs --asr sherpa --out target\wer-en
```

`score` for en-US should be well under 0.15 (Nemotron's published average is
~9%). If it is above 0.30, something is wrong (sample rate, chunking,
normalization, language option): check `hyp.txt` before going on.

Commit message: `eval: asr-wer scenario`
