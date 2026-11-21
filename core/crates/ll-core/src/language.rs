//! Language model and framework mapping.
//!
//! Ported from the Mac reference (`Sources/LiveLinguistKit/Models/Language.swift`).
//! The plan's 29 locales are the product surface; the Mac's extra `es-US` alias
//! maps onto `es-ES` because both use the same `lectura_facil` pack.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// A spoken language supported by the product (plan §12.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    DeDe,
    FrFr,
    EsEs,
    EnUs,
    ArSa,
    DaDk,
    EtEe,
    FiFi,
    HiIn,
    ItIt,
    JaJp,
    KoKr,
    NlNl,
    PtBr,
    PtPt,
    RuRu,
    SkSk,
    SvSe,
    TrTr,
    ViVn,
    ZhCn,
    UkUa,
    RoRo,
    BgBg,
    PlPl,
    HrHr,
    NbNo,
    HuHu,
    CsCz,
}

impl Language {
    /// All 29 locales in the plan's matrix order.
    pub const ALL: [Language; 29] = [
        Language::DeDe,
        Language::FrFr,
        Language::EsEs,
        Language::EnUs,
        Language::ArSa,
        Language::DaDk,
        Language::EtEe,
        Language::FiFi,
        Language::HiIn,
        Language::ItIt,
        Language::JaJp,
        Language::KoKr,
        Language::NlNl,
        Language::PtBr,
        Language::PtPt,
        Language::RuRu,
        Language::SkSk,
        Language::SvSe,
        Language::TrTr,
        Language::ViVn,
        Language::ZhCn,
        Language::UkUa,
        Language::RoRo,
        Language::BgBg,
        Language::PlPl,
        Language::HrHr,
        Language::NbNo,
        Language::HuHu,
        Language::CsCz,
    ];

