//! Session pipeline: audio -> VAD/chunker -> ASR -> segmenter -> simplification
//! -> enveloped events (plan §7.6).
//!
//! The default build is synchronous and deterministic so `ll-eval`, tests, and
//! the UI demo can replay a whole session without a device. Live capture feeds
//! the same `feed_audio` entry point from its worker thread.

use ll_asr::{AsrEngine, AsrError};
use ll_audio::{AudioChunker, VoiceActivityDetector};
use ll_config::{AudioFault, RuntimeConfig, RuntimeStatus};
use ll_core::coverage::CoverageMatrix;
use ll_core::engine::{SimplificationEngine, SimplifyConfig};
use ll_core::export;
use ll_core::metrics::{LatencyMetrics, Stage};
use ll_core::pack::FrameworkPack;
use ll_core::reversion::ReversionDetector;
use ll_core::segmenter::Segmenter;
use ll_core::session::{
    FallbackReason, FinalizedSegment, PartialHypothesis, SessionConfiguration, SessionStore,
    SimplificationMode, SimplificationOutcome, SimplifiedSegment,
};
use ll_core::validation::Severity;
use ll_llm::{QueueAction, SimplifyQueue};
use ll_protocol::{
    CoreEvent, Envelope, EventEmitter, SessionState, SimplifiedStatus, Snapshot, SnapshotPartial,
    SnapshotSegment,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("session is not listening")]
    NotListening,
    #[error("unknown segment: {0}")]
    UnknownSegment(String),
    #[error("ASR error: {0}")]
    Asr(#[from] AsrError),
    #[error("pipeline error: {0}")]
    Other(String),
}

/// What a click on a segment did.
#[derive(Debug, Clone, PartialEq)]
pub enum ClickOutcome {
    /// Already simplified; no backend call.
    Cached(SimplifiedSegment),
    /// Became the active request.
    Started { id: String },
    /// Waits in the single queued slot. `superseded` is the waiting request
    /// it replaced: that one is dropped quietly (not an error).
    Queued {
        id: String,
        superseded: Option<String>,
    },
}

/// What running the active request did.
#[derive(Debug, Clone, PartialEq)]
pub enum RunOutcome {
    Done(SimplifiedSegment),
    Cancelled { id: String },
}

/// Shared cancel flags for running requests (usable from another thread).
#[derive(Debug, Clone, Default)]
pub struct SimplifyCancel {
    ids: std::sync::Arc<std::sync::Mutex<std::collections::HashSet<String>>>,
}

impl SimplifyCancel {
    pub fn cancel(&self, id: &str) {
        if let Ok(mut ids) = self.ids.lock() {
            ids.insert(id.to_string());
        }
    }

    fn take(&self, id: &str) -> bool {
        self.ids
            .lock()
            .map(|mut ids| ids.remove(id))
            .unwrap_or(false)
    }

    fn clear(&self, id: &str) {
        let _ = self.take(id);
    }
}

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub session: SessionConfiguration,
    pub engine: SimplifyConfig,
    pub detection: bool,
    pub max_caption_lag: Option<f64>,
    pub audio_fault: AudioFault,
    pub batch_threshold: usize,
    /// Dev-only: lets `SimplificationMode::Automatic` simplify every finalized
    /// segment (stress scenario / `LL_PIPELINE=live`). Off in 1.0 builds of
    /// the app: without it, Automatic behaves like OnDemand (click-only).
    pub dev_automatic: bool,
}

impl PipelineConfig {
    pub fn new(session: SessionConfiguration) -> Self {
        Self {
            session,
            engine: SimplifyConfig::on_demand(),
            detection: false,
            max_caption_lag: None,
            audio_fault: AudioFault::None,
            batch_threshold: 3,
            dev_automatic: false,
        }
    }
}

pub struct Pipeline {
    config: PipelineConfig,
    coverage: CoverageMatrix,
    asr: Box<dyn AsrEngine>,
    engine: SimplificationEngine,
    queue: SimplifyQueue,
    cancel_handle: SimplifyCancel,
    segmenter: Segmenter,
    vad: VoiceActivityDetector,
    chunker: AudioChunker,
    reversion: ReversionDetector,
    store: SessionStore,
    metrics: LatencyMetrics,
    emitter: EventEmitter,
    events: Vec<Envelope>,
    state: SessionState,
    session_id: String,
    audio_time: f64,
    utterance: u64,
    active_partial: Option<PartialHypothesis>,
    overrun_count: u64,
    overrun_simulated: bool,
    first_partial_audio: Option<f64>,
    runtime: RuntimeConfig,
    wall_start: std::time::Instant,
}

