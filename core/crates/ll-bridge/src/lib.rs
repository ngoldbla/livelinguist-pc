//! `ll-bridge` — thin napi-rs shell over the pipeline/protocol.
//!
//! napi is a pass-through (D-043): commands and events are the transport-neutral
//! protocol, and events reach JS through a ThreadsafeFunction. No business logic
//! lives here. Audio never crosses the FFI boundary.

use ll_asr::{AsrEngine, OffAsr, ScriptedAsr};
use ll_audio::{AudioSource, SyntheticSource};
use ll_config::{
    Backend, ConfigLayer, LlmFault, RuntimeConfig, RuntimeStatus, Scenario, WindowMode,
};
use ll_core::coverage::CoverageMatrix;
use ll_core::demo::DemoConversation;
use ll_core::engine::SimplifierBackend;
use ll_core::language::Language;
use ll_core::pack::FrameworkPack;
use ll_core::session::{AudioSourceKind, SessionConfiguration, SimplificationMode};
use ll_llm::{DemoBackend, FaultyBackend};
use ll_pipeline::{Pipeline, PipelineConfig};
use ll_window::{
    apply_for_current_window, begin_drag, resize_current_window, work_area, WindowPolicyResult,
};
use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[cfg(feature = "capture")]
use std::time::{Duration, Instant};

/// How often the loopback feeder checks whether the default output changed.
#[cfg(feature = "capture")]
const DEVICE_CHECK_INTERVAL: Duration = Duration::from_secs(2);
/// How long the feeder keeps trying to reopen the default output.
#[cfg(feature = "capture")]
const DEVICE_RETRY_WINDOW: Duration = Duration::from_secs(10);
#[cfg(feature = "capture")]
const DEVICE_RETRY_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StartRequest {
    language: String,
    #[serde(default = "default_source")]
    source: String,
    #[serde(default = "default_mode")]
    mode: String,
    #[serde(default)]
    detection: bool,
    #[serde(default = "default_scenario")]
    scenario: String,
}

fn default_source() -> String {
    "demo".to_string()
}

fn default_mode() -> String {
    "on_demand".to_string()
}

