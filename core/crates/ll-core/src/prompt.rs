//! Prompt construction from a framework pack.
//!
//! Ported from `Sources/LiveLinguistKit/Simplification/PromptBuilder.swift`.
//! The fast-path is byte-compatible with the Python training renderer; the
//! golden tests pin both the lean and few-shot prompts.

use crate::pack::FrameworkPack;
use crate::validation::Violation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimplificationRequest {
    pub system_prompt: String,
    pub user_prompt: String,
    pub sentence: String,
}

#[derive(Debug, Clone)]
pub struct PromptBuilder {
    pack: FrameworkPack,
}

impl PromptBuilder {
    pub fn new(pack: &FrameworkPack) -> Self {
        Self { pack: pack.clone() }
    }

    pub fn pack(&self) -> &FrameworkPack {
        &self.pack
    }

    pub fn make_request(
        &self,
        sentence: &str,
        previous_segments: &[String],
    ) -> SimplificationRequest {
        SimplificationRequest {
            system_prompt: self.pack.system_prompt.clone(),
            user_prompt: self.body(sentence, previous_segments),
            sentence: sentence.to_string(),
        }
    }

    /// The one constrained retry, naming the violations (PRD §5.6 step 3).
    pub fn make_retry_request(
        &self,
        sentence: &str,
        previous_segments: &[String],
        failed_output: &str,
        violations: &[Violation],
    ) -> SimplificationRequest {
        let named = violations
            .iter()
            .map(|violation| violation.detail.as_str())
            .collect::<Vec<_>>()
            .join("; ");
        let retry_instruction = self
            .pack
            .retry_prompt_template
            .replace("{violations}", &named);
        let user_prompt = format!(
            "{}\n\nYour previous attempt was:\n{}\n\n{}",
            self.body(sentence, previous_segments),
            failed_output,
            retry_instruction
        );
        SimplificationRequest {
            system_prompt: self.pack.system_prompt.clone(),
            user_prompt,
            sentence: sentence.to_string(),
        }
    }

    fn body(&self, sentence: &str, previous_segments: &[String]) -> String {
        // A fine-tuned pack has no few-shots: render the trained, example-free
        // prompt, and never inject a rolling-context block it never saw.
        if self.pack.few_shots.is_empty() {
            return format!("Original: {sentence}\nRewritten:");
        }

        let mut sections = vec!["Examples:".to_string()];
        for shot in &self.pack.few_shots {
            sections.push(format!(
                "Original: {}\nRewritten: {}",
                shot.source, shot.simplified
            ));
        }
        let context = previous_segments
            .iter()
            .rev()
            .take(2)
            .rev()
            .cloned()
            .collect::<Vec<_>>();
        if !context.is_empty() {
            sections.push(format!(
                "Earlier in the conversation (context only, do not rewrite):\n{}",
                context
                    .iter()
                    .map(|segment| format!("- {segment}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }
        sections.push(format!("Original: {sentence}\nRewritten:"));
        sections.join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::Framework;
    use crate::pack::{FewShot, Thresholds};

    fn pack(few_shots: Vec<FewShot>) -> FrameworkPack {
        FrameworkPack {
            framework: Framework::LeichteSprache,
            language: "de".to_string(),
            display_name: "Test".to_string(),
            system_prompt: "SYS".to_string(),
            retry_prompt_template: "{violations}".to_string(),
            thresholds: Thresholds {
                max_words_hard: 12,
                max_words_soft: 10,
            },
            few_shots,
            base_lang: None,
            stopwords: None,
            length_unit: None,
        }
    }

    #[test]
    fn lean_render_matches_golden() {
        let builder = PromptBuilder::new(&pack(vec![]));
        let request = builder.make_request("Dies ist ein Test.", &[]);
        assert_eq!(
            request.user_prompt,
            "Original: Dies ist ein Test.\nRewritten:"
        );
    }

    #[test]
    fn lean_render_ignores_rolling_context() {
        let builder = PromptBuilder::new(&pack(vec![]));
        let request = builder.make_request(
            "Dies ist ein Test.",
            &[
                "Vorheriger Satz eins.".to_string(),
                "Vorheriger Satz zwei.".to_string(),
            ],
        );
        assert_eq!(
            request.user_prompt,
            "Original: Dies ist ein Test.\nRewritten:"
        );
        assert!(!request.user_prompt.contains("Vorheriger Satz"));
        assert!(!request.user_prompt.contains("Earlier in the conversation"));
    }

    #[test]
    fn few_shot_render_matches_golden() {
        let builder = PromptBuilder::new(&pack(vec![FewShot {
            source: "A".to_string(),
            simplified: "B".to_string(),
        }]));
        let request = builder.make_request("C", &[]);
        assert_eq!(
            request.user_prompt,
            "Examples:\n\nOriginal: A\nRewritten: B\n\nOriginal: C\nRewritten:"
        );
    }

    #[test]
    fn context_is_limited_to_last_two_segments() {
        let builder = PromptBuilder::new(&pack(vec![FewShot {
            source: "A".to_string(),
            simplified: "B".to_string(),
        }]));
        let request = builder.make_request(
            "Phrase actuelle.",
            &[
                "Une.".to_string(),
                "Deux.".to_string(),
                "Trois.".to_string(),
                "Quatre.".to_string(),
            ],
        );
        assert!(!request.user_prompt.contains("Une."));
        assert!(!request.user_prompt.contains("Deux."));
        assert!(request.user_prompt.contains("Trois."));
        assert!(request.user_prompt.contains("Quatre."));
    }

    #[test]
    fn retry_request_names_violations() {
        let builder = PromptBuilder::new(&pack(vec![]));
        let violation = Violation::hard("sentence_length", "18 words (max 15)");
        let request = builder.make_retry_request(
            "Une phrase beaucoup trop longue.",
            &[],
            "sortie ratée",
            &[violation],
        );
        assert!(request.user_prompt.contains("18 words (max 15)"));
        assert!(request
            .user_prompt
            .contains("Une phrase beaucoup trop longue."));
        assert!(request.user_prompt.contains("sortie ratée"));
    }

    #[test]
    fn real_falc_pack_includes_few_shots_and_context() {
        let pack = FrameworkPack::load_framework(Framework::Falc).unwrap();
        let builder = PromptBuilder::new(&pack);
        let request = builder.make_request(
            "Il faudrait qu'on se voie prochainement.",
            &[
                "On a parlé du projet.".to_string(),
                "C'était intéressant.".to_string(),
            ],
        );
        assert_eq!(request.system_prompt, pack.system_prompt);
        assert!(request
            .user_prompt
            .contains(pack.few_shots[0].source.as_str()));
        assert!(request.user_prompt.contains("On a parlé du projet."));
    }
}
