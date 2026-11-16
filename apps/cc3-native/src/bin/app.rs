//! cc3: the native CC3 application (Dioxus components rendered by Blitz). Work in progress, see docs/native/README.md.
use dioxus::prelude::*;

fn main() {
    // Chromium at 1x on Linux: whole-pixel glyph advances.
    parley_engine::ROUND_ADVANCES.store(true, std::sync::atomic::Ordering::Relaxed);
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut count = use_signal(|| 0);
    rsx! {
        div { style: "padding: 24px; font-family: 'Lexend Deca'; color: white; background: #111; height: 100vh;",
            h1 { "CC3 native" }
            button { onclick: move |_| count += 1, "Clicks: {count}" }
        }
    }
}
