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
use ll_window::{apply_for_current_window, WindowPolicyResult};
use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

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
}

#[napi]
impl CoreClient {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(BridgeInner::default())),
        }
    }

    /// Start a session. `onEvent` receives one JSON `Envelope` per call.
    #[napi]
    pub fn start(&self, request_json: String, on_event: ThreadsafeFunction<String>) -> Result<()> {
        let request: StartRequest = serde_json::from_str(&request_json)
            .map_err(|error| Error::from_reason(format!("invalid start request: {error}")))?;
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
        let mut session = SessionConfiguration::new(
            language,
            match request.source.as_str() {
                "demo" => AudioSourceKind::Demo,
                "silence" => AudioSourceKind::Silence,
                _other => AudioSourceKind::Demo,
            },
        );
        let _ = request.source;
        session.simplification_mode = match request.mode.as_str() {
            "live" => SimplificationMode::Automatic,
            _ => SimplificationMode::OnDemand,
        };
        session.language_detection_enabled = request.detection;
        let mut pipeline_config = PipelineConfig::new(session);
        pipeline_config.detection = request.detection;
        pipeline_config.audio_fault = runtime.fault_audio;
        let asr: Box<dyn AsrEngine> = match runtime.asr {
            ll_config::AsrMode::Off => Box::new(OffAsr),
            _ => Box::new(ScriptedAsr::from_conversation(&conversation, 4)),
        };
        let demo = DemoBackend::from_pack(&pack);
        let backend: Box<dyn SimplifierBackend> = if runtime.fault_llm == LlmFault::None {
            Box::new(demo)
        } else {
            Box::new(FaultyBackend::new(Box::new(demo), runtime.fault_llm))
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
        let callback = Arc::new(on_event);
        {
            let mut inner = self
                .inner
                .lock()
                .map_err(|_| Error::from_reason("bridge lock poisoned"))?;
            inner.pipeline = Some(pipeline);
            inner.pack = Some(pack);
            inner.callback = Some(callback.clone());
            inner.runtime = runtime;
            inner.language = language;
            inner.last_error = None;
        }
        emit(callback.as_ref(), events);
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
        let inner = self
            .inner
            .lock()
            .map_err(|_| Error::from_reason("bridge lock poisoned"))?;
        let status = match inner.pipeline.as_ref() {
            Some(pipeline) => pipeline.runtime_status(),
            None => RuntimeStatus::requested(&inner.runtime),
        };
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

    #[napi]
    pub fn list_audio_sources(&self) -> Result<String> {
        let sources = serde_json::json!([
            {"id": "demo", "label": "Demo conversation", "kind": "demo", "available": true},
            {"id": "silence", "label": "Synthetic silence", "kind": "silence", "available": true},
            {"id": "file", "label": "Audio file", "kind": "file", "available": true},
            {"id": "mic", "label": "Microphone", "kind": "mic", "available": false, "note": "device capture pending"},
            {"id": "monitor", "label": "System audio", "kind": "monitor", "available": false, "note": "device capture pending"}
        ]);
        Ok(sources.to_string())
    }

    #[napi]
    pub fn apply_runtime_config(&self, layer_json: String) -> Result<String> {
        let layer: ConfigLayer = serde_json::from_str(&layer_json)
            .map_err(|error| Error::from_reason(format!("invalid config layer: {error}")))?;
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| Error::from_reason("bridge lock poisoned"))?;
        layer.apply(&mut inner.runtime);
        let status = RuntimeStatus::requested(&inner.runtime);
        serde_json::to_string(&status).map_err(|error| Error::from_reason(error.to_string()))
    }

    #[napi]
    pub fn apply_scenario(&self, scenario: String) -> Result<String> {
        let parsed = Scenario::parse(&scenario)
            .ok_or_else(|| Error::from_reason(format!("unknown scenario: {scenario}")))?;
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| Error::from_reason("bridge lock poisoned"))?;
        parsed.apply(&mut inner.runtime);
        let status = RuntimeStatus::requested(&inner.runtime);
        serde_json::to_string(&status).map_err(|error| Error::from_reason(error.to_string()))
    }

    #[napi]
    pub fn probe_backends(&self) -> Result<String> {
        let vulkan = std::path::Path::new("/usr/share/vulkan/icd.d").exists()
            || std::env::var_os("VK_ICD_FILENAMES").is_some();
        let cuda = std::env::var_os("LL_CUDA_AVAILABLE").is_some();
        let payload = serde_json::json!({
            "backend": "auto",
            "devices": [
                {"id": "cpu", "kind": "cpu", "available": true},
                {"id": "vulkan", "kind": "gpu", "available": vulkan},
                {"id": "cuda", "kind": "gpu", "available": cuda}
            ],
            "mockCalibration": true
        });
        Ok(payload.to_string())
    }

    #[napi]
    pub fn calibrate(&self) -> Result<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_request_defaults_match_the_ui() {
        let request: StartRequest = serde_json::from_str(r#"{"language":"de-DE"}"#).unwrap();
        assert_eq!(request.source, "demo");
        assert_eq!(request.mode, "on_demand");
        assert_eq!(request.scenario, "mock");
        assert!(!request.detection);
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
