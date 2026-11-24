# ASR evaluation fixtures (FLEURS)

- Source: `google/fleurs` (Conneau et al., 2022), **test** split, dataset
  revision `70bb2e84b976b7e960aa89f1c648e09c59f894dd`.
- License: **CC-BY-4.0**. Attribution: "FLEURS, Google, CC-BY-4.0".
- `manifest.json`: for each of the 29 app locales, test clips chosen in TSV
  order up to ~300 s, with file name, duration, SHA-256, and the normalized
  public reference transcription (read speech, not user transcripts).
- Fetch: `python -I scripts/asr-eval/fetch_fleurs.py --cache target/fleurs`
  (streams the tarballs and keeps only the listed clips, verifies every hash).
  Rebuild the manifest: add `--build-manifest`.
- Score: `ll-eval --features sherpa --scenario asr-wer --asr sherpa --all --cache target/fleurs --out target/wer-all`.
- **Audio is never committed.** CI rejects `.wav`/`.flac` files.
