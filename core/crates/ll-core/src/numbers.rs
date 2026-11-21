//! Number and named-entity preservation check.
//!
//! Ported from `Sources/LiveLinguistKit/Validation/NumberGuard.swift`.
//! Values are compared, not surface forms: "forty eight" -> "48" is legitimate.

use crate::fidelity::matches;
use crate::text_analysis::{char_len, sentences, tokens, trim_token};
use crate::validation::Violation;
use std::collections::{BTreeSet, HashMap};

const MULTIPLIERS: [i64; 3] = [100, 1000, 1_000_000];
const CONNECTORS: [&str; 4] = ["et", "and", "y", "und"];

#[derive(Debug, Clone)]
pub struct NumberGuard {
    language: String,
}

impl NumberGuard {
    pub fn new(language: &str) -> Self {
        Self {
            language: language.to_string(),
        }
    }

    pub fn validate(&self, output: &str, source: &str, context: &[String]) -> Vec<Violation> {
        // Without a number lexicon we cannot distinguish a legitimate
        // word-to-digit rewrite ("fire uger" -> "4 uger") from an invention.
        if number_words(&self.language).is_empty() {
            return Vec::new();
        }
        let mut details = Vec::new();
        let anchors = format!("{} {}", source, context.join(" "));
        let source_values = number_values(&anchors, &self.language);
        let output_values = number_values(output, &self.language);
        let invented = output_values
            .difference(&source_values)
            .copied()
            .collect::<BTreeSet<_>>();
        if !invented.is_empty() {
            let listed = invented
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            details.push(format!(
                "the rewrite changed or invented numbers ({listed})"
            ));
        }
        if let Some(swapped) = self.swapped_entity(output, source, context) {
            details.push(format!("the rewrite replaced the name '{swapped}'"));
        }
        if details.is_empty() {
            return Vec::new();
        }
        vec![Violation::hard(
            "numbers_changed",
            format!(
                "{} — keep names and numbers exactly as in the sentence",
                details.join("; ")
            ),
        )]
    }

    fn swapped_entity(&self, output: &str, source: &str, context: &[String]) -> Option<String> {
        // The capitalized-word heuristic is only validated for the Latin
        // languages with a number lexicon. German capitalizes every noun;
        // other locales (Danish, Japanese, ...) are left to fidelity/language
        // checks rather than risking false entity flags.
        if !matches!(self.language.as_str(), "en" | "fr" | "es") {
            return None;
        }
        let source_entities = mid_sentence_capitalized_words(source);
        let output_entities = mid_sentence_capitalized_words(output);
        if source_entities.is_empty() || output_entities.is_empty() {
            return None;
        }
        let mut output_anchors = tokens(source);
        for segment in context {
            output_anchors.extend(tokens(segment));
        }
        let has_introduced = output_entities.iter().any(|entity| {
            !output_anchors
                .iter()
                .any(|anchor| matches(&entity.to_lowercase(), anchor))
        });
        if !has_introduced {
            return None;
        }
        let source_anchors = tokens(output);
        source_entities
            .iter()
            .find(|entity| {
                !source_anchors
                    .iter()
                    .any(|anchor| matches(&entity.to_lowercase(), anchor))
            })
            .cloned()
    }
}

/// All number values readable from text: digits, number words, and simple
/// compositions.
pub fn number_values(text: &str, language: &str) -> BTreeSet<i64> {
    let atoms = number_atoms(text, language);
    let mut values = atoms.iter().copied().collect::<BTreeSet<_>>();
    for pair in atoms.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let a_is_tens = (20..=90).contains(&a) && a % 10 == 0;
        let b_is_tens = (20..=90).contains(&b) && b % 10 == 0;
        if a_is_tens && (1..=19).contains(&b) {
            values.insert(a + b);
        }
        if b_is_tens && (1..=9).contains(&a) {
            values.insert(a + b);
        }
        if MULTIPLIERS.contains(&b) && (1..=999).contains(&a) {
            values.insert(a * b);
        }
    }
    values
}

fn number_atoms(text: &str, language: &str) -> Vec<i64> {
    let lexicon = number_words(language);
    let mut atoms = Vec::new();
    for token in tokens(text) {
        if CONNECTORS.contains(&token.as_str()) {
            continue;
        }
        if token.chars().any(|c| c.is_ascii_digit()) {
            atoms.extend(digit_runs(&token));
        } else if let Some(value) = lexicon.get(&token) {
            atoms.push(*value);
        } else if token.contains('-') {
            let parts = token.split('-').map(str::to_string).collect::<Vec<_>>();
            atoms.extend(greedy_parts(&parts, &lexicon));
        } else if language == "de" && token.contains("und") {
            let parts = token
                .split("und")
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>();
            let values = parts
                .iter()
                .map(|part| {
                    lexicon
                        .get(*part)
                        .copied()
                        .or_else(|| (*part == "ein").then_some(1))
                })
                .collect::<Option<Vec<_>>>();
            if let Some(values) = values {
                if parts.len() > 1 {
                    atoms.extend(values);
                }
            }
        }
    }
    atoms
}