fn default_scenario() -> String {
    "mock".to_string()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionSummary {
    session_id: String,
    language: String,
    segments: usize,
    simplified: usize,
    duration_s: f64,
    overrun_count: u64,
    last_seq: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SimplifyResult {
    id: String,
    text: Option<String>,
    status: String,
    flags: Vec<String>,
}

struct BridgeInner {
    pipeline: Option<Pipeline>,
    llm_status: Option<ll_llm::LoadStatus>,
    pack: Option<FrameworkPack>,
    callback: Option<Arc<ThreadsafeFunction<String>>>,
    coverage: CoverageMatrix,
    runtime: RuntimeConfig,
    language: Language,
    last_error: Option<String>,
    flood_stop: Option<Arc<AtomicBool>>,
}

impl Default for BridgeInner {
    fn default() -> Self {
        Self {
            pipeline: None,
            llm_status: None,
            pack: None,
            callback: None,
            coverage: CoverageMatrix::v1(),
            runtime: RuntimeConfig::default(),
            language: Language::FrFr,
            last_error: None,
            flood_stop: None,
        }
    }
}

#[napi]
pub struct CoreClient {
    inner: Arc<Mutex<BridgeInner>>,
    downloads: ll_models::DownloadSlot,
}

#[napi]
impl CoreClient {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(BridgeInner::default())),
            downloads: ll_models::DownloadSlot::default(),
        }
    }

    /// Start a session. `onEvent` receives one JSON `Envelope` per call.
    #[napi]
    pub fn start(&self, request_json: String, on_event: ThreadsafeFunction<String>) -> Result<()> {
        let request: StartRequest = serde_json::from_str(&request_json)
            .map_err(|error| Error::from_reason(format!("invalid start request: {error}")))?;
        if let Err(reason) = preflight_source(&request.source) {
            return Err(Error::from_reason(reason));
        }
        let callback = Arc::new(on_event);
        let language = Language::from_locale(&request.language)
            .ok_or_else(|| Error::from_reason(format!("unknown language: {}", request.language)))?;
        let scenario = Scenario::parse(&request.scenario).unwrap_or(Scenario::Mock);
        let mut runtime = RuntimeConfig::default();
        scenario.apply(&mut runtime);
        if request.mode == "live" {
            runtime.pipeline = ll_config::PipelineMode::Live;
        }
        let pack = FrameworkPack::for_language(language)
            .map_err(|error| Error::from_reason(error.to_string()))?;
        let conversation = DemoConversation::from_pack(&pack, 4);
        let source_kind = match request.source.as_str() {
            "demo" => AudioSourceKind::Demo,
            "silence" => AudioSourceKind::Silence,
            "monitor" => AudioSourceKind::SystemAudio,
            _other => AudioSourceKind::Demo,
        };
        // Real ASR for real audio: with the `sherpa` feature, system audio
        // (or any non-mock scenario) uses the pinned model. A missing model is
        // an ASR_MODEL_MISSING error; it never falls back to scripted text.
        let real_asr = use_real_asr(&request.source, scenario);
        #[cfg(feature = "sherpa")]
        let sherpa_asr: Option<Box<dyn AsrEngine>> = if real_asr {
            match sherpa_engine(language) {
                Ok(engine) => Some(engine),
                Err((code, message)) => {
                    emit_error_code(&callback, code, &message, code == "ASR_MODEL_MISSING");
                    return Err(Error::from_reason(format!("{code}: {message}")));
                }
            }
        } else {
            None
        };
        #[cfg(not(feature = "sherpa"))]
        let sherpa_asr: Option<Box<dyn AsrEngine>> = None;
        #[cfg(feature = "capture")]
        let capture = if matches!(source_kind, AudioSourceKind::SystemAudio) {
            match ll_audio::capture::LoopbackCapture::start_default() {
                Ok(capture) => Some(capture),
                Err(error) => {
                    emit_error(&callback, "AUDIO_SOURCE_UNAVAILABLE", &error.to_string());
                    return Err(Error::from_reason(format!(
                        "system audio capture failed: {error}"
                    )));
                }
            }
        } else {
            None
        };
        let mut session = SessionConfiguration::new(language, source_kind);
        session.simplification_mode = match request.mode.as_str() {
            "live" => SimplificationMode::Automatic,
            _ => SimplificationMode::OnDemand,
        };
        session.language_detection_enabled = request.detection;
        let mut pipeline_config = PipelineConfig::new(session);
        // 1.0 is click-only. Automatic simplification is a dev-only mode,
        // enabled only when the developer sets LL_MODE=live.
        pipeline_config.dev_automatic =
            request.mode == "live" && matches!(std::env::var("LL_MODE").as_deref(), Ok("live"));
        pipeline_config.detection = request.detection;
        pipeline_config.audio_fault = runtime.fault_audio;
        let asr: Box<dyn AsrEngine> = match (sherpa_asr, runtime.asr) {
            (Some(engine), _) => engine,
            (None, ll_config::AsrMode::Off) => Box::new(OffAsr),
            (None, _) if real_asr => {
                // Built without sherpa: real audio gets no fake captions.
                Box::new(OffAsr)
            }
            (None, _) => Box::new(ScriptedAsr::from_conversation(&conversation, 4)),
        };
        // Real simplifier for real sessions: with the `llama` feature the
        // pinned GGUF loads lazily on the first click, never at start.
        let (inner_backend, llm_status): (Box<dyn SimplifierBackend>, Option<ll_llm::LoadStatus>) =
            match lazy_llama(real_asr, &pack, runtime.threads) {
                Some(lazy) => {
                    pipeline_config.engine =
                        ll_core::engine::SimplifyConfig::on_demand_fine_tuned();
                    let status = lazy.status();
                    (Box::new(lazy), Some(status))
                }
                None => (Box::new(DemoBackend::from_pack(&pack)), None),
            };
        let backend: Box<dyn SimplifierBackend> = if runtime.fault_llm == LlmFault::None {
            inner_backend
        } else {
            Box::new(FaultyBackend::new(inner_backend, runtime.fault_llm))
        };
        let mut pipeline = Pipeline::new(
            pipeline_config,
            pack.clone(),
            CoverageMatrix::v1(),
            runtime.clone(),
            asr,
            backend,
        );
        pipeline
            .start()
            .map_err(|error| Error::from_reason(error.to_string()))?;
        let events = pipeline.take_events();
        {
            let mut inner = self
                .inner
                .lock()
                .map_err(|_| Error::from_reason("bridge lock poisoned"))?;
            inner.pipeline = Some(pipeline);
            inner.llm_status = llm_status;
            inner.pack = Some(pack);
            inner.callback = Some(callback.clone());
            inner.runtime = runtime;
            inner.language = language;
            inner.last_error = None;
        }
        emit(callback.as_ref(), events);
        #[cfg(feature = "capture")]
        if let Some(capture) = capture {
            start_monitor_feeder(Arc::clone(&self.inner), capture);
        }
        Ok(())
    }

    /// Feed the built-in demo conversation through the pipeline.
    #[napi]
    pub fn feed_demo(&self) -> Result<()> {
        let (events, callback) = {
            let mut inner = self
                .inner
                .lock()
                .map_err(|_| Error::from_reason("bridge lock poisoned"))?;
            let pack = inner
                .pack
                .clone()
                .ok_or_else(|| Error::from_reason("session has no pack"))?;
            let callback = inner.callback.clone();
            let Some(pipeline) = inner.pipeline.as_mut() else {
                return Err(Error::from_reason("session not started"));
            };
            let conversation = DemoConversation::from_pack(&pack, 4);
            let mut source = SyntheticSource::from_conversation(&conversation, 1, false);
            let mut events = Vec::new();
            while let Some(buffer) = source.next_buffer() {
                events.extend(pipeline.feed_audio(&buffer));
            }
            events.extend(pipeline.flush());
            (events, callback)
        };
        if let Some(callback) = callback {
            emit(&callback, events);
        }
        Ok(())
    }

    #[napi]
    pub fn stop(&self) -> Result<String> {
        let (summary, callback, events) = {
            let mut inner = self
                .inner
                .lock()
                .map_err(|_| Error::from_reason("bridge lock poisoned"))?;
            let Some(mut pipeline) = inner.pipeline.take() else {
                return Err(Error::from_reason("session not started"));
            };
            let store = pipeline.stop();
            let summary = SessionSummary {
                session_id: pipeline.session_id().to_string(),
                language: store.configuration.language.locale().to_string(),
                segments: store.segments.len(),
                simplified: store
                    .segments
                    .iter()
                    .filter(|entry| entry.outcome.display_text().is_some())
                    .count(),
                duration_s: store.duration_seconds.unwrap_or(0.0),
                overrun_count: pipeline.snapshot().overrun_count,
                last_seq: pipeline.snapshot().last_seq,
            };
            (summary, inner.callback.take(), pipeline.take_events())
        };
        if let Some(callback) = callback {
            emit(&callback, events);
        }
        serde_json::to_string(&summary).map_err(|error| Error::from_reason(error.to_string()))
    }

    #[napi]
    pub fn pause(&self) -> Result<()> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| Error::from_reason("bridge lock poisoned"))?;
        if let Some(pipeline) = inner.pipeline.as_mut() {
            pipeline.pause();
        }
        Ok(())
    }

    #[napi]
    pub fn resume(&self) -> Result<()> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| Error::from_reason("bridge lock poisoned"))?;
        if let Some(pipeline) = inner.pipeline.as_mut() {
            pipeline.resume();
        }
        Ok(())
    }

    #[napi]
    pub fn simplify(&self, segment_id: String) -> Result<String> {
        let (result, callback, events) = {
            let mut inner = self
                .inner
                .lock()
                .map_err(|_| Error::from_reason("bridge lock poisoned"))?;
            let callback = inner.callback.clone();
            let Some(pipeline) = inner.pipeline.as_mut() else {
                return Err(Error::from_reason("session not started"));
            };
            let result = pipeline
                .simplify_now(&segment_id)
                .map_err(|error| Error::from_reason(error.to_string()))?;
            let load_error = inner
                .llm_status
                .as_ref()
                .and_then(ll_llm::LoadStatus::take_error);
            let strict = inner.runtime.strict;
            let Some(pipeline) = inner.pipeline.as_mut() else {
                return Err(Error::from_reason("session not started"));
            };
            if let Some(error) = load_error {
                match ll_llm::click_failure(strict, llm_load_missing(&error)) {
                    ll_llm::ClickFailure::Verbatim { code } => {
                        pipeline.emit_event(ll_protocol::CoreEvent::Error {
                            code: code.to_string(),
                            message: error.message.clone(),
                            recoverable: true,
                        });
                    }
                    ll_llm::ClickFailure::StopSession { code } => {
                        let envelope = pipeline.emit_event(ll_protocol::CoreEvent::Error {
                            code: code.to_string(),
                            message: error.message.clone(),
                            recoverable: false,
                        });
                        let _ = pipeline.stop();
                        let mut events = pipeline.take_events();
                        events.insert(0, envelope);
                        if let Some(callback) = &callback {
                            emit(callback, events);
                        }
                        inner.pipeline = None;
                        return Err(Error::from_reason(format!(
                            "{code}: strict scenario stops on simplifier failure"
                        )));
                    }
                }
            }
            let simplified = SimplifyResult {
                id: result.segment.id.clone(),
                text: result.outcome.display_text().map(ToString::to_string),
                status: if result.confidence.is_clean() {
                    "clean".to_string()
                } else {
                    "flagged".to_string()
                },
                flags: result.confidence.reasons().to_vec(),
            };
            (simplified, callback, pipeline.take_events())
        };
        if let Some(callback) = callback {
            emit(&callback, events);
        }
        serde_json::to_string(&result).map_err(|error| Error::from_reason(error.to_string()))
    }

    /// Simplify typed/pasted text (the simplify-only locale path). This never
    /// crosses audio over FFI; it is the same engine and pack as a session.
    #[napi]
    pub fn simplify_text(&self, text: String) -> Result<String> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| Error::from_reason("bridge lock poisoned"))?;
        let language = inner.language;
        let pack = FrameworkPack::for_language(language)
            .map_err(|error| Error::from_reason(error.to_string()))?;
        let backend = DemoBackend::from_pack(&pack).with_latency(std::time::Duration::ZERO);
        let mut engine = ll_core::engine::SimplificationEngine::with_config(
            Box::new(backend),
            &pack,
            ll_core::engine::SimplifyConfig::on_demand(),
        );
        let segment = ll_core::session::FinalizedSegment::new(text, 0.0, 1.0);
        let result = engine.simplify(&segment, &[], false);
        let simplified = SimplifyResult {
            id: result.segment.id.clone(),
            text: result.outcome.display_text().map(ToString::to_string),
            status: if result.outcome.display_text().is_some() {
                if result.confidence.is_clean() {
                    "clean".to_string()
                } else {
                    "flagged".to_string()
                }
            } else {
                "verbatim".to_string()
            },
            flags: result.confidence.reasons().to_vec(),
        };
        serde_json::to_string(&simplified).map_err(|error| Error::from_reason(error.to_string()))
    }

    #[napi]
    pub fn cancel_simplify(&self, segment_id: String) -> Result<bool> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| Error::from_reason("bridge lock poisoned"))?;
        Ok(inner
            .pipeline
            .as_mut()
            .map(|pipeline| pipeline.cancel_simplify(&segment_id))
            .unwrap_or(false))
    }

    #[napi]
    pub fn snapshot(&self) -> Result<String> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| Error::from_reason("bridge lock poisoned"))?;
        let snapshot = match inner.pipeline.as_ref() {
            Some(pipeline) => pipeline.snapshot(),
            None => ll_protocol::Snapshot {
                state: ll_protocol::SessionState::Setup,
                last_seq: 0,
                session_id: None,
                segments: Vec::new(),
                partial: None,
                reversion_active: false,
                overrun_count: 0,
                asr_status: inner.coverage.status(inner.language).as_str().to_string(),
                language: inner.language.locale().to_string(),
            },
        };
        serde_json::to_string(&snapshot).map_err(|error| Error::from_reason(error.to_string()))
    }

    #[napi]
    pub fn runtime_status(&self) -> Result<String> {
        let ui = ll_accel::detect_ui();
        let inner = self
            .inner
            .lock()
            .map_err(|_| Error::from_reason("bridge lock poisoned"))?;
        let mut status = match inner.pipeline.as_ref() {
            Some(pipeline) => pipeline.runtime_status(),
            None => RuntimeStatus::requested(&inner.runtime),
        };
        status.ui = ui;
        serde_json::to_string(&status).map_err(|error| Error::from_reason(error.to_string()))
    }

    #[napi]
    pub fn drain_events(&self) -> Result<String> {
        let (events, callback) = {
            let mut inner = self
                .inner
                .lock()
                .map_err(|_| Error::from_reason("bridge lock poisoned"))?;
            let events = inner
                .pipeline
                .as_mut()
                .map(Pipeline::take_events)
                .unwrap_or_default();
            (events, inner.callback.clone())
        };
        if let Some(callback) = callback {
            emit(&callback, events.clone());
        }
        serde_json::to_string(&events).map_err(|error| Error::from_reason(error.to_string()))
    }

    #[napi]
    pub fn export(&self, format: String, path: String) -> Result<String> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| Error::from_reason("bridge lock poisoned"))?;
        let pipeline = inner
            .pipeline
            .as_ref()
            .ok_or_else(|| Error::from_reason("session not started"))?;
        let contents = match format.as_str() {
            "markdown" | "md" => pipeline.export_markdown(),
            "plain" | "txt" => pipeline.export_plain_text(),
            "csv" | "diagnostics" => pipeline.export_csv(false),
            other => {
                return Err(Error::from_reason(format!(
                    "unknown export format: {other}"
                )))
            }
        };
        std::fs::write(&path, contents)
            .map_err(|error| Error::from_reason(format!("write {path}: {error}")))?;
        Ok(path)
    }

    #[napi]
    pub fn set_window_mode(&self, mode: String, title_nonce: String) -> Result<String> {
        let mode = match mode.as_str() {
            "console" => WindowMode::Console,
            "overlay" => WindowMode::Overlay,
            "expanded" => WindowMode::Expanded,
            other => return Err(Error::from_reason(format!("unknown window mode: {other}"))),
        };
        let result: WindowPolicyResult = apply_for_current_window(mode, &title_nonce);
        serde_json::to_string(&result).map_err(|error| Error::from_reason(error.to_string()))
    }

    /// Work area of the monitor the app opens on, as JSON
    /// `{ x, y, width, height, scale }` (physical pixels) or `null`. Works
    /// before a session starts.
    #[napi]
    pub fn work_area(&self) -> String {
        work_area_json()
    }

    /// Resize the app window to a logical size (the UI fits it to the work
    /// area first). Returns `false` when no native resize path applies.
    #[napi]
    pub fn resize_window(&self, title_nonce: String, width: u32, height: u32) -> bool {
        resize_current_window(&title_nonce, width, height)
    }

    /// Which pinned models are in the cache: JSON
    /// `[{ id, title, bytes, present, verified }]` (sizes only, no hashing).
    #[napi]
    pub fn model_status(&self) -> String {
        model_status_json(&ll_models::cache_root())
    }

    /// Download the pinned model set on a worker thread; returns at once.
    /// Envelopes `model_progress` (≤ 4/s per file), `model_ready`, and
    /// `model_error` reach `onEvent`. A second call while one runs is an error.
    #[napi]
    pub fn download_models(&self, on_event: ThreadsafeFunction<String>) -> Result<()> {
        let ticket = begin_download(&self.downloads).map_err(Error::from_reason)?;
        let callback = Arc::new(on_event);
        std::thread::spawn(move || {
            let mut emitter = ll_protocol::EventEmitter::new();
            let mut send = |event: ll_models::DownloadEvent| {
                emit(&callback, vec![emitter.emit(download_event(event))]);
            };
            let root = ll_models::cache_root();
            let catalog = ll_models::ModelCatalog::plan_default();
            match ll_models::http_client() {
                Ok(client) => {
                    let _ = ll_models::download_catalog(
                        &client,
                        &catalog,
                        &root,
                        Some(ticket.cancel_flag()),
                        4,
                        &mut send,
                    );
                }
                Err(error) => send(ll_models::DownloadEvent::Error {
                    id: catalog
                        .artifacts
                        .first()
                        .map(|artifact| artifact.id.clone())
                        .unwrap_or_default(),
                    code: error.code().to_string(),
                    reason: error.to_string(),
                }),
            }
            drop(ticket);
        });
        Ok(())
    }

    /// Ask the running download to stop between chunks. Partial files stay
    /// for resume. Returns false when nothing is downloading.
    #[napi]
    pub fn cancel_download(&self) -> bool {
        self.downloads.cancel()
    }

    /// Start a native window move from the overlay drag grip.
    #[napi]
    pub fn begin_drag(&self) -> bool {
        begin_drag()
    }

    #[napi]
    pub fn list_audio_sources(&self) -> Result<String> {
        let sources = serde_json::json!([
            {"id": "demo", "label": "Demo conversation", "kind": "demo", "available": true},
            {"id": "silence", "label": "Synthetic silence", "kind": "silence", "available": true},
            {"id": "file", "label": "Audio file", "kind": "file", "available": true},
            {"id": "mic", "label": "Microphone", "kind": "mic", "available": false, "note": "post-1.0"},
            monitor_source_entry()
        ]);
        Ok(sources.to_string())
    }

    #[napi]
    pub fn apply_runtime_config(&self, layer_json: String) -> Result<String> {
        let layer: ConfigLayer = serde_json::from_str(&layer_json)
            .map_err(|error| Error::from_reason(format!("invalid config layer: {error}")))?;
        let ui = ll_accel::detect_ui();
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| Error::from_reason("bridge lock poisoned"))?;
        layer.apply(&mut inner.runtime);
        let mut status = RuntimeStatus::requested(&inner.runtime);
        status.ui = ui;
        serde_json::to_string(&status).map_err(|error| Error::from_reason(error.to_string()))
    }

    #[napi]
    pub fn apply_scenario(&self, scenario: String) -> Result<String> {
        let parsed = Scenario::parse(&scenario)
            .ok_or_else(|| Error::from_reason(format!("unknown scenario: {scenario}")))?;
        let ui = ll_accel::detect_ui();
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| Error::from_reason("bridge lock poisoned"))?;
        parsed.apply(&mut inner.runtime);
        let mut status = RuntimeStatus::requested(&inner.runtime);
        status.ui = ui;
        serde_json::to_string(&status).map_err(|error| Error::from_reason(error.to_string()))
    }

    #[napi]
    pub fn probe_backends(&self) -> Result<String> {
        let report = ll_accel::detect();
        let hardware_gpu = report.adapters.iter().any(|adapter| !adapter.software);
        let vulkan = report.vulkan.available && hardware_gpu;
        let cuda = report.cuda.available && !report.cuda.devices.is_empty();
        let payload = serde_json::json!({
            "backend": "auto",
            "devices": [
                {"id": "cpu", "kind": "cpu", "available": true},
                {"id": "vulkan", "kind": "gpu", "available": vulkan},
                {"id": "cuda", "kind": "gpu", "available": cuda}
            ],
            "mockCalibration": true,
            "ui": report.ui,
            "vulkanDetail": report.vulkan.detail,
            "cudaDetail": report.cuda.detail,
            "cudaDevices": report.cuda.devices
        });
        Ok(payload.to_string())
    }

    /// The measured renderer and accelerator inventory. UI status comes from
    /// DXGI/DRM, never from the target OS; no transcript data is involved.
    #[napi]
    pub fn probe_hardware(&self) -> Result<String> {
        let report = ll_accel::detect();
        serde_json::to_string(&report).map_err(|error| Error::from_reason(error.to_string()))
    }

    #[napi]
    pub fn calibrate(&self) -> Result<String> {
        let ui = ll_accel::detect_ui();
        let inner = self
            .inner
            .lock()
            .map_err(|_| Error::from_reason("bridge lock poisoned"))?;
        let mut status = RuntimeStatus::requested(&inner.runtime);
        status.effective_backend = if inner.runtime.backend == Backend::Auto {
            "cpu".to_string()
        } else {
            inner.runtime.backend.as_str().to_string()
        };
        status.last_calibration = Some("mock calibration (real GGUF pending)".to_string());
        status.fallback_reason = ui.degraded_reason.clone();
        status.ui = ui;
        serde_json::to_string(&status).map_err(|error| Error::from_reason(error.to_string()))
    }

    /// Emit content-free envelopes at a fixed rate through the TSFN. Used by
    /// the S-2/event-delivery harness (100 events/s x 10 min).
    #[napi]
    pub fn delivery_test(&self, events_per_second: u32, seconds: u32) -> Result<()> {
        let (callback, stop) = {
            let mut inner = self
                .inner
                .lock()
                .map_err(|_| Error::from_reason("bridge lock poisoned"))?;
            let callback = inner
                .callback
                .clone()
                .ok_or_else(|| Error::from_reason("start a session first"))?;
            if let Some(existing) = inner.flood_stop.take() {
                existing.store(true, Ordering::Relaxed);
            }
            let stop = Arc::new(AtomicBool::new(false));
            inner.flood_stop = Some(stop.clone());
            (callback, stop)
        };
        let rate = events_per_second.max(1);
        let total = rate.saturating_mul(seconds.max(1));
        let interval_ms = (1000 / rate as u64).max(1);
        std::thread::spawn(move || {
            let mut emitter = ll_protocol::EventEmitter::new();
            for _ in 0..total {
                if stop.load(Ordering::Relaxed) {
                    break;
                }
                let envelope = emitter.emit(ll_protocol::CoreEvent::Log {
                    level: ll_config::LogLevel::Info,
                    code: "delivery_test".to_string(),
                    message: "tick".to_string(),
                });
                if let Ok(json) = serde_json::to_string(&envelope) {
                    let _ = callback.call(Ok(json), ThreadsafeFunctionCallMode::NonBlocking);
                }
                std::thread::sleep(std::time::Duration::from_millis(interval_ms));
            }
        });
        Ok(())
    }

    #[napi]
    pub fn stop_delivery_test(&self) -> Result<()> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| Error::from_reason("bridge lock poisoned"))?;
        if let Some(stop) = inner.flood_stop.take() {
            stop.store(true, Ordering::Relaxed);
        }
        Ok(())
    }

    #[napi]
    pub fn last_error(&self) -> Result<Option<String>> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| Error::from_reason("bridge lock poisoned"))?;
        Ok(inner.last_error.clone())
    }
}

