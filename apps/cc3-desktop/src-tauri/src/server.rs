//! Lifecycle of the bundled T3 server (desktop mode, bootstrap over stdin).
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::time::{Duration, Instant};

pub struct Server {
    pub child: Child,
    _stdin: ChildStdin,
    pub port: u16,
    pub token: String,
}

pub fn home() -> PathBuf {
    std::env::var("CC3_HOME").map(PathBuf::from).unwrap_or_else(|_| {
        PathBuf::from(std::env::var("HOME").expect("HOME not set")).join(".cc3")
    })
}

fn entry() -> PathBuf {
    if let Ok(p) = std::env::var("CC3_SERVER_ENTRY") {
        return PathBuf::from(p);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../server/dist/bin.mjs")
}

fn random_token() -> String {
    let mut buf = [0u8; 24];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| std::io::Read::read_exact(&mut f, &mut buf))
        .expect("cannot read /dev/urandom");
    buf.iter().map(|b| format!("{b:02x}")).collect()
}

/// True once the server answers `GET /` with 200 (the port can listen before it serves).
fn http_ready(port: u16) -> bool {
    use std::io::Read;
    let Ok(mut c) = TcpStream::connect(("127.0.0.1", port)) else { return false };
    let _ = c.set_read_timeout(Some(Duration::from_secs(2)));
    if c.write_all(b"GET / HTTP/1.0\r\nHost: 127.0.0.1\r\n\r\n").is_err() {
        return false;
    }
    let mut head = [0u8; 12];
    c.read_exact(&mut head).is_ok() && &head == b"HTTP/1.1 200" || &head == b"HTTP/1.0 200"
}

pub fn start() -> Result<Server, String> {
    let port = TcpListener::bind("127.0.0.1:0")
        .and_then(|l| l.local_addr())
        .map_err(|e| e.to_string())?
        .port();
    let token = random_token();
    let home = home();
    let bootstrap = serde_json::json!({
        "mode": "desktop",
        "noBrowser": true,
        "port": port,
        "t3Home": home,
        "host": "127.0.0.1",
        "desktopBootstrapToken": token,
        "tailscaleServeEnabled": false,
        "tailscaleServePort": 443,
        "desktopAppUpdate": false,
    });
    let mut child = Command::new(std::env::var("CC3_NODE").unwrap_or_else(|_| "node".into()))
        .arg(entry())
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
