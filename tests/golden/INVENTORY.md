# Golden fixture inventory (D-049 / T-012)

Reference: `ngoldbla/live-linguist` (private Mac/iOS repo), commit
`8eecdc4c06203cd1a1277b7a1c4123ce76c2acd2`, checked out on the Linux dev host
on 2026-11-21. The framework packs and test expectations below were read from
that tree; no PII media is included, and no audio/transcript is committed.

## Framework packs

| Path | Count | Provenance |
|---|---|---|
| `assets/packs/*.json` | 29 | Copied verbatim from `Sources/LiveLinguistKit/Resources/FrameworkPacks/`; SHA-256 of each file is reproducible from the reference commit. These are the canonical register rules and few-shots. |

Curated frameworks with bespoke validators: `falc` (fr), `leichte_sprache` (de),
`easy_english` (en), `lectura_facil` (es). Every other pack decodes its national
framework string to `easyToRead` and uses the language-agnostic validators.

## Ported suites

| Swift suite | Swift path | Tests in reference | Rust status | Rust location |
|---|---|---|---|---|
| `PromptGoldenTests` | `Tests/LiveLinguistKitTests/PromptGoldenTests.swift` | 3 | ported + 3 extension tests | `ll-core/src/prompt.rs`, `tests/golden/prompt_goldens.json` |
| `ChatTemplateGoldenTests` | not in the checked-out tree | 0 | deferred to `ll-llm`; Qwen3 template pinned there | `ll-llm` |
| `SegmenterTests` | `.../SegmenterTests.swift` | 11 | ported + 3 extension tests | `ll-core/src/segmenter.rs`, `tests/golden/segmenter_cases.json` |
| `CaptionDecodingTests` | `.../CaptionDecodingTests.swift` | 15 | ported | `ll-core/src/decoding.rs`, `tests/golden/decoding_cases.json` |
| `ValidatorTests` | `.../ValidatorTests.swift` | 25 | ported (12 golden cases + inline) | `ll-core/src/validation.rs`, `tests/golden/validator_cases.json` |
| `FidelityGuardTests` | `.../FidelityGuardTests.swift` | 17 | ported | `ll-core/src/fidelity.rs` |
| `NumberGuardTests` | `.../NumberGuardTests.swift` | 19 | ported + all-29-pack gold-pair regression | `ll-core/src/numbers.rs` |
| `SimplifierProfileTests` | `.../SimplifierProfileTests.swift` | 8 | policy ported; backend identity belongs to `ll-llm` | `ll-core/src/profile.rs` |
| `VoiceActivityDetectorTests` | `.../VoiceActivityDetectorTests.swift` | 14 | scheduled for `ll-audio` (stage 3) | `ll-audio` |
| `SilenceGateTests` | `.../SilenceGateTests.swift` | 11 | scheduled for `ll-audio` (stage 3) | `ll-audio` |
| `CaptionPipelineTests` | `.../CaptionPipelineTests.swift` | 5 | scheduled for `ll-pipeline` (stage 3) | `ll-pipeline` |
| `SessionCSVExporterTests` | `.../SessionCSVExporterTests.swift` | 14 | ported (11 inline + golden header) | `ll-core/src/export.rs`, `tests/golden/export_goldens.json` |
| `SessionStoreTests` | `.../SessionStoreTests.swift` | 8 | ported (core subset) | `ll-core/src/session.rs` |
| `LanguageTests` | `.../LanguageTests.swift` | 18 | ported + 29-locale checks | `ll-core/src/language.rs`, `ll-core/src/pack.rs` |
| `UILocalizationTests` | `.../UILocalizationTests.swift` | 9 | scheduled for `ui/src/i18n` (stage 5) | `ui/src/i18n` |
| `ReversionDetectorTests` | `.../ReversionDetectorTests.swift` | 7 | ported + 2 extension tests | `ll-core/src/reversion.rs`, `tests/golden/reversion_cases.json` |
| `LatencyMetricsTests` | `.../LatencyMetricsTests.swift` | 6 | ported | `ll-core/src/metrics.rs` |
| `FrameworkPackTests` | `.../FrameworkPackTests.swift` | 7 | ported | `ll-core/src/pack.rs` |

## Golden files in this repo

| File | Contents |
|---|---|
| `tests/golden/prompt_goldens.json` | Lean, lean-with-context, few-shot, rolling-context, and retry prompt literals |
| `tests/golden/segmenter_cases.json` | Emitted-hypothesis sequences with expected finalized texts/partials |
| `tests/golden/decoding_cases.json` | Stop/first-line/collapse/think-strip cases |
| `tests/golden/validator_cases.json` | Framework/output/source → expected rule + severity |
| `tests/golden/reversion_cases.json` | LID language sequences → expected reversion spans |
| `tests/golden/export_goldens.json` | Exact CSV header, Markdown/plain required lines, reason strings |
| `tests/golden/easylanguage_prompt_goldens.json` | `render_easylanguage_prompt` output for all 29 locales with the sentence "This is a test sentence." Template from the Qwen3 EasyLanguage GGUF model card (`docs/qa/2026-11-23-easylanguage-prompt-contract.md`); regenerate with `LL_WRITE_GOLDENS=1 cargo test -p ll-core --test golden easylanguage` |

## Mac-generated samples

The Mac repo's `Tests/LiveLinguistKitTests/Resources/` contains only
`bg4_floats_frame.b64` and `bg4_floats_plain.b64` (audio-frame codec fixtures,
not transcript). They are not needed by the deterministic port; if the audio
suite is ported later, copy them with their provenance and keep them free of PII.

## Reproduce

```bash
cargo test -p ll-core                      # 143 unit tests + 6 golden tests
cargo test -p ll-core --test golden        # golden harness only
```
