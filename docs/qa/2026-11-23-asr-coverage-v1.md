# ASR coverage v1 (S3: T29, T32, T33 measurements)

Measured 2026-11-24 on the Linux dev container with
`ll-eval --scenario asr-wer --asr sherpa --all` under the tier C approximation
(`taskset -c 0-3 nice -n 10`, recognizer 2 threads), FLEURS test split, about
300 s per locale (`tests/asr-eval/manifest.json`).

- **Score (WER/CER) is real quality data.**
- **RTF and p95 chunk ms are a Linux 4-core approximation, not the Windows
  tier C gate.**

Gate: score ≤ 0.15, RTF ≤ 0.5, p95 chunk ≤ 500 ms. A row is `ok (Linux)`
only when all three pass here. The Windows re-check is **pending** for every
row, so `coverage.rs` is not changed yet: T33 step 2 is open.

| Locale | FLEURS | Metric | Score | RTF | p95 chunk ms | Clips | Status | Note |
|---|---|---|---|---|---|---|---|---|
| de-DE | de_de | wer | 0.141 | 0.211 | 264 | 23 | ok (Linux; Windows pending) |  |
| fr-FR | fr_fr | wer | 0.135 | 0.218 | 274 | 28 | ok (Linux; Windows pending) |  |
| es-ES | es_419 | wer | 0.044 | 0.203 | 244 | 29 | ok (Linux; Windows pending) | dialect mismatch: Latin-American Spanish recordings |
| en-US | en_us | wer | 0.149 | 0.206 | 253 | 33 | ok (Linux; Windows pending) |  |
| ar-SA | ar_eg | wer | 0.125 | 0.210 | 256 | 28 | ok (Linux; Windows pending) | dialect mismatch: Egyptian speakers reading MSA prompts |
| da-DK | da_dk | wer | 0.319 | 0.210 | 266 | 30 | simplify_only (score) |  |
| et-EE | et_ee | wer | 0.376 | 0.207 | 252 | 26 | simplify_only (score) |  |
| fi-FI | fi_fi | wer | 0.297 | 0.209 | 254 | 23 | simplify_only (score) |  |
| hi-IN | hi_in | wer | 0.101 | 0.213 | 265 | 29 | ok (Linux; Windows pending) |  |
| it-IT | it_it | wer | 0.061 | 0.214 | 268 | 21 | ok (Linux; Windows pending) |  |
| ja-JP | ja_jp | cer | 0.155 | 0.208 | 259 | 21 | simplify_only (score) | scored with CER |
| ko-KR | ko_kr | cer | 0.087 | 0.207 | 247 | 23 | ok (Linux; Windows pending) | scored with CER |
| nl-NL | nl_nl | wer | 0.182 | 0.207 | 250 | 31 | simplify_only (score) |  |
| pt-BR | pt_br | wer | 0.095 | 0.216 | 265 | 21 | ok (Linux; Windows pending) |  |
| pt-PT | pt_br | wer | 0.095 | 0.231 | 286 | 21 | ok (Linux; Windows pending) | mismatch: no pt_pt in FLEURS; Brazilian recordings |
| ru-RU | ru_ru | wer | 0.148 | 0.221 | 294 | 31 | ok (Linux; Windows pending) |  |
| sk-SK | sk_sk | wer | 0.309 | 0.212 | 258 | 29 | simplify_only (score) |  |
| sv-SE | sv_se | wer | 0.258 | 0.216 | 269 | 20 | simplify_only (score) |  |
| tr-TR | tr_tr | wer | 0.275 | 0.212 | 250 | 25 | simplify_only (score) |  |
| vi-VN | vi_vn | wer | 0.120 | 0.224 | 286 | 25 | ok (Linux; Windows pending) |  |
| zh-CN | cmn_hans_cn | cer | 0.266 | 0.230 | 296 | 26 | simplify_only (score) | scored with CER |
| uk-UA | uk_ua | wer | 0.196 | 0.225 | 287 | 29 | simplify_only (score) |  |
| ro-RO | ro_ro | wer | 0.436 | 0.220 | 277 | 34 | simplify_only (score) |  |
| bg-BG | bg_bg | wer | 0.255 | 0.208 | 252 | 29 | simplify_only (score) |  |
| pl-PL | pl_pl | wer | 0.297 | 0.223 | 295 | 28 | simplify_only (score) |  |
| hr-HR | hr_hr | wer | 0.505 | 0.222 | 293 | 32 | simplify_only (score) |  |
| nb-NO | nb_no | wer | 0.248 | 0.217 | 273 | 25 | simplify_only (score) |  |
| hu-HU | hu_hu | wer | 0.360 | 0.223 | 284 | 31 | simplify_only (score) |  |
| cs-CZ | cs_cz | wer | 0.283 | 0.218 | 277 | 25 | simplify_only (score) |  |

Minimum live set (T32): en, fr, de, es, ar all pass on score and on the
Linux timing approximation. en-US (0.149) and de-DE (0.141) are close to the
gate. Every locale passes the timing gate here (RTF ≈ 0.21, p95 ≈ 250–300 ms).
Failing on score: ja-JP (CER 0.155, just over), and most Nordic, Baltic and
Central European locales.
