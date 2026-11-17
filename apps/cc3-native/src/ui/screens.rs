//! Screens rendered to static HTML (server-side rendering of the same components the app uses), so
//! `cc3-native ui-report` can compare them with the Chromium golden images.
use dioxus::prelude::*;

pub struct Screen {
    /// Golden state name without theme suffix, e.g. `thread-question`.
    pub name: String,
    pub component: fn() -> Element,
    /// Set for pages taken from `fragments.rs`; rendered by `PageScreen`.
    pub page: Option<&'static str>,
}

/// A page (settings, routes, overlays) in the standard layout.
#[component]
fn PageScreen(page: String) -> Element {
    use crate::ui::composer::ComposerMode;
    rsx! {
        crate::ui::app_layout::AppLayout {
            rows: crate::ui::generated::thread_history::fixture_rows(),
            thread: crate::ui::generated::thread_history::fixture_view(),
            environment_id: "environment-render".to_string(),
            onselect: |_| {},
            composer: ComposerMode::Normal,
            page,
        }
    }
}

pub fn screens() -> Vec<Screen> {
    let mut all = vec![
        Screen { name: "thread-question".into(), component: crate::ui::generated::thread_question::ThreadQuestion, page: None },
        Screen { name: "thread-history".into(), component: crate::ui::generated::thread_history::ThreadHistory, page: None },
        Screen { name: "thread-work".into(), component: crate::ui::generated::thread_work::ThreadWork, page: None },
        Screen { name: "thread-work-expanded".into(), component: crate::ui::generated::thread_work::ThreadWork, page: None },
    ];
    for p in crate::ui::fragments::PAGES {
        all.push(Screen { name: p.name.into(), component: crate::ui::generated::thread_history::ThreadHistory, page: Some(p.name) });
    }
    all
}

/// The page the golden export contains: global CSS plus the body of the screen.
pub fn render_html(screen: &Screen, dark: bool) -> String {
    // The golden clock: 2026-03-18T09:00:30Z.
    crate::ui::clock::freeze(1_773_824_430_000);
    crate::ui::timeline::EXPAND_ALL.store(screen.name.ends_with("-expanded"), std::sync::atomic::Ordering::Relaxed);
    let mut vdom = match screen.page {
        Some(page) => VirtualDom::new_with_props(PageScreen, PageScreenProps { page: page.to_string() }),
        None => VirtualDom::new_with_props(screen.component, ()),
    };
    vdom.rebuild_in_place();
    let body = dioxus_ssr::render(&vdom);
    let css = crate::ui::golden_css();
    format!(
        r#"<!doctype html><html class="{}"><head><meta charset="utf-8"><style>{css}</style></head><body>{body}</body></html>"#,
        if dark { "dark" } else { "light" }
    )
}
