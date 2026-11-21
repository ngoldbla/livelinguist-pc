//! Simplifier engines, calibration, degradation ladder, queue policy, and
//! prefix-cache bookkeeping (plan §7.5, §11).
//!
//! The default build uses deterministic demo/mock backends. The `llama` feature
//! is reserved for the pinned `llama-cpp-2` backend once a model is installed;
//! this crate never silently falls back in strict mode.

use ll_config::{GpuLayers, LlmFault, LlmModel};
use ll_core::demo::DemoSimplifier;
use ll_core::engine::{BackendError, SimplifierBackend};
use ll_core::pack::FrameworkPack;
use ll_core::prompt::SimplificationRequest;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimplifierMode {
    Demo,
    Mock,
    LlamaCpp,
    Off,
}

/// Deterministic demo backend derived from a framework pack.
#[derive(Debug, Clone)]
pub struct DemoBackend {
    simplifier: DemoSimplifier,
    latency: Duration,
}

impl DemoBackend {
    pub fn from_pack(pack: &FrameworkPack) -> Self {
        Self {
            simplifier: DemoSimplifier::from_pack(pack),
            latency: Duration::from_millis(120),
        }
    }

    pub fn with_latency(mut self, latency: Duration) -> Self {
        self.latency = latency;
        self
    }
}

impl SimplifierBackend for DemoBackend {
    fn name(&self) -> &'static str {
        "demo"
    }

    fn simplify(&mut self, request: &SimplificationRequest) -> Result<String, BackendError> {
        if !self.latency.is_zero() {
            std::thread::sleep(self.latency);
        }
        Ok(self.simplifier.simplify(&request.sentence))
    }
}

/// A queue-backed mock backend for tests and fault scenarios.
#[derive(Debug, Clone, Default)]
pub struct MockBackend {
    responses: VecDeque<Result<String, BackendError>>,
}

impl MockBackend {
    pub fn new(responses: impl IntoIterator<Item = Result<String, BackendError>>) -> Self {
        Self {
            responses: responses.into_iter().collect(),
        }
    }

    pub fn from_strings(responses: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::new(responses.into_iter().map(|text| Ok(text.into())))
    }
}

impl SimplifierBackend for MockBackend {
    fn name(&self) -> &'static str {
        "mock"
    }

    fn simplify(&mut self, _request: &SimplificationRequest) -> Result<String, BackendError> {
        self.responses
            .pop_front()
            .unwrap_or_else(|| Ok(String::new()))
    }
}

/// Injects the configured LLM fault before delegating.
pub struct FaultyBackend {
    inner: Box<dyn SimplifierBackend>,
    fault: LlmFault,
}

impl FaultyBackend {
    pub fn new(inner: Box<dyn SimplifierBackend>, fault: LlmFault) -> Self {
        Self { inner, fault }
    }
}

impl SimplifierBackend for FaultyBackend {
    fn name(&self) -> &'static str {
        "faulty"
    }

    fn simplify(&mut self, request: &SimplificationRequest) -> Result<String, BackendError> {
        match self.fault {
            LlmFault::None => self.inner.simplify(request),
            LlmFault::Timeout => Err(BackendError::timeout()),
            LlmFault::Error => Err(BackendError::backend("injected LLM error")),
            LlmFault::Oom => Err(BackendError::out_of_memory("injected OOM")),
        }
    }
}

/// Backend modules and devices for calibration (S-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendId {
    Cpu,
    Vulkan,
    Cuda,
}

