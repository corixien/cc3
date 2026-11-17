//! cc3: the native CC3 application (Dioxus components rendered by Blitz). Work in progress, see docs/native/README.md.
//!   cc3 [t3code:... | https://...]        start, or hand the links to the running instance
//!   cc3 --alert <id> <title> <body>       the large notification card (spawned by the app itself)
use dioxus::native::Config;
use dioxus::prelude::*;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::window::{WindowAttributes, WindowLevel};

use cc3_native::shell::{self, alert, badge, instance, ShellEvent};
use cc3_native::ui::window_frame::WindowFrame;

/// The CC3 icon is also the taskbar and Alt-Tab icon.
fn window_icon() -> Option<winit::icon::Icon> {
    let rgba = image::load_from_memory(include_bytes!("../../assets/icon-128.png")).ok()?.to_rgba8();
    let (w, h) = rgba.dimensions();
    winit::icon::RgbaIcon::new(rgba.into_raw(), w, h).ok().map(winit::icon::Icon::from)
}

fn main() {
    // Chromium at 1x on Linux: whole-pixel glyph advances.
    parley_engine::ROUND_ADVANCES.store(true, std::sync::atomic::Ordering::Relaxed);
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.first().map(String::as_str) == Some("--alert") {
        let id = args.get(1).and_then(|v| v.parse().ok()).unwrap_or(0);
        ALERT.set((id, args.get(2).cloned().unwrap_or_default(), args.get(3).cloned().unwrap_or_default())).ok();
        let attributes = WindowAttributes::default()
            .with_title(alert::TITLE)
            .with_decorations(false)
            .with_transparent(true)
            .with_resizable(false)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_surface_size(LogicalSize::new(alert::WIDTH, alert::HEIGHT))
            .with_position(LogicalPosition::new(680.0, 72.0));
        alert::keep_above();
        dioxus::native::launch_cfg(AlertApp, vec![], vec![Box::new(Config::new().with_window_attributes(attributes))]);
        return;
    }

    if matches!(instance::acquire(&instance::launch_message(args.iter().cloned())), instance::Role::Secondary) {
        return;
    }
    for arg in &args {
        if shell::is_link(arg) {
            shell::push_event(ShellEvent::OpenLink(arg.clone()));
        }
    }
    let attributes = WindowAttributes::default()
        .with_window_icon(window_icon())
        .with_title("CC3")
        .with_decorations(false)
        .with_transparent(true)
        .with_surface_size(LogicalSize::new(1400.0, 900.0))
        .with_min_surface_size(LogicalSize::new(640.0, 480.0));
    dioxus::native::launch_cfg(App, vec![], vec![Box::new(Config::new().with_window_attributes(attributes))]);
}

static ALERT: std::sync::OnceLock<(u64, String, String)> = std::sync::OnceLock::new();

#[component]
fn AlertApp() -> Element {
    let (id, title, body) = ALERT.get().cloned().unwrap_or_default();
    rsx! { alert::AlertCard { id, title, body } }
}

#[component]
fn App() -> Element {
    let window = dioxus::native::use_window();
    let mut last_event = use_signal(String::new);

    // Links, activation and notification clicks arrive from other threads and processes.
    use_future({
        let window = window.clone();
        move || {
            let window = window.clone();
            async move {
                loop {
                    for event in shell::drain_events() {
                        window.set_minimized(false);
                        window.focus_window();
                        last_event.set(format!("{event:?}"));
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                }
            }
        }
    });

    // CC3_SELFTEST=1: exercises window controls, badge and the notification card without input.
    use_future({
        let window = window.clone();
        move || {
            let window = window.clone();
            async move {
                if std::env::var("CC3_SELFTEST").is_err() {
                    return;
                }
                let step = std::time::Duration::from_secs(4);
                tokio::time::sleep(step).await;
                window.set_maximized(true);
                tokio::time::sleep(step).await;
                window.set_maximized(false);
                badge::set(3);
                alert::spawn(7, "Input needed", "Which approach should I take?");
            }
        }
    });

    rsx! {
        div { style: "font-family: 'Lexend Deca'; color: white; width: 100vw; height: 100vh;",
            WindowFrame {
                dark: true,
                background: "#000".to_string(),
                title_content: rsx! { div { style: "padding: 0 16px; line-height: 40px; font-size: 14px;", "CC3" } },
                div { style: "padding: 64px 24px 24px;",
                    h1 { "CC3 native" }
                    div { "Last shell event: {last_event}" }
                }
            }
        }
    }
}
