//! Fast same-language check (stopword dominance).
//!
//! Ported from `Sources/LiveLinguistKit/Validation/LanguageGuard.swift`.
//! A wrong-language output is a hard violation: retry once, then verbatim.

use crate::text_analysis::tokens;
use crate::validation::Violation;
use std::collections::{BTreeMap, HashSet};

fn builtin_stopwords() -> BTreeMap<String, HashSet<String>> {
    let mut map = BTreeMap::new();
    map.insert(
        "fr".to_string(),
        [
            "le", "la", "les", "un", "une", "des", "de", "du", "et", "est", "sont", "je", "tu",
            "il", "elle", "on", "nous", "vous", "ils", "elles", "ne", "pas", "que", "qui", "dans",
            "sur", "avec", "pour", "ce", "cette", "mais", "où", "son", "sa", "ses", "au", "aux",
            "se", "suis", "très", "été", "être", "fait", "plus", "bien", "tout", "quand",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
    );
    map.insert(
        "en".to_string(),
        [
            "the", "a", "an", "and", "is", "are", "was", "were", "i", "you", "he", "she", "we",
            "they", "them", "not", "that", "which", "in", "on", "with", "for", "this", "but", "or",
            "his", "her", "its", "to", "of", "at", "by", "it", "am", "be", "have", "has", "will",
            "do", "did", "going", "very", "today",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
    );
    map.insert(
        "de".to_string(),
        [
            "der", "die", "das", "ein", "eine", "und", "ist", "sind", "ich", "du", "er", "wir",
            "ihr", "nicht", "dass", "auf", "mit", "für", "aber", "oder", "sein", "ihre", "zu",
            "von", "bei", "es", "den", "dem", "wird", "werden", "haben", "hat", "sehr", "auch",
            "noch", "schon", "wie", "was", "kann", "morgen",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
    );
    map.insert(
        "es".to_string(),
        [
            "el", "los", "las", "un", "una", "unos", "unas", "y", "es", "son", "yo", "él", "ella",
            "nosotros", "ellos", "no", "que", "quien", "sobre", "con", "para", "este", "esta",
            "pero", "o", "su", "sus", "al", "lo", "me", "te", "mi", "muy", "hay", "está", "más",
            "cuando", "hasta", "porque",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
    );
    map
}

/// Stopwords for one language, if the built-in table has them.
pub fn stopwords_for(language: &str) -> HashSet<String> {
    builtin_stopwords().remove(language).unwrap_or_default()
}

#[derive(Debug, Clone)]
pub struct LanguageGuard {
    expected_language: String,
    stopword_map: BTreeMap<String, HashSet<String>>,
    minimum_tokens: usize,
}

impl LanguageGuard {
    pub fn new(expected_language: &str, extra_stopwords: Vec<String>) -> Self {
        let mut map = builtin_stopwords();
        if !extra_stopwords.is_empty() {
            let entry = map.entry(expected_language.to_string()).or_default();
            for word in extra_stopwords {
                entry.insert(word.to_lowercase());
            }
        }
        Self {
            expected_language: expected_language.to_string(),
            stopword_map: map,
            minimum_tokens: 4,
        }
    }

    pub fn validate(&self, output: &str) -> Vec<Violation> {
        let words = tokens(output);
        if words.len() < self.minimum_tokens {
            return Vec::new();
        }
        let mut best_language: Option<&str> = None;
        let mut best_score = 0usize;
        let mut expected_score = 0usize;
        for (language, stopwords) in &self.stopword_map {
            let score = words
                .iter()
                .filter(|word| stopwords.contains(*word))
                .count();
            if language == &self.expected_language {
                expected_score = score;
            }
            if score > best_score {
                best_score = score;
                best_language = Some(language.as_str());
            }
        }
        let Some(best_language) = best_language else {
            return Vec::new();
        };
        if best_language == self.expected_language || best_score <= expected_score || best_score < 2
        {
            return Vec::new();
        }
        vec![Violation::hard(
            "wrong_language",
            format!(
                "output looks like {best_language}, expected {} — rewrite in {}",
                self.expected_language, self.expected_language
            ),
        )]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guard(language: &str) -> LanguageGuard {
        LanguageGuard::new(language, Vec::new())
    }

    #[test]
    fn matching_language_passes() {
        let violations =
            guard("fr").validate("Les discussions se sont bien passées et je suis contente.");
        assert!(!violations.iter().any(|v| v.rule == "wrong_language"));
    }

    #[test]
    fn english_output_in_french_session_flags() {
        let violations =
            guard("fr").validate("The discussions went well and I am very happy with them.");
        assert!(violations.iter().any(|v| v.rule == "wrong_language"));
    }

    #[test]
    fn short_output_is_not_judged() {
        assert!(guard("fr").validate("D'accord.").is_empty());
    }

    #[test]
    fn extra_pack_stopwords_help_run_two_locales() {
        // Without pack stopwords, Italian has no built-in list; the pack's own
        // words make the expected language win.
        let extra = vec![
            "che".to_string(),
            "di".to_string(),
            "è".to_string(),
            "per".to_string(),
            "con".to_string(),
        ];
        let violations = LanguageGuard::new("it", extra)
            .validate("Questo è un testo che parla di cose semplici con parole chiare.");
        assert!(!violations.iter().any(|v| v.rule == "wrong_language"));
    }

    #[test]
    fn builtin_table_is_available() {
        assert!(stopwords_for("fr").contains("et"));
        assert!(stopwords_for("xx").is_empty());
    }
}
