//! The golden thread-history state (six turns, no question banner) through the real layout component.
use dioxus::prelude::*;

use crate::model::ThreadView;
use crate::ui::app_layout::AppLayout;
use crate::ui::composer::ComposerMode;
use crate::ui::sidebar_list::ThreadRowData;

pub fn fixture_rows() -> Vec<ThreadRowData> {
    let row = |id: &str, title: &str, ui: &str, active: bool| ThreadRowData {
        thread_id: id.to_string(),
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
        row("thread-golden", "Thread thread-golden", "_r_u_", true),
        row("thread-golden-question", "Thread thread-golden-question", "_r_19_", false),
    ]
}

pub fn fixture_view() -> ThreadView {
    let projection: serde_json::Value = serde_json::from_str(include_str!("../../../fixtures/thread-golden.projection.json")).unwrap();
    ThreadView::from_projection(&projection)
}

#[component]
pub fn ThreadHistory() -> Element {
    rsx! { AppLayout { rows: fixture_rows(), thread: fixture_view(), environment_id: "environment-render".to_string(), onselect: |_| {}, composer: ComposerMode::Normal } }
}
