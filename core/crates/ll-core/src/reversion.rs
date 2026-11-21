//! English-reversion state machine.
//!
//! Ported from `Sources/LiveLinguistKit/Reversion/ReversionDetector.swift`.
//! In a non-English session, >= 2 consecutive English finalized segments
//! activate the indicator; 2 consecutive target-language segments clear it.

use crate::session::{FinalizedSegment, ReversionSpan};

#[derive(Debug, Clone)]
pub struct ReversionDetector {
    target_language: String,
    is_english_session: bool,
    consecutive_english: usize,
    consecutive_target: usize,
    pending_start: Option<f64>,
    last_english_end: Option<f64>,
    is_active: bool,
    spans: Vec<ReversionSpan>,
}

impl ReversionDetector {
    pub fn new(target_language: &str) -> Self {
        Self {
            target_language: target_language.to_string(),
            is_english_session: target_language == "en",
            consecutive_english: 0,
            consecutive_target: 0,
            pending_start: None,
            last_english_end: None,
            is_active: false,
            spans: Vec::new(),
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn spans(&self) -> &[ReversionSpan] {
        &self.spans
    }

    /// Observe a finalized segment; returns whether the indicator is active.
    pub fn observe(&mut self, segment: &FinalizedSegment) -> bool {
        if self.is_english_session {
            return self.is_active;
        }
        let Some(detected) = segment.detected_language.as_deref() else {
            return self.is_active;
        };
        if detected == "en" {
            self.consecutive_english += 1;
            self.consecutive_target = 0;
            if self.pending_start.is_none() {
                self.pending_start = Some(segment.audio_start);
            }
            self.last_english_end = Some(segment.audio_end);
            if self.consecutive_english >= 2 {
                self.is_active = true;
            }
        } else if detected == self.target_language {
            self.consecutive_target += 1;
            self.consecutive_english = 0;
            if self.is_active && self.consecutive_target >= 2 {
                self.close_span();
                self.is_active = false;
            } else if !self.is_active {
                self.pending_start = None;
                self.last_english_end = None;
            }
        } else {
            self.consecutive_english = 0;
            self.consecutive_target = 0;
            if !self.is_active {
                self.pending_start = None;
                self.last_english_end = None;
            }
        }
        self.is_active
    }

    /// End of session: close any open reversion span.
    pub fn finish(&mut self) {
        if self.is_active {
            self.close_span();
        }
        self.is_active = false;
    }

    fn close_span(&mut self) {
        if let (Some(start), Some(end)) = (self.pending_start, self.last_english_end) {
            self.spans.push(ReversionSpan { start, end });
        }
        self.pending_start = None;
        self.last_english_end = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segment(language: &str, start: f64, end: f64) -> FinalizedSegment {
        FinalizedSegment::with_language("x", Some(language.to_string()), start, end)
    }

    #[test]
    fn single_english_segment_does_not_trigger() {
        let mut detector = ReversionDetector::new("fr");
        assert!(!detector.observe(&segment("en", 0.0, 2.0)));
    }

    #[test]
    fn two_consecutive_english_segments_trigger() {
        let mut detector = ReversionDetector::new("fr");
        assert!(!detector.observe(&segment("en", 0.0, 2.0)));
        assert!(detector.observe(&segment("en", 2.0, 4.0)));
    }

    #[test]
    fn target_segment_between_english_resets_count() {
        let mut detector = ReversionDetector::new("fr");
        let _ = detector.observe(&segment("en", 0.0, 2.0));
        let _ = detector.observe(&segment("fr", 2.0, 4.0));
        assert!(!detector.observe(&segment("en", 4.0, 6.0)));
    }

    #[test]
    fn clears_after_two_consecutive_target_segments() {
        let mut detector = ReversionDetector::new("fr");
        let _ = detector.observe(&segment("en", 0.0, 2.0));
        let _ = detector.observe(&segment("en", 2.0, 4.0));
        assert!(detector.observe(&segment("fr", 4.0, 6.0)));
        assert!(!detector.observe(&segment("fr", 6.0, 8.0)));
    }

    #[test]
    fn spans_are_recorded() {
        let mut detector = ReversionDetector::new("fr");
        let _ = detector.observe(&segment("en", 10.0, 12.0));
        let _ = detector.observe(&segment("en", 12.0, 14.0));
        let _ = detector.observe(&segment("fr", 14.0, 16.0));
        let _ = detector.observe(&segment("fr", 16.0, 18.0));
        assert_eq!(detector.spans().len(), 1);
        assert_eq!(detector.spans()[0].start, 10.0);
        assert_eq!(detector.spans()[0].end, 14.0);
    }

    #[test]
    fn english_session_never_triggers() {
        let mut detector = ReversionDetector::new("en");
        let _ = detector.observe(&segment("en", 0.0, 2.0));
        assert!(!detector.observe(&segment("en", 2.0, 4.0)));
    }

    #[test]
    fn open_span_closes_on_finish() {
        let mut detector = ReversionDetector::new("fr");
        let _ = detector.observe(&segment("en", 0.0, 2.0));
        let _ = detector.observe(&segment("en", 2.0, 4.0));
        detector.finish();
        assert_eq!(detector.spans().len(), 1);
        assert_eq!(detector.spans()[0].end, 4.0);
    }

    #[test]
    fn third_language_breaks_english_run_without_clearing() {
        let mut detector = ReversionDetector::new("fr");
        let _ = detector.observe(&segment("en", 0.0, 2.0));
        let _ = detector.observe(&segment("en", 2.0, 4.0));
        assert!(detector.observe(&segment("de", 4.0, 6.0)));
        assert!(detector.observe(&segment("fr", 6.0, 8.0)));
        assert!(!detector.observe(&segment("fr", 8.0, 10.0)));
    }
}
