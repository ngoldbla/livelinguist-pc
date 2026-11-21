//! Runtime configuration, developer overrides, precedence resolution, and
//! `RuntimeStatus` (plan §7.2, §14).
//!
//! Precedence: CLI > env > dev overrides > user settings > calibration > defaults.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use ts_rs::TS;

macro_rules! string_enum {
    ($name:ident { $($variant:ident => $text:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
        #[serde(rename_all = "snake_case")]
        #[ts(export, rename_all = "snake_case", export_to = "../../../../ui/src/bridge/generated/")]
        pub enum $name { $($variant),+ }

        impl $name {
            pub fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $text),+ }
            }

            pub fn parse(value: &str) -> Option<Self> {
                match value.to_ascii_lowercase().replace('-', "_").as_str() {
                    $($text => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }
    };
}

string_enum!(Backend {
    Auto => "auto",
    Cpu => "cpu",
    Vulkan => "vulkan",
    Cuda => "cuda",
    Metal => "metal",
});
string_enum!(DevicePreference {
    Auto => "auto",
    Discrete => "discrete",
    Integrated => "integrated",
});
string_enum!(LlmModel {
    Qwen06b => "0.6b",
    Qwen17b => "1.7b",
});
string_enum!(Quant {
    Q4Km => "q4_k_m",
    Q8_0 => "q8_0",
    Iq4Xs => "iq4_xs",
});
string_enum!(AsrMode {
    Cpu => "cpu",
    Mock => "mock",
    Off => "off",
});
string_enum!(AsrModel {
    Nemotron => "nemotron",
    Zipformer => "zipformer",
    Parakeet => "parakeet",
});
string_enum!(AudioMode {
    Demo => "demo",
    File => "file",
    Mic => "mic",
    Monitor => "monitor",
    Silence => "silence",
});
string_enum!(PipelineMode {
    OnDemand => "on_demand",
    Live => "live",
});
string_enum!(CalibrationMode {
    Auto => "auto",
    Skip => "skip",
    ForceCpu => "force_cpu",
    ForceGpu => "force_gpu",
});
string_enum!(LlmFault {
    None => "none",
    Timeout => "timeout",
    Error => "error",
    Oom => "oom",
});
string_enum!(AudioFault {
    None => "none",
    Overrun => "overrun",
});
string_enum!(LogLevel {
    Error => "error",
    Warn => "warn",
    Info => "info",
    Debug => "debug",
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(
    export,
    rename_all = "snake_case",
    export_to = "../../../../ui/src/bridge/generated/"
)]
pub enum WindowMode {
    Console,
    Overlay,
    Expanded,
}

impl WindowMode {
    pub fn as_str(self) -> &'static str {
        match self {
            WindowMode::Console => "console",
            WindowMode::Overlay => "overlay",
            WindowMode::Expanded => "expanded",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(
    export,
    rename_all = "snake_case",
    export_to = "../../../../ui/src/bridge/generated/"
)]
pub enum GpuLayers {
    Auto,
    Zero,
    All,
    Count(u32),
}

impl GpuLayers {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "auto" => Some(GpuLayers::Auto),
            "0" => Some(GpuLayers::Zero),
            "all" => Some(GpuLayers::All),
            other => other.parse::<u32>().ok().map(GpuLayers::Count),
        }
    }

    pub fn effective(self, total_layers: u32) -> u32 {
        match self {
            GpuLayers::Auto => total_layers,
            GpuLayers::Zero => 0,
            GpuLayers::All => total_layers,
            GpuLayers::Count(count) => count.min(total_layers),
        }
    }

    pub fn as_string(self) -> String {
        match self {
            GpuLayers::Auto => "auto".to_string(),
            GpuLayers::Zero => "0".to_string(),
            GpuLayers::All => "all".to_string(),
            GpuLayers::Count(count) => count.to_string(),
        }
    }
}

/// Fully resolved runtime configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(
    export,
    rename_all = "camelCase",
    export_to = "../../../../ui/src/bridge/generated/"
)]
pub struct RuntimeConfig {
    pub backend: Backend,
    pub device: DevicePreference,
    pub gpu_layers: GpuLayers,
    pub threads: usize,
    pub llm_model: LlmModel,
    pub quant: Quant,
    pub context: usize,
    pub max_tokens: usize,
    pub temperature: f32,
    #[ts(type = "number")]
    #[ts(type = "number")]
    pub seed: u64,
    pub prefix_cache: bool,
    pub flash_attention: bool,
    pub asr: AsrMode,
    pub asr_model: AsrModel,
    pub asr_threads: usize,
    pub audio: AudioMode,
    pub pipeline: PipelineMode,
    pub strict: bool,
    pub calibration: CalibrationMode,
    pub fault_llm: LlmFault,
    pub fault_audio: AudioFault,
    pub validators: bool,
    pub reversion: bool,
    pub log_level: LogLevel,
    pub window_mode: WindowMode,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            backend: Backend::Auto,
            device: DevicePreference::Auto,
            gpu_layers: GpuLayers::Auto,
            threads: default_threads(),
            llm_model: LlmModel::Qwen06b,
            quant: Quant::Q4Km,
            context: 2048,
            max_tokens: 128,
            temperature: 0.0,
            seed: 42,
            prefix_cache: true,
            flash_attention: false,
            asr: AsrMode::Mock,
            asr_model: AsrModel::Nemotron,
            asr_threads: 2,
            audio: AudioMode::Demo,
            pipeline: PipelineMode::OnDemand,
            strict: false,
            calibration: CalibrationMode::Auto,
            fault_llm: LlmFault::None,
            fault_audio: AudioFault::None,
            validators: true,
            reversion: true,
            log_level: LogLevel::Info,
            window_mode: WindowMode::Console,
        }
    }
}

