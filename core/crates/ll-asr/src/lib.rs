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
    /// Streaming engines with their own endpointing need every chunk: a
    /// chunk the energy VAD skips is speech they never hear. Mock engines
    /// keep the VAD gate (it saves CPU in silence).
    fn needs_continuous_audio(&self) -> bool {
        false
    }
    /// For engines that endpoint on their own: `Some(true)` once after the
    /// engine detected the end of an utterance (it then starts a fresh
    /// hypothesis), `Some(false)` otherwise. `None`: pauses come from the
    /// pipeline's energy VAD.
    fn take_endpoint(&mut self) -> Option<bool> {
        None
    }
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

/// The four files of the pinned Nemotron 3.5 streaming transducer.
pub const SHERPA_MODEL_FILES: [&str; 4] = [
    "encoder.int8.onnx",
    "decoder.int8.onnx",
    "joiner.int8.onnx",
    "tokens.txt",
];

/// Recognizer config for the pinned model (shared by the probe and the
/// engine). CPU provider; endpointing on; everything else at defaults.
#[cfg(feature = "sherpa")]
pub fn sherpa_recognizer_config(
    model_dir: &std::path::Path,
    num_threads: i32,
) -> sherpa_onnx::OnlineRecognizerConfig {
    let path = |name: &str| Some(model_dir.join(name).display().to_string());
    let mut config = sherpa_onnx::OnlineRecognizerConfig::default();
    config.model_config.transducer.encoder = path(SHERPA_MODEL_FILES[0]);
    config.model_config.transducer.decoder = path(SHERPA_MODEL_FILES[1]);
    config.model_config.transducer.joiner = path(SHERPA_MODEL_FILES[2]);
    config.model_config.tokens = path(SHERPA_MODEL_FILES[3]);
    config.model_config.num_threads = num_threads;
    config.model_config.provider = Some("cpu".to_string());
    config.enable_endpoint = true;
    config
}

/// Download size of the pinned ASR model, for `ModelMissing`.
pub const SHERPA_MODEL_SIZE_MB: u64 = 682;

/// Streaming sherpa-onnx engine for the pinned Nemotron 3.5 model.
///
/// sherpa-onnx 2.0.1 marks `OnlineRecognizer` and `OnlineStream` as `Send`
/// (`unsafe impl Send` in the crate), so the engine moves to the capture
/// worker thread without extra wrappers. Language conditioning: none found
/// for this streaming transducer (T12), so `language` only labels errors.
#[cfg(feature = "sherpa")]
pub struct SherpaAsr {
    recognizer: sherpa_onnx::OnlineRecognizer,
    stream: sherpa_onnx::OnlineStream,
    language: Language,
    endpoint_seen: bool,
}

#[cfg(feature = "sherpa")]
impl std::fmt::Debug for SherpaAsr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SherpaAsr")
            .field("language", &self.language)
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "sherpa")]
impl SherpaAsr {
    /// Load the model from `model_dir` (2 CPU threads). Missing files are
    /// `ModelMissing`; a model that fails to load is `Unavailable`.
    pub fn new(
        model_dir: impl AsRef<std::path::Path>,
        language: Language,
    ) -> Result<Self, AsrError> {
        Self::with_threads(model_dir, language, 2)
    }

    pub fn with_threads(
        model_dir: impl AsRef<std::path::Path>,
        language: Language,
        num_threads: i32,
    ) -> Result<Self, AsrError> {
        let model_dir = model_dir.as_ref();
        if SHERPA_MODEL_FILES
            .iter()
            .any(|name| !model_dir.join(name).is_file())
        {
            return Err(AsrError::ModelMissing {
                language: language.locale().to_string(),
                size_mb: SHERPA_MODEL_SIZE_MB,
            });
        }
        let config = sherpa_recognizer_config(model_dir, num_threads);
        let recognizer = sherpa_onnx::OnlineRecognizer::create(&config)
            .ok_or_else(|| AsrError::Unavailable("sherpa recognizer failed to load".to_string()))?;
        let stream = recognizer.create_stream();
        Ok(Self {
            recognizer,
            stream,
            language,
            endpoint_seen: false,
        })
    }

    /// Signal end of input and return the final hypothesis of the current
    /// utterance (used by offline scoring; the live path never ends input).
    pub fn finish(&mut self) -> String {
        self.stream.input_finished();
        while self.recognizer.is_ready(&self.stream) {
            self.recognizer.decode(&self.stream);
        }
        let text = self
            .recognizer
            .get_result(&self.stream)
            .map(|result| result.text.trim().to_string())
            .unwrap_or_default();
        // A finished stream cannot take more audio; start a fresh one.
        self.stream = self.recognizer.create_stream();
        text
    }

    /// Whether the recognizer saw an endpoint since the last reset.
    pub fn is_endpoint(&self) -> bool {
        self.recognizer.is_endpoint(&self.stream)
    }
}

