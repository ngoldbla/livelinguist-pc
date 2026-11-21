//! Streaming ASR seam and mock/off engines.
//!
//! Real sherpa-onnx recognition is feature-gated (`sherpa`) and needs a
//! downloaded model; the default build uses the scripted demo engine so the
//! full pipeline, UI, and eval stay runnable offline. A missing model is an
//! explicit `ASR_MODEL_MISSING` error, never a fake listening state.

use ll_core::coverage::{AsrStatus, CoverageMatrix};
use ll_core::demo::DemoConversation;
use ll_core::language::Language;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum AsrError {
    #[error("ASR model missing for {language}: {size_mb} MB download required")]
    ModelMissing { language: String, size_mb: u64 },
    #[error("ASR initialization failed: {0}")]
    InitFailed(String),
    #[error("ASR engine unavailable: {0}")]
    Unavailable(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AsrEvent {
    /// Full rolling hypothesis for the current utterance.
    pub hypothesis: String,
    pub audio_time: f64,
    pub detected_language: Option<String>,
}

/// A streaming ASR engine. Chunks are 1.1 s (17,600 samples) at 16 kHz.
pub trait AsrEngine: Send {
    fn process(&mut self, chunk: &[f32], audio_time: f64) -> Result<Option<AsrEvent>, AsrError>;
    fn reset(&mut self);
    fn name(&self) -> &'static str;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptLine {
    pub text: String,
    pub language: Option<String>,
    pub chunks: usize,
}

#[derive(Debug, Clone)]
pub struct ScriptedAsr {
    lines: Vec<ScriptLine>,
    index: usize,
    chunks_seen: usize,
}

impl ScriptedAsr {
    pub fn new(lines: Vec<ScriptLine>) -> Self {
        Self {
            lines,
            index: 0,
            chunks_seen: 0,
        }
    }

    pub fn from_lines(
        lines: impl IntoIterator<Item = impl Into<String>>,
        chunks_per_line: usize,
    ) -> Self {
        Self::new(
            lines
                .into_iter()
                .map(|text| ScriptLine {
                    text: text.into(),
                    language: None,
                    chunks: chunks_per_line.max(1),
                })
                .collect(),
        )
    }

    /// Build the demo script from a pack's few-shot sources.
    pub fn from_conversation(conversation: &DemoConversation, words_per_chunk: usize) -> Self {
        let lines = conversation
            .lines
            .iter()
            .map(|line| {
                let words = line.text.split_whitespace().count();
                ScriptLine {
                    text: line.text.clone(),
                    language: Some(line.language.clone()),
                    chunks: (words as f64 / words_per_chunk.max(1) as f64).ceil() as usize,
                }
            })
            .collect::<Vec<_>>();
        Self::new(lines)
    }

    pub fn is_finished(&self) -> bool {
        self.index >= self.lines.len()
    }
}

impl AsrEngine for ScriptedAsr {
    fn process(&mut self, _chunk: &[f32], audio_time: f64) -> Result<Option<AsrEvent>, AsrError> {
        if self.index >= self.lines.len() {
            return Ok(None);
        }
        let line = &self.lines[self.index];
        let event = AsrEvent {
            hypothesis: line.text.clone(),
            audio_time,
            detected_language: line.language.clone(),
        };
        self.chunks_seen += 1;
        if self.chunks_seen >= line.chunks.max(1) {
            self.index += 1;
            self.chunks_seen = 0;
        }
        Ok(Some(event))
    }

    fn reset(&mut self) {
        self.index = 0;
        self.chunks_seen = 0;
    }

    fn name(&self) -> &'static str {
        "mock"
    }
}

/// No-op engine for simplify-only sessions.
#[derive(Debug, Default, Clone, Copy)]
pub struct OffAsr;

impl AsrEngine for OffAsr {
    fn process(&mut self, _chunk: &[f32], _audio_time: f64) -> Result<Option<AsrEvent>, AsrError> {
        Ok(None)
    }

    fn reset(&mut self) {}

    fn name(&self) -> &'static str {
        "off"
    }
}

