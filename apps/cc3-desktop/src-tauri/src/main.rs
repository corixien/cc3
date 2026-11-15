#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod native;
mod server;

use std::sync::Mutex;

use serde_json::json;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{Manager, RunEvent, WebviewUrl, WebviewWindowBuilder, WindowEvent};

const BRIDGE_JS: &str = include_str!("bridge.js");

struct ServerState(Mutex<Option<server::Server>>);

/// Calls `window.__cc3emit(channel, payload)` in the renderer.
fn emit(window: &tauri::WebviewWindow, channel: &str, payload: serde_json::Value) {
    let _ = window.eval(format!("window.__cc3emit&&window.__cc3emit({},{})", json!(channel), payload));
}

fn build_menu(app: &tauri::AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let item = |id: &str, label: &str, accel: Option<&str>| MenuItem::with_id(app, id, label, true, accel);
    let file = Submenu::with_items(
        app,
        "File",
        true,
        &[
            &item("open-settings", "Settings...", Some("CmdOrCtrl+,"))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::close_window(app, None)?,
            &PredefinedMenuItem::quit(app, None)?,
        ],
    )?;
    let edit = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(app, None)?,
            &PredefinedMenuItem::redo(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &item("paste-as-text", "Paste as Text", Some("CmdOrCtrl+Shift+V"))?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;
    let view = Submenu::with_items(
        app,
        "View",
        true,
        &[
            &item("reload", "Reload", Some("CmdOrCtrl+R"))?,
            &item("force-reload", "Force Reload", Some("CmdOrCtrl+Shift+R"))?,
            &item("devtools", "Toggle Developer Tools", Some("Ctrl+Shift+I"))?,
            &PredefinedMenuItem::separator(app)?,
            &item("zoom-reset", "Actual Size", Some("CmdOrCtrl+0"))?,
            &item("zoom-in", "Zoom In", Some("CmdOrCtrl+="))?,
            &item("zoom-out", "Zoom Out", Some("CmdOrCtrl+-"))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::fullscreen(app, None)?,
        ],
    )?;
    let window = Submenu::with_items(app, "Window", true, &[&PredefinedMenuItem::minimize(app, None)?])?;
    let help = Submenu::with_items(
        app,
        "Help",
        true,
        &[&item("check-for-updates", "Check for Updates...", None)?],
    )?;
    Menu::with_items(app, &[&file, &edit, &view, &window, &help])
}

fn on_menu(app: &tauri::AppHandle, id: &str) {
    let Some(window) = app.get_webview_window("main") else { return };
    match id {
        "reload" | "force-reload" => {
            let _ = window.eval("location.reload()");
        }
        "devtools" => {
            if window.is_devtools_open() {
                window.close_devtools();
            } else {
                window.open_devtools();
            }
        }
        "zoom-in" | "zoom-out" | "zoom-reset" => {
            emit(&window, "zoom-step", json!(id));
        }
        other => emit(&window, "menu-action", json!(other)),
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![native::cc3_native])
        .on_menu_event(|app, event| on_menu(app, event.id().as_ref()))
        .setup(|app| {
            let srv = server::start()?;
            let (port, token) = (srv.port, srv.token.clone());
            app.manage(ServerState(Mutex::new(Some(srv))));
            app.set_menu(build_menu(app.handle())?)?;

            let debug_port = if std::env::var("CC3_DEBUG").is_ok() {
                let l = std::net::TcpListener::bind("127.0.0.1:0")?;
                let port = l.local_addr()?.port();
                std::thread::spawn(move || {
                    use std::io::{BufRead, BufReader, Write};
                    for conn in l.incoming().flatten() {
                        std::thread::spawn(move || {
                            let mut conn = conn;
                            let mut line = String::new();
                            let _ = BufReader::new(&conn).read_line(&mut line);
                            eprintln!("[renderer] {}", line.trim());
                            let _ = conn.write_all(b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n");
                        });
                    }
                });
                Some(port)
            } else {
                None
            };
            let http = format!("http://127.0.0.1:{port}/");
            let init = json!({
                "version": env!("CARGO_PKG_VERSION"),
                "debugPort": debug_port,
                "arch": std::env::consts::ARCH,
                "locale": std::env::var("LANG").ok()
                    .and_then(|l| l.split('.').next().map(|s| s.replace('_', "-")))
                    .filter(|l| l != "C" && l != "POSIX"),
                "bootstraps": [{
                    "id": "primary",
                    "label": "This device",
                    "httpBaseUrl": http,
                    "wsBaseUrl": format!("ws://127.0.0.1:{port}/"),
                    "bootstrapToken": token,
                }],
            });
            let script = format!("window.__CC3__={init};\n{BRIDGE_JS}");
            let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::External(http.parse()?))
                .title("CC3")
                .inner_size(1400.0, 900.0)
                .min_inner_size(640.0, 480.0)
                .initialization_script(script)
                .build()?;
            if std::env::var("CC3_DEVTOOLS").is_ok() {
                window.open_devtools();
            }
            let w = window.clone();
            window.on_window_event(move |e| {
                if let WindowEvent::Resized(_) = e {
                    emit(&w, "fullscreen", json!(w.is_fullscreen().unwrap_or(false)));
                }
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building CC3")
        .run(|app, event| {
            if let RunEvent::Exit = event {
                if let Some(state) = app.try_state::<ServerState>() {
                    if let Some(mut srv) = state.0.lock().unwrap().take() {
                        let _ = srv.child.kill();
                        let _ = srv.child.wait();
                    }
                }
            }
        });
}
