//! The golden thread-question state rendered through the real layout component (used by `ui-report`).
use dioxus::prelude::*;

use crate::model::ThreadView;
use crate::ui::app_layout::AppLayout;
use crate::ui::composer::ComposerMode;
use crate::ui::sidebar_list::ThreadRowData;

/// The two threads of the golden fixture (apps/web/test/fakeEnvironment.ts).
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
        row("thread-golden", "Thread thread-golden", "_r_u_", false),
        row("thread-golden-question", "Thread thread-golden-question", "_r_19_", true),
    ]
}

/// The golden thread as a timeline view.
pub fn fixture_view() -> ThreadView {
    let projection: serde_json::Value = serde_json::from_str(include_str!("../../../fixtures/thread-golden-question.projection.json")).unwrap();
    ThreadView { project_name: "render".into(), monogram: "RR".into(), branch: "main".into(), ..ThreadView::from_projection(&projection) }
}

#[component]
pub fn ThreadQuestion() -> Element {
    let projection: serde_json::Value = serde_json::from_str(include_str!("../../../fixtures/thread-golden-question.projection.json")).unwrap();
    rsx! { AppLayout { rows: fixture_rows(), thread: ThreadView::from_projection(&projection), environment_id: "environment-render".to_string(), onselect: |_| {}, composer: ComposerMode::Question } }
}
