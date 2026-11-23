//! Windows renderer and accelerator inventory through DXGI and the registry.
//!
//! DXGI is the authority for adapters: a machine with the Microsoft Basic
//! Display Adapter enumerates a software adapter and must be reported as
//! degraded. Vulkan and CUDA are reported separately; neither is the UI
//! backend on Windows.

use crate::summarize_ui;
use ll_config::{AdapterInfo, HardwareReport, RuntimeSupport};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{FreeLibrary, ERROR_SUCCESS};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE,
};
use windows::Win32::System::LibraryLoader::LoadLibraryW;
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryInfoKeyW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ,
};

pub(crate) fn detect() -> HardwareReport {
    let (adapters, api_available) = enumerate_adapters();
    let ui = summarize_ui("directx", api_available, &adapters);
    let vulkan = vulkan_support();
    let cuda = cuda_support();
    HardwareReport {
        ui,
        adapters,
        vulkan,
        cuda,
    }
}

fn enumerate_adapters() -> (Vec<AdapterInfo>, bool) {
    let mut adapters = Vec::new();
    unsafe {
        let factory: IDXGIFactory1 = match CreateDXGIFactory1() {
            Ok(factory) => factory,
            Err(_) => return (adapters, false),
        };
        let mut index = 0u32;
        while let Ok(adapter) = factory.EnumAdapters1(index) {
            index += 1;
            let Ok(desc) = adapter.GetDesc1() else {
                continue;
            };
            let name = String::from_utf16_lossy(&desc.Description)
                .trim_end_matches('\0')
                .to_string();
            if name.is_empty() {
                continue;
            }
            let software = (desc.Flags & (DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32)) != 0;
            let vram_mb = (desc.DedicatedVideoMemory / (1024 * 1024)) as u64;
            adapters.push(AdapterInfo {
                name: name.clone(),
                vendor_id: desc.VendorId,
                device_id: desc.DeviceId,
                dedicated_vram_mb: vram_mb,
                software,
                kind: crate::adapter_kind(&name, vram_mb, software).to_string(),
            });
        }
    }
    (adapters, true)
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn library_present(name: &str) -> bool {
    let name = wide(name);
    unsafe {
        match LoadLibraryW(PCWSTR(name.as_ptr())) {
            Ok(module) => {
                let _ = FreeLibrary(module);
                true
            }
            Err(_) => false,
        }
    }
}

fn vulkan_icd_entries() -> Result<u32, String> {
    let path = wide("SOFTWARE\\Khronos\\Vulkan\\Drivers");
    unsafe {
        let mut key = HKEY::default();
        let status = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR(path.as_ptr()),
            Some(0),
            KEY_READ,
            &mut key,
        );
        if status != ERROR_SUCCESS {
            return Err(format!("{status:?}"));
        }
        let mut values = 0u32;
        let query = RegQueryInfoKeyW(
            key,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(&mut values),
            None,
            None,
            None,
            None,
        );
        let _ = RegCloseKey(key);
        if query != ERROR_SUCCESS {
            return Err(format!("{query:?}"));
        }
        Ok(values)
    }
}

fn vulkan_support() -> RuntimeSupport {
    let loader = library_present("vulkan-1.dll");
    match vulkan_icd_entries() {
        Ok(entries) => RuntimeSupport {
            available: loader && entries > 0,
            detail: format!(
                "Vulkan loader {}; {entries} ICD registration(s) under HKLM\\SOFTWARE\\Khronos\\Vulkan\\Drivers",
                if loader { "present" } else { "missing" }
            ),
            devices: Vec::new(),
        },
        Err(error) => RuntimeSupport {
            available: false,
            detail: format!(
                "Vulkan loader {}; ICD registry unavailable ({error})",
                if loader { "present" } else { "missing" }
            ),
            devices: Vec::new(),
        },
    }
}

