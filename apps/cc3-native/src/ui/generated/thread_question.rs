//! The golden thread-question state rendered through the real layout component (used by `ui-report`).
use dioxus::prelude::*;

use crate::ui::app_layout::AppLayout;
use crate::ui::sidebar_list::ThreadRowData;

/// The two threads of the golden fixture (apps/web/test/fakeEnvironment.ts).
pub fn fixture_rows() -> Vec<ThreadRowData> {
    let row = |id: &str, title: &str, ui: &str, active: bool| ThreadRowData {
        thread_key: format!("environment-render:{id}"),
        aria_label: format!("{title}, render"),
        ui_id: ui.into(),
        title: title.into(),
        project: "render".into(),
        monogram: "RR".into(),
        age: "13h".into(),
        branch: "main".into(),
        active,
    };
    vec![
        row("thread-golden", "Thread thread-golden", "_r_u_", false),
        row("thread-golden-question", "Thread thread-golden-question", "_r_19_", true),
    ]
}

#[component]
pub fn ThreadQuestion() -> Element {
    rsx! { AppLayout { rows: fixture_rows() } }
}
