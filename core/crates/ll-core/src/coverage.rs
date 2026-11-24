//! Per-locale ASR coverage policy (D-031 / INV-015).
//!
//! Every locale has an explicit, measured status (T33). `tbd` remains a valid
//! value for a row that has not been measured, and is treated as
//! simplify-only. Demo/mock sessions are labeled as demo, not live ASR.

use crate::language::Language;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AsrStatus {
    /// Streaming ASR passes the WER gate on the locale fixture.
    Ok,
    /// Streaming works but WER/RTF is worse; show a subtle badge.
    Degraded,
    /// No usable streaming ASR; paste/type + simplification only.
    SimplifyOnly,
    /// Not yet measured; cannot ship as `ok`.
    Tbd,
}

impl AsrStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            AsrStatus::Ok => "ok",
            AsrStatus::Degraded => "degraded",
            AsrStatus::SimplifyOnly => "simplify_only",
            AsrStatus::Tbd => "tbd",
        }
    }

    /// What the UI may promise in v1: anything not ok/degraded is simplify-only.
    pub fn effective(self) -> AsrStatus {
        match self {
            AsrStatus::Ok | AsrStatus::Degraded => self,
            AsrStatus::SimplifyOnly | AsrStatus::Tbd => AsrStatus::SimplifyOnly,
        }
    }

    pub fn listening_capable(self) -> bool {
        matches!(self.effective(), AsrStatus::Ok | AsrStatus::Degraded)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageEntry {
    pub language: Language,
    pub status: AsrStatus,
    /// Candidate model notes from S-3A; empty until measured.
    pub candidate: String,
    pub wer_fixture: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageMatrix {
    pub entries: Vec<CoverageEntry>,
}

impl CoverageMatrix {
    /// The measured v1 matrix (T33).
    ///
    /// Each row comes from `docs/qa/2026-11-23-asr-coverage-v1.md`: FLEURS
    /// test split, about 300 s per locale, measured 2026-11-24 with
    /// `ll-eval --scenario asr-wer --asr sherpa`. `Ok` means score <= 0.15,
    /// RTF <= 0.5 and p95 chunk <= 500 ms all passed; a score above the gate
    /// is `SimplifyOnly`. Timing is the Linux 4-core tier C approximation; the
    /// Windows re-check is tracked in that document. Nothing is `Tbd`.
    pub fn v1() -> Self {
        use AsrStatus::{Ok, SimplifyOnly};
        // (language, status, metric, score, RTF) — date 2026-11-24 for every row.
        const MEASURED: [(Language, AsrStatus, &str, f32, f32); 29] = [
            (Language::DeDe, Ok, "wer", 0.141, 0.211),
            (Language::FrFr, Ok, "wer", 0.135, 0.218),
            (Language::EsEs, Ok, "wer", 0.044, 0.203), // es_419 recordings
            (Language::EnUs, Ok, "wer", 0.149, 0.206),
            (Language::ArSa, Ok, "wer", 0.125, 0.210), // ar_eg speakers, MSA prompts
            (Language::DaDk, SimplifyOnly, "wer", 0.319, 0.210),
            (Language::EtEe, SimplifyOnly, "wer", 0.376, 0.207),
            (Language::FiFi, SimplifyOnly, "wer", 0.297, 0.209),
            (Language::HiIn, Ok, "wer", 0.101, 0.213),
            (Language::ItIt, Ok, "wer", 0.061, 0.214),
            (Language::JaJp, SimplifyOnly, "cer", 0.155, 0.208),
            (Language::KoKr, Ok, "cer", 0.087, 0.207),
            (Language::NlNl, SimplifyOnly, "wer", 0.182, 0.207),
            (Language::PtBr, Ok, "wer", 0.095, 0.216),
            (Language::PtPt, Ok, "wer", 0.095, 0.231), // pt_br recordings; no pt_pt in FLEURS
            (Language::RuRu, Ok, "wer", 0.148, 0.221),
            (Language::SkSk, SimplifyOnly, "wer", 0.309, 0.212),
            (Language::SvSe, SimplifyOnly, "wer", 0.258, 0.216),
            (Language::TrTr, SimplifyOnly, "wer", 0.275, 0.212),
            (Language::ViVn, Ok, "wer", 0.120, 0.224),
            (Language::ZhCn, SimplifyOnly, "cer", 0.266, 0.230),
            (Language::UkUa, SimplifyOnly, "wer", 0.196, 0.225),
            (Language::RoRo, SimplifyOnly, "wer", 0.436, 0.220),
            (Language::BgBg, SimplifyOnly, "wer", 0.255, 0.208),
            (Language::PlPl, SimplifyOnly, "wer", 0.297, 0.223),
            (Language::HrHr, SimplifyOnly, "wer", 0.505, 0.222),
            (Language::NbNo, SimplifyOnly, "wer", 0.248, 0.217),
            (Language::HuHu, SimplifyOnly, "wer", 0.360, 0.223),
            (Language::CsCz, SimplifyOnly, "wer", 0.283, 0.218),
        ];
        Self {
            entries: MEASURED
                .into_iter()
                .map(|(language, status, metric, score, rtf)| CoverageEntry {
                    language,
                    status,
                    candidate: format!(
                        "Nemotron 3.5 ASR 0.6B (560 ms int8); FLEURS {metric} {score:.3}, RTF {rtf:.3}, 2026-11-24"
                    ),
                    wer_fixture: Some(format!(
                        "docs/qa/2026-11-23-asr-coverage-v1.md#{}",
                        language.locale()
                    )),
                })
                .collect(),
        }
    }

    pub fn status(&self, language: Language) -> AsrStatus {
        self.entries
            .iter()
            .find(|entry| entry.language == language)
            .map(|entry| entry.status)
            .unwrap_or(AsrStatus::Tbd)
    }

    pub fn effective(&self, language: Language) -> AsrStatus {
        self.status(language).effective()
    }

    pub fn listening_capable(&self, language: Language) -> bool {
        self.status(language).listening_capable()
    }

    /// Update one row after the WER gate (S-3A / T-003).
    pub fn set_status(&mut self, language: Language, status: AsrStatus) {
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|entry| entry.language == language)
        {
            entry.status = status;
        }
    }

    /// Count rows that can actually listen in v1.
    pub fn listening_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.status.listening_capable())
            .count()
    }
}