impl Default for CoreClient {
    fn default() -> Self {
        Self::new()
    }
}

fn emit(callback: &ThreadsafeFunction<String>, events: Vec<ll_protocol::Envelope>) {
    for event in events {
        if let Ok(json) = serde_json::to_string(&event) {
            let _ = callback.call(Ok(json), ThreadsafeFunctionCallMode::NonBlocking);
        }
    }
}

#[cfg(feature = "capture")]
fn emit_error(callback: &ThreadsafeFunction<String>, code: &str, message: &str) {
    let mut emitter = ll_protocol::EventEmitter::new();
    let envelope = emitter.emit(ll_protocol::CoreEvent::Error {
        code: code.to_string(),
        message: message.to_string(),
        recoverable: true,
    });
    emit(callback, vec![envelope]);
}

/// `start` can only build the sources this build was compiled with.
fn preflight_source(source: &str) -> std::result::Result<(), String> {
    if source != "monitor" {
        return Ok(());
    }
    #[cfg(feature = "capture")]
    {
        Ok(())
    }
    #[cfg(not(feature = "capture"))]
    {
        Err("system audio capture not built".to_string())
    }
}

fn monitor_source_entry() -> serde_json::Value {
    let available = {
        #[cfg(feature = "capture")]
        {
            ll_audio::capture::default_output_available()
        }
        #[cfg(not(feature = "capture"))]
        {
            false
        }
    };
    if available {
        serde_json::json!({
            "id": "monitor", "label": "System audio", "kind": "monitor", "available": true
        })
    } else {
        serde_json::json!({
            "id": "monitor", "label": "System audio", "kind": "monitor", "available": false,
            "note": "device capture pending"
        })
    }
}

