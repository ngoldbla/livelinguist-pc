//! Framework rule packs, embedded from `assets/packs` verbatim.
//!
//! Ported from `Sources/LiveLinguistKit/Simplification/FrameworkPack.swift`.
//! Packs use national framework names (`letlst`, `selkokieli`, ...); unknown
//! framework strings decode to [`Framework::EasyToRead`] exactly like
//! the Swift decoder.

use crate::language::{Framework, Language};
use serde::Deserialize;
use thiserror::Error;

/// Embedded pack name -> JSON source. Sorted by name for determinism.
pub const PACKS: &[(&str, &str)] = &[
    (
        "ar",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/ar.json"
        )),
    ),
    (
        "bg",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/bg.json"
        )),
    ),
    (
        "cs",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/cs.json"
        )),
    ),
    (
        "da",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/da.json"
        )),
    ),
    (
        "easy_english",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/easy_english.json"
        )),
    ),
    (
        "et",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/et.json"
        )),
    ),
    (
        "falc",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/falc.json"
        )),
    ),
    (
        "fi",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/fi.json"
        )),
    ),
    (
        "hi",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/hi.json"
        )),
    ),
    (
        "hr",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/hr.json"
        )),
    ),
    (
        "hu",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/hu.json"
        )),
    ),
    (
        "it",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/it.json"
        )),
    ),
    (
        "ja",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/ja.json"
        )),
    ),
    (
        "ko",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/ko.json"
        )),
    ),
    (
        "lectura_facil",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/lectura_facil.json"
        )),
    ),
    (
        "leichte_sprache",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/leichte_sprache.json"
        )),
    ),
    (
        "nb",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/nb.json"
        )),
    ),
    (
        "nl",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/nl.json"
        )),
    ),
    (
        "pl",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/pl.json"
        )),
    ),
    (
        "pt-BR",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/pt-BR.json"
        )),
    ),
    (
        "pt-PT",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/pt-PT.json"
        )),
    ),
    (
        "ro",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/ro.json"
        )),
    ),
    (
        "ru",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/ru.json"
        )),
    ),
    (
        "sk",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/sk.json"
        )),
    ),
    (
        "sv",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/sv.json"
        )),
    ),
    (
        "tr",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/tr.json"
        )),
    ),
    (
        "uk",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/uk.json"
        )),
    ),
    (
        "vi",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/vi.json"
        )),
    ),
    (
        "zh-CN",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/packs/zh-CN.json"
        )),
    ),
];

