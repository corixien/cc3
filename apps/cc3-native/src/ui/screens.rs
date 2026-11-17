//! Screens rendered to static HTML (server-side rendering of the same components the app uses), so
//! `cc3-native ui-report` can compare them with the Chromium golden images.
use dioxus::prelude::*;

pub struct Screen {
    /// Golden state name without theme suffix, e.g. `thread-question`.
    pub name: &'static str,
    pub component: fn() -> Element,
}

pub const SCREENS: &[Screen] = &[Screen { name: "thread-question", component: crate::ui::generated::thread_question::ThreadQuestion }];

/// The page the golden export contains: global CSS plus the body of the screen.
pub fn render_html(screen: &Screen, dark: bool) -> String {
    // The golden clock: 2026-03-18T09:00:30Z.
    crate::ui::clock::freeze(1_773_824_430_000);
    let mut vdom = VirtualDom::new_with_props(screen.component, ());
    vdom.rebuild_in_place();
    let body = dioxus_ssr::render(&vdom);
    let css = include_str!("../../assets/app.css");
    format!(
        r#"<!doctype html><html class="{}"><head><meta charset="utf-8"><style>{css}</style></head><body>{body}</body></html>"#,
        if dark { "dark" } else { "light" }
    )
}