/// Pull loopback buffers into the running pipeline until the session stops.
///
/// When the capture ends or the default output device changes, the capture is
/// dropped, the loss is reported, and the default output is reopened once a
/// second for up to ten seconds. It never falls back to demo audio.
#[cfg(feature = "capture")]
fn start_monitor_feeder(
    inner: Arc<Mutex<BridgeInner>>,
    mut capture: ll_audio::capture::LoopbackCapture,
) {
    let _ = std::thread::Builder::new()
        .name("loopback-feed".into())
        .spawn(move || {
            let mut last_device_check = Instant::now();
            loop {
                match capture.next_buffer() {
                    Some(buffer) => {
                        if !push_capture_audio(&inner, &buffer) {
                            break;
                        }
                    }
                    None => {
                        let Some(reopened) = reopen_capture(&inner, capture, "output device lost")
                        else {
                            stop_after_device_loss(&inner);
                            break;
                        };
                        capture = reopened;
                        last_device_check = Instant::now();
                        continue;
                    }
                }

                if last_device_check.elapsed() < DEVICE_CHECK_INTERVAL {
                    continue;
                }
                last_device_check = Instant::now();
                let current = ll_audio::capture::default_device_name();
                if device_changed(capture.device_name(), current.as_deref()) {
                    let Some(reopened) =
                        reopen_capture(&inner, capture, "default output device changed")
                    else {
                        stop_after_device_loss(&inner);
                        break;
                    };
                    capture = reopened;
                    last_device_check = Instant::now();
                }
            }
        });
}