impl BackendId {
    pub fn as_str(self) -> &'static str {
        match self {
            BackendId::Cpu => "cpu",
            BackendId::Vulkan => "vulkan",
            BackendId::Cuda => "cuda",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceKind {
    Gpu,
    IntegratedGpu,
    Cpu,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub name: String,
    pub kind: DeviceKind,
    pub vram_free_mb: u64,
    pub vram_total_mb: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalibrationResult {
    pub backend: BackendId,
    pub device: String,
    pub ttft_ms: f64,
    pub tokens_per_sec: f64,
}

/// Stable cache key: CPU, GPU, driver, model hash, backend version.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalibrationKey {
    pub cpu: String,
    pub gpu: String,
    pub driver: String,
    pub model_hash: String,
    pub backend_version: String,
}

impl CalibrationKey {
    pub fn as_key(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}",
            self.cpu, self.gpu, self.driver, self.model_hash, self.backend_version
        )
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalibrationCache {
    pub entries: HashMap<String, CalibrationResult>,
}

impl CalibrationCache {
    pub fn get(&self, key: &CalibrationKey) -> Option<&CalibrationResult> {
        self.entries.get(&key.as_key())
    }

    pub fn put(&mut self, key: CalibrationKey, result: CalibrationResult) {
        self.entries.insert(key.as_key(), result);
    }

    pub fn to_json(&self) -> Result<String, CalibrationError> {
        serde_json::to_string_pretty(self)
            .map_err(|error| CalibrationError::Serialize(error.to_string()))
    }

    pub fn from_json(json: &str) -> Result<Self, CalibrationError> {
        serde_json::from_str(json).map_err(|error| CalibrationError::Serialize(error.to_string()))
    }
}

#[derive(Debug, Error)]
pub enum CalibrationError {
    #[error("calibration probe failed: {0}")]
    Probe(String),
    #[error("calibration serialization failed: {0}")]
    Serialize(String),
}

/// A hardware probe that measures a fixed prompt on one backend/device.
pub trait CalibrationProbe {
    fn probe(
        &mut self,
        backend: BackendId,
        device: &DeviceInfo,
    ) -> Result<CalibrationResult, CalibrationError>;
}

/// Prefer a discrete GPU, then integrated, then CPU.
pub fn choose_device(devices: &[DeviceInfo], prefer_discrete: bool) -> Option<&DeviceInfo> {
    if prefer_discrete {
        if let Some(device) = devices.iter().find(|device| device.kind == DeviceKind::Gpu) {
            return Some(device);
        }
    }
    devices
        .iter()
        .find(|device| device.kind == DeviceKind::Gpu)
        .or_else(|| {
            devices
                .iter()
                .find(|device| device.kind == DeviceKind::IntegratedGpu)
        })
        .or_else(|| devices.iter().find(|device| device.kind == DeviceKind::Cpu))
}

/// Calibrate CPU and the chosen GPU; return the faster result and record it.
pub fn calibrate(
    devices: &[DeviceInfo],
    probe: &mut dyn CalibrationProbe,
    key: CalibrationKey,
    cache: &mut CalibrationCache,
) -> Result<CalibrationResult, CalibrationError> {
    if let Some(cached) = cache.get(&key) {
        return Ok(cached.clone());
    }
    let cpu = devices
        .iter()
        .find(|device| device.kind == DeviceKind::Cpu)
        .cloned()
        .unwrap_or(DeviceInfo {
            name: "cpu".to_string(),
            kind: DeviceKind::Cpu,
            vram_free_mb: 0,
            vram_total_mb: 0,
        });
    let cpu_result = probe.probe(BackendId::Cpu, &cpu)?;
    let mut best = cpu_result;
    if let Some(gpu) = choose_device(devices, true) {
        if gpu.kind != DeviceKind::Cpu {
            if let Ok(gpu_result) = probe.probe(BackendId::Vulkan, gpu) {
                if gpu_result.tokens_per_sec > best.tokens_per_sec {
                    best = gpu_result;
                }
            }
        }
    }
    cache.put(key, best.clone());
    Ok(best)
}

/// One rung of the 2-D degradation ladder (D-040).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LadderStep {
    pub model: LlmModel,
    pub backend: BackendId,
    pub gpu_layers: GpuLayers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LadderOutcome {
    Step(LadderStep),
    Verbatim,
}

pub fn ladder() -> [LadderOutcome; 6] {
    use LlmModel::{Qwen06b, Qwen17b};
    [
        LadderOutcome::Step(LadderStep {
            model: Qwen17b,
            backend: BackendId::Vulkan,
            gpu_layers: GpuLayers::All,
        }),
        LadderOutcome::Step(LadderStep {
            model: Qwen17b,
            backend: BackendId::Vulkan,
            gpu_layers: GpuLayers::Count(16),
        }),
        LadderOutcome::Step(LadderStep {
            model: Qwen17b,
            backend: BackendId::Cpu,
            gpu_layers: GpuLayers::Zero,
        }),
        LadderOutcome::Step(LadderStep {
            model: Qwen06b,
            backend: BackendId::Vulkan,
            gpu_layers: GpuLayers::All,
        }),
        LadderOutcome::Step(LadderStep {
            model: Qwen06b,
            backend: BackendId::Cpu,
            gpu_layers: GpuLayers::Zero,
        }),
        LadderOutcome::Verbatim,
    ]
}

pub fn next_rung(current: LadderStep) -> LadderOutcome {
    let rungs = ladder();
    for (index, rung) in rungs.iter().enumerate() {
        if let LadderOutcome::Step(step) = rung {
            if *step == current {
                return rungs
                    .get(index + 1)
                    .copied()
                    .unwrap_or(LadderOutcome::Verbatim);
            }
        }
    }
    LadderOutcome::Verbatim
}

/// On-demand queue: one active + one queued; a newer click replaces the queued
/// item; cancel works on active or queued (D-041).
#[derive(Debug, Clone, Default)]
pub struct SimplifyQueue {
    active: Option<String>,
    queued: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueAction {
    Started(String),
    Replaced {
        active: String,
        replaced: Option<String>,
    },
    Queued(String),
    Idle,
}

impl SimplifyQueue {
    pub fn request(&mut self, id: &str) -> QueueAction {
        if self.active.is_none() {
            self.active = Some(id.to_string());
            QueueAction::Started(id.to_string())
        } else {
            let replaced = self.queued.replace(id.to_string());
            let active = self.active.clone().unwrap_or_default();
            if replaced.is_some() {
                QueueAction::Replaced { active, replaced }
            } else {
                QueueAction::Queued(id.to_string())
            }
        }
    }

    pub fn complete(&mut self) -> Option<String> {
        self.active = self.queued.take();
        self.active.clone()
    }

    pub fn active(&self) -> Option<&str> {
        self.active.as_deref()
    }

    pub fn queued(&self) -> Option<&str> {
        self.queued.as_deref()
    }

    pub fn cancel(&mut self, id: &str) -> bool {
        if self.active.as_deref() == Some(id) {
            self.active = self.queued.take();
            return true;
        }
        if self.queued.as_deref() == Some(id) {
            self.queued = None;
            return true;
        }
        false
    }

    pub fn clear(&mut self) {
        self.active = None;
        self.queued = None;
    }
}

/// Per-locale prefix-cache bookkeeping with LRU eviction (D-042).
#[derive(Debug, Clone)]
pub struct PrefixCache {
    capacity: usize,
    entries: HashMap<String, Vec<u8>>,
    order: Vec<String>,
    prefill_ms: HashMap<String, f64>,
}

impl PrefixCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            entries: HashMap::new(),
            order: Vec::new(),
            prefill_ms: HashMap::new(),
        }
    }