    pub fn locale(self) -> &'static str {
        match self {
            Language::DeDe => "de-DE",
            Language::FrFr => "fr-FR",
            Language::EsEs => "es-ES",
            Language::EnUs => "en-US",
            Language::ArSa => "ar-SA",
            Language::DaDk => "da-DK",
            Language::EtEe => "et-EE",
            Language::FiFi => "fi-FI",
            Language::HiIn => "hi-IN",
            Language::ItIt => "it-IT",
            Language::JaJp => "ja-JP",
            Language::KoKr => "ko-KR",
            Language::NlNl => "nl-NL",
            Language::PtBr => "pt-BR",
            Language::PtPt => "pt-PT",
            Language::RuRu => "ru-RU",
            Language::SkSk => "sk-SK",
            Language::SvSe => "sv-SE",
            Language::TrTr => "tr-TR",
            Language::ViVn => "vi-VN",
            Language::ZhCn => "zh-CN",
            Language::UkUa => "uk-UA",
            Language::RoRo => "ro-RO",
            Language::BgBg => "bg-BG",
            Language::PlPl => "pl-PL",
            Language::HrHr => "hr-HR",
            Language::NbNo => "nb-NO",
            Language::HuHu => "hu-HU",
            Language::CsCz => "cs-CZ",
        }
    }

    pub fn from_locale(code: &str) -> Option<Self> {
        let lower = code.to_ascii_lowercase();
        Some(match lower.as_str() {
            "de-de" | "de" => Language::DeDe,
            "fr-fr" | "fr" => Language::FrFr,
            "es-es" | "es" | "es-us" => Language::EsEs,
            "en-us" | "en" => Language::EnUs,
            "ar-sa" | "ar" => Language::ArSa,
            "da-dk" | "da" => Language::DaDk,
            "et-ee" | "et" => Language::EtEe,
            "fi-fi" | "fi" => Language::FiFi,
            "hi-in" | "hi" => Language::HiIn,
            "it-it" | "it" => Language::ItIt,
            "ja-jp" | "ja" => Language::JaJp,
            "ko-kr" | "ko" => Language::KoKr,
            "nl-nl" | "nl" => Language::NlNl,
            "pt-br" => Language::PtBr,
            "pt-pt" => Language::PtPt,
            "ru-ru" | "ru" => Language::RuRu,
            "sk-sk" | "sk" => Language::SkSk,
            "sv-se" | "sv" => Language::SvSe,
            "tr-tr" | "tr" => Language::TrTr,
            "vi-vn" | "vi" => Language::ViVn,
            "zh-cn" | "zh" => Language::ZhCn,
            "uk-ua" | "uk" => Language::UkUa,
            "ro-ro" | "ro" => Language::RoRo,
            "bg-bg" | "bg" => Language::BgBg,
            "pl-pl" | "pl" => Language::PlPl,
            "hr-hr" | "hr" => Language::HrHr,
            "nb-no" | "nb" | "no" => Language::NbNo,
            "hu-hu" | "hu" => Language::HuHu,
            "cs-cz" | "cs" => Language::CsCz,
            _ => return None,
        })
    }

    /// Two-letter base code used by ASR/LID and the validators.
    pub fn base_code(self) -> &'static str {
        match self {
            Language::DeDe => "de",
            Language::FrFr => "fr",
            Language::EsEs => "es",
            Language::EnUs => "en",
            Language::ArSa => "ar",
            Language::DaDk => "da",
            Language::EtEe => "et",
            Language::FiFi => "fi",
            Language::HiIn => "hi",
            Language::ItIt => "it",
            Language::JaJp => "ja",
            Language::KoKr => "ko",
            Language::NlNl => "nl",
            Language::PtBr | Language::PtPt => "pt",
            Language::RuRu => "ru",
            Language::SkSk => "sk",
            Language::SvSe => "sv",
            Language::TrTr => "tr",
            Language::ViVn => "vi",
            Language::ZhCn => "zh",
            Language::UkUa => "uk",
            Language::RoRo => "ro",
            Language::BgBg => "bg",
            Language::PlPl => "pl",
            Language::HrHr => "hr",
            Language::NbNo => "nb",
            Language::HuHu => "hu",
            Language::CsCz => "cs",
        }
    }

    pub fn framework(self) -> Framework {
        match self {
            Language::FrFr => Framework::Falc,
            Language::EnUs => Framework::EasyEnglish,
            Language::DeDe => Framework::LeichteSprache,
            Language::EsEs => Framework::LecturaFacil,
            _ => Framework::EasyToRead,
        }
    }

    /// The `assets/packs/<name>.json` file for this locale.
    pub fn pack_name(self) -> &'static str {
        match self {
            Language::DeDe => "leichte_sprache",
            Language::FrFr => "falc",
            Language::EsEs => "lectura_facil",
            Language::EnUs => "easy_english",
            Language::ArSa => "ar",
            Language::DaDk => "da",
            Language::EtEe => "et",
            Language::FiFi => "fi",
            Language::HiIn => "hi",
            Language::ItIt => "it",
            Language::JaJp => "ja",
            Language::KoKr => "ko",
            Language::NlNl => "nl",
            Language::PtBr => "pt-BR",
            Language::PtPt => "pt-PT",
            Language::RuRu => "ru",
            Language::SkSk => "sk",
            Language::SvSe => "sv",
            Language::TrTr => "tr",
            Language::ViVn => "vi",
            Language::ZhCn => "zh-CN",
            Language::UkUa => "uk",
            Language::RoRo => "ro",
            Language::BgBg => "bg",
            Language::PlPl => "pl",
            Language::HrHr => "hr",
            Language::NbNo => "nb",
            Language::HuHu => "hu",
            Language::CsCz => "cs",
        }
    }

    /// The UI localization code (`ui/src/i18n`) for the language picker.
    pub fn ui_code(self) -> &'static str {
        match self {
            Language::ZhCn => "zh-Hans",
            Language::PtBr => "pt-BR",
            Language::PtPt => "pt-PT",
            Language::EsEs => "es",
            other => other.base_code(),
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Language::DeDe => "Deutsch (Deutschland)",
            Language::FrFr => "Français (France)",
            Language::EsEs => "Español (España)",
            Language::EnUs => "English (US)",
            Language::ArSa => "العربية (Arabic)",
            Language::DaDk => "Dansk (Danish)",
            Language::EtEe => "Eesti (Estonian)",
            Language::FiFi => "Suomi (Finnish)",
            Language::HiIn => "हिन्दी (Hindi)",
            Language::ItIt => "Italiano (Italian)",
            Language::JaJp => "日本語 (Japanese)",
            Language::KoKr => "한국어 (Korean)",
            Language::NlNl => "Nederlands (Dutch)",
            Language::PtBr => "Português (Brasil)",
            Language::PtPt => "Português (Portugal)",
            Language::RuRu => "Русский (Russian)",
            Language::SkSk => "Slovenčina (Slovak)",
            Language::SvSe => "Svenska (Swedish)",
            Language::TrTr => "Türkçe (Turkish)",
            Language::ViVn => "Tiếng Việt (Vietnamese)",
            Language::ZhCn => "中文 (Mandarin, Simplified)",
            Language::UkUa => "Українська (Ukrainian)",
            Language::RoRo => "Română (Romanian)",
            Language::BgBg => "Български (Bulgarian)",
            Language::PlPl => "Polski (Polish)",
            Language::HrHr => "Hrvatski (Croatian)",
            Language::NbNo => "Norsk bokmål (Norwegian)",
            Language::HuHu => "Magyar (Hungarian)",
            Language::CsCz => "Čeština (Czech)",
        }
    }

    pub fn is_english(self) -> bool {
        self == Language::EnUs
    }

    /// True for right-to-left scripts (Arabic in v1).
    pub fn is_rtl(self) -> bool {
        self == Language::ArSa
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.locale())
    }
}

