//! cc3: the native CC3 application (Dioxus components rendered by Blitz). Work in progress, see docs/native/README.md.
use dioxus::prelude::*;
use dioxus::native::Config;
use winit::dpi::LogicalSize;
use winit::window::WindowAttributes;

#[path = "../ui/mod.rs"]
mod ui;
use ui::window_frame::WindowFrame;

fn main() {
    // Chromium at 1x on Linux: whole-pixel glyph advances.
    parley_engine::ROUND_ADVANCES.store(true, std::sync::atomic::Ordering::Relaxed);
    // The CC3 icon is also the taskbar and Alt-Tab icon.
    let icon = image::load_from_memory(include_bytes!("../../assets/icon-128.png"))
        .ok()
        .map(|img| img.to_rgba8())
        .and_then(|rgba| {
            let (w, h) = rgba.dimensions();
            winit::icon::RgbaIcon::new(rgba.into_raw(), w, h).ok()
        })
        .map(winit::icon::Icon::from);
    let attributes = WindowAttributes::default()
        .with_window_icon(icon)
        .with_title("CC3")
        .with_decorations(false)
        .with_transparent(true)
        .with_surface_size(LogicalSize::new(1400.0, 900.0))
        .with_min_surface_size(LogicalSize::new(640.0, 480.0));
    dioxus::native::launch_cfg(
        App,
        vec![],
        vec![Box::new(Config::new().with_window_attributes(attributes))],
    );
}

#[component]
fn App() -> Element {
    let mut count = use_signal(|| 0);
    // CC3_SELFTEST=1: maximize, restore and minimize without input so window controls can be checked headlessly.
    let window = dioxus::native::use_window();
    use_future(move || {
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
            tokio::time::sleep(step).await;
            window.set_minimized(true);
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
                    button { onclick: move |_| count += 1, "Clicks: {count}" }
                }
            }
        }
    }
}