static NEXT_SESSION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl Pipeline {
    pub fn new(
        config: PipelineConfig,
        pack: FrameworkPack,
        coverage: CoverageMatrix,
        runtime: RuntimeConfig,
        asr: Box<dyn AsrEngine>,
        backend: Box<dyn ll_core::engine::SimplifierBackend>,
    ) -> Self {
        let session_id = format!(
            "ses-{:06}",
            NEXT_SESSION.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        );
        let reversion = ReversionDetector::new(config.session.language.base_code());
        let engine = SimplificationEngine::with_config(backend, &pack, config.engine);
        Self {
            store: SessionStore::new(config.session.clone()),
            config,
            coverage,
            asr,
            engine,
            queue: SimplifyQueue::default(),
            cancel_handle: SimplifyCancel::default(),
            segmenter: Segmenter::default(),
            vad: VoiceActivityDetector::default(),
            chunker: AudioChunker::new(),
            reversion,
            metrics: LatencyMetrics::new(),
            emitter: EventEmitter::new(),
            events: Vec::new(),
            state: SessionState::Setup,
            session_id,
            audio_time: 0.0,
            utterance: 0,
            active_partial: None,
            overrun_count: 0,
            overrun_simulated: false,
            first_partial_audio: None,
            runtime,
            wall_start: std::time::Instant::now(),
        }
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn state(&self) -> SessionState {
        self.state
    }

    pub fn metrics(&self) -> &LatencyMetrics {
        &self.metrics
    }

    pub fn store(&self) -> &SessionStore {
        &self.store
    }

    pub fn audio_time(&self) -> f64 {
        self.audio_time
    }

    pub fn start(&mut self) -> Result<(), PipelineError> {
        self.state = SessionState::Listening;
        self.store.started_at_iso = Some(iso_now());
        self.emit(CoreEvent::State {
            state: self.state,
            session_id: self.session_id.clone(),
        });
        Ok(())
    }

    pub fn pause(&mut self) {
        if self.state == SessionState::Listening {
            self.state = SessionState::Paused;
            self.emit(CoreEvent::State {
                state: self.state,
                session_id: self.session_id.clone(),
            });
        }
    }

    pub fn resume(&mut self) {
        if self.state == SessionState::Paused {
            self.state = SessionState::Listening;
            self.emit(CoreEvent::State {
                state: self.state,
                session_id: self.session_id.clone(),
            });
        }
    }

    /// Feed 16 kHz mono f32 samples. Returns the events emitted by this call.
    pub fn feed_audio(&mut self, samples: &[f32]) -> Vec<Envelope> {
        if self.state != SessionState::Listening {
            return Vec::new();
        }
        self.audio_time += samples.len() as f64 / ll_audio::TARGET_SAMPLE_RATE as f64;
        let status = self.vad.process(samples);

        if self.config.audio_fault == AudioFault::Overrun && !self.overrun_simulated {
            self.overrun_simulated = true;
            self.note_overrun(1, 500, "batch");
        }

        for chunk in self.chunker.append(samples) {
            // VAD gates feeding ASR to save CPU during silence (plan §7.3).
            if !status.is_speech_active {
                continue;
            }
            match self.asr.process(&chunk, self.audio_time) {
                Ok(Some(event)) => {
                    let output = self.segmenter.ingest(
                        &event.hypothesis,
                        self.audio_time,
                        if self.config.detection {
                            event.detected_language.as_deref()
                        } else {
                            None
                        },
                    );
                    self.handle_segmenter_output(output);
                }
                Ok(None) => {}
                Err(error) => {
                    let (code, message, recoverable) = match &error {
                        AsrError::ModelMissing { language, size_mb } => (
                            "ASR_MODEL_MISSING",
                            format!("model for {language} missing ({size_mb} MB)"),
                            true,
                        ),
                        AsrError::InitFailed(message) => {
                            ("ASR_INIT_FAILED", message.clone(), false)
                        }
                        AsrError::Unavailable(message) => {
                            ("ASR_INIT_FAILED", message.clone(), true)
                        }
                    };
                    self.emit(CoreEvent::Error {
                        code: code.to_string(),
                        message,
                        recoverable,
                    });
                }
            }
        }

        let output = self
            .segmenter
            .tick(status.trailing_silence, self.audio_time);
        self.handle_segmenter_output(output);
        self.take_events_internal()
    }

    pub fn flush(&mut self) -> Vec<Envelope> {
        if let Some(segment) = self.segmenter.flush(self.audio_time) {
            self.accept_segment(segment);
        }
        self.take_events_internal()
    }

    pub fn stop(&mut self) -> SessionStore {
        let _ = self.flush();
        self.reversion.finish();
        self.store.reversion_spans = self.reversion.spans().to_vec();
        self.store.ended_at_iso = Some(iso_now());
        self.store.duration_seconds = Some(self.audio_time);
        self.state = SessionState::Summary;
        self.emit(CoreEvent::State {
            state: self.state,
            session_id: self.session_id.clone(),
        });
        self.store.clone()
    }

    /// Click on a segment: the only way simplification starts in 1.0.
    ///
    /// Rules (D-041, T19): a cached result returns at once without a backend
    /// call; a click starts at once when nothing runs; otherwise it waits in
    /// the single queued slot, and a newer click supersedes the waiting one.
    pub fn request_simplify(&mut self, id: &str) -> Result<ClickOutcome, PipelineError> {
        let entry = self
            .store
            .find(id)
            .ok_or_else(|| PipelineError::UnknownSegment(id.to_string()))?;
        if entry.outcome.display_text().is_some() {
            return Ok(ClickOutcome::Cached(entry.clone()));
        }
        if self.queue.active() == Some(id) {
            return Ok(ClickOutcome::Started { id: id.to_string() });
        }
        if self.queue.queued() == Some(id) {
            return Ok(ClickOutcome::Queued {
                id: id.to_string(),
                superseded: None,
            });
        }
        self.cancel_handle.clear(id);
        Ok(match self.queue.request(id) {
            QueueAction::Started(id) => ClickOutcome::Started { id },
            QueueAction::Queued(id) => ClickOutcome::Queued {
                id,
                superseded: None,
            },
            QueueAction::Replaced { replaced, .. } => ClickOutcome::Queued {
                id: id.to_string(),
                superseded: replaced,
            },
            QueueAction::Idle => ClickOutcome::Started { id: id.to_string() },
        })
    }

    /// Run the active item, then advance the queue. Returns `None` when
    /// nothing is active. A cancel that arrives while the backend runs (via
    /// [`SimplifyCancel`]) discards the result.
    pub fn run_active(&mut self) -> Option<RunOutcome> {
        let id = self.queue.active()?.to_string();
        if self.cancel_handle.take(&id) {
            let _ = self.queue.complete();
            return Some(RunOutcome::Cancelled { id });
        }
        let Some(entry) = self.store.find(&id).cloned() else {
            let _ = self.queue.complete();
            return Some(RunOutcome::Cancelled { id });
        };
        if entry.outcome.display_text().is_some() {
            let _ = self.queue.complete();
            return Some(RunOutcome::Done(entry));
        }
        // Click path: only the clicked sentence reaches the simplifier. No
        // rolling context from earlier segments (S4 prompt contract).
        let previous: Vec<String> = Vec::new();
        let started = self.wall_ms();
        self.metrics.record(&id, Stage::SimplifyStart, started);
        let result = self.engine.simplify(&entry.segment, &previous, false);
        if self.cancel_handle.take(&id) {
            let _ = self.queue.complete();
            return Some(RunOutcome::Cancelled { id });
        }
        let finished = self.wall_ms();
        self.metrics.record(&id, Stage::SimplifiedVisible, finished);
        self.store.update_outcome(
            &id,
            result.outcome.clone(),
            result.soft_violations.clone(),
            result.confidence.clone(),
        );
        self.emit_simplified(&result, (finished - started) as u64);
        let _ = self.queue.complete();
        Some(RunOutcome::Done(result))
    }

    /// On-demand simplification of one finalized segment: a click followed by
    /// running the queue until that segment finishes. Returns the updated
    /// segment; emits a `simplified` event when there is displayable text.
    pub fn simplify_now(&mut self, id: &str) -> Result<SimplifiedSegment, PipelineError> {
        if let ClickOutcome::Cached(entry) = self.request_simplify(id)? {
            return Ok(entry);
        }
        while let Some(outcome) = self.run_active() {
            match outcome {
                RunOutcome::Done(result) if result.id() == id => return Ok(result),
                RunOutcome::Cancelled { id: cancelled } if cancelled == id => {
                    return Err(PipelineError::Other("simplification cancelled".into()));
                }
                _ => {}
            }
        }
        Err(PipelineError::Other("simplification superseded".into()))
    }

    pub fn simplify_now_queued(&mut self, id: &str) -> Result<SimplifiedSegment, PipelineError> {
        self.simplify_now(id)
    }

    /// Cancel a queued or active on-demand request. A queued item is removed;
    /// an active item that has not started yet is dropped and the queued item
    /// moves up. To stop an item while the backend runs, use
    /// [`Pipeline::cancel_handle`] from another thread.
    pub fn cancel_simplify(&mut self, id: &str) -> bool {
        self.queue.cancel(id)
    }

    /// Thread-safe handle that cancels a running request; its result is
    /// discarded when the backend returns.
    pub fn cancel_handle(&self) -> SimplifyCancel {
        self.cancel_handle.clone()
    }

    pub fn queue(&self) -> &SimplifyQueue {
        &self.queue
    }

    pub fn snapshot(&self) -> Snapshot {
        let segments = self
            .store
            .segments
            .iter()
            .map(|entry| {
                let (status, flags) = status_and_flags(entry);
                SnapshotSegment {
                    id: entry.segment.id.clone(),
                    text: entry.segment.text.clone(),
                    simplified: entry.outcome.display_text().map(ToString::to_string),
                    status,
                    flags,
                    start_ms: (entry.segment.audio_start * 1000.0) as u64,
                    end_ms: (entry.segment.audio_end * 1000.0) as u64,
                    detected_language: entry.segment.detected_language.clone(),
                }
            })
            .collect();
        Snapshot {
            state: self.state,
            last_seq: self.emitter.last_seq(),
            session_id: if self.state == SessionState::Setup {
                None
            } else {
                Some(self.session_id.clone())
            },
            segments,
            partial: self.active_partial.as_ref().map(|partial| SnapshotPartial {
                stream_id: self.session_id.clone(),
                utterance_id: format!("utt-{:06}", self.utterance),
                text: partial.text.clone(),
            }),
            reversion_active: self.reversion.is_active(),
            overrun_count: self.overrun_count,
            asr_status: self
                .coverage
                .status(self.config.session.language)
                .as_str()
                .to_string(),
            language: self.config.session.language.locale().to_string(),
        }
    }

    pub fn runtime_status(&self) -> RuntimeStatus {
        let mut status = RuntimeStatus::requested(&self.runtime);
        status.strict = self.runtime.strict;
        status.window_policy = self.runtime.window_mode.as_str().to_string();
        status
    }

    pub fn take_events(&mut self) -> Vec<Envelope> {
        self.take_events_internal()
    }

    /// Emit a bridge-level event on the session's sequence.
    ///
    /// The envelope continues the pipeline's `seq`/`ts` numbering, so the UI
    /// does not drop it as stale.
    pub fn emit_event(&mut self, event: CoreEvent) -> Envelope {
        self.emitter.emit(event)
    }

    pub fn export_csv(&self, include_latency: bool) -> String {
        export::session_csv(&self.store, Some(&self.metrics), include_latency)
    }

    pub fn export_markdown(&self) -> String {
        export::markdown(&self.store)
    }

    pub fn export_plain_text(&self) -> String {
        export::plain_text(&self.store)
    }

    fn handle_segmenter_output(&mut self, output: ll_core::segmenter::SegmenterOutput) {
        if let Some(partial) = &output.partial {
            if !partial.text.is_empty() {
                self.active_partial = Some(partial.clone());
                if self.first_partial_audio.is_none() {
                    self.first_partial_audio = Some(partial.audio_time);
                }
                self.emit(CoreEvent::Partial {
                    stream_id: self.session_id.clone(),
                    utterance_id: format!("utt-{:06}", self.utterance),
                    text: partial.text.clone(),
                    confidence: None,
                });
            }
        }
        for segment in output.finalized {
            self.accept_segment(segment);
        }
        if output.partial.is_none() && self.active_partial.is_some() {
            self.active_partial = None;
            self.utterance += 1;
            self.emit(CoreEvent::Partial {
                stream_id: self.session_id.clone(),
                utterance_id: format!("utt-{:06}", self.utterance.saturating_sub(1)),
                text: String::new(),
                confidence: None,
            });
        }
    }

    fn accept_segment(&mut self, segment: FinalizedSegment) {
        // Tier 1 (word-to-glass) on the audio clock: segment start -> the
        // first partial that displayed its words. Tier 2 is wall-clock from
        // simplify start to paint in the auto path.
        self.metrics.record(
            &segment.id,
            Stage::AudioEnd,
            (segment.audio_start - ll_audio::CHUNK_DURATION).max(0.0),
        );
        self.metrics
            .record(&segment.id, Stage::Finalized, self.wall_ms());
        if let Some(first_partial) = self.first_partial_audio.take() {
            self.metrics
                .record(&segment.id, Stage::PartialVisible, first_partial);
        }
        let was_active = self.reversion.is_active();
        let active = self.reversion.observe(&segment);
        if active != was_active {
            self.emit(CoreEvent::Reversion {
                active,
                count: self.reversion.spans().len() as u32 + if active { 1 } else { 0 },
            });
        }
        self.emit(CoreEvent::Segment {
            id: segment.id.clone(),
            text: segment.text.clone(),
            start_ms: (segment.audio_start * 1000.0) as u64,
            end_ms: (segment.audio_end * 1000.0) as u64,
        });
        let automatic = self.config.dev_automatic
            && self.config.session.simplification_mode == SimplificationMode::Automatic;
        match automatic {
            false => {
                self.store.append(SimplifiedSegment::new(
                    segment,
                    SimplificationOutcome::Skipped {
                        reason: ll_core::session::SkipReason::NotRequested,
                    },
                ));
            }
            true => {
                let previous: Vec<String> = Vec::new();
                let started = self.wall_ms();
                self.metrics
                    .record(&segment.id, Stage::SimplifyStart, started);
                let result = self.engine.simplify(&segment, &previous, false);
                let finished = self.wall_ms();
                self.metrics
                    .record(&segment.id, Stage::SimplifiedVisible, finished);
                self.emit_simplified(&result, (finished - started) as u64);
                self.store.append(result);
            }
        }
    }

    fn emit_simplified(&mut self, result: &SimplifiedSegment, latency_ms: u64) {
        let (status, flags) = status_and_flags(result);
        if let Some(text) = result.outcome.display_text() {
            self.emit(CoreEvent::Simplified {
                id: result.segment.id.clone(),
                text: text.to_string(),
                status,
                flags,
                latency_ms,
            });
        } else if matches!(
            result.outcome,
            SimplificationOutcome::VerbatimFallback { .. }
        ) {
            self.emit(CoreEvent::Simplified {
                id: result.segment.id.clone(),
                text: result.segment.text.clone(),
                status: SimplifiedStatus::Verbatim,
                flags,
                latency_ms,
            });
        }
    }

    fn note_overrun(&mut self, count: u64, dropped_ms: u64, recovery: &str) {
        self.overrun_count = self.overrun_count.saturating_add(count);
        self.emit(CoreEvent::AudioOverrun {
            count: self.overrun_count,
            dropped_ms,
            recovery: recovery.to_string(),
        });
        self.metrics.record_degradation(
            "AUDIO_OVERRUN",
            self.audio_time,
            &format!("{dropped_ms} ms dropped; recovery {recovery}"),
        );
    }

    fn wall_ms(&self) -> f64 {
        self.wall_start.elapsed().as_secs_f64() * 1000.0
    }

    fn emit(&mut self, event: CoreEvent) {
        let envelope = self.emitter.emit(event);
        self.events.push(envelope);
    }

    fn take_events_internal(&mut self) -> Vec<Envelope> {
        std::mem::take(&mut self.events)
    }
}

fn status_and_flags(result: &SimplifiedSegment) -> (SimplifiedStatus, Vec<String>) {
    match &result.outcome {
        SimplificationOutcome::Simplified { .. } => {
            if result.confidence.is_clean() {
                (
                    SimplifiedStatus::Clean,
                    result
                        .soft_violations
                        .iter()
                        .filter(|violation| violation.severity == Severity::Soft)
                        .map(|violation| violation.rule.clone())
                        .collect(),
                )
            } else {
                (
                    SimplifiedStatus::Flagged,
                    result.confidence.reasons().to_vec(),
                )
            }
        }
        SimplificationOutcome::VerbatimFallback { reason } => (
            SimplifiedStatus::Verbatim,
            vec![verbatim_reason(*reason).to_string()],
        ),
        SimplificationOutcome::Skipped { .. } => (SimplifiedStatus::Verbatim, Vec::new()),
    }
}

fn verbatim_reason(reason: FallbackReason) -> &'static str {
    reason.as_str()
}

