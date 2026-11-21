//! X11/XWayland window policy (Linux dev only; not a release gate).
//!
//! EWMH hints: above, skip taskbar/pager, sticky desktop, utility window type,
//! no input focus, no WM_TAKE_FOCUS, `_NET_WM_USER_TIME=0`, Motif borderless,
//! and `_NET_WM_MOVERESIZE` for drag. Window discovery uses `_NET_WM_PID` plus
//! a title nonce, never the title alone.

use crate::{policy_for, WindowPolicyResult};
use ll_config::WindowMode;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt as _;
use x11rb::protocol::xproto::{
    Atom, AtomEnum, ChangeWindowAttributesAux, ClientMessageData, ClientMessageEvent, EventMask,
    PropMode, Window,
};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

pub fn apply_for_current_process(
    mode: WindowMode,
    title_nonce: &str,
) -> Result<WindowPolicyResult, String> {
    let (connection, screen) =
        x11rb::connect(None).map_err(|error| format!("X11 connect failed: {error}"))?;
    let root = connection.setup().roots[screen].root;
    let pid = std::process::id();
    let window = find_window(&connection, root, pid, title_nonce)?
        .ok_or_else(|| format!("no X11 window found for pid {pid} and nonce '{title_nonce}'"))?;
    let notes = apply_to_window(&connection, root, window, mode)?;
    Ok(WindowPolicyResult {
        applied: true,
        backend: "x11".to_string(),
        notes,
    })
}

/// Find a top-level window belonging to `pid`, preferring a title containing
/// the nonce. Returns the first PID match when no nonce match exists.
pub fn find_window(
    connection: &RustConnection,
    root: Window,
    pid: u32,
    title_nonce: &str,
) -> Result<Option<Window>, String> {
    let pid_atom = intern(connection, "_NET_WM_PID")?;
    let name_atom = intern(connection, "_NET_WM_NAME")?;
    let mut candidates = client_list(connection, root)?;
    if candidates.is_empty() {
        let tree = connection
            .query_tree(root)
            .map_err(|error| error.to_string())?
            .reply()
            .map_err(|error| error.to_string())?;
        candidates = tree.children;
    }
    let mut fallback = None;
    for window in candidates {
        let window_pid = connection
            .get_property(false, window, pid_atom, AtomEnum::CARDINAL, 0, 1)
            .ok()
            .and_then(|cookie| cookie.reply().ok())
            .and_then(|reply| reply.value32().and_then(|mut values| values.next()));
        if window_pid != Some(pid) {
            continue;
        }
        if fallback.is_none() {
            fallback = Some(window);
        }
        if title_nonce.is_empty() {
            return Ok(Some(window));
        }
        let title = window_title(connection, window, name_atom).unwrap_or_default();
        if title.contains(title_nonce) {
            return Ok(Some(window));
        }
    }
    Ok(fallback)
}

fn client_list(connection: &RustConnection, root: Window) -> Result<Vec<Window>, String> {
    let atom = intern(connection, "_NET_CLIENT_LIST")?;
    let reply = connection
        .get_property(false, root, atom, AtomEnum::WINDOW, 0, 4096)
        .map_err(|error| error.to_string())?
        .reply()
        .map_err(|error| error.to_string())?;
    Ok(reply
        .value32()
        .map(|iter| iter.collect())
        .unwrap_or_default())
}

fn window_title(connection: &RustConnection, window: Window, name_atom: Atom) -> Option<String> {
    let reply = connection
        .get_property(false, window, name_atom, AtomEnum::ANY, 0, 1024)
        .ok()?
        .reply()
        .ok()?;
    if reply.value.is_empty() {
        let fallback = connection
            .get_property(false, window, AtomEnum::WM_NAME, AtomEnum::ANY, 0, 1024)
            .ok()?
            .reply()
            .ok()?;
        return Some(String::from_utf8_lossy(&fallback.value).to_string());
    }
    Some(String::from_utf8_lossy(&reply.value).to_string())
}