fn greedy_parts(parts: &[String], lexicon: &HashMap<String, i64>) -> Vec<i64> {
    let mut values = Vec::new();
    let mut index = 0usize;
    while index < parts.len() {
        let mut matched = false;
        for end in ((index + 1)..=parts.len()).rev() {
            let joined = parts[index..end].join("-");
            if let Some(value) = lexicon.get(&joined) {
                values.push(*value);
                index = end;
                matched = true;
                break;
            }
        }
        if !matched {
            index += 1;
        }
    }
    values
}

fn digit_runs(token: &str) -> Vec<i64> {
    token
        .split(|c: char| !c.is_ascii_digit())
        .filter(|part| !part.is_empty() && part.len() <= 12)
        .filter_map(|part| part.parse::<i64>().ok())
        .collect()
}

fn number_words(language: &str) -> HashMap<String, i64> {
    let words: &[(&str, i64)] = match language {
        "en" => &[
            ("two", 2),
            ("three", 3),
            ("four", 4),
            ("five", 5),
            ("six", 6),
            ("seven", 7),
            ("eight", 8),
            ("nine", 9),
            ("ten", 10),
            ("eleven", 11),
            ("twelve", 12),
            ("thirteen", 13),
            ("fourteen", 14),
            ("fifteen", 15),
            ("sixteen", 16),
            ("seventeen", 17),
            ("eighteen", 18),
            ("nineteen", 19),
            ("twenty", 20),
            ("thirty", 30),
            ("forty", 40),
            ("fifty", 50),
            ("sixty", 60),
            ("seventy", 70),
            ("eighty", 80),
            ("ninety", 90),
            ("hundred", 100),
            ("thousand", 1000),
            ("million", 1_000_000),
        ],
        "fr" => &[
            ("deux", 2),
            ("trois", 3),
            ("quatre", 4),
            ("cinq", 5),
            ("six", 6),
            ("sept", 7),
            ("huit", 8),
            ("neuf", 9),
            ("dix", 10),
            ("onze", 11),
            ("douze", 12),
            ("treize", 13),
            ("quatorze", 14),
            ("quinze", 15),
            ("seize", 16),
            ("vingt", 20),
            ("trente", 30),
            ("quarante", 40),
            ("cinquante", 50),
            ("soixante", 60),
            ("soixante-dix", 70),
            ("quatre-vingt", 80),
            ("quatre-vingts", 80),
            ("quatre-vingt-dix", 90),
            ("cent", 100),
            ("cents", 100),
            ("mille", 1000),
            ("million", 1_000_000),
            ("millions", 1_000_000),
        ],
        "de" => &[
            ("zwei", 2),
            ("drei", 3),
            ("vier", 4),
            ("fünf", 5),
            ("sechs", 6),
            ("sieben", 7),
            ("acht", 8),
            ("neun", 9),
            ("zehn", 10),
            ("elf", 11),
            ("zwölf", 12),
            ("dreizehn", 13),
            ("vierzehn", 14),
            ("fünfzehn", 15),
            ("sechzehn", 16),
            ("siebzehn", 17),
            ("achtzehn", 18),
            ("neunzehn", 19),
            ("zwanzig", 20),
            ("dreißig", 30),
            ("dreissig", 30),
            ("vierzig", 40),
            ("fünfzig", 50),
            ("sechzig", 60),
            ("siebzig", 70),
            ("achtzig", 80),
            ("neunzig", 90),
            ("hundert", 100),
            ("tausend", 1000),
            ("million", 1_000_000),
            ("millionen", 1_000_000),
        ],
        "es" => &[
            ("dos", 2),
            ("tres", 3),
            ("cuatro", 4),
            ("cinco", 5),
            ("seis", 6),
            ("siete", 7),
            ("ocho", 8),
            ("nueve", 9),
            ("diez", 10),
            ("once", 11),
            ("doce", 12),
            ("trece", 13),
            ("catorce", 14),
            ("quince", 15),
            ("dieciséis", 16),
            ("dieciseis", 16),
            ("diecisiete", 17),
            ("dieciocho", 18),
            ("diecinueve", 19),
            ("veinte", 20),
            ("veintiuno", 21),
            ("veintidós", 22),
            ("veintidos", 22),
            ("veintitrés", 23),
            ("veintitres", 23),
            ("veinticuatro", 24),
            ("veinticinco", 25),
            ("veintiséis", 26),
            ("veintiseis", 26),
            ("veintisiete", 27),
            ("veintiocho", 28),
            ("veintinueve", 29),
            ("treinta", 30),
            ("cuarenta", 40),
            ("cincuenta", 50),
            ("sesenta", 60),
            ("setenta", 70),
            ("ochenta", 80),
            ("noventa", 90),
            ("cien", 100),
            ("ciento", 100),
            ("mil", 1000),
            ("millón", 1_000_000),
            ("millones", 1_000_000),
        ],
        _ => &[],
    };
    words
        .iter()
        .map(|(word, value)| ((*word).to_string(), *value))
        .collect()
}

