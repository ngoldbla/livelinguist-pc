# ASR coverage v1 (S3: T29, T32, T33 measurements)

Measured 2026-11-24 on the Linux dev container with
`ll-eval --scenario asr-wer --asr sherpa --all` under the tier C approximation
(`taskset -c 0-3 nice -n 10`, recognizer 2 threads), FLEURS test split, about
300 s per locale (`tests/asr-eval/manifest.json`).

- **Score (WER/CER) is real quality data.**
- **RTF and p95 chunk ms are a Linux 4-core approximation, not the Windows
  tier C gate.**

Gate: score ≤ 0.15, RTF ≤ 0.5, p95 chunk ≤ 500 ms. A row is `ok (Linux)`
only when all three pass here. `CoverageMatrix::v1()` (`coverage.rs`) and the
UI table (`ui/src/i18n/languages.ts`) now follow this table: 12 locales `ok`,
17 `simplify_only`, none `tbd` (T33, 2026-11-24). The Windows tier C re-check
(`## Windows tier C re-check` below) measured all 12 `ok` rows plus ja-JP:
timing passes everywhere, but **de-DE scores 0.157 there (Linux 0.141) and
misses the score gate**; the owner kept it `ok` on 2026-11-24 with the number
recorded here. en-US sits exactly on the
gate (0.149). Captions on Windows were confirmed by the owner retest
(`docs/qa/2026-11-24-windows-retest.md`).

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

## Windows tier C re-check (2026-11-24)

Reference Windows 11 PC (Ryzen 9 3900X, 12C/24T), one process at a time:

```powershell
pwsh scripts\tier-c-profile.ps1 -- target\release\ll-eval.exe --scenario asr-wer `
  --asr sherpa --locale <locale> --cache target\fleurs --asr-threads 2 --out target\asr-wer-win\<locale>
```

`scripts\tier-c-profile.ps1` (affinity 0xF, BelowNormal, `PeakWorkingSet64`),
same manifest, same clips (SHA-256 verified on load) and same pinned model
(`model_sha256 012e93…`). Numbers only; hypotheses stayed in `target/` and are
not recorded here or committed.

Scope: all 12 locales whose status is `ok`, plus ja-JP. ja-JP is the only
`simplify_only` row within the host-to-host score spread seen below, so it is
the only one a Windows re-score could have promoted; the other 16 are ≥ 0.18 on
Linux, more than the largest delta observed here.

| Locale | Metric | Score Linux → Windows | RTF | p95 chunk ms | Peak RSS MB | Gate (score ≤ 0.15, RTF ≤ 0.5, p95 ≤ 500) |
|---|---|---|---|---|---|---|
| de-DE | wer | 0.141 → **0.157** | 0.246 | 288.5 | 778 | **fail (score)** |
| fr-FR | wer | 0.135 → 0.1427 | 0.251 | 297.6 | 779 | pass |
| es-ES | wer | 0.044 → 0.0459 | 0.253 | 321.4 | 780 | pass |
| en-US | wer | 0.149 → 0.149 | 0.255 | 313.8 | 779 | pass (exactly on the gate) |
| ar-SA | wer | 0.125 → 0.1211 | 0.244 | 285.8 | 779 | pass |
| hi-IN | wer | 0.101 → 0.0981 | 0.244 | 289.6 | 778 | pass |
| it-IT | wer | 0.061 → 0.0651 | 0.253 | 312.5 | 779 | pass |
| ko-KR | cer | 0.087 → 0.0873 | 0.244 | 286.0 | 779 | pass |
| pt-BR | wer | 0.095 → 0.0987 | 0.245 | 287.5 | 779 | pass |
| pt-PT | wer | 0.095 → 0.0987 | 0.245 | 287.1 | 778 | pass |
| ru-RU | wer | 0.148 → 0.1439 | 0.244 | 289.1 | 779 | pass |
| vi-VN | wer | 0.120 → 0.1188 | 0.243 | 289.4 | 779 | pass |
| ja-JP | cer | 0.155 → 0.1597 | 0.241 | 282.6 | 779 | fail (score); stays `simplify_only` |

Reading it:

- **Timing passes on every row** with at least 1.5× headroom (RTF ≤ 0.26 vs
  0.5; p95 ≤ 322 ms vs 500 ms). The first five rows ran while FLEURS tarballs
  were still downloading, so their RTF is an upper bound; de-DE re-run on an
  idle host gave 0.246 (0.262 while downloading). The gate outcome does not
  depend on that.
- **The score is not perfectly portable between hosts.** Same model, clips,
  threads and code; the score still moves by up to 0.016 (de-DE 0.141 → 0.157,
  fr-FR 0.135 → 0.1427, ru-RU 0.148 → 0.1439). onnxruntime's CPU path is
  numerically host-dependent, and a handful of tokens flip. de-DE is the one
  row that crosses the gate: 76 edits / 484 reference units = 0.157, the same
  in three runs (one on an idle host, one with `--asr-threads 3`). en-US is
  exactly 0.149.
- **`--asr-threads 3` does not help de-DE** (same 0.157) and misses the chunk
  gate on this host (p95 501.7 ms, RTF 0.381) — 2 threads stays the right
  default.
- ja-JP would still fail after a Windows re-score (0.1597), so no
  `simplify_only` row is promoted.

**Owner decision (2026-11-24): de-DE stays `ok`.** The plan's stop rule says
the minimum live set must pass, and on this host German misses the score gate by
0.007 while captions themselves work. The owner chose to keep the row `ok`,
record the Windows number here, and treat a tighter German score as a
model-card question (re-pin or fine-tune) rather than a coverage change. No
other row moves: every other `ok` locale passes, and ja-JP would still fail
after a Windows re-score.