fn cuda_support() -> RuntimeSupport {
    let name = wide("nvcuda.dll");
    unsafe {
        let module = match LoadLibraryW(PCWSTR(name.as_ptr())) {
            Ok(module) => module,
            Err(_) => {
                return RuntimeSupport {
                    available: false,
                    detail: "nvcuda.dll not found (no NVIDIA driver API)".to_string(),
                    devices: Vec::new(),
                }
            }
        };
        let mut detail = "nvcuda.dll loaded; driver API entry points missing".to_string();
        let mut devices = Vec::new();
        if let Some(api) = CudaApi::load(module) {
            if api.init() == CUDA_SUCCESS {
                let count = api.count().unwrap_or(0).max(0);
                for index in 0..count {
                    if let Some(device) = api.device(index) {
                        devices.push(device);
                    }
                }
                detail = match api.driver_version() {
                    Some(version) => format!(
                        "CUDA driver {}.{}; {} device(s)",
                        version / 1000,
                        (version % 1000) / 10,
                        devices.len()
                    ),
                    None => format!("CUDA driver version unknown; {} device(s)", devices.len()),
                };
            } else {
                detail = "nvcuda.dll loaded; cuInit failed".to_string();
            }
        }
        let _ = FreeLibrary(module);
        RuntimeSupport {
            available: !devices.is_empty(),
            detail,
            devices,
        }
    }
}

const CUDA_SUCCESS: i32 = 0;

type CuInit = unsafe extern "system" fn(u32) -> i32;
type CuGetCount = unsafe extern "system" fn(*mut i32) -> i32;
type CuGetDevice = unsafe extern "system" fn(*mut i32, i32) -> i32;
type CuGetName = unsafe extern "system" fn(*mut u8, i32, i32) -> i32;
type CuTotalMem = unsafe extern "system" fn(*mut usize, i32) -> i32;
type CuCompute = unsafe extern "system" fn(*mut i32, *mut i32, i32) -> i32;
type CuDriverVersion = unsafe extern "system" fn(*mut i32) -> i32;

struct CudaApi {
    init: CuInit,
    count: CuGetCount,
    device: CuGetDevice,
    name: CuGetName,
    total_mem: CuTotalMem,
    compute: CuCompute,
    driver_version: CuDriverVersion,
}

impl CudaApi {
    fn load(module: windows::Win32::Foundation::HMODULE) -> Option<Self> {
        unsafe {
            Some(Self {
                init: symbol(module, b"cuInit\0")?,
                count: symbol(module, b"cuDeviceGetCount\0")?,
                device: symbol(module, b"cuDeviceGet\0")?,
                name: symbol(module, b"cuDeviceGetName\0")?,
                total_mem: symbol(module, b"cuDeviceTotalMem_v2\0")
                    .or_else(|| symbol(module, b"cuDeviceTotalMem\0"))?,
                compute: symbol(module, b"cuDeviceComputeCapability\0")?,
                driver_version: symbol(module, b"cuDriverGetVersion\0")?,
            })
        }
    }

    fn init(&self) -> i32 {
        unsafe { (self.init)(0) }
    }

    fn count(&self) -> Option<i32> {
        let mut count = 0i32;
        (unsafe { (self.count)(&mut count) } == CUDA_SUCCESS).then_some(count)
    }

    fn driver_version(&self) -> Option<i32> {
        let mut version = 0i32;
        (unsafe { (self.driver_version)(&mut version) } == CUDA_SUCCESS).then_some(version)
    }

    fn device(&self, index: i32) -> Option<ll_config::AcceleratorDevice> {
        unsafe {
            let mut device = 0i32;
            if (self.device)(&mut device, index) != CUDA_SUCCESS {
                return None;
            }
            let mut name = [0u8; 256];
            if (self.name)(name.as_mut_ptr(), name.len() as i32, device) != CUDA_SUCCESS {
                return None;
            }
            let end = name
                .iter()
                .position(|byte| *byte == 0)
                .unwrap_or(name.len());
            let name = String::from_utf8_lossy(&name[..end]).trim().to_string();
            let mut bytes = 0usize;
            let vram_total_mb = if (self.total_mem)(&mut bytes, device) == CUDA_SUCCESS {
                (bytes / (1024 * 1024)) as u64
            } else {
                0
            };
            let mut major = 0i32;
            let mut minor = 0i32;
            let compute = ((self.compute)(&mut major, &mut minor, device) == CUDA_SUCCESS)
                .then(|| format!("{major}.{minor}"));
            Some(ll_config::AcceleratorDevice {
                name,
                vram_total_mb,
                compute,
            })
        }
    }
}

unsafe fn symbol<T: Copy>(module: windows::Win32::Foundation::HMODULE, name: &[u8]) -> Option<T> {
    use windows::core::PCSTR;
    use windows::Win32::System::LibraryLoader::GetProcAddress;
    let proc = GetProcAddress(module, PCSTR(name.as_ptr()))?;
    Some(std::mem::transmute_copy(&proc))
}
