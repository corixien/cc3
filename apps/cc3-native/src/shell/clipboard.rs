//! System clipboard through the tools the desktop session provides (Wayland first, then X11).
use std::io::Write;
use std::process::{Command, Stdio};

/// Copies `text`; returns whether a clipboard tool accepted it.
pub fn copy(text: &str) -> bool {
    for (program, args) in [("wl-copy", &[][..]), ("xclip", &["-selection", "clipboard"][..]), ("xsel", &["--clipboard", "--input"][..])] {
        let Ok(mut child) = Command::new(program).args(args).stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).spawn() else { continue };
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        // wl-copy and xclip stay alive to serve the selection; do not wait for them.
        return true;
    }
    false
}