fn default_threads() -> usize {
    let cores = std::thread::available_parallelism()
        .map(|value| value.get())
        .unwrap_or(4);
    (cores / 2).max(2)
}

/// A sparse layer: only `Some` fields override. Used by env, dev, and user.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(
    export,
    rename_all = "camelCase",
    export_to = "../../../../ui/src/bridge/generated/"
)]
pub struct ConfigLayer {
    pub backend: Option<Backend>,
    pub device: Option<DevicePreference>,
    pub gpu_layers: Option<GpuLayers>,
    pub threads: Option<usize>,
    pub llm_model: Option<LlmModel>,
    pub quant: Option<Quant>,
    pub context: Option<usize>,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
    #[ts(type = "number")]
    pub seed: Option<u64>,
    pub prefix_cache: Option<bool>,
    pub flash_attention: Option<bool>,
    pub asr: Option<AsrMode>,
    pub asr_model: Option<AsrModel>,
    pub asr_threads: Option<usize>,
    pub audio: Option<AudioMode>,
    pub pipeline: Option<PipelineMode>,
    pub strict: Option<bool>,
    pub calibration: Option<CalibrationMode>,
    pub fault_llm: Option<LlmFault>,
    pub fault_audio: Option<AudioFault>,
    pub validators: Option<bool>,
    pub reversion: Option<bool>,
    pub log_level: Option<LogLevel>,
    pub window_mode: Option<WindowMode>,
}

impl ConfigLayer {
    pub fn apply(&self, config: &mut RuntimeConfig) {
        macro_rules! set {
            ($($field:ident),+ $(,)?) => {
                $(if let Some(value) = self.$field { config.$field = value; })+
            };
        }
        set!(
            backend,
            device,
            gpu_layers,
            threads,
            llm_model,
            quant,
            context,
            max_tokens,
            temperature,
            seed,
            prefix_cache,
            flash_attention,
            asr,
            asr_model,
            asr_threads,
            audio,
            pipeline,
            strict,
            calibration,
            fault_llm,
            fault_audio,
            validators,
            reversion,
            log_level,
            window_mode
        );
    }