#[cfg(feature = "sherpa")]
impl AsrEngine for SherpaAsr {
    fn process(&mut self, chunk: &[f32], audio_time: f64) -> Result<Option<AsrEvent>, AsrError> {
        self.stream.accept_waveform(16_000, chunk);
        while self.recognizer.is_ready(&self.stream) {
            self.recognizer.decode(&self.stream);
        }
        let text = self
            .recognizer
            .get_result(&self.stream)
            .map(|result| result.text.trim().to_string())
            .unwrap_or_default();
        let event = (!text.is_empty()).then_some(AsrEvent {
            hypothesis: text,
            audio_time,
            detected_language: None,
        });
        if self.recognizer.is_endpoint(&self.stream) {
            self.recognizer.reset(&self.stream);
            self.endpoint_seen = true;
        }
        Ok(event)
    }

    fn reset(&mut self) {
        self.recognizer.reset(&self.stream);
        self.endpoint_seen = false;
    }

    fn name(&self) -> &'static str {
        "sherpa-onnx"
    }

    fn needs_continuous_audio(&self) -> bool {
        true
    }

    fn take_endpoint(&mut self) -> Option<bool> {
        Some(std::mem::take(&mut self.endpoint_seen))
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

#[cfg(all(test, feature = "sherpa"))]
mod sherpa_tests {
    use super::*;

    #[test]
    fn missing_model_dir_is_model_missing() {
        let dir = std::env::temp_dir().join("ll-asr-no-model-here");
        match SherpaAsr::new(&dir, Language::EnUs) {
            Err(AsrError::ModelMissing { size_mb, language }) => {
                assert_eq!(size_mb, SHERPA_MODEL_SIZE_MB);
                assert_eq!(language, "en-US");
            }
            other => panic!("expected ModelMissing, got {other:?}"),
        }
    }

    /// Needs `LL_ASR_MODEL_DIR` and `LL_ASR_SMOKE_WAV` (a FLEURS en_us test
    /// clip; see docs/qa/2026-11-23-asr-runtime.md). `LL_ASR_SMOKE_WORD` is a
    /// word the clip contains (default "meeting", the TTS sentence).
    #[test]
    fn transcribes_tts_clip() {
        let (Some(model_dir), Some(wav)) = (
            std::env::var_os("LL_ASR_MODEL_DIR"),
            std::env::var_os("LL_ASR_SMOKE_WAV"),
        ) else {
            println!("skipped: set LL_ASR_MODEL_DIR and LL_ASR_SMOKE_WAV");
            return;
        };
        let word = std::env::var("LL_ASR_SMOKE_WORD").unwrap_or_else(|_| "meeting".to_string());
        let mut asr = SherpaAsr::new(std::path::PathBuf::from(model_dir), Language::EnUs)
            .expect("model loads");
        let bytes = std::fs::read(std::path::PathBuf::from(wav)).expect("wav");
        let samples = decode_wav_16k_mono(&bytes);
        let mut transcript = String::new();
        let mut last = String::new();
        for (index, chunk) in samples.chunks(17_600).enumerate() {
            let time = (index + 1) as f64 * 1.1;
            if let Some(event) = asr.process(chunk, time).unwrap() {
                if !event.hypothesis.starts_with(&last) && !last.is_empty() {
                    transcript.push_str(&last);
                    transcript.push(' ');
                }
                last = event.hypothesis;
            }
        }
        transcript.push_str(&last);
        transcript.push(' ');
        transcript.push_str(&asr.finish());
        println!(
            "transcript words={} samples={}",
            transcript.split_whitespace().count(),
            samples.len()
        );
        assert!(
            transcript.to_lowercase().contains(&word.to_lowercase()),
            "expected word not recognized"
        );
    }

    /// Minimal WAV reader for the test: 16 kHz mono, PCM16 or float32
    /// (FLEURS clips are float32).
    fn decode_wav_16k_mono(bytes: &[u8]) -> Vec<f32> {
        let mut float = false;
        let mut offset = 12;
        while offset + 8 <= bytes.len() {
            let id = &bytes[offset..offset + 4];
            let size =
                u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap()) as usize;
            if id == b"fmt " {
                let format = u16::from_le_bytes([bytes[offset + 8], bytes[offset + 9]]);
                float = format == 3;
            }
            if id == b"data" {
                let data = &bytes[offset + 8..(offset + 8 + size).min(bytes.len())];
                return if float {
                    data.chunks_exact(4)
                        .map(|quad| f32::from_le_bytes([quad[0], quad[1], quad[2], quad[3]]))
                        .collect()
                } else {
                    data.chunks_exact(2)
                        .map(|pair| i16::from_le_bytes([pair[0], pair[1]]) as f32 / 32_768.0)
                        .collect()
                };
            }
            offset += 8 + size + (size % 2);
        }
        panic!("no data chunk");
    }
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
