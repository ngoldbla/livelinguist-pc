//! Per-language deterministic register checks (FALC, Leichte Sprache, Easy
//! English, Lectura Fácil). All are soft violations; hard severity is reserved
//! for length, wrong language, fidelity, and numbers.
//!
//! Ported from `Sources/LiveLinguistKit/Validation/LanguageRuleCheckers.swift`.
//! Tokens that also appear in the source are exempt.

use crate::text_analysis::{char_len, sentences, syllables, tokens};
use crate::validation::Violation;
use std::collections::HashSet;

pub struct LanguageRuleCheckers;

impl LanguageRuleCheckers {
    // MARK: French — FALC

    const FRENCH_SUBJUNCTIVE: &[&str] = &[
        "soit",
        "sois",
        "soient",
        "soyons",
        "soyez",
        "fasse",
        "fasses",
        "fassent",
        "fassions",
        "fassiez",
        "puisse",
        "puisses",
        "puissent",
        "puissions",
        "puissiez",
        "aille",
        "ailles",
        "aillent",
        "allions",
        "alliez",
        "ait",
        "aient",
        "aies",
        "ayons",
        "ayez",
        "sache",
        "saches",
        "sachent",
        "sachions",
        "sachiez",
        "veuille",
        "veuilles",
        "veuillent",
        "voulions",
        "vouliez",
        "vienne",
        "viennes",
        "viennent",
        "venions",
        "veniez",
        "prenne",
        "prennes",
        "prennent",
        "prenions",
        "preniez",
        "doive",
        "doives",
        "doivent",
        "devions",
        "deviez",
        "reçoive",
        "reçoives",
        "reçoivent",
    ];

    const FRENCH_ETRE: &[&str] = &[
        "suis", "es", "est", "sommes", "êtes", "sont", "était", "étais", "étaient", "étions",
        "étiez", "été", "être", "sera", "seras", "seront", "serons", "serez",
    ];

    pub fn french(output: &str, source_tokens: &HashSet<String>) -> Vec<Violation> {
        let words = tokens(output);
        let mut violations = Vec::new();
        for token in &words {
            if Self::FRENCH_SUBJUNCTIVE.contains(&token.as_str()) && !source_tokens.contains(token)
            {
                violations.push(Violation::soft(
                    "subjunctive",
                    format!("subjunctive form “{token}” — use the indicative"),
                ));
            }
        }
        let participle_suffixes = ["é", "ée", "és", "ées", "ie", "ies", "ue", "ues", "is", "us"];
        for (index, token) in words.iter().enumerate() {
            if !Self::FRENCH_ETRE.contains(&token.as_str()) {
                continue;
            }
            let Some(next) = words.get(index + 1) else {
                continue;
            };
            if participle_suffixes
                .iter()
                .any(|suffix| next.ends_with(suffix))
            {
                violations.push(Violation::soft(
                    "passive_voice",
                    format!("passive construction “{token} {next}” — use the active voice"),
                ));
            }
        }
        violations
    }

    // MARK: German — Leichte Sprache

    const GERMAN_KONJUNKTIV: &[&str] = &[
        "würde",
        "würdest",
        "würden",
        "würdet",
        "hätte",
        "hättest",
        "hätten",
        "hättet",
        "wäre",
        "wärst",
        "wären",
        "wärt",
        "sei",
        "seist",
        "seien",
        "könnte",
        "könntest",
        "könnten",
        "sollte",
        "solltest",
        "sollten",
        "müsste",
        "müsstest",
        "müssten",
        "dürfte",
        "dürften",
    ];

    const GERMAN_WERDEN: &[&str] = &[
        "wird", "werden", "wurde", "wurden", "worden", "wirst", "werdet",
    ];

