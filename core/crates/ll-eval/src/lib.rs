//! `ll-eval` — headless replay/eval CLI (plan §7.10).
//!
//! Runs the real pipeline without the UI, writes per-segment CSV, latency JSON,
//! a summary, a content-free diagnostics JSON, and an explicit transcript.
//! Demo/mock audio is used for CI; `--wav` replays a non-PII fixture.

pub mod asr_wer;
pub mod bench;
pub mod wer;

use ll_asr::{AsrEngine, OffAsr, ScriptedAsr};
use ll_audio::{AudioSource, FileSource, SyntheticSource};
use ll_config::{RuntimeConfig, RuntimeStatus, Scenario};
use ll_core::coverage::CoverageMatrix;
use ll_core::demo::DemoConversation;
use ll_core::engine::SimplifierBackend;
use ll_core::language::Language;
use ll_core::pack::FrameworkPack;
use ll_core::session::{
    AudioSourceKind, FinalizedSegment, SessionConfiguration, SimplificationMode,
};
use ll_llm::{DemoBackend, FaultyBackend};
use ll_pipeline::{Pipeline, PipelineConfig};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EvalError {
    #[error("invalid argument: {0}")]
    Argument(String),
    #[error("scenario {0} is unavailable: {1}")]
    ScenarioUnavailable(String, String),
    #[error("audio error: {0}")]
    Audio(String),
    #[error("pipeline error: {0}")]
    Pipeline(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("unsupported scenario for this run: {0}")]
    Unsupported(String),
}