    /// Parse the `LL_*` environment contract (plan §14.5).
    pub fn from_env_map(env: &BTreeMap<String, String>) -> Self {
        let get = |key: &str| env.get(key).map(String::as_str);
        let mut layer = ConfigLayer::default();
        if let Some(value) = get("LL_BACKEND") {
            layer.backend = Backend::parse(value);
        }
        if let Some(value) = get("LL_DEVICE") {
            layer.device = DevicePreference::parse(value)
                .or_else(|| value.parse::<u32>().ok().map(|_| DevicePreference::Auto));
        }
        if let Some(value) = get("LL_GPU_LAYERS") {
            layer.gpu_layers = GpuLayers::parse(value);
        }
        layer.threads = get("LL_THREADS").and_then(|value| value.parse().ok());
        if let Some(value) = get("LL_LLM_MODEL") {
            layer.llm_model = LlmModel::parse(value);
        }
        if let Some(value) = get("LL_QUANT") {
            layer.quant = Quant::parse(value);
        }
        if let Some(value) = get("LL_ASR") {
            layer.asr = AsrMode::parse(value);
        }
        if let Some(value) = get("LL_ASR_MODEL") {
            layer.asr_model = AsrModel::parse(value);
        }
        if let Some(value) = get("LL_AUDIO") {
            layer.audio = AudioMode::parse(value);
        }
        if let Some(value) = get("LL_PIPELINE") {
            layer.pipeline = PipelineMode::parse(value);
        }
        layer.strict = get("LL_STRICT_BACKEND").map(|value| value == "1" || value == "true");
        if let Some(value) = get("LL_CALIBRATION") {
            layer.calibration = CalibrationMode::parse(value);
        }
        if let Some(value) = get("LL_FAULT_LLM") {
            layer.fault_llm = LlmFault::parse(value);
        }
        if let Some(value) = get("LL_FAULT_AUDIO") {
            layer.fault_audio = AudioFault::parse(value);
        }
        if let Some(value) = get("LL_LOG_LEVEL") {
            layer.log_level = LogLevel::parse(value);
        }
        layer
    }

    pub fn from_process_env() -> Self {
        let env = std::env::vars().collect::<BTreeMap<_, _>>();
        Self::from_env_map(&env)
    }
}

/// Persisted developer overrides (`dev-overrides.json`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(
    export,
    rename_all = "camelCase",
    export_to = "../../../../ui/src/bridge/generated/"
)]
pub struct DevOverrides {
    pub schema_version: u32,
    #[serde(flatten)]
    pub layer: ConfigLayer,
}

/// Persisted user settings (non-developer).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(
    export,
    rename_all = "camelCase",
    export_to = "../../../../ui/src/bridge/generated/"
)]
pub struct UserSettings {
    #[serde(flatten)]
    pub layer: ConfigLayer,
}

/// Resolve the config by layering CLI, env, dev, user, calibration, defaults.
pub fn resolve(
    cli: Option<&ConfigLayer>,
    env: Option<&ConfigLayer>,
    dev: Option<&DevOverrides>,
    user: Option<&UserSettings>,
    calibration: Option<&ConfigLayer>,
) -> RuntimeConfig {
    let mut config = RuntimeConfig::default();
    if let Some(layer) = calibration {
        layer.apply(&mut config);
    }
    if let Some(settings) = user {
        settings.layer.apply(&mut config);
    }
    if let Some(overrides) = dev {
        overrides.layer.apply(&mut config);
    }
    if let Some(layer) = env {
        layer.apply(&mut config);
    }
    if let Some(layer) = cli {
        layer.apply(&mut config);
    }
    config
}

/// Developer scenario presets (plan §14.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(
    export,
    rename_all = "kebab-case",
    export_to = "../../../../ui/src/bridge/generated/"
)]
pub enum Scenario {
    Auto,
    CpuStrict,
    NoGpu,
    VulkanStrict,
    CudaStrict,
    IgpuOnly,
    DgpuOnly,
    Mock,
    TierC,
    Stress,
    AsrOnly,
    LlmOnly,
    SimplifyOnly,
}

