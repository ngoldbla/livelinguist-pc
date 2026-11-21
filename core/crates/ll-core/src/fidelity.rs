//! Faithfulness check: catch outputs unrelated to the source sentence,
//! including few-shot example parroting.
//!
//! Ported from `Sources/LiveLinguistKit/Validation/FidelityGuard.swift`.

use crate::language_guard::stopwords_for;
use crate::pack::FrameworkPack;
use crate::text_analysis::{char_len, char_prefix, tokens};
use crate::validation::Violation;

const MIN_ANCHORED_COUNT: usize = 3;
const MIN_ANCHORED_RATIO: f64 = 0.3;
const STEM_LENGTH: usize = 6;
const CONTAINMENT_LENGTH: usize = 5;
const MIN_CONTENT_TOKENS: usize = 2;
const FRAGMENT_TOKENS: usize = 4;
const UNSPACED_FRAGMENT_CHARACTERS: usize = 8;

#[derive(Debug, Clone)]
struct FewShotEcho {
    normalized_answer: String,
    source_content_tokens: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FidelityGuard {
    language: String,
    few_shot_echoes: Vec<FewShotEcho>,
}

impl FidelityGuard {
    pub fn new(pack: &FrameworkPack) -> Self {
        let few_shot_echoes = pack
            .few_shots
            .iter()
            .map(|shot| FewShotEcho {
                normalized_answer: normalize(&shot.simplified),
                source_content_tokens: content_tokens(&shot.source, &pack.language),
            })
            .collect();
        Self {
            language: pack.language.clone(),
            few_shot_echoes,
        }
    }

    pub fn validate(&self, output: &str, source: &str, context: &[String]) -> Vec<Violation> {
        let output_tokens = content_tokens(output, &self.language);
        let source_tokens = content_tokens(source, &self.language);
        if output_tokens.len() < MIN_CONTENT_TOKENS || source_tokens.len() < MIN_CONTENT_TOKENS {
            return Vec::new();
        }
        let mut anchors = source_tokens.clone();
        for segment in context {
            anchors.extend(content_tokens(segment, &self.language));
        }
        if passes_anchoring(&output_tokens, &anchors) {
            return Vec::new();
        }

        let normalized = normalize(output);
        let detail = if let Some(echo) = self
            .few_shot_echoes
            .iter()
            .find(|echo| echo.normalized_answer == normalized)
        {
            // A gold rewrite can share zero words with its source, so only the
            // source distinguishes "answered example k" from "parroted example k".
            if passes_anchoring(&source_tokens, &echo.source_content_tokens) {
                return Vec::new();
            }
            "you repeated an example answer instead of rewriting the sentence — rewrite the actual sentence, keeping its key words"
        } else {
            "the rewrite does not keep any key words from the sentence — rewrite it, keeping its names, numbers, and key words"
        };
        vec![Violation::hard("unfaithful", detail)]
    }

