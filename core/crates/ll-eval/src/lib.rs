//! `ll-eval` — headless replay/eval CLI (plan §7.10).
//!
//! Runs the real pipeline without the UI, writes per-segment CSV, latency JSON,
//! a summary, a content-free diagnostics JSON, and an explicit transcript.
//! Demo/mock audio is used for CI; `--wav` replays a non-PII fixture.

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
}

#[derive(Debug, Clone)]
pub struct EvalOptions {
    pub scenario: Scenario,
    pub language: Language,
    pub wav: Option<PathBuf>,
    pub out: PathBuf,
    pub text: Option<String>,
}

impl Default for EvalOptions {
    fn default() -> Self {
        Self {
            scenario: Scenario::Mock,
            language: Language::FrFr,
            wav: None,
            out: PathBuf::from("eval-out"),
            text: None,
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
                    options.scenario = Scenario::parse(value)
                        .ok_or_else(|| EvalError::Argument(format!("unknown scenario: {value}")))?;
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
    config.detection = runtime.reversion;
    config.audio_fault = runtime.fault_audio;

    let asr: Box<dyn AsrEngine> = match runtime.asr {
        ll_config::AsrMode::Off => Box::new(OffAsr),
        _ => Box::new(ScriptedAsr::from_conversation(&conversation, 4)),
    };
    let demo = DemoBackend::from_pack(&pack).with_latency(std::time::Duration::ZERO);
    let backend: Box<dyn SimplifierBackend> = if runtime.fault_llm == ll_config::LlmFault::None {
        Box::new(demo)
    } else {
        Box::new(FaultyBackend::new(Box::new(demo), runtime.fault_llm))
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
        Box::new(DemoBackend::from_pack(&pack).with_latency(std::time::Duration::ZERO)),
        &pack,
        ll_core::engine::SimplifyConfig::on_demand(),
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
    event_count += pipeline.flush().len();
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
        "asrEngine": "mock",
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
        asr_engine: "mock".to_string(),
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
    run_audio(options, &runtime, &coverage, &options.out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn out_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ll-eval-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
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