impl Default for CoverageMatrix {
    fn default() -> Self {
        Self::v1()
    }
}

/// The exact user-facing promise for a simplify-only locale.
pub const SIMPLIFY_ONLY_NOTICE: &str =
    "Live listening is not available for this language yet. Paste or type text to simplify it.";

#[cfg(test)]
mod tests {
    use super::*;

    /// The 12 locales that passed the gate in
    /// `docs/qa/2026-11-23-asr-coverage-v1.md`.
    const MEASURED_OK: [Language; 12] = [
        Language::DeDe,
        Language::FrFr,
        Language::EsEs,
        Language::EnUs,
        Language::ArSa,
        Language::HiIn,
        Language::ItIt,
        Language::KoKr,
        Language::PtBr,
        Language::PtPt,
        Language::RuRu,
        Language::ViVn,
    ];

    #[test]
    fn every_locale_has_one_measured_row_and_none_is_tbd() {
        let matrix = CoverageMatrix::v1();
        assert_eq!(matrix.entries.len(), 29);
        for language in Language::ALL {
            let rows = matrix
                .entries
                .iter()
                .filter(|entry| entry.language == language)
                .count();
            assert_eq!(rows, 1, "{}", language.locale());
            let status = matrix.status(language);
            assert_ne!(status, AsrStatus::Tbd, "{}", language.locale());
            let expected = if MEASURED_OK.contains(&language) {
                AsrStatus::Ok
            } else {
                AsrStatus::SimplifyOnly
            };
            assert_eq!(status, expected, "{}", language.locale());
            assert_eq!(
                matrix.listening_capable(language),
                expected == AsrStatus::Ok
            );
        }
        assert_eq!(matrix.listening_count(), MEASURED_OK.len());
    }

    #[test]
    fn the_minimum_live_set_can_listen() {
        let matrix = CoverageMatrix::v1();
        for language in [
            Language::EnUs,
            Language::FrFr,
            Language::DeDe,
            Language::EsEs,
            Language::ArSa,
        ] {
            assert_eq!(
                matrix.status(language),
                AsrStatus::Ok,
                "{}",
                language.locale()
            );
        }
    }

    #[test]
    fn tbd_is_still_simplify_only() {
        let mut matrix = CoverageMatrix::v1();
        matrix.set_status(Language::EnUs, AsrStatus::Tbd);
        assert_eq!(matrix.effective(Language::EnUs), AsrStatus::SimplifyOnly);
        assert!(!matrix.listening_capable(Language::EnUs));
    }

    #[test]
    fn a_promoted_locale_can_listen() {
        let mut matrix = CoverageMatrix::v1();
        let before = matrix.listening_count();
        matrix.set_status(Language::DaDk, AsrStatus::Ok);
        assert!(matrix.listening_capable(Language::DaDk));
        assert_eq!(matrix.listening_count(), before + 1);
        assert_eq!(matrix.effective(Language::DaDk), AsrStatus::Ok);
    }

    #[test]
    fn status_strings_are_stable() {
        assert_eq!(AsrStatus::Ok.as_str(), "ok");
        assert_eq!(AsrStatus::Degraded.as_str(), "degraded");
        assert_eq!(AsrStatus::SimplifyOnly.as_str(), "simplify_only");
        assert_eq!(AsrStatus::Tbd.as_str(), "tbd");
        assert!(SIMPLIFY_ONLY_NOTICE.contains("Paste or type"));
    }
}