impl Scenario {
    pub const ALL: [Scenario; 13] = [
        Scenario::Auto,
        Scenario::CpuStrict,
        Scenario::NoGpu,
        Scenario::VulkanStrict,
        Scenario::CudaStrict,
        Scenario::IgpuOnly,
        Scenario::DgpuOnly,
        Scenario::Mock,
        Scenario::TierC,
        Scenario::Stress,
        Scenario::AsrOnly,
        Scenario::LlmOnly,
        Scenario::SimplifyOnly,
    ];

    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().replace('_', "-").as_str() {
            "auto" => Some(Scenario::Auto),
            "cpu-strict" => Some(Scenario::CpuStrict),
            "no-gpu" => Some(Scenario::NoGpu),
            "vulkan-strict" => Some(Scenario::VulkanStrict),
            "cuda-strict" => Some(Scenario::CudaStrict),
            "igpu-only" => Some(Scenario::IgpuOnly),
            "dgpu-only" => Some(Scenario::DgpuOnly),
            "mock" | "mock-pipeline" => Some(Scenario::Mock),
            "tier-c" => Some(Scenario::TierC),
            "stress" => Some(Scenario::Stress),
            "asr-only" => Some(Scenario::AsrOnly),
            "llm-only" => Some(Scenario::LlmOnly),
            "simplify-only" => Some(Scenario::SimplifyOnly),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Scenario::Auto => "auto",
            Scenario::CpuStrict => "cpu-strict",
            Scenario::NoGpu => "no-gpu",
            Scenario::VulkanStrict => "vulkan-strict",
            Scenario::CudaStrict => "cuda-strict",
            Scenario::IgpuOnly => "igpu-only",
            Scenario::DgpuOnly => "dgpu-only",
            Scenario::Mock => "mock",
            Scenario::TierC => "tier-c",
            Scenario::Stress => "stress",
            Scenario::AsrOnly => "asr-only",
            Scenario::LlmOnly => "llm-only",
            Scenario::SimplifyOnly => "simplify-only",
        }
    }

    pub fn is_strict(self) -> bool {
        matches!(
            self,
            Scenario::CpuStrict
                | Scenario::NoGpu
                | Scenario::VulkanStrict
                | Scenario::CudaStrict
                | Scenario::IgpuOnly
                | Scenario::DgpuOnly
        )
    }

    pub fn apply(self, config: &mut RuntimeConfig) {
        config.strict = self.is_strict();
        match self {
            Scenario::Auto => {
                config.backend = Backend::Auto;
                config.calibration = CalibrationMode::Auto;
            }
            Scenario::CpuStrict | Scenario::NoGpu => {
                config.backend = Backend::Cpu;
                config.gpu_layers = GpuLayers::Zero;
                config.device = DevicePreference::Auto;
                config.calibration = CalibrationMode::Skip;
            }
            Scenario::VulkanStrict => {
                config.backend = Backend::Vulkan;
                config.calibration = CalibrationMode::Skip;
            }
            Scenario::CudaStrict => {
                config.backend = Backend::Cuda;
                config.calibration = CalibrationMode::Skip;
            }
            Scenario::IgpuOnly => {
                config.device = DevicePreference::Integrated;
                config.calibration = CalibrationMode::Skip;
            }
            Scenario::DgpuOnly => {
                config.device = DevicePreference::Discrete;
                config.calibration = CalibrationMode::Skip;
            }
            Scenario::Mock => {
                config.asr = AsrMode::Mock;
                config.backend = Backend::Cpu;
                config.audio = AudioMode::Demo;
                config.pipeline = PipelineMode::OnDemand;
            }
            Scenario::TierC => {
                config.backend = Backend::Cpu;
                config.gpu_layers = GpuLayers::Zero;
                config.llm_model = LlmModel::Qwen06b;
                config.quant = Quant::Q4Km;
                config.threads = 3;
                config.asr_threads = 2;
                config.pipeline = PipelineMode::OnDemand;
                config.calibration = CalibrationMode::ForceCpu;
            }
            Scenario::Stress => {
                config.llm_model = LlmModel::Qwen17b;
                config.pipeline = PipelineMode::Live;
                config.validators = true;
                config.reversion = true;
            }
            Scenario::AsrOnly => {
                config.asr = AsrMode::Mock;
                config.validators = false;
            }
            Scenario::LlmOnly => {
                config.asr = AsrMode::Off;
                config.audio = AudioMode::Silence;
            }
            Scenario::SimplifyOnly => {
                config.asr = AsrMode::Off;
                config.audio = AudioMode::Silence;
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(
    export,
    rename_all = "camelCase",
    export_to = "../../../../ui/src/bridge/generated/"
)]
pub struct EventDeliveryStatus {
    #[ts(type = "number")]
    pub last_seq: u64,
    #[ts(type = "number")]
    pub last_event_age_ms: u64,
    #[ts(type = "number")]
    pub dropped: u64,
}

/// The dev panel / health card status (plan §14.4).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(
    export,
    rename_all = "camelCase",
    export_to = "../../../../ui/src/bridge/generated/"
)]
pub struct RuntimeStatus {
    pub requested_backend: String,
    pub effective_backend: String,
    pub requested_device: String,
    pub effective_device: String,
    pub requested_gpu_layers: String,
    pub effective_gpu_layers: u32,
    pub loaded_backend_modules: Vec<String>,
    pub cpu_variant: String,
    pub ui_backend: String,
    pub window_policy: String,
    pub model: String,
    pub quant: String,
    pub model_hash: Option<String>,
    pub ttft_ms: Option<f64>,
    pub tokens_per_sec: Option<f64>,
    pub last_calibration: Option<String>,
    pub fallback_reason: Option<String>,
    pub strict: bool,
    pub event_delivery: EventDeliveryStatus,
}

