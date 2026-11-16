//! Single instance through a Unix socket in the runtime dir. The first process listens; later
//! launches hand over their links (or just "activate") and exit. The notification card process
//! reports clicks the same way.
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;

use super::{is_link, push_event, ShellEvent};

fn socket_path() -> PathBuf {
    let dir = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    dir.join("cc3-native.sock")
}

pub enum Role {
    /// This process owns the socket and receives messages from other launches.
    Primary,
    /// Another instance is running; the message was delivered.
    Secondary,
}

/// Becomes the primary instance, or hands `message` to the running one.
pub fn acquire(message: &str) -> Role {
    let path = socket_path();
    if let Ok(mut stream) = UnixStream::connect(&path) {
        let _ = writeln!(stream, "{message}");
        return Role::Secondary;
    }
    // Stale socket from a crashed run.
    let _ = std::fs::remove_file(&path);
    let Ok(listener) = UnixListener::bind(&path) else { return Role::Primary };
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            for line in BufReader::new(stream).lines().map_while(Result::ok) {
                handle(&line);
            }
        }
    });
    Role::Primary
}

/// Sends one message to the primary instance. Returns false when none is running.
pub fn send(message: &str) -> bool {
    UnixStream::connect(socket_path()).and_then(|mut s| writeln!(s, "{message}")).is_ok()
}

/// Message for a launch: `open <url>` lines, or `activate` when there is no link.
pub fn launch_message(args: impl IntoIterator<Item = String>) -> String {
    let links: Vec<String> = args.into_iter().filter(|a| is_link(a)).map(|a| format!("open {a}")).collect();
    if links.is_empty() { "activate".into() } else { links.join("\n") }
}

fn handle(line: &str) {
    if let Some(url) = line.strip_prefix("open ") {
        push_event(ShellEvent::OpenLink(url.to_string()));
    } else if let Some(id) = line.strip_prefix("alert-click ").and_then(|v| v.trim().parse().ok()) {
        push_event(ShellEvent::NotificationClick(id));
    } else if line == "activate" {
        push_event(ShellEvent::Activate);
    }
}
