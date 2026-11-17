//! Scrollable thread list of the sidebar (`sidebar-content`). Rows are data, the surrounding markup is the golden one.
use dioxus::prelude::*;

use crate::ui::thread_row::ThreadRow;

#[derive(Clone, PartialEq)]
pub struct ThreadRowData {
    /// Thread id without the environment prefix.
    pub thread_id: String,
    pub thread_key: String,
    pub aria_label: String,
    pub ui_id: String,
    pub title: String,
    pub project: String,
    pub monogram: String,
    pub age: String,
    pub branch: String,
    pub active: bool,
}

#[component]
pub fn SidebarList(rows: Vec<ThreadRowData>, #[props(default)] settled: Vec<ThreadRowData>, #[props(default)] shelf_open: bool, onselect: EventHandler<String>, #[props(default)] onaction: EventHandler<String>) -> Element {
    rsx! {
    div {
        "class": "flex w-full min-w-0 flex-col [overflow-anchor:none] group-data-[collapsible=icon]:overflow-hidden [&>[data-sidebar=group]+[data-sidebar=group]]:pt-0 [&>[data-sidebar=group]:first-child]:pt-0 min-h-full",
        "data-sidebar": "content",
        "data-slot": "sidebar-content",
        div {
            "class": "relative flex w-full min-w-0 flex-col p-[var(--sidebar-content-inset)] flex-1",
            "data-sidebar": "group",
            "data-slot": "sidebar-group",
            "role": "presentation",
            ul {
                "role": "presentation",
                "class": "relative flex flex-col gap-px flex-1",
                li {
                    "data-thread-selection-safe": "true",
                    "data-testid": "sidebar-pinned-header",
                    "class": "list-none pointer-events-none relative mx-0.5 -mb-px h-0",
                }
                li {
                    "data-thread-selection-safe": "true",
                    "data-testid": "sidebar-pinned-divider",
                    "class": "list-none pointer-events-none relative mx-0.5 -mb-px h-0",
                }
                li {
                    "data-thread-selection-safe": "true",
                    "data-testid": "sidebar-active-placeholder",
                    "class": "list-none relative mx-0.5 -mb-px h-0",
                    "style": "transition: none;",
                }
                for row in rows.iter() {
                    ThreadRow { key: "{row.thread_key}", onselect: { let id = row.thread_id.clone(); move |_| onselect.call(id.clone()) }, onaction: { let id = row.thread_id.clone(); move |a: String| onaction.call(format!("row:{a}:{id}")) }, thread_key: row.thread_key.clone(), aria_label: row.aria_label.clone(), ui_id: row.ui_id.clone(), title: row.title.clone(), project: row.project.clone(), monogram: row.monogram.clone(), age: row.age.clone(), branch: row.branch.clone(), active: row.active }
                }
                li {
                    "data-thread-selection-safe": "true",
                    "data-testid": "sidebar-settled-header",
                    "class": "list-none relative mx-0.5 h-8 mt-auto",
                    button {
                        "data-sidebar-row": "shelf:settled",
                        "data-testid": "sidebar-settled-shelf-toggle",
                        "type": "button",
                        "aria-expanded": "{shelf_open}",
                        onclick: move |_| onaction.call("ui:shelf".to_string()),
                        "class": "flex h-8 w-full cursor-pointer items-center gap-2 rounded-md text-left text-xs font-medium focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring/70 px-2 text-sidebar-muted-foreground",
                        span { "class": "min-w-0 truncate", "Settled" }
                        span {
                            "aria-hidden": "true",
                            "class": "h-px min-w-2 flex-1 bg-sidebar-border/50 invisible",
                        }
                        span { "class": "font-normal tabular-nums", "{settled.len()}" }
                        svg {
                            "width": "24",
                            "height": "24",
                            "viewBox": "0 0 24 24",
                            "fill": "none",
                            "stroke": "currentColor",
                            "stroke-width": "2",
                            "stroke-linecap": "round",
                            "stroke-linejoin": "round",
                            "class": if shelf_open { "lucide lucide-chevron-down size-3 shrink-0 transition-transform" } else { "lucide lucide-chevron-down size-3 shrink-0 transition-transform -rotate-90" },
                            "aria-hidden": "true",
                            path {
                                "d": "m6 9 6 6 6-6",
                            }
                        }
                    }
                }
                if shelf_open {
                    for row in settled.iter() {
                        ThreadRow { key: "{row.thread_key}", onselect: { let id = row.thread_id.clone(); move |_| onselect.call(id.clone()) }, onaction: { let id = row.thread_id.clone(); move |a: String| onaction.call(format!("row:{a}:{id}")) }, thread_key: row.thread_key.clone(), aria_label: row.aria_label.clone(), ui_id: row.ui_id.clone(), title: row.title.clone(), project: row.project.clone(), monogram: row.monogram.clone(), age: row.age.clone(), branch: row.branch.clone(), active: row.active }
                    }
                }
                li {
                    "data-thread-selection-safe": "true",
                    "data-testid": "sidebar-settled-placeholder",
                    "class": "list-none relative mx-0.5 -mb-px h-0",
                    "style": "transition: none;",
                }
            }
            div {
                "id": "DndDescribedBy-0",
                "style": "display: none;",
                "\n    To pick up a draggable item, press the space bar.\n    While dragging, use the arrow keys to move the item.\n    Press space again to drop the item in its new position, or press escape to cancel.\n  "
            }
            div {
                "id": "DndLiveRegion-0",
                "role": "status",
                "aria-live": "assertive",
                "aria-atomic": "true",
                "style": "position: fixed; top: 0px; left: 0px; width: 1px; height: 1px; margin: -1px; border: 0px; padding: 0px; overflow: hidden; clip: rect(0px, 0px, 0px, 0px); clip-path: inset(100%); white-space: nowrap;",
            }
        }
    }

    }
}