    pub fn german(output: &str, source_tokens: &HashSet<String>) -> Vec<Violation> {
        let words = tokens(output);
        let mut violations = Vec::new();
        for token in &words {
            if Self::GERMAN_KONJUNKTIV.contains(&token.as_str()) && !source_tokens.contains(token) {
                violations.push(Violation::soft(
                    "konjunktiv",
                    format!("Konjunktiv form “{token}” — use the indicative"),
                ));
            }
        }
        for (index, token) in words.iter().enumerate() {
            if !Self::GERMAN_WERDEN.contains(&token.as_str()) {
                continue;
            }
            for next in words.iter().skip(index + 1).take(4) {
                if next.starts_with("ge") && (next.ends_with('t') || next.ends_with("en")) {
                    violations.push(Violation::soft(
                        "passive_voice",
                        format!("passive construction “{token} … {next}” — use the active voice"),
                    ));
                    break;
                }
            }
        }
        for (index, token) in words.iter().enumerate() {
            if token != "des" && token != "eines" {
                continue;
            }
            if let Some(next) = words.get(index + 1) {
                if next.ends_with('s') {
                    violations.push(Violation::soft(
                        "genitive",
                        format!("genitive “{token} {next}” — prefer “von” + dative"),
                    ));
                }
            }
        }
        for token in &words {
            if char_len(token) >= 18 && !source_tokens.contains(token) {
                violations.push(Violation::soft(
                    "compound_length",
                    format!(
                        "long compound “{token}” ({} characters) — split it",
                        char_len(token)
                    ),
                ));
            }
        }
        violations
    }

    // MARK: English — Easy English

    const ENGLISH_BE: &[&str] = &["is", "are", "was", "were", "been", "being", "be", "am"];
    const ENGLISH_IRREGULAR_PARTICIPLES: &[&str] = &[
        "done", "made", "given", "taken", "seen", "known", "found", "told", "shown", "written",
        "held", "brought", "kept", "left", "put", "set", "sent", "built", "spoken", "chosen",
        "broken", "paid", "sold", "lost",
    ];

    pub fn english(output: &str, _source_tokens: &HashSet<String>) -> Vec<Violation> {
        let words = tokens(output);
        let mut violations = Vec::new();
        for (index, token) in words.iter().enumerate() {
            if !Self::ENGLISH_BE.contains(&token.as_str()) {
                continue;
            }
            let Some(next) = words.get(index + 1) else {
                continue;
            };
            let looks_like_participle = next.ends_with("ed")
                || Self::ENGLISH_IRREGULAR_PARTICIPLES.contains(&next.as_str());
            if looks_like_participle {
                violations.push(Violation::soft(
                    "passive_voice",
                    format!("passive construction “{token} {next}” — use the active voice"),
                ));
            }
        }

        // Flesch-Kincaid grade <= 6 (soft gate).
        let sentence_count = sentences(output).len();
        if sentence_count > 0 && !words.is_empty() {
            let word_count = words.len() as f64;
            let syllable_count = words.iter().map(|word| syllables(word)).sum::<usize>() as f64;
            let grade = 0.39 * (word_count / sentence_count as f64)
                + 11.8 * (syllable_count / word_count)
                - 15.59;
            if grade > 6.0 {
                violations.push(Violation::soft(
                    "readability",
                    format!(
                        "Flesch-Kincaid grade {grade:.1} (target ≤ 6) — use shorter, simpler words"
                    ),
                ));
            }
        }
        violations
    }

    // MARK: Spanish — Lectura Fácil

    const SPANISH_SUBJUNCTIVE: &[&str] = &[
        "sea", "seas", "sean", "seamos", "seáis", "vaya", "vayas", "vayan", "vayamos", "haya",
        "hayas", "hayan", "hayamos", "pueda", "puedas", "puedan", "podamos", "tenga", "tengas",
        "tengan", "tengamos", "haga", "hagas", "hagan", "hagamos", "venga", "vengas", "vengan",
        "vengamos", "diga", "digas", "digan", "digamos", "quiera", "quieras", "quieran", "sepa",
        "sepas", "sepan", "esté", "estés", "estén",
    ];

    const SPANISH_SER: &[&str] = &[
        "es", "son", "fue", "fueron", "era", "eran", "será", "serán", "soy", "eres", "somos",
    ];