    pub fn get(&mut self, locale: &str) -> Option<&Vec<u8>> {
        if !self.entries.contains_key(locale) {
            return None;
        }
        self.touch(locale);
        self.entries.get(locale)
    }

    pub fn put(&mut self, locale: &str, state: Vec<u8>, prefill_ms: f64) {
        if !self.entries.contains_key(locale) && self.entries.len() >= self.capacity {
            if let Some(oldest) = self.order.first().cloned() {
                self.order.remove(0);
                self.entries.remove(&oldest);
                self.prefill_ms.remove(&oldest);
            }
        }
        self.entries.insert(locale.to_string(), state);
        self.prefill_ms.insert(locale.to_string(), prefill_ms);
        self.touch(locale);
    }

    pub fn prefill_ms(&self, locale: &str) -> Option<f64> {
        self.prefill_ms.get(locale).copied()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn touch(&mut self, locale: &str) {
        self.order.retain(|entry| entry != locale);
        self.order.push(locale.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ll_core::language::Framework;
    use ll_core::pack::FrameworkPack;
    use ll_core::prompt::PromptBuilder;

    #[test]
    fn demo_backend_returns_pack_reference_rewrite() {
        let pack = FrameworkPack::load_framework(Framework::Falc).unwrap();
        let shot = &pack.few_shots[0];
        let mut backend = DemoBackend::from_pack(&pack).with_latency(Duration::ZERO);
        let request = PromptBuilder::new(&pack).make_request(&shot.source, &[]);
        assert_eq!(backend.simplify(&request).unwrap(), shot.simplified);
    }

    #[test]
    fn faulty_backend_injects_each_fault() {
        let pack = FrameworkPack::load_framework(Framework::Falc).unwrap();
        let request = PromptBuilder::new(&pack).make_request(&pack.few_shots[0].source, &[]);
        for (fault, kind) in [
            (LlmFault::Timeout, BackendErrorKind::Timeout),
            (LlmFault::Error, BackendErrorKind::Backend),
            (LlmFault::Oom, BackendErrorKind::OutOfMemory),
        ] {
            let mut backend = FaultyBackend::new(
                Box::new(DemoBackend::from_pack(&pack).with_latency(Duration::ZERO)),
                fault,
            );
            assert_eq!(backend.simplify(&request).unwrap_err().kind, kind);
        }
    }

    use ll_core::engine::BackendErrorKind;

    #[test]
    fn queue_has_one_active_and_one_queued_with_newest_wins() {
        let mut queue = SimplifyQueue::default();
        assert_eq!(queue.request("a"), QueueAction::Started("a".into()));
        assert_eq!(queue.request("b"), QueueAction::Queued("b".into()));
        assert_eq!(
            queue.request("c"),
            QueueAction::Replaced {
                active: "a".into(),
                replaced: Some("b".into())
            }
        );
        assert_eq!(queue.active(), Some("a"));
        assert_eq!(queue.queued(), Some("c"));
        assert_eq!(queue.complete(), Some("c".to_string()));
        assert_eq!(queue.active(), Some("c"));
    }

    #[test]
    fn queue_cancel_works_for_active_and_queued() {
        let mut queue = SimplifyQueue::default();
        let _ = queue.request("a");
        let _ = queue.request("b");
        assert!(queue.cancel("b"));
        assert_eq!(queue.queued(), None);
        assert!(queue.cancel("a"));
        assert_eq!(queue.active(), None);
        assert!(!queue.cancel("nope"));
    }

    #[test]
    fn ladder_matches_the_2d_plan_and_ends_verbatim() {
        let steps = ladder();
        assert_eq!(steps.len(), 6);
        assert_eq!(steps[5], LadderOutcome::Verbatim);
        let first = match steps[0] {
            LadderOutcome::Step(step) => step,
            _ => panic!("first rung"),
        };
        assert_eq!(first.model, LlmModel::Qwen17b);
        assert_eq!(first.backend, BackendId::Vulkan);
        assert_eq!(first.gpu_layers, GpuLayers::All);
        let mut current = first;
        let mut seen = 0;
        while let LadderOutcome::Step(step) = next_rung(current) {
            current = step;
            seen += 1;
        }
        assert_eq!(seen, 4);
    }

    #[test]
    fn calibration_picks_the_faster_gpu_and_caches() {
        struct Probe;
        impl CalibrationProbe for Probe {
            fn probe(
                &mut self,
                backend: BackendId,
                device: &DeviceInfo,
            ) -> Result<CalibrationResult, CalibrationError> {
                let tokens = if backend == BackendId::Cpu {
                    10.0
                } else {
                    30.0
                };
                Ok(CalibrationResult {
                    backend,
                    device: device.name.clone(),
                    ttft_ms: 100.0,
                    tokens_per_sec: tokens,
                })
            }
        }
        let devices = vec![
            DeviceInfo {
                name: "cpu".into(),
                kind: DeviceKind::Cpu,
                vram_free_mb: 0,
                vram_total_mb: 0,
            },
            DeviceInfo {
                name: "gpu".into(),
                kind: DeviceKind::Gpu,
                vram_free_mb: 8000,
                vram_total_mb: 12000,
            },
        ];
        let key = CalibrationKey {
            cpu: "test".into(),
            gpu: "test".into(),
            driver: "test".into(),
            model_hash: "test".into(),
            backend_version: "test".into(),
        };
        let mut cache = CalibrationCache::default();
        let result = calibrate(&devices, &mut Probe, key.clone(), &mut cache).unwrap();
        assert_eq!(result.backend, BackendId::Vulkan);
        assert!(cache.get(&key).is_some());
        let json = cache.to_json().unwrap();
        let restored = CalibrationCache::from_json(&json).unwrap();
        assert!(restored.get(&key).is_some());
    }

    #[test]
    fn prefix_cache_evicts_lru_and_tracks_prefill() {
        let mut cache = PrefixCache::new(2);
        cache.put("de", vec![1], 40.0);
        cache.put("fr", vec![2], 50.0);
        assert!(cache.get("de").is_some());
        cache.put("es", vec![3], 60.0);
        assert!(cache.get("fr").is_none());
        assert_eq!(cache.prefill_ms("de"), Some(40.0));
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn choose_device_prefers_discrete_then_integrated_then_cpu() {
        let cpu = DeviceInfo {
            name: "cpu".into(),
            kind: DeviceKind::Cpu,
            vram_free_mb: 0,
            vram_total_mb: 0,
        };
        let igpu = DeviceInfo {
            name: "igpu".into(),
            kind: DeviceKind::IntegratedGpu,
            vram_free_mb: 4000,
            vram_total_mb: 4000,
        };
        let dgpu = DeviceInfo {
            name: "dgpu".into(),
            kind: DeviceKind::Gpu,
            vram_free_mb: 8000,
            vram_total_mb: 8000,
        };
        assert_eq!(
            choose_device(&[cpu.clone(), igpu.clone(), dgpu.clone()], true)
                .unwrap()
                .name,
            "dgpu"
        );
        assert_eq!(
            choose_device(&[cpu.clone(), igpu.clone()], true)
                .unwrap()
                .name,
            "igpu"
        );
        assert_eq!(choose_device(&[cpu], true).unwrap().name, "cpu");
    }
}
