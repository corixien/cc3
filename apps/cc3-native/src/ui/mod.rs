//! Shared UI pieces of the native app. Class strings are copied from apps/web so the same CSS applies.
pub mod window_frame;
pub mod generated;
pub mod screens;
pub mod sidebar_list;
pub mod thread_row;
pub mod app_layout;

/// Application stylesheet. `:root` becomes `.cc3-root` so the theme classes (`dark`) can sit on a wrapper
/// element, which is all a native app can set (there is no `<html>` to toggle).
pub fn app_css() -> &'static str {
    static CSS: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| include_str!("../../assets/app.css").replace(":root", ".cc3-root"));
    &CSS
}
pub mod markdown;
pub mod timeline_rows;
pub mod clock;
pub mod timeline;
