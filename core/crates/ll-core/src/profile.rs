//! MLX-free description of how a resolved simplifier must be driven, plus the
//! per-language fine-tune gate.
//!
//! Ported from `Sources/LiveLinguistKit/Simplification/SimplifierProfile.swift`.
//! The concrete model identity lives in `ll-llm`; the policy lives here so it is
//! testable without a backend.

use crate::decoding::DecodingStyle;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimplifierProfile {
    /// Prompt with `FrameworkPack::lean_variant` (no few-shots).
    pub uses_lean_pack: bool,
    pub decoding_style: DecodingStyle,
    pub max_tokens: usize,
}

impl SimplifierProfile {
    pub const fn stock() -> Self {
        Self {
            uses_lean_pack: false,
            decoding_style: DecodingStyle::FirstSentence,
            max_tokens: 48,
        }
    }

    pub const fn fine_tuned() -> Self {
        Self {
            uses_lean_pack: true,
            decoding_style: DecodingStyle::SingleLine,
            max_tokens: 128,
        }
    }

    pub fn for_fine_tuned(fine_tuned: bool) -> Self {
        if fine_tuned {
            Self::fine_tuned()
        } else {
            Self::stock()
        }
    }
}

/// A fine-tuned model is used only when the language passed the eval gate and
/// the model is installed. Everything else stays byte-identical to stock.
pub fn uses_fine_tuned(
    language: &str,
    enabled_languages: &HashSet<String>,
    fine_tuned_installed: bool,
) -> bool {
    let base = language
        .split(['-', '_'])
        .next()
        .unwrap_or(language)
        .to_lowercase();
    enabled_languages.contains(&base) && fine_tuned_installed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enabled() -> HashSet<String> {
        ["de", "fr", "es", "en"]
            .into_iter()
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn enabled_language_with_model_installed_uses_fine_tuned() {
        assert!(uses_fine_tuned("de-DE", &enabled(), true));
        assert!(uses_fine_tuned("es-US", &enabled(), true));
    }

    #[test]
    fn disabled_or_missing_falls_back_to_stock() {
        assert!(!uses_fine_tuned("it-IT", &enabled(), true));
        assert!(!uses_fine_tuned("fr-FR", &enabled(), false));
        assert!(!uses_fine_tuned("de-DE", &HashSet::<String>::new(), true));
    }

    #[test]
    fn profiles_are_distinct() {
        assert_eq!(
            SimplifierProfile::fine_tuned().decoding_style,
            DecodingStyle::SingleLine
        );
        assert!(SimplifierProfile::fine_tuned().uses_lean_pack);
        assert_eq!(SimplifierProfile::fine_tuned().max_tokens, 128);
        assert_eq!(
            SimplifierProfile::for_fine_tuned(true),
            SimplifierProfile::fine_tuned()
        );
        assert_eq!(
            SimplifierProfile::stock().decoding_style,
            DecodingStyle::FirstSentence
        );
        assert!(!SimplifierProfile::stock().uses_lean_pack);
        assert_eq!(SimplifierProfile::stock().max_tokens, 48);
    }
}
