# T29 — FLEURS evaluation manifest (S3)

**Needs:** T14 merged. FLEURS is approved by the product owner (ADR 0011
addendum).

**Goal:** a committed manifest of ~5 minutes of FLEURS test clips per locale,
and a script that fetches exactly those clips into a local cache. **No audio
is committed.**

**Files:** create `scripts/asr-eval/fetch_fleurs.py`,
`tests/asr-eval/manifest.json`, `tests/asr-eval/README.md`; add
`target/` or the cache dir to `.gitignore` if it isn't already ignored.

## Locale map (verify every code exists under `data/` in the `google/fleurs` dataset repo)

| App | FLEURS | Note |
|---|---|---|
| de-DE | de_de | |
| fr-FR | fr_fr | |
| es-ES | es_419 | Latin-American Spanish; record as dialect mismatch |
| en-US | en_us | |
| ar-SA | ar_eg | Egyptian recordings of MSA prompts; record as mismatch |
| da-DK | da_dk | |
| et-EE | et_ee | |
| fi-FI | fi_fi | |
| hi-IN | hi_in | |
| it-IT | it_it | |
| ja-JP | ja_jp | CER |
| ko-KR | ko_kr | CER |
| nl-NL | nl_nl | |
| pt-BR | pt_br | |
| pt-PT | pt_br | no pt_pt in FLEURS; record as mismatch |
| ru-RU | ru_ru | |
| sk-SK | sk_sk | |
| sv-SE | sv_se | |
| tr-TR | tr_tr | |
| vi-VN | vi_vn | |
| zh-CN | cmn_hans_cn | CER |
| uk-UA | uk_ua | |
| ro-RO | ro_ro | |
| bg-BG | bg_bg | |
| pl-PL | pl_pl | |
| hr-HR | hr_hr | |
| nb-NO | nb_no | |
| hu-HU | hu_hu | |
| cs-CZ | cs_cz | |

If a code is missing, mark that locale `no_fixture` in the manifest; it will
ship `simplify_only`.

## Steps

1. `fetch_fleurs.py` (Python 3, stdlib + `requests` only), run as
   `python -I scripts\asr-eval\fetch_fleurs.py --cache <dir> [--locales en-US,fr-FR] [--build-manifest]`:
   - for each locale download `data/<code>/test.tsv` and
     `data/<code>/audio/test.tar.gz` from
     `https://huggingface.co/datasets/google/fleurs/resolve/<revision>/...`
     (pin `<revision>` to the dataset commit you see today; write it in the
     manifest);
   - **one locale at a time**: open the tar, extract only the chosen clips into
     `<cache>/<locale>/`, then delete the tarball (disk space);
   - `--build-manifest`: choose clips in TSV order until the total duration
     reaches 300 s (duration = `num_samples / 16000` from the TSV; check the
     header row for column names), and write their ids, file names, SHA-256,
     and the reference transcription column (use the `transcription`
     /normalized column if present) into `tests/asr-eval/manifest.json`;
   - without `--build-manifest`: fetch only the clips listed in the manifest
     and verify each SHA-256.
2. `tests/asr-eval/README.md`: source, license (CC-BY-4.0, attribution line),
   dataset revision, how to fetch, and "audio is never committed".
3. Build the manifest for all 29 locales. Reference transcripts are public
   read-speech text, allowed in the repo; they are **not** user transcripts.

## Done when

- `manifest.json` has 29 locale entries (clips or `no_fixture`), each ~300 s.
- Running the script again without `--build-manifest` re-verifies all hashes.
- `git status` shows no `.wav`/`.flac` files staged (CI rejects them).

Commit message: `eval: FLEURS manifest and fetch script`
