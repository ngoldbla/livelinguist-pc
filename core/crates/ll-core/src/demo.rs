//! Deterministic, model-free demo simplifier and per-pack demo conversation.
//!
//! Ported from `Sources/LiveLinguistKit/Simplification/DemoSimplifierBackend.swift`
//! and `DemoConversation.swift`. This is the UI/test double, not FALC quality.

use crate::disfluency::clean;
use crate::pack::FrameworkPack;
use std::collections::HashMap;

/// One scripted ASR line for the demo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemoLine {
    pub text: String,
    pub language: String,
}

/// A speech/silence phase; `ll-audio` turns this into synthetic samples.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DemoPhase {
    Speech(f64),
    Silence(f64),
}

/// The built-in demo for one learning language, derived from its rule pack:
/// the pack's few-shot sources are "spoken", and their reference rewrites are
/// the curated simplifications.
#[derive(Debug, Clone)]
pub struct DemoConversation {
    pub lines: Vec<DemoLine>,
    pub curated: HashMap<String, String>,
    pub audio_pattern: Vec<DemoPhase>,
    pub words_per_chunk: usize,
}

impl DemoConversation {
    pub fn from_pack(pack: &FrameworkPack, words_per_chunk: usize) -> Self {
        const CHUNK_DURATION: f64 = 1.1;
        let language = pack.base_code();
        let mut lines = Vec::new();
        let mut curated = HashMap::new();
        let mut pattern = Vec::new();
        for shot in &pack.few_shots {
            let word_count = shot.source.split_whitespace().count();
            if word_count == 0 {
                continue;
            }
            lines.push(DemoLine {
                text: shot.source.clone(),
                language: language.clone(),
            });
            curated.insert(key(&shot.source), shot.simplified.clone());
            let cleaned = clean(&shot.source, &pack.language);
            curated.insert(key(&cleaned), shot.simplified.clone());

            let chunks = (word_count as f64 / words_per_chunk as f64).ceil();
            pattern.push(DemoPhase::Speech(chunks * CHUNK_DURATION));
            pattern.push(DemoPhase::Silence(CHUNK_DURATION));
        }
        Self {
            lines,
            curated,
            audio_pattern: pattern,
            words_per_chunk,
        }
    }

    /// Lookup key: letters and digits only, case-folded.
    pub fn key(sentence: &str) -> String {
        key(sentence)
    }
}

/// Letters and digits only, case-folded — immune to spacing/punctuation changes.
pub fn key(sentence: &str) -> String {
    sentence
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

fn fillers(language: &str) -> &'static [&'static str] {
    match language {
        "fr" => &[
            "euh", "ben", "bah", "quoi", "hein", "voilà", "enfin", "donc",
        ],
        "en" => &["um", "uh", "like", "basically", "actually", "so", "well"],
        "de" => &["äh", "ähm", "halt", "also", "naja", "quasi"],
        "es" => &["eh", "pues", "bueno", "vale", "o sea", "este"],
        _ => &[],
    }
}

fn conjunctions(language: &str) -> &'static [&'static str] {
    match language {
        "fr" => &["et", "mais", "donc", "alors", "puis"],
        "en" => &["and", "but", "so", "then"],
        "de" => &["und", "aber", "dann", "denn"],
        "es" => &["y", "pero", "entonces", "luego"],
        _ => &[],
    }
}

/// Rules-based demo simplifier: strips fillers, splits long sentences at
/// clause boundaries, enforces the word cap.
#[derive(Debug, Clone)]
pub struct DemoSimplifier {
    language: String,
    max_words_per_sentence: usize,
    curated: HashMap<String, String>,
}

impl DemoSimplifier {
    pub fn new(
        language: &str,
        max_words_per_sentence: usize,
        curated: HashMap<String, String>,
    ) -> Self {
        Self {
            language: language.to_string(),
            max_words_per_sentence,
            curated,
        }
    }

