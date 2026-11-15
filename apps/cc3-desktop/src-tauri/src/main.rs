#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::sync::Mutex;
use std::time::Duration;

use tauri::{Manager, RunEvent, WebviewUrl, WebviewWindowBuilder};

/// Holds the T3 server child process so it is stopped when the app exits.
struct Server(Mutex<Option<Child>>);

fn server_entry() -> PathBuf {
    if let Ok(p) = std::env::var("CC3_SERVER_ENTRY") {
        return PathBuf::from(p);
    }
    // Dev layout: apps/cc3-desktop/src-tauri -> apps/server/dist/bin.mjs
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../server/dist/bin.mjs")
}

/// Data dir. Separate from `~/.t3` so CC3 can run next to the Electron T3 Code app
/// (the server allows only one instance per data dir). Override with CC3_HOME.
fn base_dir() -> PathBuf {
    std::env::var("CC3_HOME").map(PathBuf::from).unwrap_or_else(|_| {
        PathBuf::from(std::env::var("HOME").expect("HOME not set")).join(".cc3")
    })
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .and_then(|l| l.local_addr())
        .map(|a| a.port())
        .expect("no free port")
}

/// Starts the T3 server and returns it with its one-time pairing URL.
fn start_server() -> Result<(Child, String), String> {
    let mut child = Command::new(std::env::var("CC3_NODE").unwrap_or_else(|_| "node".into()))
        .arg(server_entry())
        .args(["--no-browser", "--port", &free_port().to_string(), "--base-dir"])
        .arg(base_dir())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("failed to start node server: {e}"))?;

    let stdout = child.stdout.take().ok_or("no server stdout")?;
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if let Some(url) = line.trim().strip_prefix("pairingUrl:") {
                let _ = tx.send(url.trim().to_string());
            }
        }
    });

    match rx.recv_timeout(Duration::from_secs(90)) {
        Ok(url) => Ok((child, url)),
        Err(_) => {
            let _ = child.kill();
            Err("server did not announce a pairing URL in time".into())
        }
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let (child, url) = start_server()?;
            app.manage(Server(Mutex::new(Some(child))));
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url.parse()?))
                .title("CC3")
                .inner_size(1400.0, 900.0)
                .min_inner_size(640.0, 480.0)
                .build()?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building CC3")
        .run(|app, event| {
            if let RunEvent::Exit = event {
                if let Some(server) = app.try_state::<Server>() {
                    if let Some(mut child) = server.0.lock().unwrap().take() {
                        let _ = child.kill();
                        let _ = child.wait();
                    }
                }
            }
        });
}
