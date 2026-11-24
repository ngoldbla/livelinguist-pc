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
const WM_MOUSEACTIVATE: u32 = 0x0021;
const MA_NOACTIVATE: isize = 3;
const VK_LBUTTON: i32 = 0x01;
const SWP_NOSIZE: u32 = 0x0001;
const SWP_NOMOVE: u32 = 0x0002;
const SWP_NOZORDER: u32 = 0x0004;
const SWP_NOACTIVATE: u32 = 0x0010;
const SWP_FRAMECHANGED: u32 = 0x0020;
const GWL_EXSTYLE: i32 = -20;
const WS_EX_NOACTIVATE: isize = 0x0800_0000;
const WS_EX_TOOLWINDOW: isize = 0x0000_0080;
#[cfg(test)]
const WS_EX_TOPMOST: isize = 0x0000_0008;
const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
const DWMWCP_ROUND: u32 = 2;
// GPUIX installs its own window subclasses; a low id such as 1 can collide and
// make SetWindowSubclass fail silently. Use an app-private id and record the
// result.
const OVERLAY_SUBCLASS_ID: usize = 0x4C4C_0001;
#[cfg(test)]
const WM_LL_SUBCLASS_PROBE: u32 = 0x8001;

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
    fn IsWindowVisible(hwnd: Hwnd) -> i32;
    fn GetCursorPos(point: *mut Point) -> i32;
    fn GetAsyncKeyState(key: i32) -> i16;
    fn GetWindowRect(hwnd: Hwnd, rect: *mut Rect) -> i32;
    #[cfg(test)]
    fn SendMessageW(hwnd: Hwnd, msg: u32, wparam: usize, lparam: isize) -> isize;
}

#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcessId() -> u32;
}

#[repr(C)]
struct MonitorInfo {
    cb_size: u32,
    rc_monitor: Rect,
    rc_work: Rect,
    flags: u32,
}

const MONITOR_DEFAULTTOPRIMARY: u32 = 0x0000_0001;
const MDT_EFFECTIVE_DPI: i32 = 0;

#[link(name = "user32")]
extern "system" {
    fn MonitorFromPoint(point: Point, flags: u32) -> isize;
    fn GetDpiForWindow(hwnd: Hwnd) -> u32;
    fn GetMonitorInfoW(monitor: isize, info: *mut MonitorInfo) -> i32;
}

#[link(name = "shcore")]
extern "system" {
    fn GetDpiForMonitor(monitor: isize, dpi_type: i32, dpi_x: *mut u32, dpi_y: *mut u32)
        -> Hresult;
}