/// Feed one loopback buffer into the running pipeline; false when it is gone.
#[cfg(feature = "capture")]
fn push_capture_audio(inner: &Arc<Mutex<BridgeInner>>, buffer: &[f32]) -> bool {
    let (events, callback) = {
        let Ok(mut guard) = inner.lock() else {
            return false;
        };
        let Some(pipeline) = guard.pipeline.as_mut() else {
            return false;
        };
        let events = if buffer.is_empty() {
            Vec::new()
        } else {
            pipeline.feed_audio(buffer)
        };
        (events, guard.callback.clone())
    };
    if let Some(callback) = callback {
        emit(&callback, events);
    }
    true
}

/// True when the capture no longer follows the default output device.
#[cfg(feature = "capture")]
fn device_changed(captured: &str, current: Option<&str>) -> bool {
    match current {
        // No default output at all: the device we captured is gone.
        None => true,
        Some(name) => name != captured,
    }
}

/// Report the loss, drop the dead capture, and reopen the default output.
///
/// Returns the new capture, or `None` when nothing reopened within
/// [`DEVICE_RETRY_WINDOW`].
#[cfg(feature = "capture")]
fn reopen_capture(
    inner: &Arc<Mutex<BridgeInner>>,
    stale: ll_audio::capture::LoopbackCapture,
    reason: &str,
) -> Option<ll_audio::capture::LoopbackCapture> {
    emit_capture_event(inner, "AUDIO_DEVICE_LOST", reason);
    drop(stale);

    let deadline = Instant::now() + DEVICE_RETRY_WINDOW;
    while Instant::now() < deadline {
        std::thread::sleep(DEVICE_RETRY_INTERVAL);
        if let Ok(capture) = ll_audio::capture::LoopbackCapture::start_default() {
            emit_capture_event(
                inner,
                "AUDIO_DEVICE_CHANGED",
                &format!("capturing {}", capture.device_name()),
            );
            return Some(capture);
        }
    }
    None
}

