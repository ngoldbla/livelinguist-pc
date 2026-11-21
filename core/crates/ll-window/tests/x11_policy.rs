#![cfg(target_os = "linux")]

use ll_config::WindowMode;
use ll_window::x11::{apply_to_window, find_window};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    AtomEnum, ConnectionExt as _, CreateWindowAux, EventMask, PropMode, WindowClass,
};
use x11rb::wrapper::ConnectionExt as _;

#[test]
fn applies_ewmh_overlay_hints_to_a_real_x11_window() {
    if std::env::var_os("DISPLAY").is_none() || std::env::var_os("WAYLAND_DISPLAY").is_some() {
        eprintln!("skipping: X11 display not available");
        return;
    }
    let (connection, screen) = x11rb::connect(None).expect("connect X11");
    let root = connection.setup().roots[screen].root;
    let window = connection.generate_id().unwrap();
    connection
        .create_window(
            x11rb::COPY_DEPTH_FROM_PARENT,
            window,
            root,
            0,
            0,
            420,
            140,
            0,
            WindowClass::INPUT_OUTPUT,
            x11rb::COPY_FROM_PARENT,
            &CreateWindowAux::new().event_mask(EventMask::EXPOSURE),
        )
        .unwrap();
    let nonce = format!("ll-window-test-{}", std::process::id());
    let title = format!("Live Linguist — {nonce}");
    connection
        .change_property8(
            PropMode::REPLACE,
            window,
            AtomEnum::WM_NAME,
            AtomEnum::STRING,
            title.as_bytes(),
        )
        .unwrap();
    let pid_atom = connection
        .intern_atom(false, b"_NET_WM_PID")
        .unwrap()
        .reply()
        .unwrap()
        .atom;
    connection
        .change_property32(
            PropMode::REPLACE,
            window,
            pid_atom,
            AtomEnum::CARDINAL,
            &[std::process::id()],
        )
        .unwrap();
    connection.map_window(window).unwrap();
    connection.flush().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(150));

    let found = find_window(&connection, root, std::process::id(), &nonce).unwrap();
    assert_eq!(found, Some(window));
    let notes = apply_to_window(&connection, root, window, WindowMode::Overlay).unwrap();
    assert!(notes.iter().any(|note| note.contains("above=true")));
    assert!(notes
        .iter()
        .any(|note| note.contains("window_type=utility")));
    connection.flush().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(150));

    let type_atom = connection
        .intern_atom(false, b"_NET_WM_WINDOW_TYPE")
        .unwrap()
        .reply()
        .unwrap()
        .atom;
    let utility = connection
        .intern_atom(false, b"_NET_WM_WINDOW_TYPE_UTILITY")
        .unwrap()
        .reply()
        .unwrap()
        .atom;
    let reply = connection
        .get_property(false, window, type_atom, AtomEnum::ATOM, 0, 8)
        .unwrap()
        .reply()
        .unwrap();
    let types = reply
        .value32()
        .map(|values| values.collect::<Vec<_>>())
        .unwrap_or_default();
    assert!(types.contains(&utility));

    let desktop = connection
        .intern_atom(false, b"_NET_WM_DESKTOP")
        .unwrap()
        .reply()
        .unwrap()
        .atom;
    let reply = connection
        .get_property(false, window, desktop, AtomEnum::CARDINAL, 0, 1)
        .unwrap()
        .reply()
        .unwrap();
    assert_eq!(
        reply.value32().and_then(|mut values| values.next()),
        Some(0xFFFF_FFFF)
    );

    connection.destroy_window(window).unwrap();
    connection.flush().unwrap();
}
