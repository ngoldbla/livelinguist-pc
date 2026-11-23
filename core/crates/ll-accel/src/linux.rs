//! Linux development inventory: DRM render nodes, Vulkan ICDs, and CUDA.
//!
//! Linux dev uses Vulkan; lavapipe works but is software-class, so a machine
//! without a render node is reported as degraded rather than GPU accelerated.

use crate::summarize_ui;
use ll_config::{AdapterInfo, HardwareReport, RuntimeSupport};
use std::path::Path;

const ICD_DIRS: [&str; 4] = [
    "/usr/share/vulkan/icd.d",
    "/usr/local/share/vulkan/icd.d",
    "/etc/vulkan/icd.d",
    "/run/opengl-driver/share/vulkan/icd.d",
];

pub(crate) fn detect() -> HardwareReport {
    let adapters = drm_adapters();
    let render_nodes = read_dir_count("/dev/dri", |name| name.starts_with("renderD"));
    let icd_files = ICD_DIRS
        .iter()
        .map(|dir| read_dir_count(dir, |name| name.ends_with(".json")))
        .sum::<usize>();
    let env_icd = std::env::var_os("VK_ICD_FILENAMES").is_some();
    let loader = [
        "/usr/lib/x86_64-linux-gnu/libvulkan.so.1",
        "/usr/lib/aarch64-linux-gnu/libvulkan.so.1",
        "/usr/lib64/libvulkan.so.1",
        "/usr/lib/libvulkan.so.1",
    ]
    .iter()
    .any(|path| Path::new(path).exists());
    let api_available = loader || icd_files > 0 || env_icd;
    let uevent_driver = drm_driver();
    let ui = summarize_ui(
        "vulkan",
        api_available && (render_nodes > 0 || icd_files > 0),
        &adapters,
    );
    let mut ui = ui;
    if ui.hardware && render_nodes == 0 {
        ui.hardware = false;
        ui.degraded_reason = Some(
            "a DRM adapter is listed but no /dev/dri/renderD node is usable; software rendering is degraded"
                .to_string(),
        );
    }
    if let Some(driver) = uevent_driver {
        ui.driver = Some(driver);
    }
    ui.notes.push(format!("render nodes: {render_nodes}"));
    ui.notes.push(format!("Vulkan ICD files: {icd_files}"));

    HardwareReport {
        ui,
        adapters,
        vulkan: RuntimeSupport {
            available: api_available,
            detail: format!(
                "libvulkan {}; {icd_files} ICD file(s) in {}",
                if loader { "present" } else { "missing" },
                ICD_DIRS[0]
            ),
            devices: Vec::new(),
        },
        cuda: cuda_support(),
    }
}

fn read_dir_count(path: &str, matches: impl Fn(&str) -> bool) -> usize {
    std::fs::read_dir(path)
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| matches(&entry.file_name().to_string_lossy()))
                .count()
        })
        .unwrap_or(0)
}

fn drm_adapters() -> Vec<AdapterInfo> {
    let entries = match std::fs::read_dir("/sys/class/drm") {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };
    let mut adapters = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }
        let device = entry.path().join("device");
        if !device.exists() {
            continue;
        }
        let uevent = std::fs::read_to_string(device.join("uevent")).unwrap_or_default();
        let driver = uevent_value(&uevent, "DRIVER").unwrap_or_else(|| "unknown".to_string());
        let pci = uevent_value(&uevent, "PCI_ID").unwrap_or_else(|| "0000:0000".to_string());
        let (vendor_id, device_id) = parse_pci_id(&pci);
        let vram_bytes = std::fs::read_to_string(device.join("mem_info_vram_total"))
            .ok()
            .and_then(|value| value.trim().parse::<u64>().ok())
            .unwrap_or(0);
        let vram_mb = vram_bytes / (1024 * 1024);
        let label = vendor_label(vendor_id);
        let adapter_name = format!("{label} {pci} ({driver})");
        let kind = crate::adapter_kind(&adapter_name, vram_mb, false);
        adapters.push(AdapterInfo {
            name: adapter_name,
            vendor_id,
            device_id,
            dedicated_vram_mb: vram_mb,
            software: false,
            kind: kind.to_string(),
        });
    }
    adapters
}

fn drm_driver() -> Option<String> {
    let entries = std::fs::read_dir("/sys/class/drm").ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }
        let uevent = std::fs::read_to_string(entry.path().join("device").join("uevent")).ok()?;
        if let Some(driver) = uevent_value(&uevent, "DRIVER") {
            return Some(driver);
        }
    }
    None
}

fn uevent_value(uevent: &str, key: &str) -> Option<String> {
    uevent.lines().find_map(|line| {
        line.strip_prefix(&format!("{key}="))
            .map(|value| value.trim().to_string())
    })
}

fn parse_pci_id(value: &str) -> (u32, u32) {
    let mut parts = value.split(':');
    let vendor = parts
        .next()
        .and_then(|part| u32::from_str_radix(part, 16).ok())
        .unwrap_or(0);
    let device = parts
        .next()
        .and_then(|part| u32::from_str_radix(part, 16).ok())
        .unwrap_or(0);
    (vendor, device)
}

fn vendor_label(vendor_id: u32) -> &'static str {
    match vendor_id {
        0x10de => "NVIDIA",
        0x1002 | 0x1022 => "AMD",
        0x8086 => "Intel",
        0x5143 => "Qualcomm",
        _ => "GPU",
    }
}

fn cuda_support() -> RuntimeSupport {
    let present =
        Path::new("/proc/driver/nvidia/version").exists() || Path::new("/dev/nvidia0").exists();
    RuntimeSupport {
        available: present,
        detail: if present {
            "NVIDIA driver interface present".to_string()
        } else {
            "no NVIDIA driver interface".to_string()
        },
        devices: Vec::new(),
    }
}