impl Serialize for Language {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.locale())
    }
}

impl<'de> Deserialize<'de> for Language {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Language::from_locale(&raw)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown language: {raw}")))
    }
}

/// The official easy-language register (plan §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Framework {
    Falc,
    LeichteSprache,
    EasyEnglish,
    LecturaFacil,
    #[default]
    EasyToRead,
}

impl Framework {
    pub fn from_name(name: &str) -> Self {
        match name {
            "falc" => Framework::Falc,
            "leichteSprache" => Framework::LeichteSprache,
            "easyEnglish" => Framework::EasyEnglish,
            "lecturaFacil" => Framework::LecturaFacil,
            _ => Framework::EasyToRead,
        }
    }

    pub fn resource_name(self) -> &'static str {
        match self {
            Framework::Falc => "falc",
            Framework::LeichteSprache => "leichte_sprache",
            Framework::EasyEnglish => "easy_english",
            Framework::LecturaFacil => "lectura_facil",
            Framework::EasyToRead => "easy_to_read",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Framework::Falc => "FALC (Facile à Lire et à Comprendre)",
            Framework::LeichteSprache => "Leichte Sprache",
            Framework::EasyEnglish => "Easy English / Plain English",
            Framework::LecturaFacil => "Lectura Fácil",
            Framework::EasyToRead => "Easy-to-Read / Plain Language",
        }
    }

    /// The four curated frameworks with bespoke validators.
    pub fn curated() -> [Framework; 4] {
        [
            Framework::Falc,
            Framework::LeichteSprache,
            Framework::EasyEnglish,
            Framework::LecturaFacil,
        ]
    }

    pub fn is_curated(self) -> bool {
        self != Framework::EasyToRead
    }
}

impl Serialize for Framework {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let value = match self {
            Framework::Falc => "falc",
            Framework::LeichteSprache => "leichteSprache",
            Framework::EasyEnglish => "easyEnglish",
            Framework::LecturaFacil => "lecturaFacil",
            Framework::EasyToRead => "easyToRead",
        };
        serializer.serialize_str(value)
    }
}

impl<'de> Deserialize<'de> for Framework {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Ok(Framework::from_name(&raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_locales_are_unique_and_map_to_packs() {
        let mut seen = std::collections::HashSet::new();
        for language in Language::ALL {
            assert!(seen.insert(language.locale()));
            assert!(Language::from_locale(language.locale()).is_some());
            assert!(!language.display_name().is_empty());
            assert!(!language.pack_name().is_empty());
        }
        assert_eq!(seen.len(), 29);
    }

    #[test]
    fn regional_aliases_and_base_codes() {
        assert_eq!(Language::from_locale("es-US"), Some(Language::EsEs));
        assert_eq!(Language::PtBr.base_code(), "pt");
        assert_eq!(Language::PtBr.pack_name(), "pt-BR");
        assert_eq!(Language::PtPt.pack_name(), "pt-PT");
        assert_eq!(Language::ZhCn.ui_code(), "zh-Hans");
        assert_eq!(Language::NbNo.ui_code(), "nb");
        assert!(Language::ArSa.is_rtl());
        assert!(!Language::KoKr.is_rtl());
    }

    #[test]
    fn framework_mapping_and_unknown_decode() {
        assert_eq!(Language::FrFr.framework(), Framework::Falc);
        assert_eq!(Language::DeDe.framework(), Framework::LeichteSprache);
        assert_eq!(Language::EnUs.framework(), Framework::EasyEnglish);
        assert_eq!(Language::EsEs.framework(), Framework::LecturaFacil);
        assert_eq!(Language::ItIt.framework(), Framework::EasyToRead);
        // Packs use national names ("letlst", "selkokieli", ...); all map to easyToRead.
        assert_eq!(Framework::from_name("selkokieli"), Framework::EasyToRead);
        assert_eq!(
            Framework::from_name("leichteSprache"),
            Framework::LeichteSprache
        );
    }
}
