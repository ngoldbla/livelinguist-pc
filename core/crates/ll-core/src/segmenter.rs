//! Segmenter: rolling ASR hypotheses -> finalized sentence units.
//!
//! Ported from `Sources/LiveLinguistKit/Segmentation/Segmenter.swift`.
//! Finalization triggers: a stable terminal-punctuation token, a VAD pause
//! >= 500 ms, or the 12 s maximum segment duration.

use crate::session::{FinalizedSegment, PartialHypothesis};

#[derive(Debug, Clone, Copy)]
pub struct SegmenterConfig {
    pub pause_threshold: f64,
    pub max_segment_duration: f64,
}

impl Default for SegmenterConfig {
    fn default() -> Self {
        Self {
            pause_threshold: 0.5,
            max_segment_duration: 12.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmenterOutput {
    pub partial: Option<PartialHypothesis>,
    pub finalized: Vec<FinalizedSegment>,
}

#[derive(Debug, Clone)]
pub struct Segmenter {
    config: SegmenterConfig,
    tokens: Vec<String>,
    previous_tokens: Vec<String>,
    finalized_token_count: usize,
    segment_start_time: Option<f64>,
    current_language: Option<String>,
}

impl Default for Segmenter {
    fn default() -> Self {
        Self::new(SegmenterConfig::default())
    }
}

impl Segmenter {
    pub fn new(config: SegmenterConfig) -> Self {
        Self {
            config,
            tokens: Vec::new(),
            previous_tokens: Vec::new(),
            finalized_token_count: 0,
            segment_start_time: None,
            current_language: None,
        }
    }

    pub fn config(&self) -> SegmenterConfig {
        self.config
    }

    /// Feed a new ASR emission (the full rolling hypothesis so far).
    pub fn ingest(
        &mut self,
        hypothesis: &str,
        audio_time: f64,
        detected_language: Option<&str>,
    ) -> SegmenterOutput {
        self.previous_tokens = std::mem::take(&mut self.tokens);
        self.tokens = hypothesis
            .split_whitespace()
            .map(ToOwned::to_owned)
            .collect();
        if let Some(language) = detected_language {
            self.current_language = Some(language.to_string());
        }

        if self.segment_start_time.is_none() && self.tokens.len() > self.finalized_token_count {
            self.segment_start_time = Some(audio_time);
        }

        let mut finalized = Vec::new();
        if let Some(start) = self.segment_start_time {
            if audio_time - start >= self.config.max_segment_duration {
                finalized = self.finalize_open_region(self.tokens.len(), audio_time, false);
            }
        }
        if finalized.is_empty() {
            if let Some(last_punctuated) = self.last_stable_punctuated_index() {
                finalized = self.finalize_open_region(last_punctuated + 1, audio_time, true);
            }
        }

        if !finalized.is_empty() && self.tokens.len() > self.finalized_token_count {
            self.segment_start_time = Some(audio_time);
        }
        SegmenterOutput {
            partial: self.current_partial(audio_time),
            finalized,
        }
    }

    /// Called on the audio clock even when no new text arrived.
    pub fn tick(&mut self, trailing_silence: f64, audio_time: f64) -> SegmenterOutput {
        let mut finalized = Vec::new();
        if self.tokens.len() > self.finalized_token_count
            && trailing_silence >= self.config.pause_threshold
        {
            finalized = self.finalize_open_region(self.tokens.len(), audio_time, false);
        }
        SegmenterOutput {
            partial: self.current_partial(audio_time),
            finalized,
        }
    }

    /// End of session: finalize whatever remains as one segment.
    pub fn flush(&mut self, audio_time: f64) -> Option<FinalizedSegment> {
        self.finalize_open_region(self.tokens.len(), audio_time, false)
            .into_iter()
            .next()
    }

    fn last_stable_punctuated_index(&self) -> Option<usize> {
        let mut last_index = None;
        let mut index = self.finalized_token_count;
        while index < self.tokens.len() && index < self.previous_tokens.len() {
            if self.tokens[index] != self.previous_tokens[index] {
                break;
            }
            if ends_with_terminal_punctuation(&self.tokens[index]) {
                last_index = Some(index);
            }
            index += 1;
        }
        last_index
    }

    fn finalize_open_region(
        &mut self,
        end_index: usize,
        audio_time: f64,
        split_sentences: bool,
    ) -> Vec<FinalizedSegment> {
        if end_index <= self.finalized_token_count {
            return Vec::new();
        }
        let region = self.tokens[self.finalized_token_count..end_index].to_vec();
        let groups = if split_sentences {
            sentences(&region)
        } else {
            vec![region]
        };
        let start = self.segment_start_time.unwrap_or(audio_time);
        let segments = groups
            .into_iter()
            .map(|group| {
                FinalizedSegment::with_language(
                    group.join(" "),
                    self.current_language.clone(),
                    start,
                    audio_time,
                )
            })
            .collect::<Vec<_>>();
        self.finalized_token_count = end_index;
        if self.finalized_token_count >= self.tokens.len() {
            self.segment_start_time = None;
            self.current_language = None;
        }
        segments
    }

    fn current_partial(&self, audio_time: f64) -> Option<PartialHypothesis> {
        if self.tokens.len() <= self.finalized_token_count {
            return None;
        }
        let open = &self.tokens[self.finalized_token_count..];
        let mut stable = Vec::new();
        let mut index = self.finalized_token_count;
        while index < self.tokens.len() && index < self.previous_tokens.len() {
            if self.tokens[index] != self.previous_tokens[index] {
                break;
            }
            stable.push(self.tokens[index].clone());
            index += 1;
        }
        Some(PartialHypothesis::new(
            open.join(" "),
            stable.join(" "),
            audio_time,
        ))
    }
}

fn sentences(region: &[String]) -> Vec<Vec<String>> {
    let mut groups = Vec::new();
    let mut current = Vec::new();
    for token in region {
        current.push(token.clone());
        if ends_with_terminal_punctuation(token) {
            groups.push(current);
            current = Vec::new();
        }
    }
    if !current.is_empty() {
        groups.push(current);
    }
    groups
}

fn ends_with_terminal_punctuation(token: &str) -> bool {
    for character in token.chars().rev() {
        if ".!?…".contains(character) {
            return true;
        }
        if "\"'»«)]".contains(character) {
            continue;
        }
        return false;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segmenter() -> Segmenter {
        Segmenter::default()
    }

    #[test]
    fn partial_updates_without_finalization() {
        let mut segmenter = segmenter();
        let output = segmenter.ingest("Bonjour tout", 1.1, None);
        assert!(output.finalized.is_empty());
        assert_eq!(output.partial.unwrap().text, "Bonjour tout");
    }

    #[test]
    fn punctuation_finalizes_once_stable() {
        let mut segmenter = segmenter();
        let _ = segmenter.ingest("Bonjour tout", 1.1, None);
        let second = segmenter.ingest("Bonjour tout le monde.", 2.2, None);
        assert!(second.finalized.is_empty());
        let third = segmenter.ingest("Bonjour tout le monde. Comment", 3.3, None);
        assert_eq!(third.finalized.len(), 1);
        assert_eq!(third.finalized[0].text, "Bonjour tout le monde.");
        assert_eq!(third.partial.unwrap().text, "Comment");
    }

    #[test]
    fn multiple_stable_sentences_finalize_separately() {
        let mut segmenter = segmenter();
        let _ = segmenter.ingest("Oui. Bien sûr.", 1.1, None);
        let output = segmenter.ingest("Oui. Bien sûr. Et toi", 2.2, None);
        assert_eq!(
            output
                .finalized
                .iter()
                .map(|segment| segment.text.as_str())
                .collect::<Vec<_>>(),
            vec!["Oui.", "Bien sûr."]
        );
        assert_eq!(output.partial.unwrap().text, "Et toi");
    }

    #[test]
    fn pause_finalizes_without_punctuation() {
        let mut segmenter = segmenter();
        let _ = segmenter.ingest("Ça va bien", 1.1, None);
        let output = segmenter.tick(0.6, 2.2);
        assert_eq!(output.finalized.len(), 1);
        assert_eq!(output.finalized[0].text, "Ça va bien");
        assert!(output.partial.is_none());
    }

    #[test]
    fn short_pause_does_not_finalize() {
        let mut segmenter = segmenter();
        let _ = segmenter.ingest("Ça va bien", 1.1, None);
        assert!(segmenter.tick(0.3, 1.6).finalized.is_empty());
    }

    #[test]
    fn max_duration_cap_finalizes_runaway_segment() {
        let mut segmenter = segmenter();
        let _ = segmenter.ingest("il parle sans arrêt", 1.1, None);
        let mut output = segmenter.ingest("il parle sans arrêt et continue encore", 6.0, None);
        assert!(output.finalized.is_empty());
        output = segmenter.ingest(
            "il parle sans arrêt et continue encore et toujours plus",
            13.5,
            None,
        );
        assert_eq!(output.finalized.len(), 1);
        assert_eq!(
            output.finalized[0].text,
            "il parle sans arrêt et continue encore et toujours plus"
        );
    }

    #[test]
    fn stable_text_lags_revisable_tail() {
        let mut segmenter = segmenter();
        let _ = segmenter.ingest("Bonjour a", 1.1, None);
        let output = segmenter.ingest("Bonjour à tous", 2.2, None);
        let partial = output.partial.unwrap();
        assert_eq!(partial.stable_text, "Bonjour");
        assert_eq!(partial.text, "Bonjour à tous");
    }

    #[test]
    fn detected_language_attaches_to_finalized_segment() {
        let mut segmenter = segmenter();
        let _ = segmenter.ingest("Okay so anyway", 1.1, Some("en"));
        let output = segmenter.tick(0.7, 2.0);
        assert_eq!(output.finalized[0].detected_language.as_deref(), Some("en"));
    }

    #[test]
    fn audio_times_span_the_segment() {
        let mut segmenter = segmenter();
        let _ = segmenter.ingest("Ça va", 1.1, None);
        let output = segmenter.tick(0.8, 3.3);
        let segment = &output.finalized[0];
        assert_eq!(segment.audio_start, 1.1);
        assert_eq!(segment.audio_end, 3.3);
    }

    #[test]
    fn flush_finalizes_remaining_text() {
        let mut segmenter = segmenter();
        let _ = segmenter.ingest("dernier mot", 1.1, None);
        let segment = segmenter.flush(2.0).expect("segment");
        assert_eq!(segment.text, "dernier mot");
        assert!(segmenter.flush(2.1).is_none());
    }

    #[test]
    fn empty_hypothesis_produces_nothing() {
        let mut segmenter = segmenter();
        let output = segmenter.ingest("", 1.1, None);
        assert!(output.finalized.is_empty());
        assert!(output.partial.is_none());
    }

    #[test]
    fn closures_and_quotes_are_skipped_when_checking_punctuation() {
        assert!(ends_with_terminal_punctuation("monde.»"));
        assert!(ends_with_terminal_punctuation("\"fini.\")"));
        assert!(!ends_with_terminal_punctuation("fini"));
    }
}