#[derive(Debug, Error)]
pub enum PackError {
    #[error("pack not found: {0}")]
    NotFound(String),
    #[error("invalid pack {name}: {source}")]
    Parse {
        name: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("framework has no single pack: {0}")]
    FrameworkWithoutPack(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thresholds {
    pub max_words_hard: usize,
    pub max_words_soft: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct FewShot {
    pub source: String,
    pub simplified: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameworkPack {
    pub framework: Framework,
    pub language: String,
    pub display_name: String,
    pub system_prompt: String,
    pub retry_prompt_template: String,
    pub thresholds: Thresholds,
    #[serde(default)]
    pub few_shots: Vec<FewShot>,
    #[serde(default)]
    pub base_lang: Option<String>,
    #[serde(default)]
    pub stopwords: Option<Vec<String>>,
    #[serde(default)]
    pub length_unit: Option<String>,
}

impl FrameworkPack {
    /// Parse a pack by `assets/packs/<name>.json` name.
    pub fn load(name: &str) -> Result<Self, PackError> {
        let json = pack_json(name).ok_or_else(|| PackError::NotFound(name.to_string()))?;
        serde_json::from_str(json).map_err(|source| PackError::Parse {
            name: name.to_string(),
            source,
        })
    }

    pub fn load_framework(framework: Framework) -> Result<Self, PackError> {
        if framework == Framework::EasyToRead {
            return Err(PackError::FrameworkWithoutPack(
                framework.resource_name().to_string(),
            ));
        }
        Self::load(framework.resource_name())
    }

    pub fn for_language(language: Language) -> Result<Self, PackError> {
        Self::load(language.pack_name())
    }

    /// Base language code ("pt" for a "pt-BR" pack).
    pub fn base_code(&self) -> String {
        let raw = self.base_lang.as_deref().unwrap_or(&self.language);
        raw.split(['-', '_']).next().unwrap_or(raw).to_lowercase()
    }

    /// The lean variant used with a fine-tuned simplifier: no few-shots,
    /// everything validators rely on unchanged.
    pub fn lean_variant(&self) -> Self {
        let mut copy = self.clone();
        copy.few_shots.clear();
        copy
    }

    pub fn has_few_shots(&self) -> bool {
        !self.few_shots.is_empty()
    }
}

/// The JSON source for a pack name, if it is embedded.
pub fn pack_json(name: &str) -> Option<&'static str> {
    PACKS
        .iter()
        .find(|(pack_name, _)| *pack_name == name)
        .map(|(_, json)| *json)
}

/// All embedded pack names, sorted.
pub fn pack_names() -> Vec<&'static str> {
    PACKS.iter().map(|(name, _)| *name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_29_packs_load_and_have_required_content() {
        assert_eq!(PACKS.len(), 29);
        for (name, _) in PACKS {
            let pack = FrameworkPack::load(name).expect(name);
            assert!(!pack.system_prompt.is_empty(), "{name} prompt");
            assert!(
                pack.retry_prompt_template.contains("{violations}"),
                "{name} retry template"
            );
            assert!(pack.thresholds.max_words_hard > 0, "{name} hard cap");
            assert!(
                pack.thresholds.max_words_hard >= pack.thresholds.max_words_soft,
                "{name} thresholds"
            );
            assert!(!pack.display_name.is_empty(), "{name} display name");
        }
    }

    #[test]
    fn every_language_resolves_to_a_loadable_pack() {
        for language in Language::ALL {
            let pack = FrameworkPack::for_language(language)
                .unwrap_or_else(|_| panic!("{}", language.locale()));
            assert_eq!(
                pack.base_code(),
                language.base_code(),
                "{}",
                language.locale()
            );
        }
    }

    #[test]
    fn curated_thresholds_match_the_prd() {
        let cases = [
            (Framework::Falc, 15, 12),
            (Framework::LeichteSprache, 12, 10),
            (Framework::EasyEnglish, 18, 15),
            (Framework::LecturaFacil, 15, 12),
        ];
        for (framework, hard, soft_) in cases {
            let pack = FrameworkPack::load_framework(framework).unwrap();
            assert_eq!(pack.thresholds.max_words_hard, hard);
            assert_eq!(pack.thresholds.max_words_soft, soft_);
        }
    }

    #[test]
    fn curated_packs_have_at_least_six_few_shots_and_same_language_twice() {
        for framework in Framework::curated() {
            let pack = FrameworkPack::load_framework(framework).unwrap();
            assert!(pack.few_shots.len() >= 6, "{framework:?}");
            assert!(
                pack.system_prompt.matches("same language").count() >= 2,
                "{framework:?} must state the same-language rule twice"
            );
            for shot in &pack.few_shots {
                assert!(!shot.source.is_empty());
                assert!(!shot.simplified.is_empty());
            }
        }
    }

    #[test]
    fn lean_variant_drops_only_few_shots() {
        let full = FrameworkPack::load_framework(Framework::LeichteSprache).unwrap();
        let lean = full.lean_variant();
        assert!(lean.few_shots.is_empty());
        assert_eq!(lean.system_prompt, full.system_prompt);
        assert_eq!(lean.retry_prompt_template, full.retry_prompt_template);
        assert_eq!(lean.thresholds, full.thresholds);
        assert!(!full.few_shots.is_empty());
    }

    #[test]
    fn unknown_pack_is_an_error() {
        assert!(matches!(
            FrameworkPack::load("nope"),
            Err(PackError::NotFound(_))
        ));
    }
}