pub fn apply_to_window(
    connection: &RustConnection,
    root: Window,
    window: Window,
    mode: WindowMode,
) -> Result<Vec<String>, String> {
    let policy = policy_for(mode);
    let mut notes = Vec::new();
    let above = intern(connection, "_NET_WM_STATE_ABOVE")?;
    let skip_taskbar = intern(connection, "_NET_WM_STATE_SKIP_TASKBAR")?;
    let skip_pager = intern(connection, "_NET_WM_STATE_SKIP_PAGER")?;
    let sticky = intern(connection, "_NET_WM_STATE_STICKY")?;
    let net_wm_state = intern(connection, "_NET_WM_STATE")?;
    let desktop = intern(connection, "_NET_WM_DESKTOP")?;
    let user_time = intern(connection, "_NET_WM_USER_TIME")?;
    let window_type = intern(connection, "_NET_WM_WINDOW_TYPE")?;
    let utility = intern(connection, "_NET_WM_WINDOW_TYPE_UTILITY")?;
    let normal = intern(connection, "_NET_WM_WINDOW_TYPE_NORMAL")?;
    let motif = intern(connection, "_MOTIF_WM_HINTS")?;
    let wm_hints = intern(connection, "WM_HINTS")?;
    let wm_protocols = intern(connection, "WM_PROTOCOLS")?;
    let take_focus = intern(connection, "WM_TAKE_FOCUS")?;

    send_state(
        connection,
        root,
        window,
        net_wm_state,
        above,
        if policy.topmost { 1 } else { 0 },
    )?;
    notes.push(format!("above={}", policy.topmost));
    for atom in [skip_taskbar, skip_pager] {
        send_state(
            connection,
            root,
            window,
            net_wm_state,
            atom,
            if policy.skip_taskbar { 1 } else { 0 },
        )?;
    }
    notes.push(format!("skip_taskbar={}", policy.skip_taskbar));
    send_state(
        connection,
        root,
        window,
        net_wm_state,
        sticky,
        if policy.sticky { 1 } else { 0 },
    )?;
    notes.push(format!("sticky={}", policy.sticky));

    let type_atom = if policy.utility_window {
        utility
    } else {
        normal
    };
    connection
        .change_property32(
            PropMode::REPLACE,
            window,
            window_type,
            AtomEnum::ATOM,
            &[type_atom],
        )
        .map_err(|error| error.to_string())?;
    notes.push(format!(
        "window_type={}",
        if policy.utility_window {
            "utility"
        } else {
            "normal"
        }
    ));

    let desktop_value: u32 = if policy.sticky { 0xFFFF_FFFF } else { 0 };
    connection
        .change_property32(
            PropMode::REPLACE,
            window,
            desktop,
            AtomEnum::CARDINAL,
            &[desktop_value],
        )
        .map_err(|error| error.to_string())?;
    connection
        .change_property32(
            PropMode::REPLACE,
            window,
            user_time,
            AtomEnum::CARDINAL,
            &[0],
        )
        .map_err(|error| error.to_string())?;
    notes.push("user_time=0".to_string());

    if policy.no_activate {
        // WM_HINTS: flags=InputHint, input=False.
        connection
            .change_property32(
                PropMode::REPLACE,
                window,
                wm_hints,
                wm_hints,
                &[1, 0, 0, 0, 0, 0, 0, 0, 0],
            )
            .map_err(|error| error.to_string())?;
        remove_protocol(connection, window, wm_protocols, take_focus)?;
        notes.push("input=false, WM_TAKE_FOCUS removed".to_string());
    }

    if policy.borderless {
        // Motif hints: decorations flag, no decorations.
        connection
            .change_property32(PropMode::REPLACE, window, motif, motif, &[2, 0, 0, 0, 0])
            .map_err(|error| error.to_string())?;
        notes.push("borderless".to_string());
    }

    let _ = connection
        .change_window_attributes(
            window,
            &ChangeWindowAttributesAux::new().event_mask(EventMask::PROPERTY_CHANGE),
        )
        .map_err(|error| error.to_string())?;
    connection.flush().map_err(|error| error.to_string())?;
    Ok(notes)
}

fn remove_protocol(
    connection: &RustConnection,
    window: Window,
    protocols_atom: Atom,
    remove: Atom,
) -> Result<(), String> {
    let reply = connection
        .get_property(false, window, protocols_atom, AtomEnum::ATOM, 0, 64)
        .map_err(|error| error.to_string())?
        .reply()
        .map_err(|error| error.to_string())?;
    let values = reply
        .value32()
        .map(|iter| iter.filter(|atom| *atom != remove).collect::<Vec<_>>())
        .unwrap_or_default();
    connection
        .change_property32(
            PropMode::REPLACE,
            window,
            protocols_atom,
            AtomEnum::ATOM,
            &values,
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn send_state(
    connection: &RustConnection,
    root: Window,
    window: Window,
    state_atom: Atom,
    property: Atom,
    action: u32,
) -> Result<(), String> {
    let event = ClientMessageEvent::new(
        32,
        window,
        state_atom,
        ClientMessageData::from([action, property, 0, 1, 0]),
    );
    connection
        .send_event(
            false,
            root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            event,
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

/// Ask the window manager to begin a move (caption drag).
pub fn begin_moveresize(window: Window) -> Result<(), String> {
    let (connection, _) = x11rb::connect(None).map_err(|error| error.to_string())?;
    let root = connection.setup().roots[0].root;
    let moveresize = intern(&connection, "_NET_WM_MOVERESIZE")?;
    let event = ClientMessageEvent::new(
        32,
        window,
        moveresize,
        ClientMessageData::from([0u32, 0, 8, 1, 1]),
    );
    connection
        .send_event(
            false,
            root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            event,
        )
        .map_err(|error| error.to_string())?;
    connection.flush().map_err(|error| error.to_string())
}

fn intern(connection: &RustConnection, name: &str) -> Result<Atom, String> {
    connection
        .intern_atom(false, name.as_bytes())
        .map_err(|error| error.to_string())?
        .reply()
        .map(|reply| reply.atom)
        .map_err(|error| error.to_string())
}
