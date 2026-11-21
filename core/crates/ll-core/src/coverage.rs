//! Per-locale ASR coverage policy (D-031 / INV-015).
//!
//! Every locale has an explicit status. `tbd` is honest but not shippable as
//! `ok`; the effective v1 behavior for `tbd` is simplify-only until the S-3A
//! WER gate is recorded. Demo/mock sessions are labeled as demo, not live ASR.

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
    /// Honest v1 default: no locale has a measured WER gate yet.
    pub fn v1() -> Self {
        Self {
            entries: Language::ALL
                .into_iter()
                .map(|language| CoverageEntry {
                    language,
                    status: AsrStatus::Tbd,
                    candidate: "Nemotron 3.5 / zipformer TBD".to_string(),
                    wer_fixture: None,
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

    #[test]
    fn every_locale_has_a_row_and_tbd_is_simplify_only() {
        let matrix = CoverageMatrix::v1();
        assert_eq!(matrix.entries.len(), 29);
        for language in Language::ALL {
            assert_eq!(matrix.status(language), AsrStatus::Tbd);
            assert_eq!(matrix.effective(language), AsrStatus::SimplifyOnly);
            assert!(!matrix.listening_capable(language));
        }
        assert_eq!(matrix.listening_count(), 0);
    }

    #[test]
    fn a_promoted_locale_can_listen() {
        let mut matrix = CoverageMatrix::v1();
        matrix.set_status(Language::DeDe, AsrStatus::Ok);
        assert!(matrix.listening_capable(Language::DeDe));
        assert_eq!(matrix.listening_count(), 1);
        assert_eq!(matrix.effective(Language::DeDe), AsrStatus::Ok);
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