fn iso_now() -> String {
    // Content-free timestamp; avoids a chrono dependency.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("unix:{}", now.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ll_audio::AudioSource;
    use ll_core::demo::DemoConversation;
    use ll_core::language::{Framework, Language};
    use ll_core::session::AudioSourceKind;
    use ll_llm::DemoBackend;

    fn demo_pipeline(mode: SimplificationMode) -> Pipeline {
        let language = Language::FrFr;
        let pack = FrameworkPack::load_framework(Framework::Falc).unwrap();
        let conversation = DemoConversation::from_pack(&pack, 4);
        let asr = ll_asr::ScriptedAsr::from_conversation(&conversation, 4);
        let mut config =
            PipelineConfig::new(SessionConfiguration::new(language, AudioSourceKind::Demo));
        config.session.simplification_mode = mode;
        config.detection = true;
        let backend = DemoBackend::from_pack(&pack).with_latency(std::time::Duration::ZERO);
        Pipeline::new(
            config,
            pack,
            CoverageMatrix::v1(),
            RuntimeConfig::default(),
            Box::new(asr),
            Box::new(backend),
        )
    }

    fn feed_demo(pipeline: &mut Pipeline) {
        let pack = FrameworkPack::load_framework(Framework::Falc).unwrap();
        let conversation = DemoConversation::from_pack(&pack, 4);
        let mut source = ll_audio::SyntheticSource::from_conversation(&conversation, 1, false)
            .with_amplitude(0.3);
        while let Some(buffer) = source.next_buffer() {
            let _ = pipeline.feed_audio(&buffer);
        }
        let _ = pipeline.flush();
    }

    #[test]
    fn on_demand_session_collects_verbatim_segments_without_fake_simplification() {
        let mut pipeline = demo_pipeline(SimplificationMode::OnDemand);
        pipeline.start().unwrap();
        feed_demo(&mut pipeline);
        let _ = pipeline.stop();
        assert!(pipeline.store().segments.len() >= 2);
        assert!(pipeline
            .store()
            .segments
            .iter()
            .all(|entry| entry.outcome.display_text().is_none()));
        let snapshot = pipeline.snapshot();
        assert_eq!(snapshot.state, SessionState::Summary);
        assert!(snapshot.last_seq > 0);
        assert!(pipeline.store().duration_seconds.unwrap() > 0.0);
    }

    #[test]
    fn auto_session_emits_simplified_events_only_in_dev_scenario() {
        let mut pipeline = demo_pipeline(SimplificationMode::Automatic);
        pipeline.config.dev_automatic = true;
        pipeline.start().unwrap();
        let pack = FrameworkPack::load_framework(Framework::Falc).unwrap();
        let conversation = DemoConversation::from_pack(&pack, 4);
        let mut source = ll_audio::SyntheticSource::from_conversation(&conversation, 1, false);
        let mut events = Vec::new();
        while let Some(buffer) = source.next_buffer() {
            events.extend(pipeline.feed_audio(&buffer));
        }
        events.extend(pipeline.flush());
        let _ = pipeline.stop();
        events.extend(pipeline.take_events());
        assert!(events
            .iter()
            .any(|envelope| matches!(envelope.event, CoreEvent::Simplified { .. })));
        assert!(pipeline
            .store()
            .segments
            .iter()
            .any(|entry| entry.outcome.display_text().is_some()));
    }

    #[test]
    fn on_demand_simplify_updates_the_segment() {
        let mut pipeline = demo_pipeline(SimplificationMode::OnDemand);
        pipeline.start().unwrap();
        let pack = FrameworkPack::load_framework(Framework::Falc).unwrap();
        let conversation = DemoConversation::from_pack(&pack, 4);
        let mut source = ll_audio::SyntheticSource::from_conversation(&conversation, 1, false);
        while let Some(buffer) = source.next_buffer() {
            let _ = pipeline.feed_audio(&buffer);
        }
        let _ = pipeline.flush();
        let id = pipeline.store().segments[0].id().to_string();
        let result = pipeline.simplify_now(&id).unwrap();
        assert!(result.outcome.display_text().is_some());
        assert!(pipeline
            .store()
            .find(&id)
            .unwrap()
            .outcome
            .display_text()
            .is_some());
        let events = pipeline.take_events();
        assert!(events.iter().any(|envelope| matches!(
            &envelope.event,
            CoreEvent::Simplified { id: event_id, .. } if event_id == &id
        )));
    }

    #[test]
    fn pause_resume_changes_state() {
        let mut pipeline = demo_pipeline(SimplificationMode::OnDemand);
        pipeline.start().unwrap();
        pipeline.pause();
        assert_eq!(pipeline.state(), SessionState::Paused);
        assert!(pipeline.feed_audio(&[0.0; 1600]).is_empty());
        pipeline.resume();
        assert_eq!(pipeline.state(), SessionState::Listening);
    }

    #[test]
    fn audio_overrun_is_surfaced_and_counted() {
        let mut pipeline = demo_pipeline(SimplificationMode::OnDemand);
        pipeline.config.audio_fault = AudioFault::Overrun;
        pipeline.start().unwrap();
        let events = pipeline.feed_audio(&[0.3; 1600]);
        assert!(events
            .iter()
            .any(|envelope| matches!(envelope.event, CoreEvent::AudioOverrun { count: 1, .. })));
        assert_eq!(pipeline.snapshot().overrun_count, 1);
    }

    #[test]
    fn csv_and_markdown_exports_are_available() {
        let mut pipeline = demo_pipeline(SimplificationMode::OnDemand);
        pipeline.start().unwrap();
        feed_demo(&mut pipeline);
        assert!(pipeline.export_csv(false).starts_with("segment_index,"));
        assert!(pipeline
            .export_markdown()
            .contains("# Live Linguist — Transcript"));
        assert!(!pipeline.export_plain_text().is_empty());
    }

    #[test]
    fn seq_is_monotonic() {
        let mut pipeline = demo_pipeline(SimplificationMode::OnDemand);
        pipeline.start().unwrap();
        let first = pipeline.feed_audio(&[0.3; 1600]);
        let second = pipeline.feed_audio(&[0.3; 1600]);
        let mut all = first;
        all.extend(second);
        let mut last = 0;
        for envelope in all {
            assert!(envelope.seq > last);
            last = envelope.seq;
        }
    }

    /// Pipeline with a recording `MockBackend` and `count` finalized segments
    /// pushed straight into the store.
    fn mock_pipeline(
        mode: SimplificationMode,
        texts: &[&str],
    ) -> (Pipeline, ll_llm::MockRecorder, Vec<String>) {
        let language = Language::FrFr;
        let pack = FrameworkPack::load_framework(Framework::Falc).unwrap();
        let conversation = DemoConversation::from_pack(&pack, 4);
        let asr = ll_asr::ScriptedAsr::from_conversation(&conversation, 4);
        let mut config =
            PipelineConfig::new(SessionConfiguration::new(language, AudioSourceKind::Demo));
        config.session.simplification_mode = mode;
        let backend = ll_llm::MockBackend::from_strings(
            (0..64).map(|index| format!("Phrase simple numéro {index}.")),
        );
        let recorder = backend.recorder();
        let mut pipeline = Pipeline::new(
            config,
            pack,
            CoverageMatrix::v1(),
            RuntimeConfig::default(),
            Box::new(asr),
            Box::new(backend),
        );
        pipeline.start().unwrap();
        let mut ids = Vec::new();
        for (index, text) in texts.iter().enumerate() {
            let segment = FinalizedSegment::new(
                text.to_string(),
                index as f64 * 2.0,
                index as f64 * 2.0 + 1.5,
            );
            ids.push(segment.id.clone());
            pipeline.accept_segment(segment);
        }
        (pipeline, recorder, ids)
    }

    const THREE: [&str; 3] = [
        "Le premier orateur parle du budget annuel.",
        "La deuxième personne répond sur les impôts locaux.",
        "Nous voterons la proposition jeudi prochain.",
    ];

    #[test]
    fn click_sends_only_the_clicked_sentence_without_rolling_context() {
        let (mut pipeline, recorder, ids) = mock_pipeline(SimplificationMode::OnDemand, &THREE);
        let _ = pipeline.simplify_now(&ids[2]);
        // The engine may retry after a validator rejection; every attempt
        // must carry only the clicked sentence.
        assert!(recorder.calls() >= 1);
        for request in recorder.requests() {
            assert_eq!(request.sentence, THREE[2]);
            assert!(!request.user_prompt.contains(THREE[0]));
            assert!(!request.user_prompt.contains(THREE[1]));
        }
        let request = recorder.last_request().unwrap();
        assert!(request.user_prompt.contains(THREE[2]));
        assert!(!request.user_prompt.contains(THREE[0]));
        assert!(!request.user_prompt.contains(THREE[1]));
        assert!(!request.system_prompt.contains(THREE[0]));
        assert!(!request.system_prompt.contains(THREE[1]));
    }

    #[test]
    fn click_starts_at_once_when_idle() {
        let (mut pipeline, recorder, ids) = mock_pipeline(SimplificationMode::OnDemand, &THREE);
        assert_eq!(
            pipeline.request_simplify(&ids[0]).unwrap(),
            ClickOutcome::Started { id: ids[0].clone() }
        );
        assert_eq!(pipeline.queue().active(), Some(ids[0].as_str()));
        assert!(matches!(pipeline.run_active(), Some(RunOutcome::Done(_))));
        assert!(recorder.calls() >= 1);
        assert_eq!(pipeline.queue().active(), None);
    }

    #[test]
    fn click_while_running_queues_and_newer_click_supersedes_queued() {
        let (mut pipeline, recorder, ids) = mock_pipeline(SimplificationMode::OnDemand, &THREE);
        let (a, b, c) = (&ids[0], &ids[1], &ids[2]);
        let _ = pipeline.request_simplify(a).unwrap();
        assert_eq!(
            pipeline.request_simplify(b).unwrap(),
            ClickOutcome::Queued {
                id: b.clone(),
                superseded: None
            }
        );
        assert_eq!(
            pipeline.request_simplify(c).unwrap(),
            ClickOutcome::Queued {
                id: c.clone(),
                superseded: Some(b.clone())
            }
        );
        assert_eq!(pipeline.queue().queued(), Some(c.as_str()));
        // Superseded is quiet: no error event.
        assert!(!pipeline
            .take_events()
            .iter()
            .any(|envelope| matches!(envelope.event, CoreEvent::Error { .. })));
        let mut done = Vec::new();
        while let Some(outcome) = pipeline.run_active() {
            if let RunOutcome::Done(result) = outcome {
                done.push(result.id().to_string());
            }
        }
        assert_eq!(done, vec![a.clone(), c.clone()]);
        let sentences: std::collections::BTreeSet<String> = recorder
            .requests()
            .into_iter()
            .map(|request| request.sentence)
            .collect();
        assert!(sentences.contains(THREE[0]) && sentences.contains(THREE[2]));
        assert!(
            !sentences.contains(THREE[1]),
            "superseded item reached the backend"
        );
        assert!(pipeline
            .store()
            .find(b)
            .unwrap()
            .outcome
            .display_text()
            .is_none());
    }

    /// Backend that cancels its own request mid-run, standing in for a cancel
    /// from the UI thread while the model generates.
    struct CancellingBackend {
        cancel: Option<(SimplifyCancel, String)>,
        calls: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }

    impl ll_core::engine::SimplifierBackend for CancellingBackend {
        fn name(&self) -> &'static str {
            "cancelling"
        }

        fn simplify(
            &mut self,
            _request: &ll_core::prompt::SimplificationRequest,
        ) -> Result<String, ll_core::engine::BackendError> {
            self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if let Some((cancel, id)) = self.cancel.take() {
                cancel.cancel(&id);
            }
            Ok("Phrase simple.".to_string())
        }
    }

    #[test]
    fn cancel_while_running_discards_result_and_starts_queued() {
        let (mut pipeline, _recorder, ids) = mock_pipeline(SimplificationMode::OnDemand, &THREE);
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let pack = FrameworkPack::load_framework(Framework::Falc).unwrap();
        pipeline.engine = SimplificationEngine::with_config(
            Box::new(CancellingBackend {
                cancel: Some((pipeline.cancel_handle(), ids[0].clone())),
                calls: calls.clone(),
            }),
            &pack,
            pipeline.config.engine,
        );
        let _ = pipeline.request_simplify(&ids[0]).unwrap();
        let _ = pipeline.request_simplify(&ids[1]).unwrap();
        let _ = pipeline.take_events();
        assert_eq!(
            pipeline.run_active(),
            Some(RunOutcome::Cancelled { id: ids[0].clone() })
        );
        assert!(pipeline
            .store()
            .find(&ids[0])
            .unwrap()
            .outcome
            .display_text()
            .is_none());
        assert!(!pipeline.take_events().iter().any(|envelope| matches!(
            &envelope.event,
            CoreEvent::Simplified { id, .. } if id == &ids[0]
        )));
        // The queued item moved up and runs next.
        assert_eq!(pipeline.queue().active(), Some(ids[1].as_str()));
        assert!(matches!(pipeline.run_active(), Some(RunOutcome::Done(_))));
        assert!(calls.load(std::sync::atomic::Ordering::SeqCst) >= 2);
    }

    #[test]
    fn cancel_removes_queued_and_unstarted_active() {
        let (mut pipeline, recorder, ids) = mock_pipeline(SimplificationMode::OnDemand, &THREE);
        let _ = pipeline.request_simplify(&ids[0]).unwrap();
        let _ = pipeline.request_simplify(&ids[1]).unwrap();
        assert!(pipeline.cancel_simplify(&ids[1]));
        assert_eq!(pipeline.queue().queued(), None);
        assert!(pipeline.cancel_simplify(&ids[0]));
        assert_eq!(pipeline.queue().active(), None);
        assert!(pipeline.run_active().is_none());
        assert_eq!(recorder.calls(), 0);
        assert!(!pipeline.cancel_simplify("nope"));
    }

    #[test]
    fn click_on_simplified_segment_returns_cache_without_backend_call() {
        let (mut pipeline, recorder, ids) = mock_pipeline(SimplificationMode::OnDemand, &THREE);
        let first = pipeline.simplify_now(&ids[0]).unwrap();
        assert!(first.outcome.display_text().is_some());
        let calls = recorder.calls();
        assert!(calls >= 1);
        match pipeline.request_simplify(&ids[0]).unwrap() {
            ClickOutcome::Cached(entry) => assert_eq!(entry, first),
            other => panic!("expected cached result, got {other:?}"),
        }
        let again = pipeline.simplify_now(&ids[0]).unwrap();
        assert_eq!(again, first);
        assert_eq!(recorder.calls(), calls);
    }

    fn feed_seconds(pipeline: &mut Pipeline, seconds: f64) {
        let pack = FrameworkPack::load_framework(Framework::Falc).unwrap();
        let conversation = DemoConversation::from_pack(&pack, 4);
        while pipeline.audio_time() < seconds {
            let mut source = ll_audio::SyntheticSource::from_conversation(&conversation, 1, false)
                .with_amplitude(0.3);
            while let Some(buffer) = source.next_buffer() {
                let _ = pipeline.feed_audio(&buffer);
                if pipeline.audio_time() >= seconds {
                    break;
                }
            }
        }
        let _ = pipeline.flush();
    }

    #[test]
    fn no_background_work_60s_without_clicks_even_in_automatic_mode_outside_dev_scenario() {
        for mode in [SimplificationMode::OnDemand, SimplificationMode::Automatic] {
            let (mut pipeline, recorder, _ids) = mock_pipeline(mode, &[]);
            feed_seconds(&mut pipeline, 60.0);
            assert!(pipeline.audio_time() >= 60.0);
            assert!(
                !pipeline.store().segments.is_empty(),
                "demo audio must produce finalized segments"
            );
            assert_eq!(recorder.calls(), 0, "mode {mode:?} made backend calls");
        }
    }

    #[test]
    fn unknown_segment_errors() {
        let mut pipeline = demo_pipeline(SimplificationMode::OnDemand);
        assert!(matches!(
            pipeline.simplify_now("nope"),
            Err(PipelineError::UnknownSegment(_))
        ));
    }
}
