//! Golden-fixture harness for the deterministic core.
//!
//! The fixtures in `tests/golden/` were ported from the Mac reference test
//! suites; these tests prove the Rust port reads and satisfies them.

use ll_core::decoding::{
    collapse_repeated_sentences, first_line, reached_sentence_stop, reached_stop,
    strip_think_block, DecodingStyle,
};
use ll_core::export::{markdown, plain_text, session_csv};
use ll_core::language::{Framework, Language};
use ll_core::pack::{FewShot, FrameworkPack, Thresholds};
use ll_core::prompt::{render_easylanguage_prompt, PromptBuilder, EMPTY_THINK_BLOCK};
use ll_core::reversion::ReversionDetector;
use ll_core::segmenter::Segmenter;
use ll_core::session::{
    AudioSourceKind, FallbackReason, FinalizedSegment, SessionConfiguration, SessionStore,
    SimplificationOutcome, SimplifiedSegment, SkipReason,
};
use ll_core::validation::{Severity, ValidatorSuite};
use serde_json::Value;
use std::path::PathBuf;

fn golden(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../tests/golden")
        .join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

fn strings(value: &Value, key: &str) -> Vec<String> {
    value[key]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn prompt_goldens_match_the_mac_renderer() {
    let root = golden("prompt_goldens.json");
    let pack_json = &root["pack"];
    let base = FrameworkPack {
        framework: Framework::from_name(pack_json["framework"].as_str().unwrap()),
        language: pack_json["language"].as_str().unwrap().to_string(),
        display_name: pack_json["display_name"].as_str().unwrap().to_string(),
        system_prompt: pack_json["system_prompt"].as_str().unwrap().to_string(),
        retry_prompt_template: pack_json["retry_prompt_template"]
            .as_str()
            .unwrap()
            .to_string(),
        thresholds: Thresholds {
            max_words_hard: pack_json["max_words_hard"].as_u64().unwrap() as usize,
            max_words_soft: pack_json["max_words_soft"].as_u64().unwrap() as usize,
        },
        few_shots: Vec::new(),
        base_lang: None,
        stopwords: None,
        length_unit: None,
    };
    for case in root["cases"].as_array().unwrap() {
        let mut pack = base.clone();
        pack.few_shots = case["few_shots"]
            .as_array()
            .map(|shots| {
                shots
                    .iter()
                    .map(|shot| FewShot {
                        source: shot["source"].as_str().unwrap().to_string(),
                        simplified: shot["simplified"].as_str().unwrap().to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        let builder = PromptBuilder::new(&pack);
        let request = builder.make_request(
            case["sentence"].as_str().unwrap(),
            &strings(case, "previous"),
        );
        if let Some(expected) = case["expected"].as_str() {
            assert_eq!(request.user_prompt, expected, "{}", case["name"]);
        }
        for needle in strings(case, "expected_contains") {
            assert!(request.user_prompt.contains(&needle), "{}", case["name"]);
        }
        for needle in strings(case, "expected_excludes") {
            assert!(!request.user_prompt.contains(&needle), "{}", case["name"]);
        }
    }

    let retry = &root["retry"];
    let builder = PromptBuilder::new(&base);
    let violations = strings(retry, "violations")
        .into_iter()
        .map(|detail| ll_core::validation::Violation::hard("sentence_length", detail))
        .collect::<Vec<_>>();
    let request = builder.make_retry_request(
        retry["sentence"].as_str().unwrap(),
        &[],
        retry["failed_output"].as_str().unwrap(),
        &violations,
    );
    assert_eq!(request.user_prompt, retry["expected"].as_str().unwrap());
}

#[test]
fn segmenter_goldens_match_the_mac_behavior() {
    let root = golden("segmenter_cases.json");
    for case in root["cases"].as_array().unwrap() {
        let mut segmenter = Segmenter::default();
        let mut finalized = Vec::new();
        let mut partial: Option<String> = None;
        for step in case["steps"].as_array().unwrap() {
            let time = step["time"].as_f64().unwrap();
            match step["kind"].as_str().unwrap() {
                "ingest" => {
                    let output = segmenter.ingest(step["text"].as_str().unwrap(), time, None);
                    finalized.extend(output.finalized.iter().map(|segment| segment.text.clone()));
                    partial = output.partial.map(|partial| partial.text);
                }
                "tick" => {
                    let output = segmenter.tick(step["silence"].as_f64().unwrap(), time);
                    finalized.extend(output.finalized.iter().map(|segment| segment.text.clone()));
                    partial = output.partial.map(|partial| partial.text);
                }
                "flush" => {
                    if let Some(segment) = segmenter.flush(time) {
                        finalized.push(segment.text);
                    }
                    partial = None;
                }
                other => panic!("unknown step {other}"),
            }
        }
        assert_eq!(
            finalized,
            strings(case, "expected_finalized"),
            "{}",
            case["name"]
        );
        let expected_partial = case["expected_partial"].as_str().map(ToString::to_string);
        assert_eq!(partial, expected_partial, "{}", case["name"]);
    }
}

#[test]
fn decoding_goldens_match_the_mac_behavior() {
    let root = golden("decoding_cases.json");
    for case in root["cases"].as_array().unwrap() {
        let input = case["input"].as_str().unwrap();
        match case["fn"].as_str().unwrap() {
            "reached_sentence_stop" => {
                assert_eq!(
                    reached_sentence_stop(input),
                    case["expected"].as_bool().unwrap(),
                    "{input}"
                );
            }
            "reached_stop" => {
                let style = match case["style"].as_str().unwrap() {
                    "single_line" => DecodingStyle::SingleLine,
                    _ => DecodingStyle::FirstSentence,
                };
                assert_eq!(
                    reached_stop(input, style),
                    case["expected"].as_bool().unwrap(),
                    "{input}"
                );
            }
            "first_line" => {
                assert_eq!(first_line(input), case["expected"].as_str().unwrap());
            }
            "collapse" => {
                assert_eq!(
                    collapse_repeated_sentences(input),
                    case["expected"].as_str().unwrap()
                );
            }
            "strip_think" => {
                assert_eq!(strip_think_block(input), case["expected"].as_str().unwrap());
            }
            other => panic!("unknown fn {other}"),
        }
    }
}

#[test]
fn validator_goldens_match_the_mac_behavior() {
    let root = golden("validator_cases.json");
    for case in root["cases"].as_array().unwrap() {
        let pack = FrameworkPack::load(case["framework"].as_str().unwrap()).unwrap();
        let suite = ValidatorSuite::new(&pack);
        let violations = suite.validate(
            case["output"].as_str().unwrap(),
            case["source"].as_str().unwrap(),
            &[],
        );
        for expected in case["expected"].as_array().unwrap() {
            let rule = expected["rule"].as_str().unwrap();
            let severity = match expected["severity"].as_str().unwrap() {
                "hard" => Severity::Hard,
                "soft" => Severity::Soft,
                other => panic!("unknown severity {other}"),
            };
            assert!(
                violations
                    .iter()
                    .any(|violation| violation.rule == rule && violation.severity == severity),
                "{}: expected {rule}/{severity:?}, got {violations:?}",
                case["name"]
            );
        }
    }
}

#[test]
fn reversion_goldens_match_the_mac_behavior() {
    let root = golden("reversion_cases.json");
    for case in root["cases"].as_array().unwrap() {
        let mut detector = ReversionDetector::new(case["target"].as_str().unwrap());
        let detected = strings(case, "detected");
        for (index, language) in detected.iter().enumerate() {
            let start = (index * 2) as f64;
            let segment =
                FinalizedSegment::with_language("x", Some(language.clone()), start, start + 2.0);
            detector.observe(&segment);
        }
        detector.finish();
        let expected = case["expected_spans"].as_array().unwrap();
        assert_eq!(detector.spans().len(), expected.len(), "{}", case["name"]);
        for (span, expected) in detector.spans().iter().zip(expected) {
            assert_eq!(span.start, expected[0].as_f64().unwrap());
            assert_eq!(span.end, expected[1].as_f64().unwrap());
        }
    }
}

#[test]
fn export_goldens_match_the_mac_format() {
    let root = golden("export_goldens.json");
    let mut store = SessionStore::new(SessionConfiguration::new(
        Language::FrFr,
        AudioSourceKind::Microphone,
    ));
    store.duration_seconds = Some(125.0);
    store.append(SimplifiedSegment::new(
        FinalizedSegment::with_language("Bonjour.", Some("fr".into()), 1.0, 3.5),
        SimplificationOutcome::Simplified {
            text: "Salut.".into(),
        },
    ));
    store.append(SimplifiedSegment::new(
        FinalizedSegment::with_language("Au revoir.", Some("fr".into()), 4.0, 5.0),
        SimplificationOutcome::VerbatimFallback {
            reason: FallbackReason::Timeout,
        },
    ));
    store.append(SimplifiedSegment::new(
        FinalizedSegment::with_language("Okay.", Some("en".into()), 6.0, 7.0),
        SimplificationOutcome::Skipped {
            reason: SkipReason::OffTargetLanguage,
        },
    ));
    let csv = session_csv(&store, None, false);
    assert!(csv.starts_with(root["csv_header"].as_str().unwrap()));
    let markdown_text = markdown(&store);
    for needle in strings(&root, "markdown_required") {
        assert!(markdown_text.contains(&needle), "missing {needle}");
    }
    let plain = plain_text(&store);
    for needle in strings(&root, "plain_required") {
        assert!(plain.contains(&needle), "missing {needle}");
    }
    let reasons = &root["fallback_reasons"];
    assert_eq!(
        reasons["timeout"].as_str().unwrap(),
        FallbackReason::Timeout.as_str()
    );
    assert_eq!(
        reasons["off_target"].as_str().unwrap(),
        SkipReason::OffTargetLanguage.as_str()
    );
}

const EASYLANGUAGE_SENTENCE: &str = "This is a test sentence.";

fn easylanguage_goldens() -> Vec<Value> {
    Language::ALL
        .iter()
        .map(|language| {
            let pack = FrameworkPack::for_language(*language).unwrap();
            serde_json::json!({
                "locale": language.locale(),
                "prompt": render_easylanguage_prompt(&pack, EASYLANGUAGE_SENTENCE),
            })
        })
        .collect()
}

/// Set `LL_WRITE_GOLDENS=1` once to (re)write the file; otherwise compare.
#[test]
fn easylanguage_prompt_matches_goldens() {
    let rendered = easylanguage_goldens();
    assert_eq!(rendered.len(), 29);
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../tests/golden/easylanguage_prompt_goldens.json");
    if std::env::var_os("LL_WRITE_GOLDENS").is_some() {
        let json = serde_json::to_string_pretty(&Value::Array(rendered.clone())).unwrap();
        std::fs::write(&path, json + "\n").unwrap();
    }
    let expected = golden("easylanguage_prompt_goldens.json");
    assert_eq!(Value::Array(rendered), expected);
}

#[test]
fn easylanguage_prompt_has_no_context() {
    for language in Language::ALL {
        let pack = FrameworkPack::for_language(language).unwrap();
        let sentence = "Le conseil municipal se réunit mardi à 18 heures.";
        let prompt = render_easylanguage_prompt(&pack, sentence);
        assert_eq!(prompt.matches(sentence).count(), 1, "{language:?}");
        for shot in &pack.few_shots {
            assert!(
                !prompt.contains(&shot.source),
                "{language:?} few-shot leaked"
            );
            assert!(
                !prompt.contains(&shot.simplified),
                "{language:?} few-shot leaked"
            );
        }
        assert_eq!(prompt.matches("/no_think").count(), 1, "{language:?}");
        assert_eq!(prompt.matches(EMPTY_THINK_BLOCK).count(), 1, "{language:?}");
        assert!(prompt.ends_with("<|im_start|>assistant\n<think>\n\n</think>\n\n"));
        assert!(prompt.contains(&format!("{} /no_think<|im_end|>", pack.system_prompt)));
    }
}

#[test]
fn easylanguage_prompt_matches_the_model_card_template_literally() {
    // The template block from the model card, with the placeholders filled.
    let pack = FrameworkPack::for_language(Language::FrFr).unwrap();
    let template = "<|im_start|>system\n{SYSTEM_PROMPT}<|im_end|>\n<|im_start|>user\nOriginal: {SENTENCE}\nRewritten:<|im_end|>\n<|im_start|>assistant\n<think>\n\n</think>\n\n";
    let expected = template
        .replace(
            "{SYSTEM_PROMPT}",
            &format!("{} /no_think", pack.system_prompt),
        )
        .replace("{SENTENCE}", "Bonjour.");
    assert_eq!(render_easylanguage_prompt(&pack, "Bonjour."), expected);
}
