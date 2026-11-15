//! `t3code://` app links and web links handed over by the OS (argv of this or a second launch).
//! Queued until the renderer reports ready through `webLinks.setReady`.
use std::sync::Mutex;

use serde_json::json;
use tauri::{AppHandle, Manager};

use crate::emit;

#[derive(Default)]
pub struct Links {
    inner: Mutex<(bool, Vec<String>)>,
}

fn accepts(arg: &str) -> bool {
    let a = arg.to_ascii_lowercase();
    a.starts_with("t3code:") || a.starts_with("http://") || a.starts_with("https://")
}

fn web_page_file(arg: &str) -> Option<String> {
    let a = arg.to_ascii_lowercase();
    let path = std::path::Path::new(arg);
    if !(a.ends_with(".html") || a.ends_with(".htm") || a.ends_with(".xhtml")) || !path.is_file() {
        return None;
    }
    Some(format!("file://{}", path.canonicalize().ok()?.display()))
}

/// Queues every link in `args` (program name excluded). Returns whether any was found.
pub fn receive_args(app: &AppHandle, args: impl IntoIterator<Item = String>) -> bool {
    let links: Vec<String> = args
        .into_iter()
        .filter_map(|a| if accepts(&a) { Some(a) } else { web_page_file(&a) })
        .collect();
    let found = !links.is_empty();
    app.state::<Links>().inner.lock().unwrap().1.extend(links);
    flush(app);
    found
}

pub fn set_ready(app: &AppHandle, ready: bool) {
    app.state::<Links>().inner.lock().unwrap().0 = ready;
    flush(app);
}

fn flush(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else { return };
    let pending = {
        let state = app.state::<Links>();
        let mut guard = state.inner.lock().unwrap();
        if !guard.0 || guard.1.is_empty() {
            return;
        }
        std::mem::take(&mut guard.1)
    };
    for url in pending {
        emit(&window, "web-link-open", json!(url));
    }
    reveal(&window);
}

pub fn reveal(window: &tauri::WebviewWindow) {
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}