impl Default for RuntimeStatus {
    fn default() -> Self {
        Self {
            requested_backend: "auto".to_string(),
            effective_backend: "cpu".to_string(),
            requested_device: "auto".to_string(),
            effective_device: "cpu".to_string(),
            requested_gpu_layers: "auto".to_string(),
            effective_gpu_layers: 0,
            loaded_backend_modules: vec!["cpu".to_string()],
            cpu_variant: cpu_variant(),
            ui_backend: default_ui_backend(),
            window_policy: "console".to_string(),
            model: "qwen3-0.6b-easylanguage".to_string(),
            quant: "q4_k_m".to_string(),
            model_hash: None,
            ttft_ms: None,
            tokens_per_sec: None,
            last_calibration: None,
            fallback_reason: None,
            strict: false,
            event_delivery: EventDeliveryStatus::default(),
        }
    }
}

impl RuntimeStatus {
    pub fn requested(config: &RuntimeConfig) -> Self {
        Self {
            requested_backend: config.backend.as_str().to_string(),
            requested_device: config.device.as_str().to_string(),
            requested_gpu_layers: config.gpu_layers.as_string(),
            effective_backend: match config.backend {
                Backend::Auto | Backend::Metal => "cpu".to_string(),
                other => other.as_str().to_string(),
            },
            effective_device: match config.device {
                DevicePreference::Discrete => "discrete-gpu".to_string(),
                DevicePreference::Integrated => "integrated-gpu".to_string(),
                DevicePreference::Auto => "cpu".to_string(),
            },
            effective_gpu_layers: if config.backend == Backend::Cpu {
                0
            } else {
                config.gpu_layers.effective(28)
            },
            strict: config.strict,
            model: format!("qwen3-{}-easylanguage", config.llm_model.as_str()),
            quant: config.quant.as_str().to_string(),
            window_policy: config.window_mode.as_str().to_string(),
            ..Self::default()
        }
    }

    pub fn with_effective(
        mut self,
        backend: &str,
        device: &str,
        modules: Vec<String>,
        fallback_reason: Option<String>,
    ) -> Self {
        self.effective_backend = backend.to_string();
        self.effective_device = device.to_string();
        self.loaded_backend_modules = modules;
        self.fallback_reason = fallback_reason;
        self
    }
}

fn cpu_variant() -> String {
    #[cfg(target_arch = "x86_64")]
    {
        if std::is_x86_feature_detected!("avx512f") {
            return "avx512".to_string();
        }
        if std::is_x86_feature_detected!("avx2") {
            return "avx2".to_string();
        }
        return "x86_64".to_string();
    }
    #[allow(unreachable_code)]
    std::env::consts::ARCH.to_string()
}

