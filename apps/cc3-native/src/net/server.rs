//! Starts the bundled Node server the way apps/cc3-desktop/src-tauri/src/server.rs does and returns
//! its address and one-time bootstrap token. The server is the same unchanged T3 server.
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

pub struct Server {
    pub child: Child,
    /// Kept open: the server treats stdin EOF as its parent going away.
    _stdin: std::process::ChildStdin,
    pub port: u16,
    pub token: String,
}

impl Server {
    pub fn http_base(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
    pub fn ws_base(&self) -> String {
        format!("ws://127.0.0.1:{}", self.port)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub fn home() -> PathBuf {
    std::env::var("CC3_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(std::env::var("HOME").expect("HOME not set")).join(".cc3"))
}

/// Node binary and server entry: `CC3_NODE`/`CC3_SERVER_ENTRY`, else the installed CC3 app, else the repo build.
fn locate() -> Result<(PathBuf, PathBuf), String> {
    if let (Ok(node), Ok(entry)) = (std::env::var("CC3_NODE"), std::env::var("CC3_SERVER_ENTRY")) {
        return Ok((node.into(), entry.into()));
    }
    let home = PathBuf::from(std::env::var("HOME").map_err(|e| e.to_string())?);
    let installed = home.join(".local/share/cc3-app/usr/lib/CC3/resources");
    let node = installed.join("node");
    let entry = installed.join("app/apps/server/dist/bin.mjs");
    if node.is_file() && entry.is_file() {
        return Ok((node, entry));
    }
    Err("no server found: set CC3_NODE and CC3_SERVER_ENTRY".into())
}

fn token() -> String {
    let mut buf = [0u8; 24];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| std::io::Read::read_exact(&mut f, &mut buf))
        .expect("cannot read /dev/urandom");
    buf.iter().map(|b| format!("{b:02x}")).collect()
}

fn http_ready(port: u16) -> bool {
    use std::io::Read;
    let Ok(mut c) = TcpStream::connect(("127.0.0.1", port)) else { return false };
    let _ = c.set_read_timeout(Some(Duration::from_secs(2)));
    if c.write_all(b"GET / HTTP/1.0\r\nHost: 127.0.0.1\r\n\r\n").is_err() {
        return false;
    }
    let mut head = [0u8; 12];
    c.read_exact(&mut head).is_ok() && (&head == b"HTTP/1.1 200" || &head == b"HTTP/1.0 200")
}

pub fn start() -> Result<Server, String> {
    let (node, entry) = locate()?;
    let port = TcpListener::bind("127.0.0.1:0").and_then(|l| l.local_addr()).map_err(|e| e.to_string())?.port();
    let token = token();
    let home = home();
    let bootstrap = serde_json::json!({
        "mode": "desktop", "noBrowser": true, "port": port, "t3Home": home, "host": "127.0.0.1",
        "desktopBootstrapToken": token, "tailscaleServeEnabled": false, "tailscaleServePort": 443,
        "desktopAppUpdate": false,
    });
    let mut child = Command::new(node)
        // Small V8 young generation: about 100 MB less resident memory (docs/perf/SERVER-MEMORY.md).
        .arg("--max-semi-space-size=1")
        .arg(entry)
        .args(["--bootstrap-fd", "0"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("failed to start node server: {e}"))?;
    let mut stdin = child.stdin.take().ok_or("no server stdin")?;
    writeln!(stdin, "{bootstrap}").map_err(|e| e.to_string())?;
    stdin.flush().map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(90);
    loop {
        if http_ready(port) {
            return Ok(Server { child, _stdin: stdin, port, token });
        }
        if let Ok(Some(status)) = child.try_wait() {
            return Err(format!("server exited early: {status}"));
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            return Err("server did not start in time".into());
        }
        std::thread::sleep(Duration::from_millis(150));
    }
}

static RUNNING: std::sync::Mutex<Option<Server>> = std::sync::Mutex::new(None);

/// Starts the server once and keeps it for the lifetime of the process (see `stop`).
pub fn start_global() -> Result<(String, String, u16), String> {
    let server = start()?;
    let info = (server.http_base(), server.token.clone(), server.port);
    *RUNNING.lock().unwrap() = Some(server);
    Ok(info)
}

/// Kills the server child. `std::process::exit` skips destructors, so quitting must call this.
pub fn stop() {
    if let Some(server) = RUNNING.lock().unwrap().take() {
        drop(server);
    }
}
