//! Shared UI pieces of the native app. Class strings are copied from apps/web so the same CSS applies.
pub mod window_frame;
pub mod generated;
pub mod screens;
pub mod sidebar_list;
pub mod thread_row;
pub mod app_layout;

/// Application stylesheet. `:root` becomes `.cc3-root` so the theme classes (`dark`) can sit on a wrapper
/// element, which is all a native app can set (there is no `<html>` to toggle).
/// Blitz has no `scrollbar-gutter`; `stable both-edges` reserves 6px per side with an overlay scrollbar.
pub const EXTRA_CSS: &str = ".scrollbar-gutter-both > .legend-list-content-container{margin-left:6px;margin-right:6px}.scrollbar-gutter-stable > .legend-list-content-container{margin-right:6px}";

/// The production stylesheet with container queries rewritten (registers their thresholds with the document).
fn processed() -> &'static str {
    static CSS: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
        let (css, thresholds) = cq::rewrite(include_str!("../../assets/app.css"));
        blitz_dom::set_container_thresholds(thresholds);
        css + EXTRA_CSS
    });
    &CSS
}

/// For the app: `:root` variables move to the `.cc3-root` wrapper (where the theme class sits).
pub fn app_css() -> &'static str {
    static CSS: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
        // No backdrop-filter in the renderer yet: glass surfaces would show unblurred content behind them.
        let css = processed();
        // SVG `var()` lookups (stage art) only see `:root`; the app is dark, so the dark values go there.
        let marker = ":root:is(.dark, .dark *) {";
        let stage_vars: String = css
            .find(marker)
            .map(|i| css[i + marker.len()..].split('}').next().unwrap_or_default().split(';').filter(|d| d.trim_start().starts_with("--stage-")).map(|d| format!("{};", d.trim())).collect())
            .unwrap_or_default();
        format!("{}\n:root{{{stage_vars}}}\n.dialog-glass{{background:color-mix(in srgb,var(--background) 96%,transparent)!important}}.dropdown-glass{{background:color-mix(in srgb,var(--popover) 96%,transparent)!important}}", css.replace(":root", ".cc3-root"))
    });
    &CSS
}

/// For the golden pages (theme class on `<html>`).
pub fn golden_css() -> &'static str {
    processed()
}
pub mod markdown;
pub mod minimap;
pub mod timeline_rows;
pub mod clock;
pub mod timeline;
pub mod composer;
pub mod cq;
pub mod fragments;
pub mod add_project;
pub mod model_picker;
pub mod diff_panel;
pub mod page;
pub mod terminal_panel;
pub mod root;
pub mod palette;
pub mod settings;
