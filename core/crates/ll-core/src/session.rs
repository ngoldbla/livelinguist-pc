//! Session models: finalized segments, simplification outcomes, in-memory store.
//!
//! Ported from `Sources/LiveLinguistKit/Models/Segment.swift`,
//! `SessionModels.swift`, and `Session/SessionStore.swift`. Transcripts are
//! memory-only until an explicit export (INV-010).

use crate::language::{Framework, Language};
use crate::validation::Violation;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_SEGMENT_ID: AtomicU64 = AtomicU64::new(1);

/// A process-local, monotonic segment id.
pub fn next_segment_id() -> String {
    format!("seg-{:06}", NEXT_SEGMENT_ID.fetch_add(1, Ordering::Relaxed))
}

/// A live, revisable ASR hypothesis rendered in Tier 1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartialHypothesis {
    pub text: String,
    pub stable_text: String,
    /// Audio-clock time (seconds since session start).
    pub audio_time: f64,
}

impl PartialHypothesis {
    pub fn new(text: impl Into<String>, stable_text: impl Into<String>, audio_time: f64) -> Self {
        Self {
            text: text.into(),
            stable_text: stable_text.into(),
            audio_time,
        }
    }
}

/// A finalized sentence unit produced by the segmenter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FinalizedSegment {
    pub id: String,
    pub text: String,
    pub detected_language: Option<String>,
    pub audio_start: f64,
    pub audio_end: f64,
}

impl FinalizedSegment {
    pub fn new(text: impl Into<String>, audio_start: f64, audio_end: f64) -> Self {
        Self {
            id: next_segment_id(),
            text: text.into(),
            detected_language: None,
            audio_start,
            audio_end,
        }
    }

    pub fn with_language(
        text: impl Into<String>,
        detected_language: Option<String>,
        audio_start: f64,
        audio_end: f64,
    ) -> Self {
        Self {
            id: next_segment_id(),
            text: text.into(),
            detected_language,
            audio_start,
            audio_end,
        }
    }

    pub fn duration(&self) -> f64 {
        self.audio_end - self.audio_start
    }
}

/// Why a segment fell back to verbatim display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FallbackReason {
    Timeout,
    RetryFailed,
    WrongLanguage,
    Unfaithful,
    BackendError,
    DegradationSkip,
}

impl FallbackReason {
    pub fn as_str(self) -> &'static str {
        match self {
            FallbackReason::Timeout => "timeout",
            FallbackReason::RetryFailed => "retry_failed",
            FallbackReason::WrongLanguage => "wrong_language",
            FallbackReason::Unfaithful => "unfaithful",
            FallbackReason::BackendError => "backend_error",
            FallbackReason::DegradationSkip => "degradation_skip",
        }
    }
}

/// Why a segment was intentionally not simplified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkipReason {
    OffTargetLanguage,
    NotRequested,
    Fragment,
}

impl SkipReason {
    pub fn as_str(self) -> &'static str {
        match self {
            SkipReason::OffTargetLanguage => "skipped_off_target_language",
            SkipReason::NotRequested => "not_requested",
            SkipReason::Fragment => "skipped_fragment",
        }
    }
}

/// The result of running one finalized segment through the engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SimplificationOutcome {
    Simplified { text: String },
    VerbatimFallback { reason: FallbackReason },
    Skipped { reason: SkipReason },
}

impl SimplificationOutcome {
    pub fn display_text(&self) -> Option<&str> {
        match self {
            SimplificationOutcome::Simplified { text } => Some(text),
            _ => None,
        }
    }

    pub fn is_simplified(&self) -> bool {
        matches!(self, SimplificationOutcome::Simplified { .. })
    }
}

/// How much the validators trust a displayed simplification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SimplificationConfidence {
    Clean,
    LowConfidence { reasons: Vec<String> },
}

impl SimplificationConfidence {
    pub fn is_clean(&self) -> bool {
        matches!(self, SimplificationConfidence::Clean)
    }

    pub fn reasons(&self) -> &[String] {
        match self {
            SimplificationConfidence::Clean => &[],
            SimplificationConfidence::LowConfidence { reasons } => reasons,
        }
    }
}

/// A finalized segment paired with its simplification outcome.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimplifiedSegment {
    pub segment: FinalizedSegment,
    pub outcome: SimplificationOutcome,
    pub soft_violations: Vec<Violation>,
    pub confidence: SimplificationConfidence,
}

impl SimplifiedSegment {
    pub fn new(segment: FinalizedSegment, outcome: SimplificationOutcome) -> Self {
        Self {
            segment,
            outcome,
            soft_violations: Vec::new(),
            confidence: SimplificationConfidence::Clean,
        }
    }

    pub fn with_validation(
        segment: FinalizedSegment,
        outcome: SimplificationOutcome,
        soft_violations: Vec<Violation>,
        confidence: SimplificationConfidence,
    ) -> Self {
        Self {
            segment,
            outcome,
            soft_violations,
            confidence,
        }
    }

    pub fn id(&self) -> &str {
        &self.segment.id
    }
}

/// A completed Tier 2 caption line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptionLine {
    pub id: String,
    pub text: String,
    pub is_simplified: bool,
}

/// How Tier 2 simplification is triggered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimplificationMode {
    Automatic,
    #[default]
    OnDemand,
}

/// Where the session audio comes from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AudioSourceKind {
    Microphone,
    SystemAudio,
    File { path: String },
    Demo,
    Silence,
}

/// Everything chosen in the pre-session UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionConfiguration {
    pub language: Language,
    pub source: AudioSourceKind,
    pub language_detection_enabled: bool,
    pub simplification_mode: SimplificationMode,
}

