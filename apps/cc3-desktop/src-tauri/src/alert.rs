//! Large always-on-top notification card. Clicking it reveals the main window and tells the page
//! which notification was clicked so the web UI can open the thread.
use serde_json::{json, Value};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::emit;

const LABEL: &str = "alert";
const WIDTH: f64 = 560.0;
const HEIGHT: f64 = 190.0;

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn page(id: u64, title: &str, body: &str) -> String {
    let html = format!(
        r#"<!doctype html><meta charset="utf-8"><style>
html,body{{margin:0;height:100%;background:transparent;font-family:"Lexend Deca","Inter",system-ui,sans-serif}}
a{{display:flex;flex-direction:column;justify-content:center;gap:10px;box-sizing:border-box;height:100%;padding:24px 32px;
background:#0b0b0b;color:#f5f5f5;border:1px solid #252525;border-radius:22px;text-decoration:none;cursor:pointer;
box-shadow:0 0 0 1px #000;overflow:hidden}}
a:hover{{background:#141414;border-color:#3a3a3a}}
small{{font-size:13px;letter-spacing:.08em;text-transform:uppercase;color:#8ab4ff}}
h1{{margin:0;font-size:30px;line-height:1.15;font-weight:600}}
p{{margin:0;font-size:18px;color:#bdbdbd;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}}
</style><a href="cc3-alert://open/{id}"><small>CC3</small><h1>{title}</h1><p>{body}</p></a>"#,
        id = id,
        title = escape(title),
        body = escape(body),
    );
    let mut url = String::from("data:text/html;charset=utf-8,");
    for b in html.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
            url.push(b as char);
        } else {
            url.push_str(&format!("%{b:02X}"));
        }
    }
    url
}

pub fn close(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(LABEL) {
        let _ = w.destroy();
    }
}

pub fn show(app: &AppHandle, args: &Value) -> Result<(), String> {
    let id = args["id"].as_u64().ok_or("id missing")?;
    let title = args["title"].as_str().unwrap_or("CC3");
    let body = args["body"].as_str().unwrap_or("");
    close(app);
    let handle = app.clone();
    let url = page(id, title, body).parse().map_err(|e| format!("{e}"))?;
    let window = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::External(url))
        .title("CC3 Alert")
        .inner_size(WIDTH, HEIGHT)
        .decorations(false)
        .transparent(true)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .on_navigation(move |url| {
            if url.scheme() == "cc3-alert" {
                close(&handle);
                if let Some(main) = handle.get_webview_window("main") {
                    crate::links::reveal(&main);
                    emit(&main, "notification-click", json!(id));
                }
                return false;
            }
            url.scheme() == "data"
        })
        .build()
        .map_err(|e| e.to_string())?;
    if let Ok(Some(monitor)) = window.primary_monitor() {
        let scale = monitor.scale_factor();
        let size = monitor.size().to_logical::<f64>(scale);
        let _ = window.set_position(tauri::LogicalPosition::new((size.width - WIDTH) / 2.0, 72.0));
    }
    let _ = window.hide_menu();
    let _ = window.show();
    keep_above();
    let handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(30));
        if let Some(w) = handle.get_webview_window(LABEL) {
            let _ = w.destroy();
        }
    });
    Ok(())
}

/// Wayland ignores client-side stacking and placement; KWin's scripting CLI can still do both.
fn keep_above() {
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(350));
        let Ok(out) = std::process::Command::new("kdotool").args(["search", "--name", "CC3 Alert"]).output()
        else {
            return;
        };
        for id in String::from_utf8_lossy(&out.stdout).lines() {
            let run = |args: &[&str]| {
                let _ = std::process::Command::new("kdotool").args(args).output();
            };
            run(&["windowstate", "--add", "ABOVE", id]);
            run(&["windowstate", "--add", "SKIP_TASKBAR", id]);
            run(&["windowmove", id, "680", "72"]);
        }
    });
}