fn mid_sentence_capitalized_words(text: &str) -> Vec<String> {
    const PRONOUN_FORMS: [&str; 5] = ["i", "i'm", "i've", "i'll", "i'd"];
    let mut entities = Vec::new();
    for sentence in sentences(text) {
        let words = sentence
            .split_whitespace()
            .map(trim_token)
            .filter(|word| !word.is_empty())
            .collect::<Vec<_>>();
        for word in words.iter().skip(1) {
            let Some(first) = word.chars().next() else {
                continue;
            };
            if char_len(word) < 3
                || !first.is_uppercase()
                || word.chars().any(|c| c.is_numeric())
                || PRONOUN_FORMS.contains(&word.to_lowercase().as_str())
            {
                continue;
            }
            entities.push((*word).to_string());
        }
    }
    entities
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guard(language: &str) -> NumberGuard {
        NumberGuard::new(language)
    }

    #[test]
    fn changed_number_word_is_flagged() {
        let violations = guard("en").validate(
            "The budget is twenty-eight K this year.",
            "He said the budget is forty eight K for the year.",
            &[],
        );
        let flag = violations
            .iter()
            .find(|v| v.rule == "numbers_changed")
            .expect("numbers");
        assert!(flag.detail.contains("28"));
    }

    #[test]
    fn invented_number_is_flagged() {
        let violations = guard("en").validate(
            "The meeting starts at 12.",
            "The meeting starts around noon today.",
            &[],
        );
        assert!(violations.iter().any(|v| v.rule == "numbers_changed"));
    }

    #[test]
    fn value_rewrites_pass() {
        assert!(guard("en")
            .validate(
                "The budget is 48 K.",
                "He said the budget is forty eight K for the year.",
                &[],
            )
            .is_empty());
        assert!(guard("de")
            .validate(
                "Das Projekt kostet 48 Euro.",
                "Das Projekt kostet ungefähr achtundvierzig Euro insgesamt.",
                &[],
            )
            .is_empty());
        assert!(guard("fr")
            .validate(
                "75 personnes sont venues.",
                "Il y avait soixante-quinze personnes à la fête hier soir.",
                &[],
            )
            .is_empty());
        assert!(guard("es")
            .validate(
                "Hay 35 personas en la sala.",
                "Hay treinta y cinco personas en la sala ahora mismo.",
                &[],
            )
            .is_empty());
    }

    #[test]
    fn dropped_numbers_pass() {
        assert!(guard("en")
            .validate(
                "The deadline moved.",
                "The deadline moved by 14 days to March 15.",
                &[],
            )
            .is_empty());
    }

    #[test]
    fn context_anchors_numbers() {
        assert!(guard("en")
            .validate(
                "The price is 48 K.",
                "And that is the final price they offered us.",
                &["The offer was forty eight K.".to_string()],
            )
            .is_empty());
    }

    #[test]
    fn swapped_proper_noun_is_flagged() {
        let violations = guard("en").validate(
            "We take the train to London tomorrow.",
            "We are taking the train to Paris tomorrow morning.",
            &[],
        );
        let flag = violations
            .iter()
            .find(|v| v.rule == "numbers_changed")
            .expect("entity");
        assert!(flag.detail.contains("Paris"));
    }

    #[test]
    fn dropped_entity_alone_and_acronym_expansion_pass() {
        assert!(guard("en")
            .validate(
                "We take the train tomorrow.",
                "We are taking the train to Paris tomorrow morning.",
                &[],
            )
            .is_empty());
        assert!(guard("en")
            .validate(
                "The United Nations meeting starts soon.",
                "The UN meeting starts in a few minutes.",
                &[],
            )
            .is_empty());
    }

    #[test]
    fn german_skips_entity_check() {
        assert!(guard("de")
            .validate(
                "Wir gehen morgen ins Kino in Hamburg.",
                "Wir besuchen morgen das Museum in Berlin.",
                &[],
            )
            .is_empty());
    }

    #[test]
    fn english_pronoun_contractions_are_not_entities() {
        assert!(guard("en")
            .validate(
                "Tomorrow I'm going to the market.",
                "So yeah tomorrow I'm heading over to the market.",
                &[],
            )
            .is_empty());
    }

    #[test]
    fn gold_pairs_are_never_flagged() {
        use crate::language::{Framework, Language};
        use crate::pack::FrameworkPack;
        for framework in Framework::curated() {
            let pack = FrameworkPack::load_framework(framework).unwrap();
            for shot in &pack.few_shots {
                assert!(
                    guard(&pack.language)
                        .validate(&shot.simplified, &shot.source, &[])
                        .is_empty(),
                    "{framework:?}: {}",
                    shot.source
                );
            }
        }
        // Also cover every locale pack.
        for language in Language::ALL {
            let pack = FrameworkPack::for_language(language).unwrap();
            for shot in &pack.few_shots {
                assert!(
                    guard(&pack.language)
                        .validate(&shot.simplified, &shot.source, &[])
                        .is_empty(),
                    "{}: {}",
                    language.locale(),
                    shot.source
                );
            }
        }
    }
}
