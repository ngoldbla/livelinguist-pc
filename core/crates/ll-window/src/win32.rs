//! Win32 overlay policy (the release-gated path).
//!
//! `SetWindowPos(HWND_TOPMOST)` + `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW` and a
//! `SetWindowSubclass` hook returning `MA_NOACTIVATE` from `WM_MOUSEACTIVATE`,
//! so clicking a caption cannot steal focus. Raw FFI keeps the target code
//! small and stable across `windows` crate releases; it is checked by the
//! Windows CI job.

#![cfg(target_os = "windows")]

use crate::{policy_for, WindowPolicyResult};
use ll_config::WindowMode;
use std::ffi::c_void;

type Hwnd = isize;
type Hresult = i32;

const HWND_TOPMOST: Hwnd = -1;
const HWND_NOTOPMOST: Hwnd = -2;
const SWP_NOSIZE: u32 = 0x0001;
const SWP_NOMOVE: u32 = 0x0002;
const SWP_NOACTIVATE: u32 = 0x0010;
const GWL_EXSTYLE: i32 = -20;
const WS_EX_NOACTIVATE: isize = 0x0800_0000;
const WS_EX_TOOLWINDOW: isize = 0x0000_0080;
const WM_MOUSEACTIVATE: u32 = 0x0021;
const MA_NOACTIVATE: isize = 3;
const WM_NCLBUTTONDOWN: u32 = 0x00A1;
const HTCAPTION: usize = 2;
const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
const DWMWCP_ROUND: u32 = 2;

#[link(name = "user32")]
extern "system" {
    fn SetWindowPos(
        hwnd: Hwnd,
        insert_after: Hwnd,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        flags: u32,
    ) -> i32;
    fn GetWindowLongPtrW(hwnd: Hwnd, index: i32) -> isize;
    fn SetWindowLongPtrW(hwnd: Hwnd, index: i32, value: isize) -> isize;
    fn EnumWindows(
        callback: Option<unsafe extern "system" fn(Hwnd, isize) -> i32>,
        lparam: isize,
    ) -> i32;
    fn GetWindowThreadProcessId(hwnd: Hwnd, process_id: *mut u32) -> u32;
    fn GetWindowTextW(hwnd: Hwnd, text: *mut u16, max_count: i32) -> i32;
    fn ReleaseCapture() -> i32;
    fn SendMessageW(hwnd: Hwnd, msg: u32, wparam: usize, lparam: isize) -> isize;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcessId() -> u32;
}

#[link(name = "comctl32")]
extern "system" {
    fn SetWindowSubclass(
        hwnd: Hwnd,
        callback: Option<unsafe extern "system" fn(Hwnd, u32, usize, isize, usize, usize) -> isize>,
        id: usize,
        data: usize,
    ) -> i32;
    fn DefSubclassProc(hwnd: Hwnd, msg: u32, wparam: usize, lparam: isize) -> isize;
}

#[link(name = "dwmapi")]
extern "system" {
    fn DwmSetWindowAttribute(
        hwnd: Hwnd,
        attribute: u32,
        value: *const c_void,
        size: u32,
    ) -> Hresult;
}

struct FindState {
    pid: u32,
    nonce: String,
    found: Hwnd,
}

unsafe extern "system" fn enum_callback(hwnd: Hwnd, lparam: isize) -> i32 {
    let state = &mut *(lparam as *mut FindState);
    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, &mut pid);
    if pid != state.pid {
        return 1;
    }
    let mut title = [0u16; 512];
    let length = GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32);
    let text = String::from_utf16_lossy(&title[..length.max(0) as usize]);
    if state.nonce.is_empty() || text.contains(&state.nonce) {
        state.found = hwnd;
        return 0;
    }
    1
}

fn find_window(pid: u32, nonce: &str) -> Hwnd {
    let mut state = FindState {
        pid,
        nonce: nonce.to_string(),
        found: 0,
    };
    unsafe {
        EnumWindows(Some(enum_callback), &mut state as *mut FindState as isize);
    }
    state.found
}

unsafe extern "system" fn subclass_proc(
    hwnd: Hwnd,
    msg: u32,
    wparam: usize,
    lparam: isize,
    _id: usize,
    _data: usize,
) -> isize {
    if msg == WM_MOUSEACTIVATE {
        return MA_NOACTIVATE;
    }
    DefSubclassProc(hwnd, msg, wparam, lparam)
}

pub fn apply_for_current_process(
    mode: WindowMode,
    title_nonce: &str,
) -> Result<WindowPolicyResult, String> {
    let pid = unsafe { GetCurrentProcessId() };
    let hwnd = find_window(pid, title_nonce);
    if hwnd == 0 && !title_nonce.is_empty() {
        // GPUI may not have set the nonce in the title yet; retry by PID.
        let fallback = find_window(pid, "");
        if fallback == 0 {
            return Err(format!("no Win32 window found for pid {pid}"));
        }
        return apply_to_window(fallback, mode);
    }
    if hwnd == 0 {
        return Err(format!("no Win32 window found for pid {pid}"));
    }
    apply_to_window(hwnd, mode)
}

pub fn apply_to_window(hwnd: Hwnd, mode: WindowMode) -> Result<WindowPolicyResult, String> {
    let policy = policy_for(mode);
    let mut notes = Vec::new();
    unsafe {
        let mut ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        if policy.no_activate {
            ex_style |= WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW;
            notes.push("WS_EX_NOACTIVATE|WS_EX_TOOLWINDOW".to_string());
            SetWindowSubclass(hwnd, Some(subclass_proc), 1, 0);
            notes.push("WM_MOUSEACTIVATE -> MA_NOACTIVATE".to_string());
        } else {
            ex_style &= !WS_EX_NOACTIVATE;
            if !policy.utility_window {
                ex_style &= !WS_EX_TOOLWINDOW;
            }
        }
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style);
        let after = if policy.topmost {
            HWND_TOPMOST
        } else {
            HWND_NOTOPMOST
        };
        SetWindowPos(
            hwnd,
            after,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
        notes.push(format!("topmost={}", policy.topmost));
        if policy.borderless {
            let preference = DWMWCP_ROUND;
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_WINDOW_CORNER_PREFERENCE,
                &preference as *const u32 as *const c_void,
                std::mem::size_of::<u32>() as u32,
            );
            notes.push("DWM rounded corners".to_string());
        }
    }
    Ok(WindowPolicyResult {
        applied: true,
        backend: "win32".to_string(),
        notes,
    })
}

/// Caption drag: release capture and ask the non-client area to move.
pub fn begin_drag_current() -> bool {
    let pid = unsafe { GetCurrentProcessId() };
    let hwnd = find_window(pid, "");
    if hwnd == 0 {
        return false;
    }
    unsafe {
        ReleaseCapture();
        SendMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION, 0) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_policy_is_derived_from_the_shared_table() {
        let policy = policy_for(WindowMode::Overlay);
        assert!(policy.topmost && policy.no_activate && policy.borderless);
        let console = policy_for(WindowMode::Console);
        assert!(!console.topmost && !console.no_activate);
    }
}
