//! The chat timeline (`legend-list-content-container`): rows in normal flow. The web UI positions rows
//! absolutely from measured heights (virtualized list); the resulting positions are the same.
use dioxus::prelude::*;

use crate::model::{ThreadView, TimelineRow};
use crate::ui::clock::{day_aware, duration_label, now_ms};
use crate::ui::markdown;
use crate::ui::timeline_rows::{AssistantMessageRow, UserMessageRow, WorkSummaryRow};

#[component]
pub fn Timeline(view: ThreadView, environment_id: String) -> Element {
    let now = now_ms();
    rsx! {
        div {
            "class": "legend-list-content-container",
            "style": "display: block; min-height: 100%; box-sizing: border-box; overflow-anchor: none;",
            div { div { "class": "h-[var(--workspace-titlebar-scroll-fade-height)]" } }
            div { "style": "position: relative;",
                for (index, row) in view.rows.iter().enumerate() {
                    div { key: "{index}", "style": "contain: layout style paint;",
                        match row {
                            TimelineRow::User { id, text, at } => rsx! {
                                UserMessageRow { message_id: id.clone(), time: day_aware(at, now), uid: format!("-r{index}"), body: markdown::render(text) }
                            },
                            TimelineRow::Work { run_id, duration_ms, at } => rsx! {
                                WorkSummaryRow { run_id: run_id.clone(), label: format!("Worked for {}", duration_label(*duration_ms)), time: day_aware(at, now), uid: format!("-r{index}") }
                            },
                            TimelineRow::Assistant { id, text, at } => rsx! {
                                AssistantMessageRow { message_id: id.clone(), environment_id: environment_id.clone(), thread_id: view.thread_id.clone(), time: day_aware(at, now), uid: format!("-r{index}"), body: markdown::render(text) }
                            },
                        }
                    }
                }
            }
            div {
                div { }
                div { "aria-hidden": "true", "style": "height: 281px;" }
                div { "aria-hidden": "true", "class": "h-3 sm:h-4" }
            }
        }
    }
}
