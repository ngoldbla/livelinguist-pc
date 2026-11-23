//! Measured UI renderer and accelerator inventory.
//!
//! The renderer is never inferred from `cfg!(target_os)`: a Windows build on a
//! machine without a display driver must report a software adapter and a
//! degraded reason, not "d3d12". Values here are evidence for the dev panel and
//! the backend probe; they are not performance measurements.

use ll_config::{AdapterInfo, HardwareReport, UiRendererStatus};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

/// Probe the renderer and accelerator runtimes once.
pub fn detect() -> HardwareReport {
    #[cfg(target_os = "windows")]
    {
        windows::detect()
    }
    #[cfg(target_os = "linux")]
    {
        linux::detect()
    }
    #[cfg(target_os = "macos")]
    {
        HardwareReport {
            ui: summarize_ui("metal", true, &[]),
            adapters: Vec::new(),
            vulkan: ll_config::RuntimeSupport {
                available: false,
                detail: "Vulkan is not the UI backend on macOS".to_string(),
                devices: Vec::new(),
            },
            cuda: ll_config::RuntimeSupport {
                available: false,
                detail: "CUDA is not bundled on macOS".to_string(),
                devices: Vec::new(),
            },
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        HardwareReport {
            ui: summarize_ui("unknown", false, &[]),
            adapters: Vec::new(),
            vulkan: ll_config::RuntimeSupport::default(),
            cuda: ll_config::RuntimeSupport::default(),
        }
    }
}

/// Probe only the UI renderer status.
pub fn detect_ui() -> UiRendererStatus {
    detect().ui
}

/// Classify an adapter without pretending a software device is a GPU.
pub fn adapter_kind(name: &str, dedicated_vram_mb: u64, software: bool) -> &'static str {
    if software {
        return "software";
    }
    let lower = name.to_ascii_lowercase();
    let looks_integrated = (lower.contains("intel") && dedicated_vram_mb < 1024)
        || (lower.contains("radeon") && lower.contains("graphics") && dedicated_vram_mb < 1024)
        || (lower.contains("vega") && dedicated_vram_mb < 1024);
    if looks_integrated {
        "integrated"
    } else {
        "discrete"
    }
}

/// Build the UI status from enumerated adapters.
///
/// This is shared by every platform so the "software is degraded" rule is
/// tested once, on every OS.
pub fn summarize_ui(
    backend: &str,
    api_available: bool,
    adapters: &[AdapterInfo],
) -> UiRendererStatus {
    let hardware_adapter = adapters.iter().find(|adapter| !adapter.software);
    let software_adapter = adapters.iter().find(|adapter| adapter.software);
    let chosen = hardware_adapter.or(software_adapter);
    let mut notes = Vec::new();
    if let Some(adapter) = hardware_adapter {
        notes.push(format!("hardware adapter: {}", adapter.name));
    }
    if hardware_adapter.is_some() {
        for adapter in adapters.iter().filter(|adapter| adapter.software) {
            notes.push(format!("software adapter also present: {}", adapter.name));
        }
    }
    if adapters.is_empty() {
        notes.push("no adapters enumerated".to_string());
    }

    let degraded_reason = if !api_available {
        Some(format!("{backend} runtime is not available"))
    } else if hardware_adapter.is_none() {
        Some(match software_adapter {
            Some(adapter) => format!(
                "only a software adapter ({}) is available; rendering is not GPU accelerated",
                adapter.name
            ),
            None => "no adapter is available; rendering is not GPU accelerated".to_string(),
        })
    } else {
        None
    };

    UiRendererStatus {
        backend: backend.to_string(),
        api_available,
        hardware: hardware_adapter.is_some(),
        adapter: chosen.map(|adapter| adapter.name.clone()),
        driver: None,
        degraded_reason,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn adapter(name: &str, software: bool, vram_mb: u64) -> AdapterInfo {
        AdapterInfo {
            name: name.to_string(),
            vendor_id: 0,
            device_id: 0,
            dedicated_vram_mb: vram_mb,
            software,
            kind: adapter_kind(name, vram_mb, software).to_string(),
        }
    }

    #[test]
    fn no_adapters_is_degraded() {
        let ui = summarize_ui("directx", true, &[]);
        assert!(!ui.hardware);
        assert!(ui.degraded_reason.is_some());
        assert!(ui.notes.iter().any(|note| note.contains("no adapters")));
    }

    #[test]
    fn software_only_is_degraded_and_named() {
        let basic = adapter("Microsoft Basic Display Adapter", true, 0);
        let ui = summarize_ui("directx", true, std::slice::from_ref(&basic));
        assert!(!ui.hardware);
        assert_eq!(
            ui.adapter.as_deref(),
            Some("Microsoft Basic Display Adapter")
        );
        let reason = ui.degraded_reason.unwrap();
        assert!(reason.contains("not GPU accelerated"));
        assert!(!ui.notes.iter().any(|note| note.contains("no adapters")));
    }

    #[test]
    fn hardware_wins_over_software() {
        let basic = adapter("Microsoft Basic Display Adapter", true, 0);
        let gpu = adapter("NVIDIA GeForce RTX 3050", false, 6144);
        let ui = summarize_ui("directx", true, &[basic.clone(), gpu.clone()]);
        assert!(ui.hardware);
        assert_eq!(ui.adapter.as_deref(), Some("NVIDIA GeForce RTX 3050"));
        assert!(ui.degraded_reason.is_none());
        assert!(ui.notes.iter().any(|note| note.contains("Basic Display")));
    }

    #[test]
    fn unavailable_runtime_is_degraded_even_with_an_adapter() {
        let gpu = adapter("NVIDIA GeForce RTX 3050", false, 6144);
        let ui = summarize_ui("vulkan", false, std::slice::from_ref(&gpu));
        assert!(!ui.api_available);
        assert!(ui.hardware);
        assert!(ui.degraded_reason.unwrap().contains("not available"));
    }

    #[test]
    fn adapter_kind_never_calls_software_a_gpu() {
        assert_eq!(
            adapter_kind("Microsoft Basic Display Adapter", 0, true),
            "software"
        );
        assert_eq!(
            adapter_kind("Intel UHD Graphics 630", 128, false),
            "integrated"
        );
        assert_eq!(
            adapter_kind("AMD Radeon Graphics", 512, false),
            "integrated"
        );
        assert_eq!(
            adapter_kind("AMD Radeon RX 7900 XT", 20480, false),
            "discrete"
        );
        assert_eq!(
            adapter_kind("NVIDIA GeForce RTX 3050", 6144, false),
            "discrete"
        );
    }

    #[test]
    fn detection_is_total_and_never_panics() {
        let report = detect();
        assert!(!report.ui.backend.is_empty());
        if !report.ui.api_available {
            assert!(report.ui.degraded_reason.is_some());
        }
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"ui\":"));
    }
}