/// A sherpa-onnx-backed engine placeholder. The feature can be enabled once a
/// pinned model is installed; without a model it fails loudly.
#[cfg(feature = "sherpa")]
#[derive(Debug)]
pub struct SherpaAsr {
    model_dir: std::path::PathBuf,
    language: Language,
}

#[cfg(feature = "sherpa")]
impl SherpaAsr {
    pub fn new(model_dir: impl Into<std::path::PathBuf>, language: Language) -> Self {
        Self {
            model_dir: model_dir.into(),
            language,
        }
    }
}

#[cfg(feature = "sherpa")]
impl AsrEngine for SherpaAsr {
    fn process(&mut self, _chunk: &[f32], _audio_time: f64) -> Result<Option<AsrEvent>, AsrError> {
        if !self.model_dir.exists() {
            return Err(AsrError::ModelMissing {
                language: self.language.locale().to_string(),
                size_mb: 600,
            });
        }
        Err(AsrError::Unavailable(
            "sherpa-onnx recognizer wiring is pending the S-3A export".to_string(),
        ))
    }

    fn reset(&mut self) {}

    fn name(&self) -> &'static str {
        "sherpa-onnx"
    }
}

/// What the product may promise for a locale.
pub fn asr_status(language: Language, coverage: &CoverageMatrix) -> AsrStatus {
    coverage.effective(language)
}

/// Whether the UI may show a listening state without a demo/mock label.
pub fn can_promise_live_listening(language: Language, coverage: &CoverageMatrix) -> bool {
    coverage.status(language).listening_capable()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ll_core::coverage::{AsrStatus, CoverageMatrix};

    #[test]
    fn scripted_engine_emits_lines_then_stops() {
        let mut asr = ScriptedAsr::from_lines(["Bonjour tout le monde.", "Comment ça va ?"], 2);
        assert_eq!(
            asr.process(&[0.0; 10], 1.1).unwrap().unwrap().hypothesis,
            "Bonjour tout le monde."
        );
        assert_eq!(
            asr.process(&[0.0; 10], 2.2).unwrap().unwrap().hypothesis,
            "Bonjour tout le monde."
        );
        assert_eq!(
            asr.process(&[0.0; 10], 3.3).unwrap().unwrap().hypothesis,
            "Comment ça va ?"
        );
        assert!(!asr.is_finished());
        let _ = asr.process(&[0.0; 10], 4.4).unwrap();
        assert!(asr.is_finished());
    }

    #[test]
    fn off_engine_is_noop() {
        let mut asr = OffAsr;
        assert!(asr.process(&[0.0; 10], 1.0).unwrap().is_none());
        assert_eq!(asr.name(), "off");
    }

    #[test]
    fn demo_conversation_becomes_scripted_lines() {
        use ll_core::language::Framework;
        use ll_core::pack::FrameworkPack;
        let pack = FrameworkPack::load_framework(Framework::Falc).unwrap();
        let conversation = DemoConversation::from_pack(&pack, 4);
        let asr = ScriptedAsr::from_conversation(&conversation, 4);
        assert_eq!(asr.lines.len(), conversation.lines.len());
        assert!(asr.lines.iter().all(|line| line.chunks >= 1));
    }

    #[test]
    fn coverage_gate_is_honest_before_s3a() {
        let matrix = CoverageMatrix::v1();
        assert!(!can_promise_live_listening(Language::DeDe, &matrix));
        assert_eq!(asr_status(Language::DeDe, &matrix), AsrStatus::SimplifyOnly);
    }

    #[test]
    fn reset_rewinds_the_script() {
        let mut asr = ScriptedAsr::from_lines(["a"], 1);
        let _ = asr.process(&[0.0; 4], 1.0).unwrap();
        asr.reset();
        assert!(!asr.is_finished());
    }
}