fn default_ui_backend() -> String {
    if cfg!(target_os = "windows") {
        "d3d12".to_string()
    } else if cfg!(target_os = "macos") {
        "metal".to_string()
    } else {
        "vulkan".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layer(backend: Option<Backend>) -> ConfigLayer {
        ConfigLayer {
            backend,
            ..ConfigLayer::default()
        }
    }

    #[test]
    fn defaults_are_tier_a_friendly() {
        let config = RuntimeConfig::default();
        assert_eq!(config.backend, Backend::Auto);
        assert_eq!(config.llm_model, LlmModel::Qwen06b);
        assert_eq!(config.pipeline, PipelineMode::OnDemand);
        assert!(!config.strict);
        assert!(config.threads >= 2);
    }

    #[test]
    fn precedence_cli_over_env_over_dev_over_user() {
        let user = UserSettings {
            layer: layer(Some(Backend::Cpu)),
        };
        let dev = DevOverrides {
            schema_version: 1,
            layer: layer(Some(Backend::Vulkan)),
        };
        let env = layer(Some(Backend::Cuda));
        let cli = layer(Some(Backend::Metal));
        assert_eq!(
            resolve(Some(&cli), Some(&env), Some(&dev), Some(&user), None).backend,
            Backend::Metal
        );
        assert_eq!(
            resolve(None, Some(&env), Some(&dev), Some(&user), None).backend,
            Backend::Cuda
        );
        assert_eq!(
            resolve(None, None, Some(&dev), Some(&user), None).backend,
            Backend::Vulkan
        );
        assert_eq!(
            resolve(None, None, None, Some(&user), None).backend,
            Backend::Cpu
        );
    }

    #[test]
    fn env_contract_parses() {
        let mut env = BTreeMap::new();
        env.insert("LL_BACKEND".to_string(), "vulkan".to_string());
        env.insert("LL_GPU_LAYERS".to_string(), "24".to_string());
        env.insert("LL_THREADS".to_string(), "6".to_string());
        env.insert("LL_LLM_MODEL".to_string(), "1.7b".to_string());
        env.insert("LL_PIPELINE".to_string(), "live".to_string());
        env.insert("LL_STRICT_BACKEND".to_string(), "1".to_string());
        env.insert("LL_FAULT_LLM".to_string(), "oom".to_string());
        let layer = ConfigLayer::from_env_map(&env);
        let config = resolve(Some(&layer), None, None, None, None);
        assert_eq!(config.backend, Backend::Vulkan);
        assert_eq!(config.gpu_layers, GpuLayers::Count(24));
        assert_eq!(config.threads, 6);
        assert_eq!(config.llm_model, LlmModel::Qwen17b);
        assert_eq!(config.pipeline, PipelineMode::Live);
        assert!(config.strict);
        assert_eq!(config.fault_llm, LlmFault::Oom);
    }

    #[test]
    fn scenarios_force_expected_settings() {
        for (scenario, backend, strict) in [
            (Scenario::CpuStrict, Backend::Cpu, true),
            (Scenario::NoGpu, Backend::Cpu, true),
            (Scenario::VulkanStrict, Backend::Vulkan, true),
            (Scenario::CudaStrict, Backend::Cuda, true),
            (Scenario::Mock, Backend::Cpu, false),
        ] {
            let mut config = RuntimeConfig::default();
            scenario.apply(&mut config);
            assert_eq!(config.backend, backend, "{}", scenario.as_str());
            assert_eq!(config.strict, strict, "{}", scenario.as_str());
        }
        let mut tier_c = RuntimeConfig::default();
        Scenario::TierC.apply(&mut tier_c);
        assert_eq!(tier_c.llm_model, LlmModel::Qwen06b);
        assert_eq!(tier_c.threads, 3);
        assert_eq!(tier_c.pipeline, PipelineMode::OnDemand);
    }

    #[test]
    fn scenario_parse_accepts_plan_names() {
        for scenario in Scenario::ALL {
            assert_eq!(Scenario::parse(scenario.as_str()), Some(scenario));
        }
        assert_eq!(Scenario::parse("mock-pipeline"), Some(Scenario::Mock));
        assert_eq!(Scenario::parse("nope"), None);
    }

    #[test]
    fn runtime_status_reports_requested_and_effective() {
        let config = RuntimeConfig {
            backend: Backend::Cpu,
            ..RuntimeConfig::default()
        };
        let status = RuntimeStatus::requested(&config);
        assert_eq!(status.requested_backend, "cpu");
        assert_eq!(status.effective_backend, "cpu");
        assert_eq!(status.effective_gpu_layers, 0);
        assert!(!status.cpu_variant.is_empty());
        let status = status.with_effective("cpu", "cpu", vec!["cpu".into()], None);
        assert_eq!(status.loaded_backend_modules, vec!["cpu"]);
    }

    #[test]
    fn gpu_layers_parse_and_effective() {
        assert_eq!(GpuLayers::parse("auto"), Some(GpuLayers::Auto));
        assert_eq!(GpuLayers::parse("all"), Some(GpuLayers::All));
        assert_eq!(GpuLayers::parse("24"), Some(GpuLayers::Count(24)));
        assert_eq!(GpuLayers::Count(24).effective(28), 24);
        assert_eq!(GpuLayers::All.effective(28), 28);
        assert_eq!(GpuLayers::Zero.effective(28), 0);
    }
}