    pub fn spanish(output: &str, source_tokens: &HashSet<String>) -> Vec<Violation> {
        let words = tokens(output);
        let mut violations = Vec::new();
        for token in &words {
            if Self::SPANISH_SUBJUNCTIVE.contains(&token.as_str()) && !source_tokens.contains(token)
            {
                violations.push(Violation::soft(
                    "subjunctive",
                    format!("subjunctive form “{token}” — use the indicative"),
                ));
            }
        }
        let participle_suffixes = ["ado", "ada", "ados", "adas", "ido", "ida", "idos", "idas"];
        for (index, token) in words.iter().enumerate() {
            if !Self::SPANISH_SER.contains(&token.as_str()) {
                continue;
            }
            let Some(next) = words.get(index + 1) else {
                continue;
            };
            if participle_suffixes
                .iter()
                .any(|suffix| next.ends_with(suffix))
            {
                violations.push(Violation::soft(
                    "passive_voice",
                    format!("passive construction “{token} {next}” — use the active voice"),
                ));
            }
        }
        violations
    }
}

/// Shared helper used by the tests and by any caller that needs a source set.
pub fn source_token_set(text: &str) -> HashSet<String> {
    tokens(text).into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(text: &str) -> HashSet<String> {
        source_token_set(text)
    }

    #[test]
    fn french_subjunctive_is_soft() {
        let violations = LanguageRuleCheckers::french(
            "Il faut que tu fasses attention.",
            &source("fais attention"),
        );
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].severity, crate::validation::Severity::Soft);
    }

    #[test]
    fn french_source_retained_token_is_exempt() {
        let text = "Soit on part, soit on reste.";
        let violations = LanguageRuleCheckers::french(text, &source(text));
        assert!(!violations.iter().any(|v| v.rule == "subjunctive"));
    }

    #[test]
    fn french_passive_is_flagged() {
        let violations = LanguageRuleCheckers::french(
            "La réunion a été annulée par le directeur.",
            &source("le directeur a annulé la réunion"),
        );
        assert!(violations.iter().any(|v| v.rule == "passive_voice"));
    }

    #[test]
    fn german_konjunktiv_passive_genitive_and_compound() {
        let violations = LanguageRuleCheckers::german(
            "Das Haus wurde gebaut. Das Auto des Lehrers ist neu. Die Geschwindigkeitsbegrenzung gilt hier.",
            &source("man baute das Haus"),
        );
        assert!(violations.iter().any(|v| v.rule == "passive_voice"));
        assert!(violations.iter().any(|v| v.rule == "genitive"));
        assert!(violations.iter().any(|v| v.rule == "compound_length"));
    }

    #[test]
    fn simple_german_passes() {
        let violations = LanguageRuleCheckers::german(
            "Wir fahren morgen an den See.",
            &source("morgen fahren wir an den See"),
        );
        assert!(violations.is_empty(), "{violations:?}");
    }

    #[test]
    fn english_passive_and_readability() {
        let passive = LanguageRuleCheckers::english(
            "The meeting was cancelled by the director.",
            &source("the director cancelled the meeting"),
        );
        assert!(passive.iter().any(|v| v.rule == "passive_voice"));
        let dense = LanguageRuleCheckers::english(
            "Organizational restructuring necessitates comprehensive stakeholder communication initiatives.",
            &source(""),
        );
        assert!(dense.iter().any(|v| v.rule == "readability"));
    }

    #[test]
    fn simple_english_passes() {
        let violations = LanguageRuleCheckers::english(
            "We will start the new steps soon.",
            &source("we will endeavor to facilitate implementation"),
        );
        assert!(!violations.iter().any(|v| v.rule == "readability"));
    }

    #[test]
    fn spanish_subjunctive_and_passive() {
        let subjunctive =
            LanguageRuleCheckers::spanish("Espero que vengas pronto.", &source("ven pronto"));
        assert!(subjunctive.iter().any(|v| v.rule == "subjunctive"));
        let passive = LanguageRuleCheckers::spanish(
            "La reunión fue cancelada por la dirección.",
            &source("la dirección canceló la reunión"),
        );
        assert!(passive.iter().any(|v| v.rule == "passive_voice"));
    }
}