    pub fn from_pack(pack: &FrameworkPack) -> Self {
        Self::new(
            &pack.base_code(),
            pack.thresholds.max_words_soft,
            DemoConversation::from_pack(pack, 4).curated,
        )
    }

    pub fn simplify(&self, sentence: &str) -> String {
        if let Some(reference) = self.curated.get(&key(sentence)) {
            return reference.clone();
        }
        let fillers = fillers(&self.language);
        let conjunctions = conjunctions(&self.language);
        let words = sentence
            .split_whitespace()
            .filter(|word| {
                let bare = crate::text_analysis::trim_token(&word.to_lowercase()).to_string();
                !fillers.contains(&bare.as_str()) && !bare.is_empty()
            })
            .collect::<Vec<_>>();

        let mut sentences: Vec<Vec<&str>> = Vec::new();
        let mut current: Vec<&str> = Vec::new();
        for word in words {
            let bare = crate::text_analysis::trim_token(&word.to_lowercase()).to_string();
            let conjunction_break = conjunctions.contains(&bare.as_str()) && current.len() >= 4;
            let clause_break = word.ends_with(',') || word.ends_with(';');
            if conjunction_break {
                sentences.push(std::mem::take(&mut current));
                continue; // drop the conjunction itself
            }
            let break_after = clause_break && current.len() + 1 >= 4;
            current.push(word);
            if break_after || current.len() >= self.max_words_per_sentence {
                sentences.push(std::mem::take(&mut current));
            }
        }
        if !current.is_empty() {
            sentences.push(current);
        }

        sentences
            .into_iter()
            .map(|sentence| {
                let mut text = sentence.join(" ");
                while text.ends_with([',', ';', ':']) {
                    text.pop();
                }
                let mut chars = text.chars();
                let rendered = match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => text,
                };
                if !rendered.ends_with(['.', '!', '?', '…']) {
                    format!("{rendered}.")
                } else {
                    rendered
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::Framework;

    #[test]
    fn curated_demo_uses_pack_reference_rewrites() {
        let pack = FrameworkPack::load_framework(Framework::Falc).unwrap();
        let simplifier = DemoSimplifier::from_pack(&pack);
        let shot = &pack.few_shots[0];
        assert_eq!(simplifier.simplify(&shot.source), shot.simplified);
        // The cleaned spelling is indexed too.
        let conversation = DemoConversation::from_pack(&pack, 4);
        assert_eq!(conversation.lines.len(), pack.few_shots.len());
    }

    #[test]
    fn rules_path_splits_and_caps() {
        let simplifier = DemoSimplifier::new("en", 8, HashMap::new());
        let output = simplifier
            .simplify("So we went to the market and we bought some bread and then we went home");
        assert!(!output.contains("So "));
        assert!(output.ends_with('.'));
        for sentence in output.split_terminator('.') {
            if sentence.trim().is_empty() {
                continue;
            }
            assert!(
                sentence.split_whitespace().count() <= 8,
                "too long: {sentence}"
            );
        }
    }

    #[test]
    fn key_ignores_case_spacing_and_punctuation() {
        assert_eq!(
            DemoConversation::key("Bonjour, le monde !"),
            DemoConversation::key("bonjour le monde")
        );
        assert!(DemoConversation::key("é").contains('é'));
    }

    #[test]
    fn demo_audio_pattern_has_one_pause_per_line() {
        let pack = FrameworkPack::load_framework(Framework::Falc).unwrap();
        let conversation = DemoConversation::from_pack(&pack, 4);
        assert_eq!(
            conversation.audio_pattern.len(),
            conversation.lines.len() * 2
        );
        assert!(matches!(
            conversation.audio_pattern[0],
            DemoPhase::Speech(_)
        ));
        assert!(matches!(
            conversation.audio_pattern[1],
            DemoPhase::Silence(_)
        ));
    }

    #[test]
    fn unknown_locale_has_no_fillers() {
        assert_eq!(fillers("xx").len(), 0);
        assert_eq!(conjunctions("xx").len(), 0);
    }
}