impl From<std::io::Error> for EvalError {
    fn from(error: std::io::Error) -> Self {
        EvalError::Io(error.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalSummary {
    pub scenario: String,
    pub language: String,
    pub effective_backend: String,
    pub asr_engine: String,
    pub asr_status: String,
    pub segments: usize,
    pub simplified: usize,
    pub duration_s: f64,
    pub tier1_p50_ms: Option<f64>,
    pub tier1_p95_ms: Option<f64>,
    pub tier2_p50_ms: Option<f64>,
    pub tier2_p95_ms: Option<f64>,
    pub overrun_count: u64,
    pub event_count: usize,
    pub content_free_diagnostics: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capture_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capture_overrun_events: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_rate: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct EvalOptions {
    pub scenario: Scenario,
    pub language: Language,
    pub wav: Option<PathBuf>,
    pub out: PathBuf,
    pub text: Option<String>,
    /// Capture system audio instead of replaying a file or the demo.
    pub loopback: bool,
    /// How long the loopback capture runs.
    pub seconds: u64,
    /// `--asr sherpa`: the real streaming engine with the pinned model
    /// (needs the `sherpa` feature and downloaded models).
    pub sherpa: bool,
    /// `--scenario asr-wer`: FLEURS WER/CER scoring (needs `--asr sherpa`).
    pub asr_wer: Option<asr_wer::AsrWerArgs>,
    /// `--llm llama`: the pinned GGUF on CPU (needs the `llama` feature).
    pub llama: bool,
    /// `--threads`: llama.cpp CPU threads.
    pub threads: i32,
    /// Eval-only benchmark (`--scenario click-bench|simplify-matrix`).
    pub bench: Option<bench::Bench>,
    /// `--click-all`: after the audio run, click every finalized segment.
    pub click_all: bool,
}

impl Default for EvalOptions {
    fn default() -> Self {
        Self {
            scenario: Scenario::Mock,
            language: Language::FrFr,
            wav: None,
            out: PathBuf::from("eval-out"),
            text: None,
            loopback: false,
            seconds: 60,
            sherpa: false,
            asr_wer: None,
            llama: false,
            threads: DEFAULT_LLM_THREADS,
            bench: None,
            click_all: false,
        }
    }
}

impl EvalOptions {
    pub fn parse(args: &[String]) -> Result<Self, EvalError> {
        let mut options = Self::default();
        let mut index = 0usize;
        while index < args.len() {
            match args[index].as_str() {
                "--scenario" => {
                    let value = args
                        .get(index + 1)
                        .ok_or_else(|| EvalError::Argument("--scenario needs a value".into()))?;
                    if value == "asr-wer" {
                        options.asr_wer = Some(asr_wer::AsrWerArgs {
                            asr_threads: 2,
                            ..Default::default()
                        });
                        options.scenario = Scenario::Mock;
                    } else if let Some(bench) = bench::Bench::parse(value) {
                        options.bench = Some(bench);
                        options.scenario = Scenario::Mock;
                    } else {
                        options.scenario = Scenario::parse(value).ok_or_else(|| {
                            EvalError::Argument(format!("unknown scenario: {value}"))
                        })?;
                    }
                    index += 2;
                }
                "--language" => {
                    let value = args
                        .get(index + 1)
                        .ok_or_else(|| EvalError::Argument("--language needs a value".into()))?;
                    options.language = Language::from_locale(value)
                        .ok_or_else(|| EvalError::Argument(format!("unknown language: {value}")))?;
                    index += 2;
                }
                "--wav" => {
                    let value = args
                        .get(index + 1)
                        .ok_or_else(|| EvalError::Argument("--wav needs a path".into()))?;
                    options.wav = Some(PathBuf::from(value));
                    index += 2;
                }
                "--source" => {
                    let value = args
                        .get(index + 1)
                        .ok_or_else(|| EvalError::Argument("--source needs a value".into()))?;
                    match value.as_str() {
                        "loopback" => options.loopback = true,
                        other => {
                            return Err(EvalError::Argument(format!(
                                "unknown source: {other} (expected loopback)"
                            )))
                        }
                    }
                    index += 2;
                }
                "--seconds" => {
                    let value = args
                        .get(index + 1)
                        .ok_or_else(|| EvalError::Argument("--seconds needs a value".into()))?;
                    options.seconds = value.parse::<u64>().map_err(|_| {
                        EvalError::Argument(format!(
                            "--seconds needs a whole number of seconds, got {value}"
                        ))
                    })?;
                    if options.seconds == 0 {
                        return Err(EvalError::Argument("--seconds must be at least 1".into()));
                    }
                    index += 2;
                }
                "--asr" => {
                    let value = args
                        .get(index + 1)
                        .ok_or_else(|| EvalError::Argument("--asr needs a value".into()))?;
                    match value.as_str() {
                        "mock" => options.sherpa = false,
                        "sherpa" => {
                            if !cfg!(feature = "sherpa") {
                                return Err(EvalError::Argument(
                                    "--asr sherpa: not built with sherpa; rebuild with --features sherpa"
                                        .into(),
                                ));
                            }
                            options.sherpa = true;
                        }
                        other => {
                            return Err(EvalError::Argument(format!(
                                "unknown ASR engine: {other} (expected mock or sherpa)"
                            )))
                        }
                    }
                    index += 2;
                }
                "--locale" | "--manifest" | "--cache" | "--asr-threads" => {
                    let flag = args[index].clone();
                    let value = args
                        .get(index + 1)
                        .ok_or_else(|| EvalError::Argument(format!("{flag} needs a value")))?;
                    let wer = options.asr_wer.as_mut().ok_or_else(|| {
                        EvalError::Argument(format!("{flag} needs --scenario asr-wer first"))
                    })?;
                    match flag.as_str() {
                        "--locale" => {
                            Language::from_locale(value).ok_or_else(|| {
                                EvalError::Argument(format!("unknown locale: {value}"))
                            })?;
                            wer.locale = Some(value.clone());
                        }
                        "--manifest" => wer.manifest = Some(PathBuf::from(value)),
                        "--cache" => wer.cache = Some(PathBuf::from(value)),
                        _ => {
                            wer.asr_threads = value
                                .parse::<i32>()
                                .ok()
                                .filter(|threads| (1..=16).contains(threads))
                                .ok_or_else(|| {
                                    EvalError::Argument(format!(
                                        "--asr-threads needs 1..=16, got {value}"
                                    ))
                                })?;
                        }
                    }
                    index += 2;
                }
                "--llm" => {
                    let value = args
                        .get(index + 1)
                        .ok_or_else(|| EvalError::Argument("--llm needs a value".into()))?;
                    match value.as_str() {
                        "demo" => options.llama = false,
                        "llama" => {
                            if !cfg!(feature = "llama") {
                                return Err(EvalError::Argument(
                                    "--llm llama: not built with llama; rebuild with --features llama"
                                        .into(),
                                ));
                            }
                            options.llama = true;
                        }
                        other => {
                            return Err(EvalError::Argument(format!(
                                "unknown simplifier: {other} (expected demo or llama)"
                            )))
                        }
                    }
                    index += 2;
                }
                "--all" => {
                    options
                        .asr_wer
                        .as_mut()
                        .ok_or_else(|| {
                            EvalError::Argument("--all needs --scenario asr-wer first".into())
                        })?
                        .all = true;
                    index += 1;
                }
                "--click-all" => {
                    options.click_all = true;
                    index += 1;
                }
                "--clicks" => {
                    let value = args
                        .get(index + 1)
                        .ok_or_else(|| EvalError::Argument("--clicks needs a value".into()))?;
                    let clicks = value
                        .parse::<usize>()
                        .ok()
                        .filter(|clicks| *clicks > 0)
                        .ok_or_else(|| {
                            EvalError::Argument(format!(
                                "--clicks needs a positive number, got {value}"
                            ))
                        })?;
                    match options.bench {
                        Some(bench::Bench::ClickBench { .. }) => {
                            options.bench = Some(bench::Bench::ClickBench { clicks })
                        }
                        _ => {
                            return Err(EvalError::Argument(
                                "--clicks needs --scenario click-bench first".into(),
                            ))
                        }
                    }
                    index += 2;
                }
                "--threads" => {
                    let value = args
                        .get(index + 1)
                        .ok_or_else(|| EvalError::Argument("--threads needs a value".into()))?;
                    options.threads = value
                        .parse::<i32>()
                        .ok()
                        .filter(|threads| (1..=64).contains(threads))
                        .ok_or_else(|| {
                            EvalError::Argument(format!("--threads needs 1..=64, got {value}"))
                        })?;
                    index += 2;
                }
                "--out" => {
                    let value = args
                        .get(index + 1)
                        .ok_or_else(|| EvalError::Argument("--out needs a path".into()))?;
                    options.out = PathBuf::from(value);
                    index += 2;
                }
                "--text" => {
                    let value = args
                        .get(index + 1)
                        .ok_or_else(|| EvalError::Argument("--text needs a value".into()))?;
                    options.text = Some(value.clone());
                    index += 2;
                }
                "--help" | "-h" => {
                    return Err(EvalError::Argument(help_text()));
                }
                other => return Err(EvalError::Argument(format!("unknown argument: {other}"))),
            }
        }
        Ok(options)
    }
}

pub fn help_text() -> String {
    [
        "ll-eval — headless Live Linguist evaluation",
        "",
        "Usage:",
        "  ll-eval [--scenario mock|cpu-strict|no-gpu|vulkan-strict|tier-c|simplify-only|asr-only|llm-only]",
        "          [--language de-DE] [--wav tests/fixtures/sample.wav] [--out DIR] [--text TEXT]",
        "          [--source loopback] [--seconds 60]  (loopback needs --features capture)",
        "          [--asr mock|sherpa]  (sherpa needs --features sherpa + downloaded models)",
        "          --scenario asr-wer --asr sherpa --cache DIR [--locale xx-XX|--all] [--manifest tests/asr-eval/manifest.json] [--asr-threads 2]",
        "          [--click-all]  (click every segment after an audio run)",
        "          [--llm demo|llama] [--threads 3]  (llama needs --features llama + downloaded models)",
        "",
        "Outputs: segments.csv, latency.json, summary.json, diagnostics.json, transcript.md",
    ]
    .join("\n")
}

fn check_scenario(scenario: Scenario, runtime: &RuntimeConfig) -> Result<(), EvalError> {
    match scenario {
        Scenario::CpuStrict | Scenario::NoGpu => {
            if runtime.backend != ll_config::Backend::Cpu {
                return Err(EvalError::ScenarioUnavailable(
                    scenario.as_str().into(),
                    "CPU backend was not selected".into(),
                ));
            }
        }
        Scenario::VulkanStrict => {
            // Linux dev has Vulkan (lavapipe is best-effort); do not claim a GPU
            // benchmark, only that the scenario resolves to the Vulkan backend.
            if runtime.backend != ll_config::Backend::Vulkan {
                return Err(EvalError::ScenarioUnavailable(
                    scenario.as_str().into(),
                    "Vulkan backend was not selected".into(),
                ));
            }
        }
        Scenario::CudaStrict => {
            return Err(EvalError::ScenarioUnavailable(
                scenario.as_str().into(),
                "CUDA pack is not installed on this machine".into(),
            ));
        }
        Scenario::IgpuOnly | Scenario::DgpuOnly => {
            // No device enumeration in the mock eval path; report honestly.
            return Err(EvalError::ScenarioUnavailable(
                scenario.as_str().into(),
                "no device enumeration in headless mock run".into(),
            ));
        }
        _ => {}
    }
    Ok(())
}

/// Nearest-rank percentile in ms (None when empty).
pub fn wer_percentile(values: &[f64], p: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let rank = ((p / 100.0) * sorted.len() as f64).ceil().max(1.0) as usize;
    Some((sorted[rank.min(sorted.len()) - 1] * 10.0).round() / 10.0)
}

/// Pinned SHA-256 of the ASR encoder (from the catalog, not re-hashed).
#[cfg(feature = "sherpa")]
pub fn asr_encoder_sha256() -> Option<String> {
    ll_models::ModelCatalog::plan_default()
        .artifacts
        .iter()
        .find(|artifact| artifact.role == ll_models::ModelRole::Asr)
        .and_then(|artifact| artifact.files.first())
        .and_then(|file| file.sha256.clone())
}

#[cfg(not(feature = "sherpa"))]
pub fn asr_encoder_sha256() -> Option<String> {
    None
}

/// `--scenario asr-wer`: needs `--asr sherpa` (feature `sherpa`).
pub fn run_asr_wer(options: &EvalOptions) -> Result<serde_json::Value, EvalError> {
    let args = options
        .asr_wer
        .clone()
        .ok_or_else(|| EvalError::Argument("not an asr-wer run".into()))?;
    if !options.sherpa {
        return Err(EvalError::Argument(
            "asr-wer scores the real engine: pass --asr sherpa".into(),
        ));
    }
    run_asr_wer_sherpa(options, &args)
}

#[cfg(feature = "sherpa")]
fn run_asr_wer_sherpa(
    options: &EvalOptions,
    args: &asr_wer::AsrWerArgs,
) -> Result<serde_json::Value, EvalError> {
    let threads = args.asr_threads;
    asr_wer::run(&options.out, options.language.locale(), args, || {
        let mut engine =
            ll_asr::SherpaAsr::with_threads(asr_model_dir(), options.language, threads).map_err(
                |error| EvalError::ScenarioUnavailable("asr-wer".into(), error.to_string()),
            )?;
        Ok(Box::new(move |samples: &[f32]| {
            asr_wer::sherpa_transcribe(&mut engine, samples)
        }))
    })
}

#[cfg(not(feature = "sherpa"))]
fn run_asr_wer_sherpa(
    _options: &EvalOptions,
    _args: &asr_wer::AsrWerArgs,
) -> Result<serde_json::Value, EvalError> {
    Err(EvalError::Argument(
        "--asr sherpa: not built with sherpa; rebuild with --features sherpa".into(),
    ))
}

/// Pipeline for benchmarks: no ASR, the run's simplifier and engine config.
pub(crate) fn bench_pipeline(
    options: &EvalOptions,
    pack: &FrameworkPack,
) -> Result<Pipeline, EvalError> {
    let mut session = SessionConfiguration::new(options.language, AudioSourceKind::Demo);
    session.simplification_mode = SimplificationMode::OnDemand;
    let mut config = PipelineConfig::new(session);
    config.engine = engine_config(options);
    let mut runtime = RuntimeConfig::default();
    options.scenario.apply(&mut runtime);
    Ok(Pipeline::new(
        config,
        pack.clone(),
        CoverageMatrix::v1(),
        runtime,
        Box::new(OffAsr),
        simplifier(options, pack)?,
    ))
}

/// Default llama.cpp CPU threads (T21 decides; see ll-config).
pub const DEFAULT_LLM_THREADS: i32 = ll_config::DEFAULT_LLM_THREADS as i32;

/// The simplifier for this run: the demo backend, or the pinned GGUF.
fn simplifier(
    options: &EvalOptions,
    pack: &FrameworkPack,
) -> Result<Box<dyn SimplifierBackend>, EvalError> {
    if options.llama {
        return llama_backend(pack, options.threads);
    }
    Ok(Box::new(
        DemoBackend::from_pack(pack).with_latency(std::time::Duration::ZERO),
    ))
}

/// Engine settings: the fine-tuned GGUF gets no retries (greedy + fixed
/// prompt would repeat the same text).
fn engine_config(options: &EvalOptions) -> ll_core::engine::SimplifyConfig {
    if options.llama {
        ll_core::engine::SimplifyConfig::on_demand_fine_tuned()
    } else {
        ll_core::engine::SimplifyConfig::on_demand()
    }
}

/// The pinned GGUF path in the model cache.
#[cfg(feature = "llama")]
pub fn llm_model_path() -> PathBuf {
    let catalog = ll_models::ModelCatalog::plan_default();
    let llm = catalog
        .artifacts
        .iter()
        .find(|artifact| artifact.role == ll_models::ModelRole::Llm)
        .expect("catalog has an LLM");
    ll_models::model_dir(&ll_models::cache_root(), &llm.repo, &llm.revision)
        .join(&llm.files[0].path)
}

#[cfg(feature = "llama")]
fn llama_backend(
    pack: &FrameworkPack,
    threads: i32,
) -> Result<Box<dyn SimplifierBackend>, EvalError> {
    let backend = ll_llm::LlamaBackend::load(&llm_model_path(), threads).map_err(|error| {
        let code = if ll_llm::llama::is_model_missing(&error) {
            "LLM_MODEL_MISSING"
        } else {
            "LLM_INIT_FAILED"
        };
        EvalError::ScenarioUnavailable("llm llama".into(), format!("{code}: {}", error.message))
    })?;
    Ok(Box::new(backend.with_pack(pack)))
}

#[cfg(not(feature = "llama"))]
fn llama_backend(
    _pack: &FrameworkPack,
    _threads: i32,
) -> Result<Box<dyn SimplifierBackend>, EvalError> {
    Err(EvalError::Argument(
        "--llm llama: not built with llama; rebuild with --features llama".into(),
    ))
}

/// The pinned ASR model directory in the model cache.
#[cfg(feature = "sherpa")]
pub fn asr_model_dir() -> PathBuf {
    let catalog = ll_models::ModelCatalog::plan_default();
    let asr = catalog
        .artifacts
        .iter()
        .find(|artifact| artifact.role == ll_models::ModelRole::Asr)
        .expect("catalog has an ASR model");
    ll_models::model_dir(&ll_models::cache_root(), &asr.repo, &asr.revision)
}

#[cfg(feature = "sherpa")]
fn sherpa_engine(language: Language) -> Result<Box<dyn AsrEngine>, EvalError> {
    let engine = ll_asr::SherpaAsr::new(asr_model_dir(), language).map_err(|error| {
        let code = match error {
            ll_asr::AsrError::ModelMissing { .. } => "ASR_MODEL_MISSING",
            _ => "ASR_INIT_FAILED",
        };
        EvalError::ScenarioUnavailable("asr sherpa".into(), format!("{code}: {error}"))
    })?;
    Ok(Box::new(engine))
}

#[cfg(not(feature = "sherpa"))]
fn sherpa_engine(_language: Language) -> Result<Box<dyn AsrEngine>, EvalError> {
    Err(EvalError::Argument(
        "--asr sherpa: not built with sherpa; rebuild with --features sherpa".into(),
    ))
}

fn build_pipeline(
    options: &EvalOptions,
    runtime: &RuntimeConfig,
    coverage: &CoverageMatrix,
) -> Result<Pipeline, EvalError> {
    let pack = FrameworkPack::for_language(options.language)
        .map_err(|error| EvalError::Argument(error.to_string()))?;
    let conversation = DemoConversation::from_pack(&pack, 4);
    let source_kind = match &options.wav {
        Some(path) => AudioSourceKind::File {
            path: path.display().to_string(),
        },
        None => AudioSourceKind::Demo,
    };
    let mut session = SessionConfiguration::new(options.language, source_kind);
    session.simplification_mode = match runtime.pipeline {
        ll_config::PipelineMode::Live => SimplificationMode::Automatic,
        ll_config::PipelineMode::OnDemand => SimplificationMode::OnDemand,
    };
    session.language_detection_enabled = runtime.reversion;
    let mut config = PipelineConfig::new(session);
    // Automatic simplification exists only for the dev stress scenario.
    config.dev_automatic = runtime.pipeline == ll_config::PipelineMode::Live;
    config.detection = runtime.reversion;
    config.audio_fault = runtime.fault_audio;

    let asr: Box<dyn AsrEngine> = if options.sherpa {
        sherpa_engine(options.language)?
    } else {
        match runtime.asr {
            ll_config::AsrMode::Off => Box::new(OffAsr),
            _ => Box::new(ScriptedAsr::from_conversation(&conversation, 4)),
        }
    };
    config.engine = engine_config(options);
    let inner = simplifier(options, &pack)?;
    let backend: Box<dyn SimplifierBackend> = if runtime.fault_llm == ll_config::LlmFault::None {
        inner
    } else {
        Box::new(FaultyBackend::new(inner, runtime.fault_llm))
    };
    Ok(Pipeline::new(
        config,
        pack,
        coverage.clone(),
        runtime.clone(),
        asr,
        backend,
    ))
}

fn run_text(
    options: &EvalOptions,
    runtime: &RuntimeConfig,
    out: &Path,
) -> Result<EvalSummary, EvalError> {
    let text = options.text.clone().unwrap_or_default();
    let pack = FrameworkPack::for_language(options.language)
        .map_err(|error| EvalError::Argument(error.to_string()))?;
    let mut engine = ll_core::engine::SimplificationEngine::with_config(
        simplifier(options, &pack)?,
        &pack,
        engine_config(options),
    );
    let segment = FinalizedSegment::new(text.clone(), 0.0, 2.0);
    let result = engine.simplify(&segment, &[], false);
    std::fs::create_dir_all(out)?;
    let simplified = result.outcome.display_text().unwrap_or("").to_string();
    std::fs::write(out.join("simplification.txt"), &simplified)?;
    let summary = EvalSummary {
        scenario: options.scenario.as_str().to_string(),
        language: options.language.locale().to_string(),
        effective_backend: RuntimeStatus::requested(runtime).effective_backend,
        asr_engine: "off".to_string(),
        asr_status: "simplify_only".to_string(),
        segments: 1,
        simplified: usize::from(result.outcome.display_text().is_some()),
        duration_s: 0.0,
        tier1_p50_ms: None,
        tier1_p95_ms: None,
        tier2_p50_ms: None,
        tier2_p95_ms: None,
        overrun_count: 0,
        event_count: 0,
        content_free_diagnostics: true,
        capture_seconds: None,
        capture_overrun_events: None,
        device_rate: None,
    };
    write_json(out.join("summary.json"), &summary)?;
    write_json(
        out.join("diagnostics.json"),
        &serde_json::json!({
            "scenario": summary.scenario,
            "effectiveBackend": summary.effective_backend,
            "asrEngine": summary.asr_engine,
            "asrStatus": summary.asr_status,
            "segments": summary.segments,
            "contentFree": true,
        }),
    )?;
    Ok(summary)
}

fn run_audio(
    options: &EvalOptions,
    runtime: &RuntimeConfig,
    coverage: &CoverageMatrix,
    out: &Path,
) -> Result<EvalSummary, EvalError> {
    let mut pipeline = build_pipeline(options, runtime, coverage)?;
    pipeline
        .start()
        .map_err(|error| EvalError::Pipeline(error.to_string()))?;
    #[cfg(feature = "capture")]
    if options.loopback {
        return run_loopback_audio(options, pipeline, out);
    }
    let mut source: Box<dyn AudioSource> = match &options.wav {
        Some(path) => Box::new(
            FileSource::from_path(path, false)
                .map_err(|error| EvalError::Audio(error.to_string()))?,
        ),
        None => {
            let pack = FrameworkPack::for_language(options.language)
                .map_err(|error| EvalError::Argument(error.to_string()))?;
            let conversation = DemoConversation::from_pack(&pack, 4);
            Box::new(SyntheticSource::from_conversation(&conversation, 1, false))
        }
    };
    let mut event_count = 0usize;
    while let Some(buffer) = source.next_buffer() {
        event_count += pipeline.feed_audio(&buffer).len();
    }
    finish_run(options, pipeline, event_count, out, None)
}

/// Capture system audio for `--seconds` and feed it like the `--wav` path.
#[cfg(feature = "capture")]
fn run_loopback_audio(
    options: &EvalOptions,
    mut pipeline: Pipeline,
    out: &Path,
) -> Result<EvalSummary, EvalError> {
    use std::time::{Duration, Instant};

    let mut capture = ll_audio::capture::LoopbackCapture::start_default()
        .map_err(|error| EvalError::Audio(error.to_string()))?;
    let device_rate = capture.device_rate();
    let started = Instant::now();
    let deadline = started + Duration::from_secs(options.seconds);
    let mut event_count = 0usize;
    while Instant::now() < deadline {
        let Some(buffer) = capture.next_buffer() else {
            break;
        };
        if !buffer.is_empty() {
            event_count += pipeline.feed_audio(&buffer).len();
        }
    }
    let overruns = capture.overrun_events();
    drop(capture);
    finish_run(
        options,
        pipeline,
        event_count,
        out,
        Some((started.elapsed().as_secs_f64(), overruns, device_rate)),
    )
}

fn finish_run(
    options: &EvalOptions,
    mut pipeline: Pipeline,
    mut event_count: usize,
    out: &Path,
    capture: Option<(f64, u64, u32)>,
) -> Result<EvalSummary, EvalError> {
    event_count += pipeline.flush().len();
    if options.click_all {
        // Click every finalized segment through the UI path (privacy check).
        let ids: Vec<String> = pipeline
            .store()
            .segments
            .iter()
            .map(|entry| entry.id().to_string())
            .collect();
        for id in ids {
            let _ = pipeline.simplify_now(&id);
            event_count += pipeline.take_events().len();
        }
    }
    let store = pipeline.stop();
    event_count += pipeline.take_events().len();
    let report = pipeline.metrics().report();
    std::fs::create_dir_all(out)?;
    std::fs::write(out.join("segments.csv"), pipeline.export_csv(false))?;
    std::fs::write(out.join("transcript.md"), pipeline.export_markdown())?;
    std::fs::write(out.join("latency.json"), pipeline.metrics().export_json())?;
    let status = pipeline.runtime_status();
    let diagnostics = serde_json::json!({
        "scenario": options.scenario.as_str(),
        "effectiveBackend": status.effective_backend,
        "asrEngine": if options.sherpa { "sherpa-onnx" } else { "mock" },
        "asrStatus": pipeline.snapshot().asr_status,
        "segments": store.segments.len(),
        "overrunCount": pipeline.snapshot().overrun_count,
        "eventCount": event_count,
        "contentFree": true,
    });
    write_json(out.join("diagnostics.json"), &diagnostics)?;
    let summary = EvalSummary {
        scenario: options.scenario.as_str().to_string(),
        language: options.language.locale().to_string(),
        effective_backend: status.effective_backend,
        asr_engine: if options.sherpa {
            "sherpa-onnx"
        } else {
            "mock"
        }
        .to_string(),
        asr_status: pipeline.snapshot().asr_status,
        segments: store.segments.len(),
        simplified: store
            .segments
            .iter()
            .filter(|entry| entry.outcome.display_text().is_some())
            .count(),
        duration_s: store.duration_seconds.unwrap_or(pipeline.audio_time()),
        tier1_p50_ms: report.tier1_p50,
        tier1_p95_ms: report.tier1_p95,
        tier2_p50_ms: report.tier2_p50,
        tier2_p95_ms: report.tier2_p95,
        overrun_count: pipeline.snapshot().overrun_count,
        event_count,
        content_free_diagnostics: true,
        capture_seconds: capture.map(|(seconds, _, _)| seconds),
        capture_overrun_events: capture.map(|(_, overruns, _)| overruns),
        device_rate: capture.map(|(_, _, rate)| rate),
    };
    write_json(out.join("summary.json"), &summary)?;
    Ok(summary)
}

fn write_json<T: Serialize>(path: PathBuf, value: &T) -> Result<(), EvalError> {
    let json =
        serde_json::to_string_pretty(value).map_err(|error| EvalError::Io(error.to_string()))?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Run one eval. Returns the content-free summary.
pub fn run(options: &EvalOptions) -> Result<EvalSummary, EvalError> {
    let mut runtime = RuntimeConfig::default();
    options.scenario.apply(&mut runtime);
    check_scenario(options.scenario, &runtime)?;
    let coverage = CoverageMatrix::v1();
    if let Some(text) = &options.text {
        if text.trim().is_empty() {
            return Err(EvalError::Argument("--text is empty".into()));
        }
        return run_text(options, &runtime, &options.out);
    }
    if options.scenario == Scenario::SimplifyOnly {
        return Err(EvalError::Unsupported(
            "simplify-only needs --text (typed/pasted input)".into(),
        ));
    }
    #[cfg(not(feature = "capture"))]
    if options.loopback {
        return Err(EvalError::Audio(
            "system audio capture not built; rebuild with --features capture".into(),
        ));
    }
    run_audio(options, &runtime, &coverage, &options.out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asr_wer_arguments_parse() {
        let options = EvalOptions::parse(&[
            "--scenario".into(),
            "asr-wer".into(),
            "--locale".into(),
            "fr-FR".into(),
            "--manifest".into(),
            "m.json".into(),
            "--cache".into(),
            "target/fleurs".into(),
            "--asr-threads".into(),
            "3".into(),
            "--all".into(),
        ])
        .unwrap();
        let args = options.asr_wer.clone().unwrap();
        assert_eq!(args.locale.as_deref(), Some("fr-FR"));
        assert_eq!(args.manifest, Some(PathBuf::from("m.json")));
        assert_eq!(args.cache, Some(PathBuf::from("target/fleurs")));
        assert_eq!(args.asr_threads, 3);
        assert!(args.all);
        assert!(EvalOptions::parse(&["--cache".into(), "x".into()]).is_err());
        assert!(EvalOptions::parse(&[
            "--scenario".into(),
            "asr-wer".into(),
            "--locale".into(),
            "xx-YY".into()
        ])
        .is_err());
        // Scoring the mock engine would be meaningless.
        assert!(run_asr_wer(&options).is_err());
    }

    #[test]
    #[cfg(not(feature = "llama"))]
    fn llm_llama_is_rejected_without_the_feature() {
        let error = EvalOptions::parse(&["--llm".into(), "llama".into()]).unwrap_err();
        assert!(
            error.to_string().contains("not built with llama"),
            "{error}"
        );
        let options = EvalOptions::parse(&[
            "--llm".into(),
            "demo".into(),
            "--threads".into(),
            "2".into(),
        ])
        .unwrap();
        assert!(!options.llama);
        assert_eq!(options.threads, 2);
        assert!(EvalOptions::parse(&["--threads".into(), "0".into()]).is_err());
    }

    #[test]
    #[cfg(not(feature = "sherpa"))]
    fn asr_sherpa_is_rejected_without_the_feature() {
        let error = EvalOptions::parse(&["--asr".into(), "sherpa".into()]).unwrap_err();
        assert!(
            error.to_string().contains("not built with sherpa"),
            "{error}"
        );
        assert!(
            !EvalOptions::parse(&["--asr".into(), "mock".into()])
                .unwrap()
                .sherpa
        );
        assert!(EvalOptions::parse(&["--asr".into(), "whisper".into()]).is_err());
    }

    fn out_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ll-eval-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn loopback_source_flags_parse() {
        let options = EvalOptions::parse(&[
            "--source".into(),
            "loopback".into(),
            "--seconds".into(),
            "5".into(),
        ])
        .unwrap();
        assert!(options.loopback);
        assert_eq!(options.seconds, 5);
        assert_eq!(EvalOptions::default().seconds, 60);

        assert!(matches!(
            EvalOptions::parse(&["--source".into(), "microphone".into()]),
            Err(EvalError::Argument(_))
        ));
        assert!(matches!(
            EvalOptions::parse(&["--seconds".into(), "soon".into()]),
            Err(EvalError::Argument(_))
        ));
    }

    #[test]
    fn parse_accepts_plan_flags() {
        let options = EvalOptions::parse(&[
            "--scenario".into(),
            "mock".into(),
            "--language".into(),
            "de-DE".into(),
            "--out".into(),
            "/tmp/x".into(),
        ])
        .unwrap();
        assert_eq!(options.scenario, Scenario::Mock);
        assert_eq!(options.language, Language::DeDe);
        assert_eq!(options.out, PathBuf::from("/tmp/x"));
    }

    #[test]
    fn demo_run_writes_content_free_outputs() {
        let out = out_dir("demo");
        let options = EvalOptions {
            scenario: Scenario::Mock,
            language: Language::FrFr,
            wav: None,
            out: out.clone(),
            text: None,
            ..EvalOptions::default()
        };
        let summary = run(&options).unwrap();
        assert!(summary.segments >= 2, "{summary:?}");
        assert!(summary.event_count > 0);
        for file in [
            "segments.csv",
            "latency.json",
            "summary.json",
            "diagnostics.json",
            "transcript.md",
        ] {
            assert!(out.join(file).exists(), "{file}");
        }
        let diagnostics = std::fs::read_to_string(out.join("diagnostics.json")).unwrap();
        assert!(!diagnostics.contains("Bonjour"));
        assert!(!diagnostics.contains("médina"));
        let _ = std::fs::remove_dir_all(&out);
    }

    #[test]
    fn cpu_strict_and_no_gpu_resolve() {
        for scenario in [Scenario::CpuStrict, Scenario::NoGpu, Scenario::VulkanStrict] {
            let out = out_dir(scenario.as_str());
            let options = EvalOptions {
                scenario,
                language: Language::DeDe,
                wav: None,
                out: out.clone(),
                text: None,
                ..EvalOptions::default()
            };
            let summary = run(&options).unwrap();
            assert_eq!(summary.scenario, scenario.as_str());
            let _ = std::fs::remove_dir_all(&out);
        }
    }

    #[test]
    fn simplify_only_text_uses_the_engine() {
        let out = out_dir("text");
        let options = EvalOptions {
            scenario: Scenario::SimplifyOnly,
            language: Language::FrFr,
            wav: None,
            out: out.clone(),
            text: Some(
                "Il faudrait que tu viennes nous rendre visite dès que l'occasion se présentera."
                    .into(),
            ),
            ..EvalOptions::default()
        };
        let summary = run(&options).unwrap();
        assert_eq!(summary.segments, 1);
        assert!(out.join("simplification.txt").exists());
        let _ = std::fs::remove_dir_all(&out);
    }

    #[test]
    fn cuda_strict_fails_loudly() {
        let options = EvalOptions {
            scenario: Scenario::CudaStrict,
            ..EvalOptions::default()
        };
        assert!(matches!(
            run(&options),
            Err(EvalError::ScenarioUnavailable(_, _))
        ));
    }

    #[test]
    fn missing_wav_is_an_error() {
        let out = out_dir("wav");
        let options = EvalOptions {
            scenario: Scenario::Mock,
            wav: Some(PathBuf::from("/does/not/exist.wav")),
            out: out.clone(),
            ..EvalOptions::default()
        };
        assert!(run(&options).is_err());
        let _ = std::fs::remove_dir_all(&out);
    }
}