/// Work area (`rcWork`) of the monitor under the cursor, or the primary
/// monitor. Coordinates follow the process DPI awareness: physical pixels in
/// the per-monitor-aware app; in an unaware process Windows virtualizes both
/// the rectangle and the DPI (96), so `width / scale` is the logical size
/// either way.
pub fn work_area() -> Option<crate::WorkArea> {
    unsafe {
        let mut cursor = Point { x: 0, y: 0 };
        if GetCursorPos(&mut cursor) == 0 {
            cursor = Point { x: 0, y: 0 };
        }
        let monitor = MonitorFromPoint(cursor, MONITOR_DEFAULTTOPRIMARY);
        if monitor == 0 {
            return None;
        }
        let mut info = MonitorInfo {
            cb_size: std::mem::size_of::<MonitorInfo>() as u32,
            rc_monitor: Rect {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            rc_work: Rect {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            flags: 0,
        };
        if GetMonitorInfoW(monitor, &mut info) == 0 {
            return None;
        }
        let (mut dpi_x, mut dpi_y) = (0u32, 0u32);
        let scale = if GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) >= 0
            && dpi_x > 0
        {
            dpi_x as f64 / 96.0
        } else {
            1.0
        };
        let work = &info.rc_work;
        let width = (work.right - work.left).max(0) as u32;
        let height = (work.bottom - work.top).max(0) as u32;
        if width == 0 || height == 0 {
            return None;
        }
        Some(crate::WorkArea {
            x: work.left,
            y: work.top,
            width,
            height,
            scale,
        })
    }
}

#[link(name = "comctl32")]
extern "system" {
    fn SetWindowSubclass(
        hwnd: Hwnd,
        callback: Option<unsafe extern "system" fn(Hwnd, u32, usize, isize, usize, usize) -> isize>,
        id: usize,
        data: usize,
    ) -> i32;
    fn RemoveWindowSubclass(
        hwnd: Hwnd,
        callback: Option<unsafe extern "system" fn(Hwnd, u32, usize, isize, usize, usize) -> isize>,
        id: usize,
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
    if state.nonce.is_empty() {
        // GPUI also owns a visible helper window with an empty title. Prefer
        // the visible titled window; otherwise drag/policy can target the
        // helper and silently do nothing.
        if IsWindowVisible(hwnd) != 0 && !text.is_empty() {
            state.found = hwnd;
            return 0;
        }
        return 1;
    }
    if text.contains(&state.nonce) {
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
    #[cfg(test)]
    if msg == WM_LL_SUBCLASS_PROBE {
        return 42;
    }
    DefSubclassProc(hwnd, msg, wparam, lparam)
}

/// Resize `hwnd` to a logical size (96-DPI pixels), scaled by the window's
/// DPI. Position and z-order are kept; the window is not activated.
pub fn resize_hwnd_logical(hwnd: Hwnd, width: u32, height: u32) -> bool {
    unsafe {
        let dpi = match GetDpiForWindow(hwnd) {
            0 => 96,
            dpi => dpi,
        };
        let cx = (u64::from(width) * u64::from(dpi) / 96) as i32;
        let cy = (u64::from(height) * u64::from(dpi) / 96) as i32;
        SetWindowPos(
            hwnd,
            0,
            0,
            0,
            cx,
            cy,
            SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
        ) != 0
    }
}

/// Resize this process's app window (found by title nonce) to a logical size.
pub fn resize_current(title_nonce: &str, width: u32, height: u32) -> bool {
    let pid = unsafe { GetCurrentProcessId() };
    let hwnd = find_window(pid, title_nonce);
    hwnd != 0 && resize_hwnd_logical(hwnd, width, height)
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
            let installed =
                SetWindowSubclass(hwnd, Some(subclass_proc), OVERLAY_SUBCLASS_ID, 0) != 0;
            notes.push(if installed {
                "WM_MOUSEACTIVATE -> MA_NOACTIVATE".to_string()
            } else {
                "SetWindowSubclass failed; WM_MOUSEACTIVATE is not intercepted".to_string()
            });
        } else {
            ex_style &= !WS_EX_NOACTIVATE;
            // Leaving overlay mode must stop intercepting WM_MOUSEACTIVATE,
            // otherwise expanded/console can never be activated again.
            RemoveWindowSubclass(hwnd, Some(subclass_proc), OVERLAY_SUBCLASS_ID);
            if policy.utility_window {
                ex_style |= WS_EX_TOOLWINDOW;
            } else {
                ex_style &= !WS_EX_TOOLWINDOW;
            }
        }
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style);
        let after = if policy.topmost {
            HWND_TOPMOST
        } else {
            HWND_NOTOPMOST
        };
        let mut flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE;
        if !policy.no_activate {
            // Clearing WS_EX_NOACTIVATE is cached by the window manager until
            // the frame is refreshed; otherwise WM_MOUSEACTIVATE keeps
            // answering MA_NOACTIVATE after leaving overlay mode.
            flags |= SWP_FRAMECHANGED;
        }
        SetWindowPos(hwnd, after, 0, 0, 0, 0, flags);
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

/// Caption drag: follow the cursor until the primary button is released.
///
/// `DefWindowProc(WM_NCLBUTTONDOWN)` starts a modal move loop that depends on
/// activation, which a `WS_EX_NOACTIVATE` overlay does not get. This custom
/// loop keeps the topmost window under the cursor without activating it.
pub fn begin_drag_current() -> bool {
    let pid = unsafe { GetCurrentProcessId() };
    let hwnd = find_window(pid, "");
    if hwnd == 0 {
        return false;
    }
    unsafe {
        let mut cursor = Point { x: 0, y: 0 };
        let mut rect = Rect {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if GetCursorPos(&mut cursor) == 0 || GetWindowRect(hwnd, &mut rect) == 0 {
            return false;
        }
        let offset_x = cursor.x - rect.left;
        let offset_y = cursor.y - rect.top;
        std::thread::spawn(move || {
            while (GetAsyncKeyState(VK_LBUTTON) as u16 & 0x8000) != 0 {
                let mut cursor = Point { x: 0, y: 0 };
                if GetCursorPos(&mut cursor) == 0 {
                    break;
                }
                SetWindowPos(
                    hwnd,
                    HWND_TOPMOST,
                    cursor.x - offset_x,
                    cursor.y - offset_y,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOACTIVATE,
                );
                std::thread::sleep(std::time::Duration::from_millis(8));
            }
        });
        true
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

    #[repr(C)]
    struct WndClassExW {
        cb_size: u32,
        style: u32,
        lpfn_wnd_proc: Option<unsafe extern "system" fn(Hwnd, u32, usize, isize) -> isize>,
        cb_cls_extra: i32,
        cb_wnd_extra: i32,
        h_instance: isize,
        h_icon: isize,
        h_cursor: isize,
        hbr_background: isize,
        lpsz_menu_name: *const u16,
        lpsz_class_name: *const u16,
        h_icon_sm: isize,
    }

    #[link(name = "user32")]
    extern "system" {
        fn RegisterClassExW(class: *const WndClassExW) -> u16;
        fn CreateWindowExW(
            ex_style: u32,
            class_name: *const u16,
            window_name: *const u16,
            style: u32,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            parent: Hwnd,
            menu: isize,
            instance: isize,
            param: *mut c_void,
        ) -> Hwnd;
        fn DefWindowProcW(hwnd: Hwnd, msg: u32, wparam: usize, lparam: isize) -> isize;
        fn ShowWindow(hwnd: Hwnd, cmd: i32) -> i32;
        fn DestroyWindow(hwnd: Hwnd) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetModuleHandleW(name: *const u16) -> isize;
    }

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }

    #[test]
    fn work_area_reports_the_monitor() {
        let area = work_area().expect("work area on a Windows desktop");
        assert!(area.width > 0 && area.height > 0, "{area:?}");
        assert!((1.0..=3.0).contains(&area.scale), "{area:?}");
    }

    /// Same-process Win32 integration check: the overlay policy must set
    /// topmost/no-activate/toolwindow and intercept WM_MOUSEACTIVATE. A
    /// cross-process SendMessage is not a valid probe for this message.
    #[test]
    fn overlay_window_sets_styles_and_answers_mouse_activate() {
        unsafe {
            let class_name = wide("LiveLinguistOverlayPolicyTest");
            let window_name = wide("Live Linguist policy test");
            let instance = GetModuleHandleW(std::ptr::null());
            let class = WndClassExW {
                cb_size: std::mem::size_of::<WndClassExW>() as u32,
                style: 0,
                lpfn_wnd_proc: Some(DefWindowProcW),
                cb_cls_extra: 0,
                cb_wnd_extra: 0,
                h_instance: instance,
                h_icon: 0,
                h_cursor: 0,
                hbr_background: 0,
                lpsz_menu_name: std::ptr::null(),
                lpsz_class_name: class_name.as_ptr(),
                h_icon_sm: 0,
            };
            let _ = RegisterClassExW(&class);
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                window_name.as_ptr(),
                0x00CF_0000,
                0,
                0,
                320,
                200,
                0,
                0,
                instance,
                std::ptr::null_mut(),
            );
            assert_ne!(hwnd, 0, "CreateWindowExW failed");
            ShowWindow(hwnd, 5);
            let baseline = SendMessageW(hwnd, WM_LL_SUBCLASS_PROBE, 0, 0);
            assert_ne!(baseline, 42, "probe message already answered");
            let result = apply_to_window(hwnd, WindowMode::Overlay).expect("overlay policy");
            assert!(result.applied);
            let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            assert_ne!(ex_style & WS_EX_TOPMOST, 0, "topmost missing");
            assert_ne!(ex_style & WS_EX_NOACTIVATE, 0, "no-activate missing");
            assert_ne!(ex_style & WS_EX_TOOLWINDOW, 0, "toolwindow missing");
            let probe = SendMessageW(hwnd, WM_LL_SUBCLASS_PROBE, 0, 0);
            assert_eq!(probe, 42, "subclass not installed");
            let answer = SendMessageW(hwnd, WM_MOUSEACTIVATE, 0, 0x0201_0002);
            assert_eq!(answer, MA_NOACTIVATE, "WM_MOUSEACTIVATE not intercepted");

            apply_to_window(hwnd, WindowMode::Expanded).expect("expanded policy");
            let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            assert_eq!(ex_style & WS_EX_NOACTIVATE, 0, "no-activate not cleared");
            assert_ne!(
                ex_style & WS_EX_TOOLWINDOW,
                0,
                "expanded toolwindow missing"
            );
            let probe = SendMessageW(hwnd, WM_LL_SUBCLASS_PROBE, 0, 0);
            assert_ne!(probe, 42, "subclass survived overlay exit");

            apply_to_window(hwnd, WindowMode::Console).expect("console policy");
            let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            assert_eq!(
                ex_style & WS_EX_TOOLWINDOW,
                0,
                "console toolwindow not cleared"
            );
            assert_eq!(ex_style & WS_EX_TOPMOST, 0, "console topmost not cleared");

            assert!(resize_hwnd_logical(hwnd, 400, 300), "resize failed");
            let dpi = match GetDpiForWindow(hwnd) {
                0 => 96,
                dpi => dpi,
            } as i32;
            let mut rect = Rect {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            assert_ne!(GetWindowRect(hwnd, &mut rect), 0);
            assert_eq!(rect.right - rect.left, 400 * dpi / 96, "resized width");
            assert_eq!(rect.bottom - rect.top, 300 * dpi / 96, "resized height");
            let _ = DestroyWindow(hwnd);
        }
    }
}