impl SessionConfiguration {
    pub fn new(language: Language, source: AudioSourceKind) -> Self {
        Self {
            language,
            source,
            language_detection_enabled: false,
            simplification_mode: SimplificationMode::OnDemand,
        }
    }

    pub fn framework(&self) -> Framework {
        self.language.framework()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReversionSpan {
    pub start: f64,
    pub end: f64,
}

/// In-memory session record. Nothing touches disk until an explicit export.
#[derive(Debug, Clone)]
pub struct SessionStore {
    pub configuration: SessionConfiguration,
    pub started_at_iso: Option<String>,
    pub ended_at_iso: Option<String>,
    pub duration_seconds: Option<f64>,
    pub segments: Vec<SimplifiedSegment>,
    pub reversion_spans: Vec<ReversionSpan>,
}

impl SessionStore {
    pub fn new(configuration: SessionConfiguration) -> Self {
        Self {
            configuration,
            started_at_iso: None,
            ended_at_iso: None,
            duration_seconds: None,
            segments: Vec::new(),
            reversion_spans: Vec::new(),
        }
    }

    pub fn append(&mut self, segment: SimplifiedSegment) {
        self.segments.push(segment);
    }

    pub fn update_outcome(
        &mut self,
        id: &str,
        outcome: SimplificationOutcome,
        soft_violations: Vec<Violation>,
        confidence: SimplificationConfidence,
    ) {
        if let Some(entry) = self.segments.iter_mut().find(|entry| entry.id() == id) {
            entry.outcome = outcome;
            entry.soft_violations = soft_violations;
            entry.confidence = confidence;
        }
    }

    pub fn find(&self, id: &str) -> Option<&SimplifiedSegment> {
        self.segments.iter().find(|entry| entry.id() == id)
    }

    pub fn duration(&self) -> Option<f64> {
        self.duration_seconds
    }

    /// % of LID-identified segments in the session target language.
    pub fn percent_target_language(&self) -> Option<u8> {
        let detected = self
            .segments
            .iter()
            .filter_map(|entry| entry.segment.detected_language.as_deref())
            .collect::<Vec<_>>();
        if detected.is_empty() {
            return None;
        }
        let target = self.configuration.language.base_code();
        let in_target = detected.iter().filter(|code| **code == target).count();
        Some(((in_target as f64 / detected.len() as f64) * 100.0).round() as u8)
    }

    pub fn is_in_reversion_span(&self, segment: &FinalizedSegment) -> bool {
        self.reversion_spans
            .iter()
            .any(|span| segment.audio_start >= span.start && segment.audio_start < span.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_duration_and_outcome_helpers() {
        let segment = FinalizedSegment::new("Bonjour tout le monde.", 1.0, 3.5);
        assert!((segment.duration() - 2.5).abs() < f64::EPSILON);
        assert_eq!(
            SimplificationOutcome::Simplified {
                text: "Il fait beau.".into()
            }
            .display_text(),
            Some("Il fait beau.")
        );
        assert!(SimplificationOutcome::VerbatimFallback {
            reason: FallbackReason::Timeout
        }
        .display_text()
        .is_none());
    }

    #[test]
    fn session_configuration_defaults_to_on_demand() {
        let config = SessionConfiguration::new(Language::FrFr, AudioSourceKind::Microphone);
        assert_eq!(config.framework(), Framework::Falc);
        assert_eq!(config.simplification_mode, SimplificationMode::OnDemand);
        assert!(!config.language_detection_enabled);
    }

    #[test]
    fn percent_target_language_counts_detected_segments() {
        let mut store = SessionStore::new(SessionConfiguration::new(
            Language::FrFr,
            AudioSourceKind::Microphone,
        ));
        store.append(SimplifiedSegment::new(
            FinalizedSegment::with_language("Bonjour.", Some("fr".into()), 0.0, 1.0),
            SimplificationOutcome::Skipped {
                reason: SkipReason::NotRequested,
            },
        ));
        store.append(SimplifiedSegment::new(
            FinalizedSegment::with_language("Hello.", Some("en".into()), 1.0, 2.0),
            SimplificationOutcome::Skipped {
                reason: SkipReason::OffTargetLanguage,
            },
        ));
        assert_eq!(store.percent_target_language(), Some(50));
    }

    #[test]
    fn update_outcome_replaces_by_id() {
        let mut store = SessionStore::new(SessionConfiguration::new(
            Language::FrFr,
            AudioSourceKind::Demo,
        ));
        let segment = FinalizedSegment::new("Bonjour.", 0.0, 1.0);
        let id = segment.id.clone();
        store.append(SimplifiedSegment::new(
            segment,
            SimplificationOutcome::Skipped {
                reason: SkipReason::NotRequested,
            },
        ));
        store.update_outcome(
            &id,
            SimplificationOutcome::Simplified {
                text: "Salut.".into(),
            },
            Vec::new(),
            SimplificationConfidence::Clean,
        );
        assert_eq!(
            store.find(&id).unwrap().outcome.display_text(),
            Some("Salut.")
        );
    }

    #[test]
    fn reversion_span_membership() {
        let mut store = SessionStore::new(SessionConfiguration::new(
            Language::FrFr,
            AudioSourceKind::Demo,
        ));
        store.reversion_spans = vec![ReversionSpan {
            start: 5.0,
            end: 10.0,
        }];
        assert!(store.is_in_reversion_span(&FinalizedSegment::new("x", 6.0, 8.0)));
        assert!(!store.is_in_reversion_span(&FinalizedSegment::new("x", 12.0, 14.0)));
    }
}