    /// A source with (almost) nothing in it has nothing to simplify.
    pub fn is_fragment(source: &str, language: &str) -> bool {
        let unspaced = source.chars().filter(|c| is_unspaced_script(*c)).count();
        if unspaced >= 2 {
            return unspaced < UNSPACED_FRAGMENT_CHARACTERS;
        }
        tokens(source).len() < FRAGMENT_TOKENS
            || content_tokens(source, language).len() < MIN_CONTENT_TOKENS
    }
}

/// Han, Hiragana, Katakana — scripts written without word spaces.
fn is_unspaced_script(character: char) -> bool {
    matches!(
        character as u32,
        0x3040..=0x30FF | 0x3400..=0x4DBF | 0x4E00..=0x9FFF
    )
}

/// Lowercased tokens minus stopwords, keeping tokens with a digit or length >= 3.
pub fn content_tokens(text: &str, language: &str) -> Vec<String> {
    let stopwords = stopwords_for(language);
    tokens(text)
        .into_iter()
        .filter(|token| {
            !stopwords.contains(token)
                && (char_len(token) >= 3 || token.chars().any(|c| c.is_numeric()))
        })
        .collect()
}

fn normalize(text: &str) -> String {
    tokens(text).join(" ")
}

fn passes_anchoring(output_tokens: &[String], anchors: &[String]) -> bool {
    if output_tokens.is_empty() {
        return false;
    }
    let anchored = output_tokens
        .iter()
        .filter(|token| anchors.iter().any(|anchor| matches(token, anchor)))
        .count();
    anchored >= MIN_ANCHORED_COUNT
        || anchored as f64 / output_tokens.len() as f64 >= MIN_ANCHORED_RATIO
}

/// Shared anchor matching (exact / stem / compound containment).
pub fn matches(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    let stem_a = char_prefix(a, STEM_LENGTH);
    let stem_b = char_prefix(b, STEM_LENGTH);
    let prefix_match = if stem_a.chars().count() <= stem_b.chars().count() {
        stem_b.starts_with(&stem_a)
    } else {
        stem_a.starts_with(&stem_b)
    };
    if prefix_match {
        return true;
    }
    if char_len(a) >= CONTAINMENT_LENGTH && b.contains(a) {
        return true;
    }
    if char_len(b) >= CONTAINMENT_LENGTH && a.contains(b) {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::Framework;
    use crate::pack::{FewShot, Thresholds};

    fn guard(framework: Framework) -> FidelityGuard {
        FidelityGuard::new(&FrameworkPack::load_framework(framework).unwrap())
    }

    fn bare_guard(language: &str) -> FidelityGuard {
        FidelityGuard::new(&FrameworkPack {
            framework: Framework::LeichteSprache,
            language: language.to_string(),
            display_name: String::new(),
            system_prompt: String::new(),
            retry_prompt_template: "{violations}".to_string(),
            thresholds: Thresholds {
                max_words_hard: 12,
                max_words_soft: 10,
            },
            few_shots: Vec::new(),
            base_lang: None,
            stopwords: None,
            length_unit: None,
        })
    }

    #[test]
    fn german_few_shot_echo_is_unfaithful() {
        let violations = guard(Framework::LeichteSprache).validate(
            "Wir machen das nächste Woche.",
            "Dafür erheben Sie tagtäglich Daten und entwickeln Empfehlungen für die Politik.",
            &[],
        );
        let flag = violations
            .iter()
            .find(|v| v.rule == "unfaithful")
            .expect("unfaithful");
        assert!(flag.detail.contains("example answer"));
    }

    #[test]
    fn english_few_shot_echo_is_unfaithful() {
        let violations = guard(Framework::EasyEnglish).validate(
            "The deadline moved to Friday.",
            "We collect data every morning and develop recommendations from it.",
            &[],
        );
        let flag = violations
            .iter()
            .find(|v| v.rule == "unfaithful")
            .expect("unfaithful");
        assert!(flag.detail.contains("example answer"));
    }

    #[test]
    fn unanchored_non_echo_gets_generic_detail() {
        let violations = guard(Framework::EasyEnglish).validate(
            "The deadline is on Friday.",
            "We collect data every morning and develop recommendations from it.",
            &[],
        );
        let flag = violations
            .iter()
            .find(|v| v.rule == "unfaithful")
            .expect("unfaithful");
        assert!(!flag.detail.contains("example answer"));
        assert!(flag.detail.contains("key words"));
    }

    #[test]
    fn gold_pairs_are_never_flagged() {
        for framework in Framework::curated() {
            let pack = FrameworkPack::load_framework(framework).unwrap();
            let guard = FidelityGuard::new(&pack);
            for shot in &pack.few_shots {
                let violations = guard.validate(&shot.simplified, &shot.source, &[]);
                assert!(violations.is_empty(), "{framework:?}: {}", shot.source);
            }
        }
    }

    #[test]
    fn german_gold_pairs_pass_on_anchoring_alone() {
        let pack = FrameworkPack::load_framework(Framework::LeichteSprache).unwrap();
        let guard = bare_guard("de");
        for shot in &pack.few_shots {
            assert!(
                guard
                    .validate(&shot.simplified, &shot.source, &[])
                    .is_empty(),
                "anchoring false positive: {}",
                shot.source
            );
        }
    }

    #[test]
    fn stems_anchor_inflections() {
        let violations = bare_guard("de").validate(
            "Der Vorstand hat das entschieden. Die Mitarbeiter finden es gut.",
            "Die Entscheidung des Vorstands wurde von allen Mitarbeitern begrüßt.",
            &[],
        );
        assert!(violations.is_empty(), "{violations:?}");
    }

    #[test]
    fn compound_containment_anchors_split_compounds() {
        let violations = bare_guard("de").validate(
            "Die Öffentlichkeit redet über die Ergebnisse.",
            "Die Fachöffentlichkeit diskutiert die neuen Ergebnisse heute.",
            &[],
        );
        assert!(violations.is_empty(), "{violations:?}");
    }

    #[test]
    fn numbers_anchor_exactly() {
        let violations = bare_guard("de").validate(
            "Das Treffen ist am 15. März um 14 Uhr.",
            "Wir verschieben das Treffen auf den 15. März um 14 Uhr.",
            &[],
        );
        assert!(violations.is_empty(), "{violations:?}");
    }

    #[test]
    fn context_rescues_context_referring_output() {
        let source = "Alle Mitarbeiter kommen mit ihren Familien dorthin.";
        let output = "Das Sommerfest ist im Stadtpark.";
        let guard = bare_guard("de");
        assert!(!guard.validate(output, source, &[]).is_empty());
        assert!(guard
            .validate(
                output,
                source,
                &["Wir planen das große Sommerfest im Stadtpark.".to_string()]
            )
            .is_empty());
    }

    #[test]
    fn short_output_and_source_are_not_judged() {
        assert!(bare_guard("de")
            .validate("Gut.", "Die Lage ist wirklich kompliziert geworden.", &[])
            .is_empty());
        assert!(bare_guard("de")
            .validate(
                "Das Wetter ist heute wirklich schön draußen.",
                "Na gut.",
                &[],
            )
            .is_empty());
    }

    #[test]
    fn anchored_echo_is_not_flagged() {
        let violations = guard(Framework::LeichteSprache).validate(
            "Wir machen das nächste Woche.",
            "ähm also wir haben dann halt beschlossen, das Ganze auf nächste Woche zu verschieben",
            &[],
        );
        assert!(violations.is_empty(), "{violations:?}");
    }

    #[test]
    fn fragment_detection() {
        assert!(FidelityGuard::is_fragment("Politik.", "de"));
        assert!(!FidelityGuard::is_fragment(
            "Die Forscher veröffentlichen ihre Daten regelmäßig.",
            "de"
        ));
        assert!(!FidelityGuard::is_fragment(
            "台風の接近に伴い、不要不急の外出は控えてください。",
            "ja"
        ));
        assert!(!FidelityGuard::is_fragment(
            "由于天气原因，今天的会议已经取消了。",
            "zh"
        ));
        assert!(FidelityGuard::is_fragment("政治。", "ja"));
        assert!(FidelityGuard::is_fragment("Das ist es auch schon.", "de"));
    }

    #[test]
    fn echo_detection_uses_pack_examples() {
        let guard = guard(Framework::LeichteSprache);
        let echo = FewShot {
            source: "x".into(),
            simplified: "Wir machen das nächste Woche.".into(),
        };
        assert_eq!(guard.few_shot_echoes.len(), 8);
        assert!(echo.simplified.contains("Wir"));
    }
}