/// The device did not come back: stop the session rather than use demo audio.
#[cfg(feature = "capture")]
fn stop_after_device_loss(inner: &Arc<Mutex<BridgeInner>>) {
    emit_capture_event(inner, "AUDIO_DEVICE_LOST", "no output device after 10 s");
    if let Ok(mut guard) = inner.lock() {
        if let Some(mut pipeline) = guard.pipeline.take() {
            let _ = pipeline.stop();
        }
        guard.last_error = Some("system audio device lost".into());
    }
}

/// Loopback status and error events on the running session's callback.
#[cfg(feature = "capture")]
fn emit_capture_event(inner: &Arc<Mutex<BridgeInner>>, code: &str, message: &str) {
    let event = if code == "AUDIO_DEVICE_CHANGED" {
        ll_protocol::CoreEvent::Log {
            level: ll_config::LogLevel::Info,
            code: code.to_string(),
            message: message.to_string(),
        }
    } else {
        ll_protocol::CoreEvent::Error {
            code: code.to_string(),
            message: message.to_string(),
            recoverable: true,
        }
    };
    let (envelope, callback) = {
        let Ok(mut guard) = inner.lock() else {
            return;
        };
        let Some(pipeline) = guard.pipeline.as_mut() else {
            return;
        };
        // Emitting through the pipeline keeps `seq`/`ts` monotonic with the
        // session's other events.
        (pipeline.emit_event(event), guard.callback.clone())
    };
    if let Some(callback) = callback {
        emit(&callback, vec![envelope]);
    }
}

