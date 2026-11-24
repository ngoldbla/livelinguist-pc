//! `ll-core` — pure deterministic logic for Live Linguist PC.
//!
//! This crate owns framework packs, prompt construction, decoding, the
//! segmenter, validators, reversion detection, session metrics, demo
//! simplification, and exporters. It has no audio, ML, OS-window, or UI
//! dependency (plan §4.4, §7.1).
//!
//! The implementation is ported from the Mac reference; the Swift tests are
//! the behavioral spec. See `tests/golden/INVENTORY.md` for provenance.

pub mod coverage;
pub mod decoding;
pub mod demo;
pub mod disfluency;
pub mod engine;
pub mod export;
pub mod fidelity;
pub mod language;
pub mod language_guard;
pub mod metrics;
pub mod numbers;
pub mod pack;
pub mod profile;
pub mod prompt;
pub mod reversion;
pub mod rule_checkers;
pub mod segmenter;
pub mod session;
pub mod text_analysis;
pub mod validation;

pub use coverage::{AsrStatus, CoverageMatrix, SIMPLIFY_ONLY_NOTICE};
pub use decoding::{collapse_repeated_sentences, first_line, reached_stop, strip_think_block};
pub use demo::{DemoConversation, DemoSimplifier};
pub use engine::{
    BackendError, BackendErrorKind, ScriptedBackend, SimplificationEngine, SimplifierBackend,
    SimplifyConfig,
};
pub use export::{markdown, plain_text, session_csv};
pub use language::{Framework, Language};
pub use metrics::{LatencyMetrics, LatencyReport, Stage};
pub use pack::{FrameworkPack, PackError, Thresholds};
pub use profile::{uses_fine_tuned, SimplifierProfile};
pub use prompt::{render_easylanguage_prompt, PromptBuilder, SimplificationRequest};
pub use reversion::ReversionDetector;
pub use segmenter::{Segmenter, SegmenterConfig, SegmenterOutput};
pub use session::{
    next_segment_id, AudioSourceKind, CaptionLine, FallbackReason, FinalizedSegment,
    PartialHypothesis, ReversionSpan, SessionConfiguration, SessionStore, SimplificationConfidence,
    SimplificationMode, SimplificationOutcome, SimplifiedSegment, SkipReason,
};
pub use validation::{Severity, ValidatorSuite, Violation};

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
