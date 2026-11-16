//! Large always-on-top notification card, run as its own short-lived process (`cc3 --alert ...`)
//! so it never competes with the main window. A click tells the main instance which notification
//! was clicked and exits. Wayland ignores client-side stacking and placement, so KWin's scripting
//! CLI (`kdotool`) pins the card above other windows and centres it near the top.
use dioxus::prelude::*;

pub const WIDTH: f64 = 560.0;
pub const HEIGHT: f64 = 190.0;
pub const TITLE: &str = "CC3 Alert";

/// Starts the card process for notification `id`.
pub fn spawn(id: u64, title: &str, body: &str) {
    if let Ok(exe) = std::env::current_exe() {
        let _ = std::process::Command::new(exe).args(["--alert", &id.to_string(), title, body]).spawn();
    }
}

/// Pins the card on KWin/Wayland after the compositor mapped it.
pub fn keep_above() {
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(500));
        let Ok(out) = std::process::Command::new("kdotool").args(["search", "--name", TITLE]).output() else { return };
        let pid = std::process::id().to_string();
        for id in String::from_utf8_lossy(&out.stdout).lines() {
            let owned = std::process::Command::new("kdotool").args(["getwindowpid", id]).output().ok();
            if owned.map(|o| String::from_utf8_lossy(&o.stdout).trim() == pid) != Some(true) {
                continue;
            }
            let run = |args: &[&str]| {
                let _ = std::process::Command::new("kdotool").args(args).output();
            };
            run(&["windowstate", "--add", "ABOVE", id]);
            run(&["windowstate", "--add", "SKIP_TASKBAR", id]);
            run(&["windowmove", id, "680", "72"]);
        }
    });
}

#[component]
pub fn AlertCard(id: u64, title: String, body: String) -> Element {
    use_future(|| async {
        // Gone after 30 seconds like the Tauri card.
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        crate::shell::quit();
    });
    rsx! {
        style { "html, body {{ margin: 0; background: transparent; }} a.card:hover {{ background: #141414; border-color: #3a3a3a; }}" }
        div {
            class: "card",
            style: "display:flex;flex-direction:column;justify-content:center;gap:10px;box-sizing:border-box;width:100vw;height:100vh;padding:24px 32px;background:#0b0b0b;color:#f5f5f5;border:1px solid #252525;border-radius:22px;cursor:pointer;overflow:hidden;font-family:'Lexend Deca',sans-serif;",
            onclick: move |_| {
                crate::shell::instance::send(&format!("alert-click {id}"));
                crate::shell::quit();
            },
            div { style: "font-size:13px;letter-spacing:.08em;text-transform:uppercase;color:#8ab4ff;", "CC3" }
            div { style: "font-size:30px;line-height:1.15;font-weight:600;", "{title}" }
            div { style: "font-size:18px;color:#bdbdbd;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;", "{body}" }
        }
    }
}