fn model_status_json(root: &std::path::Path) -> String {
    serde_json::to_string(&ll_models::ModelCatalog::plan_default().status(root))
        .unwrap_or_else(|_| "[]".to_string())
}

fn begin_download(
    slot: &ll_models::DownloadSlot,
) -> std::result::Result<ll_models::DownloadTicket, String> {
    slot.try_begin()
        .ok_or_else(|| "a model download is already running".to_string())
}

fn download_event(event: ll_models::DownloadEvent) -> ll_protocol::CoreEvent {
    match event {
        ll_models::DownloadEvent::Progress {
            id,
            file,
            done_bytes,
            total_bytes,
        } => ll_protocol::CoreEvent::ModelProgress {
            id,
            file,
            done_bytes,
            total_bytes,
        },
        ll_models::DownloadEvent::Ready { id } => ll_protocol::CoreEvent::ModelReady { id },
        ll_models::DownloadEvent::Error { id, code, reason } => {
            ll_protocol::CoreEvent::ModelError { id, code, reason }
        }
    }
}

/// Real ASR applies to system audio and to every non-mock scenario. Demo
/// and silence sources under the mock scenario keep the scripted engine.
fn use_real_asr(source: &str, scenario: Scenario) -> bool {
    source == "monitor" || (scenario != Scenario::Mock && source != "demo" && source != "silence")
}

#[cfg(feature = "sherpa")]
fn sherpa_engine(
    language: Language,
) -> std::result::Result<Box<dyn AsrEngine>, (&'static str, String)> {
    let catalog = ll_models::ModelCatalog::plan_default();
    let Some(asr) = catalog
        .artifacts
        .iter()
        .find(|artifact| artifact.role == ll_models::ModelRole::Asr)
    else {
        return Err(("ASR_INIT_FAILED", "no ASR model in the catalog".to_string()));
    };
    let dir = ll_models::model_dir(&ll_models::cache_root(), &asr.repo, &asr.revision);
    match ll_asr::SherpaAsr::new(dir, language) {
        Ok(engine) => Ok(Box::new(engine)),
        Err(error @ ll_asr::AsrError::ModelMissing { .. }) => {
            Err(("ASR_MODEL_MISSING", error.to_string()))
        }
        Err(error) => Err(("ASR_INIT_FAILED", error.to_string())),
    }
}

#[cfg(feature = "sherpa")]
fn emit_error_code(
    callback: &ThreadsafeFunction<String>,
    code: &str,
    message: &str,
    recoverable: bool,
) {
    let mut emitter = ll_protocol::EventEmitter::new();
    let envelope = emitter.emit(ll_protocol::CoreEvent::Error {
        code: code.to_string(),
        message: message.to_string(),
        recoverable,
    });
    emit(callback, vec![envelope]);
}

#[cfg(feature = "llama")]
fn llm_load_missing(error: &ll_core::engine::BackendError) -> bool {
    ll_llm::llama::is_model_missing(error)
}

#[cfg(not(feature = "llama"))]
fn llm_load_missing(_error: &ll_core::engine::BackendError) -> bool {
    false
}

