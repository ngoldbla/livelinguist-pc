//! Native window policy: Win32 primary overlay, X11/XWayland dev-only.
//!
//! The policy is applied after GPUI maps the window. Linux has no
//! cross-compositor always-on-top guarantee; failures are reported honestly
//! and the UI falls back to a normal window with an in-app banner.

use ll_config::WindowMode;
use serde::{Deserialize, Serialize};

#[cfg(target_os = "windows")]
pub mod win32;
#[cfg(target_os = "linux")]
pub mod x11;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowPolicyResult {
    pub applied: bool,
    pub backend: String,
    pub notes: Vec<String>,
}

impl WindowPolicyResult {
    pub fn fallback(backend: &str, note: impl Into<String>) -> Self {
        Self {
            applied: false,
            backend: backend.to_string(),
            notes: vec![note.into()],
        }
    }
}

/// The usable area of the monitor the app opens on (taskbar excluded), in
/// physical pixels, plus the display scale (1.0 = 96 DPI).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkArea {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale: f64,
}

/// Monitor work area: Windows uses the monitor under the cursor (primary as
/// fallback) and its effective DPI; Linux dev uses `_NET_WORKAREA` or the X11
/// root window at scale 1.0. `None` when the platform cannot tell.
pub fn work_area() -> Option<WorkArea> {
    #[cfg(target_os = "windows")]
    {
        win32::work_area()
    }
    #[cfg(target_os = "linux")]
    {
        std::env::var_os("DISPLAY")?;
        x11::work_area()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        None
    }
}

/// Resize the app window to a logical size (96-DPI pixels). Returns `false`
/// when the window cannot be found or the platform has no resize path.
pub fn resize_current_window(title_nonce: &str, width: u32, height: u32) -> bool {
    #[cfg(target_os = "windows")]
    {
        win32::resize_current(title_nonce, width, height)
    }
    #[cfg(target_os = "linux")]
    {
        std::env::var_os("DISPLAY").is_some()
            && x11::resize_current(title_nonce, width, height).is_ok()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        let _ = (title_nonce, width, height);
        false
    }
}

/// The policy table from plan §10.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Policy {
    pub topmost: bool,
    pub no_activate: bool,
    pub skip_taskbar: bool,
    pub sticky: bool,
    pub borderless: bool,
    pub utility_window: bool,
}

pub fn policy_for(mode: WindowMode) -> Policy {
    match mode {
        WindowMode::Console => Policy {
            topmost: false,
            no_activate: false,
            skip_taskbar: false,
            sticky: false,
            borderless: false,
            utility_window: false,
        },
        WindowMode::Overlay => Policy {
            topmost: true,
            no_activate: true,
            skip_taskbar: true,
            sticky: true,
            borderless: true,
            utility_window: true,
        },
        WindowMode::Expanded => Policy {
            topmost: true,
            no_activate: false,
            skip_taskbar: true,
            sticky: false,
            borderless: true,
            utility_window: true,
        },
    }
}

/// Apply the current mode to the GPUI window. `title_nonce` disambiguates the
/// window; discovery never relies on a generic title alone.
pub fn apply_for_current_window(mode: WindowMode, title_nonce: &str) -> WindowPolicyResult {
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("WAYLAND_DISPLAY").is_some() && std::env::var_os("DISPLAY").is_none() {
            return WindowPolicyResult::fallback(
                "wayland",
                "native Wayland layer-shell is not in v1; run with X11/XWayland for the dev overlay",
            );
        }
        if std::env::var_os("DISPLAY").is_none() {
            return WindowPolicyResult::fallback("none", "no DISPLAY; window policy not applied");
        }
        match x11::apply_for_current_process(mode, title_nonce) {
            Ok(result) => result,
            Err(error) => WindowPolicyResult::fallback("x11", error),
        }
    }
    #[cfg(target_os = "windows")]
    {
        match win32::apply_for_current_process(mode, title_nonce) {
            Ok(result) => result,
            Err(error) => WindowPolicyResult::fallback("win32", error),
        }
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        let _ = (mode, title_nonce);
        WindowPolicyResult::fallback("unsupported", "window policy is Windows + Linux dev only")
    }
}

/// Drag helper: begin a window move from a caption drag.
pub fn begin_drag() -> bool {
    #[cfg(target_os = "linux")]
    {
        false // the UI uses _NET_WM_MOVERESIZE through the X11 helper
    }
    #[cfg(target_os = "windows")]
    {
        win32::begin_drag_current()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_table_matches_the_plan() {
        let console = policy_for(WindowMode::Console);
        assert!(!console.topmost && !console.no_activate && !console.skip_taskbar);
        let overlay = policy_for(WindowMode::Overlay);
        assert!(overlay.topmost && overlay.no_activate && overlay.skip_taskbar && overlay.sticky);
        let expanded = policy_for(WindowMode::Expanded);
        assert!(expanded.topmost && !expanded.no_activate && expanded.skip_taskbar);
    }

    #[test]
    fn work_area_serializes_camel_case() {
        let area = WorkArea {
            x: 0,
            y: 0,
            width: 1920,
            height: 1032,
            scale: 1.5,
        };
        let json = serde_json::to_string(&area).unwrap();
        assert_eq!(
            json,
            r#"{"x":0,"y":0,"width":1920,"height":1032,"scale":1.5}"#
        );
    }

    #[test]
    fn work_area_is_sane_when_present() {
        if let Some(area) = work_area() {
            assert!(area.width > 0 && area.height > 0);
            assert!((1.0..=3.0).contains(&area.scale));
        }
    }

    #[test]
    fn fallback_is_explicit() {
        let result = WindowPolicyResult::fallback("x11", "no window manager");
        assert!(!result.applied);
        assert!(result.notes[0].contains("window manager"));
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"applied\":false"));
    }
}
