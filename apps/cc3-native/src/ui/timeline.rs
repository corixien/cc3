//! The chat timeline (`legend-list-content-container`): rows in normal flow. The web UI positions rows
//! absolutely from measured heights (virtualized list); the resulting positions are the same.
use dioxus::prelude::*;

use crate::model::{ThreadView, TimelineRow};
use crate::ui::clock::{day_aware, duration_label, now_ms};
use crate::ui::markdown;
use crate::ui::timeline_rows::{AssistantMessageRow, UserMessageRow, WorkDetailsRow, WorkSummaryRow, WorkToggleRow};

/// Set by `ui-report` to render work folds expanded, as in the `*-expanded` goldens.
pub static EXPAND_ALL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub static EXPAND_GROUPS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[component]
pub fn Timeline(view: ThreadView, environment_id: String, spacer: u32) -> Element {
    let now = now_ms();
    let mut groups = use_signal(|| EXPAND_ALL.load(std::sync::atomic::Ordering::Relaxed) && EXPAND_GROUPS.load(std::sync::atomic::Ordering::Relaxed));
    let mut open = use_signal(|| {
        if EXPAND_ALL.load(std::sync::atomic::Ordering::Relaxed) {
            view.rows.iter().filter_map(|r| if let TimelineRow::Work { run_id, .. } = r { Some(run_id.clone()) } else { None }).collect()
        } else {
            Vec::new()
        }
    });
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
                            TimelineRow::Work { run_id, duration_ms, at, summary, first_item_id, entries } => rsx! {
                                WorkSummaryRow {
                                    run_id: run_id.clone(),
                                    label: format!("Worked for {}", duration_label(*duration_ms)),
                                    time: day_aware(at, now),
                                    uid: format!("-r{index}"),
                                    expanded: open.read().contains(run_id),
                                    ontoggle: {
                                        let id = run_id.clone();
                                        move |_| {
                                            let mut list = open.write();
                                            match list.iter().position(|r| *r == id) {
                                                Some(p) => { list.remove(p); }
                                                None => list.push(id.clone()),
                                            }
                                        }
                                    },
                                }
                                if open.read().contains(run_id) {
                                    div { "style": "contain: layout style paint;",
                                        WorkToggleRow { first_item_id: first_item_id.clone(), label: summary.clone(), time: day_aware(at, now), uid: format!("-r{index}"), open: groups(), ontoggle: move |_| groups.set(!groups()) }
                                    }
                                    if groups() {
                                        div { "style": "contain: layout style paint;",
                                            WorkDetailsRow {
                                                first_item_id: first_item_id.clone(),
                                                entries: entries.clone(),
                                                workspace: view.workspace_name.clone(),
                                                times: entries.iter().map(|e| day_aware(&e.at, now)).collect::<Vec<_>>(),
                                                uid: format!("-r{index}"),
                                            }
                                        }
                                    }
                                }
                            },
                            TimelineRow::Assistant { id, text, at } => rsx! {
                                AssistantMessageRow { message_id: id.clone(), environment_id: environment_id.clone(), thread_id: view.thread_id.clone(), time: day_aware(at, now), uid: format!("-r{index}"), body: markdown::render(text) }
                            },
                        }
                    }
                }
            }
            div {
                div {
                    div { }
                    div { "aria-hidden": "true", "style": "height: {spacer}px;" }
                    div { "aria-hidden": "true", "class": "h-3 sm:h-4" }
                }
            }
        }
    }
}
