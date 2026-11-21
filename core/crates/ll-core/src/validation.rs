//! Deterministic validation layer: sentence length, same-language guard,
//! fidelity, numbers, and per-framework register checks.
//!
//! Ported from `Sources/LiveLinguistKit/Validation/*.swift`. Validators are
//! confidence signals, not gates: callers display imperfect output with a flag.

use crate::fidelity::FidelityGuard;
use crate::language::Framework;
use crate::language_guard::LanguageGuard;
use crate::numbers::NumberGuard;
use crate::pack::FrameworkPack;
use crate::rule_checkers::LanguageRuleCheckers;
use crate::text_analysis::{sentences, tokens, word_count};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Triggers one constrained retry, then the fallback ladder.
    Hard,
    /// Displayed anyway, logged for eval.
    Soft,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Violation {
    /// Stable rule identifier, e.g. `sentence_length`, `passive_voice`.
    pub rule: String,
    pub severity: Severity,
    /// Human-readable detail, used to name the violation in the retry prompt.
    pub detail: String,
}

impl Violation {
    pub fn hard(rule: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            rule: rule.into(),
            severity: Severity::Hard,
            detail: detail.into(),
        }
    }

    pub fn soft(rule: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            rule: rule.into(),
            severity: Severity::Soft,
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidatorSuite {
    pub framework: Framework,
    thresholds: crate::pack::Thresholds,
    language: String,
    language_guard: LanguageGuard,
    fidelity_guard: FidelityGuard,
    number_guard: NumberGuard,
}

impl ValidatorSuite {
    pub fn new(pack: &FrameworkPack) -> Self {
        let base = pack.base_code();
        Self {
            framework: pack.framework,
            thresholds: pack.thresholds,
            language: base.clone(),
            language_guard: LanguageGuard::new(&base, pack.stopwords.clone().unwrap_or_default()),
            fidelity_guard: FidelityGuard::new(pack),
            number_guard: NumberGuard::new(&base),
        }
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    /// Run every check. `context` is the rolling previous-segment window the
    /// model saw; fidelity/numbers use it as an additional anchor pool.
    pub fn validate(&self, output: &str, source: &str, context: &[String]) -> Vec<Violation> {
        let mut violations = self.sentence_length(output);
        violations.extend(self.language_guard.validate(output));
        violations.extend(self.fidelity_guard.validate(output, source, context));
        violations.extend(self.number_guard.validate(output, source, context));

        let source_tokens = tokens(source).into_iter().collect();
        match self.framework {
            Framework::Falc => {
                violations.extend(LanguageRuleCheckers::french(output, &source_tokens))
            }
            Framework::LeichteSprache => {
                violations.extend(LanguageRuleCheckers::german(output, &source_tokens));
            }
            Framework::EasyEnglish => {
                violations.extend(LanguageRuleCheckers::english(output, &source_tokens));
            }
            Framework::LecturaFacil => {
                violations.extend(LanguageRuleCheckers::spanish(output, &source_tokens));
            }
            Framework::EasyToRead => {}
        }
        violations
    }

    fn sentence_length(&self, output: &str) -> Vec<Violation> {
        let mut violations = Vec::new();
        for sentence in sentences(output) {
            let count = word_count(&sentence);
            if count > self.thresholds.max_words_hard {
                violations.push(Violation::hard(
                    "sentence_length",
                    format!(
                        "{count} words (max {}) — split into shorter sentences",
                        self.thresholds.max_words_hard
                    ),
                ));
            } else if count > self.thresholds.max_words_soft {
                violations.push(Violation::soft(
                    "sentence_length",
                    format!("{count} words (target {})", self.thresholds.max_words_soft),
                ));
            }
        }
        violations
    }
}

/// Convenience for tests and callers that only have a framework.
pub fn validate_with_pack(
    pack: &FrameworkPack,
    output: &str,
    source: &str,
    context: &[String],
) -> Vec<Violation> {
    ValidatorSuite::new(pack).validate(output, source, context)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn suite(framework: Framework) -> ValidatorSuite {
        ValidatorSuite::new(&FrameworkPack::load_framework(framework).unwrap())
    }

    #[test]
    fn short_sentence_passes() {
        let violations =
            suite(Framework::Falc).validate("Il fait beau.", "quel temps magnifique", &[]);
        assert!(!violations.iter().any(|v| v.rule == "sentence_length"));
    }

    #[test]
    fn over_hard_cap_is_hard_violation() {
        let long = "Je pense que nous devrions aller voir le nouveau film ensemble demain soir vers huit heures.";
        let violations = suite(Framework::Falc).validate(long, long, &[]);
        let length = violations
            .iter()
            .find(|v| v.rule == "sentence_length")
            .expect("length violation");
        assert_eq!(length.severity, Severity::Hard);
    }

    #[test]
    fn between_soft_and_hard_is_soft() {
        let medium =
            "Nous allons voir le nouveau film ensemble demain soir vers huit heures trente.";
        let violations = suite(Framework::Falc).validate(medium, medium, &[]);
        let length = violations
            .iter()
            .find(|v| v.rule == "sentence_length")
            .expect("length violation");
        assert_eq!(length.severity, Severity::Soft);
    }

    #[test]
    fn each_sentence_is_measured_separately() {
        let two = "Il fait beau. On va au parc.";
        let violations = suite(Framework::Falc).validate(two, two, &[]);
        assert!(!violations.iter().any(|v| v.rule == "sentence_length"));
    }

    #[test]
    fn matching_language_passes() {
        let violations = suite(Framework::Falc).validate(
            "Les discussions se sont bien passées et je suis contente.",
            "source",
            &[],
        );
        assert!(!violations.iter().any(|v| v.rule == "wrong_language"));
    }

    #[test]
    fn english_output_in_french_session_is_hard() {
        let violations = suite(Framework::Falc).validate(
            "The discussions went well and I am very happy with them.",
            "Les discussions se sont bien passées.",
            &[],
        );
        let flag = violations
            .iter()
            .find(|v| v.rule == "wrong_language")
            .expect("wrong language");
        assert_eq!(flag.severity, Severity::Hard);
    }

    #[test]
    fn very_short_output_is_not_flagged() {
        let violations = suite(Framework::Falc).validate("D'accord.", "ok d'accord", &[]);
        assert!(!violations.iter().any(|v| v.rule == "wrong_language"));
    }

    #[test]
    fn spanish_guard_catches_english() {
        let violations = suite(Framework::LecturaFacil).validate(
            "We are going to the park with the children today.",
            "Vamos al parque.",
            &[],
        );
        assert!(violations
            .iter()
            .any(|v| v.rule == "wrong_language" && v.severity == Severity::Hard));
    }

    #[test]
    fn text_analysis_module_is_reachable() {
        assert_eq!(crate::text_analysis::word_count("a b"), 2);
    }
}
