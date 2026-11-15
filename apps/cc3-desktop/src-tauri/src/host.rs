//! Node sidecar (apps/cc3-desktop/host) that serves desktop features implemented in TypeScript.
//! Started on first use; speaks line-delimited JSON over stdio.
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};

use serde_json::{json, Value};
use tauri::{AppHandle, Manager, State};

type Pending = Arc<Mutex<HashMap<u64, Sender<Result<Value, String>>>>>;

struct Running {
    child: Child,
    stdin: ChildStdin,
    pending: Pending,
}

#[derive(Default)]
pub struct Host {
    running: Mutex<Option<Running>>,
    next_id: AtomicU64,
}

impl Host {
    pub fn shutdown(&self) {
        if let Some(mut r) = self.running.lock().unwrap().take() {
            let _ = r.child.kill();
            let _ = r.child.wait();
        }
    }
}

fn host_dir() -> PathBuf {
    std::env::var("CC3_HOST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../host"))
}

fn spawn(app: &AppHandle) -> Result<Running, String> {
    let dir = host_dir();
    let mut child = Command::new(std::env::var("CC3_NODE").unwrap_or_else(|_| "node".into()))
        .arg("--import")
        .arg(dir.join("register.mjs"))
        .arg(dir.join("main.ts"))
        .env("CC3_HOME", crate::server::home())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("failed to start desktop host: {e}"))?;
    let stdin = child.stdin.take().ok_or("no host stdin")?;
    let stdout = child.stdout.take().ok_or("no host stdout")?;
    let pending: Pending = Arc::default();
    let (reader_pending, app) = (pending.clone(), app.clone());
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            let Ok(msg) = serde_json::from_str::<Value>(&line) else { continue };
            if let Some(id) = msg.get("id").and_then(Value::as_u64) {
                let outcome = match msg.get("error").and_then(Value::as_str) {
                    Some(e) => Err(e.to_string()),
                    None => Ok(msg.get("result").cloned().unwrap_or(Value::Null)),
                };
                if let Some(tx) = reader_pending.lock().unwrap().remove(&id) {
                    let _ = tx.send(outcome);
                }
            } else if let Some(event) = msg.get("event").and_then(Value::as_str) {
                if let Some(w) = app.get_webview_window("main") {
                    crate::emit(&w, event, msg.get("payload").cloned().unwrap_or(Value::Null));
                }
            }
        }
        // Host exited: fail everything still waiting.
        for (_, tx) in reader_pending.lock().unwrap().drain() {
            let _ = tx.send(Err("desktop host exited".into()));
        }
    });
    Ok(Running { child, stdin, pending })
}

#[tauri::command]
pub async fn cc3_host(
    app: AppHandle,
    host: State<'_, Host>,
    channel: String,
    arg: Value,
) -> Result<Value, String> {
    let id = host.next_id.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = mpsc::channel();
    {
        let mut guard = host.running.lock().unwrap();
        let alive = match guard.as_mut() {
            Some(r) => matches!(r.child.try_wait(), Ok(None)),
            None => false,
        };
        if !alive {
            *guard = Some(spawn(&app)?);
        }
        let r = guard.as_mut().unwrap();
        r.pending.lock().unwrap().insert(id, tx);
        let line = json!({ "id": id, "channel": channel, "arg": arg });
        writeln!(r.stdin, "{line}").map_err(|e| e.to_string())?;
        r.stdin.flush().map_err(|e| e.to_string())?;
    }
    tauri::async_runtime::spawn_blocking(move || rx.recv().map_err(|e| e.to_string())?)
        .await
        .map_err(|e| e.to_string())?
}
