//! Where the node runtime, T3 server and desktop host live: bundled resources when packaged,
//! the repository checkout in development. CC3_NODE / CC3_SERVER_ENTRY override.
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::OnceLock;

use tauri::{AppHandle, Manager};

pub struct Paths {
    pub node: PathBuf,
    pub server_entry: PathBuf,
    /// Arguments that make `node` run the desktop host.
    pub host_args: Vec<OsString>,
}

static PATHS: OnceLock<Paths> = OnceLock::new();

pub fn init(app: &AppHandle) {
    let bundled = app.path().resource_dir().ok().map(|d| d.join("resources")).filter(|d| d.join("app/apps/server/dist/bin.mjs").exists());
    let paths = match bundled {
        Some(dir) => Paths {
            node: dir.join("node"),
            server_entry: dir.join("app/apps/server/dist/bin.mjs"),
            host_args: vec![dir.join("host/main.mjs").into()],
        },
        None => {
            let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let host = manifest.join("../host");
            Paths {
                node: "node".into(),
                server_entry: manifest.join("../../server/dist/bin.mjs"),
                host_args: vec!["--import".into(), host.join("register.mjs").into(), host.join("main.ts").into()],
            }
        }
    };
    let _ = PATHS.set(Paths {
        node: std::env::var_os("CC3_NODE").map(PathBuf::from).unwrap_or(paths.node),
        server_entry: std::env::var_os("CC3_SERVER_ENTRY").map(PathBuf::from).unwrap_or(paths.server_entry),
        host_args: paths.host_args,
    });
}

pub fn get() -> &'static Paths {
    PATHS.get().expect("paths not initialised")
}
