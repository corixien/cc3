//! Native operations behind `window.desktopBridge` (dispatched from bridge.js via `cc3_native`).
use serde_json::{json, Value};
use std::io::Read as _;
use std::path::PathBuf;
use tauri::{AppHandle, Theme, WebviewWindow};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

use crate::server::home;

fn store_path(name: &str) -> Result<PathBuf, String> {
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.') {
        return Err("invalid store name".into());
    }
    Ok(home().join("userdata").join("desktop").join(name))
}

fn str_arg<'a>(args: &'a Value, i: usize) -> Option<&'a str> {
    args.get(i).and_then(Value::as_str)
}

/// Launcher badge via the Unity LauncherEntry D-Bus signal (GNOME dash-to-dock, KDE). Best effort.
pub fn set_badge(count: i64) {
    let props = format!("{{'count': <int64 {count}>, 'count-visible': <{}>}}", count > 0);
    let _ = std::process::Command::new("gdbus")
        .args([
            "emit",
            "--session",
            "--object-path",
            "/com/canonical/unity/launcherentry/1",
            "--signal",
            "com.canonical.Unity.LauncherEntry.Update",
            "application://CC3.desktop",
            &props,
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

#[tauri::command]
pub async fn cc3_native(
    app: AppHandle,
    window: WebviewWindow,
    op: String,
    args: Value,
) -> Result<Value, String> {
    match op.as_str() {
        "pick_folder" => {
            let mut dlg = app.dialog().file();
            if let Some(p) = str_arg(&args, 0) {
                dlg = dlg.set_directory(p);
            }
            Ok(json!(dlg.blocking_pick_folder().map(|p| p.to_string())))
        }
        "pick_favicon" => {
            let mut dlg = app
                .dialog()
                .file()
                .add_filter("Images", &["png", "jpg", "jpeg", "svg", "ico", "webp", "gif"]);
            if let Some(p) = str_arg(&args, 0) {
                if let Some(parent) = std::path::Path::new(p).parent() {
                    dlg = dlg.set_directory(parent);
                }
            }
            Ok(json!(dlg.blocking_pick_file().map(|p| p.to_string())))
        }
        "pick_theme_files" => {
            let picked = app
                .dialog()
                .file()
                .add_filter("Theme", &["json", "jsonc"])
                .blocking_pick_files();
            let Some(files) = picked else { return Ok(Value::Null) };
            let mut out = vec![];
            for f in files {
                let path = f.into_path().map_err(|e| e.to_string())?;
                let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
                out.push(json!({
                    "name": path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
                    "size": text.len(),
                    "text": text,
                }));
            }
            Ok(json!(out))
        }
        "open_external" => {
            let url = str_arg(&args, 0).ok_or("url missing")?;
            if !(url.starts_with("http://") || url.starts_with("https://") || url.starts_with("mailto:")) {
                return Ok(json!(false));
            }
            Ok(json!(app.opener().open_url(url, None::<&str>).is_ok()))
        }
        "reveal_window" => {
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
            Ok(Value::Null)
        }
        "set_theme" => {
            let theme = match str_arg(&args, 0) {
                Some("light") => Some(Theme::Light),
                Some("dark") => Some(Theme::Dark),
                _ => None,
            };
            let _ = window.set_theme(theme);
            Ok(Value::Null)
        }
        "set_zoom" => {
            let f = args.get(0).and_then(Value::as_f64).unwrap_or(1.0).clamp(0.25, 5.0);
            window.set_zoom(f).map_err(|e| e.to_string())?;
            Ok(json!(f))
        }
        "store_read" => {
            let path = store_path(str_arg(&args, 0).ok_or("name missing")?)?;
            Ok(std::fs::read_to_string(path).map(Value::String).unwrap_or(Value::Null))
        }
        "store_write" => {
            let path = store_path(str_arg(&args, 0).ok_or("name missing")?)?;
            let data = str_arg(&args, 1).ok_or("data missing")?;
            std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
            let tmp = path.with_extension("tmp");
            std::fs::write(&tmp, data).and_then(|_| std::fs::rename(&tmp, &path)).map_err(|e| e.to_string())?;
            Ok(json!(true))
        }
        "read_media" => {
            let url = str_arg(&args, 0).ok_or("url missing")?.to_string();
            if !(url.starts_with("http://") || url.starts_with("https://")) {
                return Ok(Value::Null);
            }
            let res = tauri::async_runtime::spawn_blocking(move || -> Option<Value> {
                let resp = match ureq::get(&url).timeout(std::time::Duration::from_secs(30)).call() {
                    Ok(r) => r,
                    Err(ureq::Error::Status(_, r)) => r,
                    Err(_) => return None,
                };
                let status = resp.status();
                let mime = resp.content_type().to_string();
                let mut data = Vec::new();
                std::io::Read::take(resp.into_reader(), 64 * 1024 * 1024)
                    .read_to_end(&mut data)
                    .ok()?;
                Some(json!({ "status": status, "mimeType": mime, "data": data }))
            })
            .await
            .map_err(|e| e.to_string())?;
            Ok(res.unwrap_or(Value::Null))
        }
        "log" => {
            eprintln!("[renderer] {}", args.get(0).and_then(Value::as_str).unwrap_or(""));
            Ok(Value::Null)
        }
        "web_links_ready" => {
            crate::links::set_ready(&app, args.get(0).and_then(Value::as_bool).unwrap_or(false));
            Ok(Value::Null)
        }
        "set_badge" => {
            // A focused window has already seen the notification.
            let count = if window.is_focused().unwrap_or(false) {
                0
            } else {
                args.get(0).and_then(|b| b["count"].as_i64()).unwrap_or(0).max(0)
            };
            set_badge(count);
            Ok(Value::Null)
        }
        "win_minimize" => {
            let _ = window.minimize();
            Ok(Value::Null)
        }
        "win_toggle_maximize" => {
            if window.is_maximized().unwrap_or(false) {
                let _ = window.unmaximize();
            } else {
                let _ = window.maximize();
            }
            Ok(json!(window.is_maximized().unwrap_or(false)))
        }
        "win_is_maximized" => Ok(json!(window.is_maximized().unwrap_or(false))),
        "win_close" => {
            let _ = window.close();
            Ok(Value::Null)
        }
        "win_start_drag" => {
            let _ = window.start_dragging();
            Ok(Value::Null)
        }
        "is_fullscreen" => Ok(json!(window.is_fullscreen().unwrap_or(false))),
        _ => Err(format!("unknown native op: {op}")),
    }
}