/// With `llama`, real sessions get the GGUF on CPU, loaded on first click.
#[cfg(feature = "llama")]
fn lazy_llama(real: bool, pack: &FrameworkPack, threads: usize) -> Option<ll_llm::LazyBackend> {
    if !real {
        return None;
    }
    let catalog = ll_models::ModelCatalog::plan_default();
    let llm = catalog
        .artifacts
        .iter()
        .find(|artifact| artifact.role == ll_models::ModelRole::Llm)?
        .clone();
    let path = ll_models::model_dir(&ll_models::cache_root(), &llm.repo, &llm.revision)
        .join(&llm.files[0].path);
    let pack = pack.clone();
    let threads = threads.clamp(1, 64) as i32;
    Some(ll_llm::LazyBackend::new(Box::new(move || {
        ll_llm::LlamaBackend::load(&path, threads)
            .map(|backend| Box::new(backend.with_pack(&pack)) as Box<dyn SimplifierBackend>)
    })))
}

#[cfg(not(feature = "llama"))]
fn lazy_llama(_real: bool, _pack: &FrameworkPack, _threads: usize) -> Option<ll_llm::LazyBackend> {
    None
}

// Plain function so unit tests stay off the napi runtime symbols.
fn work_area_json() -> String {
    serde_json::to_string(&work_area()).unwrap_or_else(|_| "null".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_asr_only_for_real_audio_or_non_mock_scenarios() {
        assert!(use_real_asr("monitor", Scenario::Mock));
        assert!(!use_real_asr("demo", Scenario::Mock));
        assert!(!use_real_asr("silence", Scenario::Mock));
        assert!(!use_real_asr("demo", Scenario::TierC));
        assert!(use_real_asr("file", Scenario::TierC));
    }

    #[test]
    fn model_status_on_empty_root_reports_both_missing() {
        let root = tempfile::tempdir().unwrap();
        let value: serde_json::Value =
            serde_json::from_str(&model_status_json(root.path())).unwrap();
        let models = value.as_array().unwrap();
        assert_eq!(models.len(), 2);
        for model in models {
            assert_eq!(model["present"], serde_json::Value::Bool(false));
            assert_eq!(model["verified"], serde_json::Value::Bool(false));
            assert!(model["bytes"].as_u64().unwrap() > 0);
            assert!(model["id"].as_str().is_some() && model["title"].as_str().is_some());
        }
    }

    #[test]
    fn second_download_while_running_is_rejected() {
        let slot = ll_models::DownloadSlot::default();
        let running = begin_download(&slot).expect("first starts");
        assert_eq!(
            begin_download(&slot).unwrap_err(),
            "a model download is already running"
        );
        drop(running);
        assert!(begin_download(&slot).is_ok());
    }

    #[test]
    fn download_events_map_to_enveloped_protocol_events() {
        let mut emitter = ll_protocol::EventEmitter::new();
        let first = emitter.emit(download_event(ll_models::DownloadEvent::Progress {
            id: "asr".into(),
            file: "tokens.txt".into(),
            done_bytes: 10,
            total_bytes: 20,
        }));
        let second = emitter.emit(download_event(ll_models::DownloadEvent::Error {
            id: "asr".into(),
            code: "CANCELLED".into(),
            reason: "download cancelled".into(),
        }));
        assert!(second.seq > first.seq);
        let json = serde_json::to_value(&first).unwrap();
        assert_eq!(json["event"]["type"], "model_progress");
        assert_eq!(json["event"]["done_bytes"], 10);
        assert!(json["v"].is_number() && json["ts"].is_number());
        let json = serde_json::to_value(&second).unwrap();
        assert_eq!(json["event"]["type"], "model_error");
        assert_eq!(json["event"]["code"], "CANCELLED");
    }

    #[test]
    fn work_area_json_is_null_or_an_area_without_a_session() {
        let json = work_area_json();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        if value.is_null() {
            return;
        }
        assert!(value["width"].as_u64().unwrap() > 0);
        assert!(value["height"].as_u64().unwrap() > 0);
        let scale = value["scale"].as_f64().unwrap();
        assert!((1.0..=3.0).contains(&scale));
    }

    #[test]
    fn start_request_defaults_match_the_ui() {
        let request: StartRequest = serde_json::from_str(r#"{"language":"de-DE"}"#).unwrap();
        assert_eq!(request.source, "demo");
        assert_eq!(request.mode, "on_demand");
        assert_eq!(request.scenario, "mock");
        assert!(!request.detection);
    }

    #[test]
    #[cfg(not(feature = "capture"))]
    fn monitor_source_is_unavailable_without_capture() {
        // The napi entry point returns this entry; testing the entry itself keeps
        // the unit test free of napi runtime symbols.
        let monitor = monitor_source_entry();
        assert_eq!(monitor["id"], "monitor");
        assert_eq!(monitor["available"], serde_json::Value::Bool(false));

        // `start` rejects "monitor" with this error before it builds any source,
        // so no demo session is created.
        assert_eq!(
            preflight_source("monitor").unwrap_err(),
            "system audio capture not built"
        );
        assert!(preflight_source("demo").is_ok());
    }

    #[test]
    #[cfg(feature = "capture")]
    fn a_new_default_output_counts_as_a_device_change() {
        assert!(!device_changed("Speakers", Some("Speakers")));
        assert!(device_changed("Speakers", Some("Headphones")));
        // No default output at all means the captured device is gone.
        assert!(device_changed("Speakers", None));
    }

    #[test]
    fn summary_is_content_free_shape() {
        let summary = SessionSummary {
            session_id: "ses-1".into(),
            language: "fr-FR".into(),
            segments: 9,
            simplified: 0,
            duration_s: 47.3,
            overrun_count: 0,
            last_seq: 12,
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"sessionId\""));
        assert!(!json.contains("Bonjour"));
    }
}
