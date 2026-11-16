//! Desktop integration of the native app: single instance, links handed over by the OS, launcher
//! badge and the large notification card. Ports apps/cc3-desktop/src-tauri/src/{links,alert,native}.rs.
pub mod alert;
pub mod badge;
pub mod instance;

use std::collections::VecDeque;
use std::sync::Mutex;

/// Something the running app has to react to, produced by other processes or threads.
#[derive(Debug, Clone, PartialEq)]
pub enum ShellEvent {
    /// A `t3code:` or http(s) link was opened (this or a second launch).
    OpenLink(String),
    /// The user clicked the notification card with this id.
    NotificationClick(u64),
    /// A second launch without links: bring the window forward.
    Activate,
}

static EVENTS: Mutex<VecDeque<ShellEvent>> = Mutex::new(VecDeque::new());

pub fn push_event(event: ShellEvent) {
    EVENTS.lock().unwrap().push_back(event);
}

pub fn drain_events() -> Vec<ShellEvent> {
    EVENTS.lock().unwrap().drain(..).collect()
}

/// Links accepted from the command line: `t3code:` app links and http(s) URLs.
pub fn is_link(arg: &str) -> bool {
    let a = arg.to_ascii_lowercase();
    a.starts_with("t3code:") || a.starts_with("http://") || a.starts_with("https://")
}

/// Ends the process. A plain `()` function so event handlers keep a unit return type.
pub fn quit() {
    std::process::exit(0)
}
