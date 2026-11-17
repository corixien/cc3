//! Toast notifications (top right), styled like the web UI's toasts.
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct ToastView {
    pub id: u64,
    /// `success`, `error` or `info`.
    pub kind: String,
    pub title: String,
    pub description: String,
}

static NOTICES: std::sync::Mutex<Vec<(String, String, String)>> = std::sync::Mutex::new(Vec::new());

/// Queues a toast from anywhere in the UI; the root shows it on its next tick.
pub fn notify(kind: &str, title: &str, description: &str) {
    NOTICES.lock().unwrap().push((kind.into(), title.into(), description.into()));
}

pub fn drain_notices() -> Vec<(String, String, String)> {
    std::mem::take(&mut *NOTICES.lock().unwrap())
}

#[component]
pub fn ToastStack(items: Vec<ToastView>, ondismiss: EventHandler<u64>) -> Element {
    rsx! {
        div {
            "role": "region",
            "aria-label": "Notifications",
            "data-slot": "toast-viewport",
            "class": "fixed z-100 flex max-w-90 flex-col gap-3",
            "style": "top: 56px; right: 16px; width: 360px;",
            for t in items.iter() {
                div {
                    key: "{t.id}",
                    "data-type": "{t.kind}",
                    "role": "dialog",
                    "class": "dropdown-glass relative w-full overflow-visible select-none rounded-lg border text-popover-foreground shadow-xl shadow-black/25",
                    div {
                        "class": "absolute z-20 -top-1.5 -right-1.5",
                        button {
                            "type": "button",
                            "aria-label": "Dismiss notification",
                            "data-slot": "toast-close",
                            "class": "inline-flex size-6 shrink-0 cursor-pointer items-center justify-center rounded-full border border-border/60 bg-popover text-muted-foreground shadow-sm outline-none hover:text-foreground",
                            onclick: { let id = t.id; move |_| ondismiss.call(id) },
                            svg { "width": "24", "height": "24", "viewBox": "0 0 24 24", "fill": "none", "stroke": "currentColor", "stroke-width": "2.25", "stroke-linecap": "round", "stroke-linejoin": "round", "class": "lucide lucide-x size-3", "aria-hidden": "true", path { "d": "M18 6 6 18" } path { "d": "m6 6 12 12" } }
                        }
                    }
                    div {
                        "class": "pointer-events-auto flex min-h-0 items-center justify-between gap-1.5 py-3 pl-3.5 pr-3.5 text-sm",
                        div {
                            "class": "flex min-h-0 min-w-0 flex-1 flex-col gap-0.5",
                            div {
                                "class": "flex min-w-0 gap-2 pr-6",
                                div {
                                    "data-slot": "toast-icon",
                                    "class": "[&>svg]:h-lh [&>svg]:w-4 [&_svg]:pointer-events-none [&_svg]:shrink-0",
                                    svg {
                                        "width": "24", "height": "24", "viewBox": "0 0 24 24", "fill": "none", "stroke": "currentColor", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round",
                                        "class": if t.kind == "error" { "lucide lucide-circle-x text-destructive" } else { "lucide lucide-circle-check text-success" },
                                        "aria-hidden": "true",
                                        circle { "cx": "12", "cy": "12", "r": "10" }
                                        if t.kind == "error" {
                                            path { "d": "m15 9-6 6" }
                                            path { "d": "m9 9 6 6" }
                                        } else {
                                            path { "d": "m9 12 2 2 4-4" }
                                        }
                                    }
                                }
                                h2 { "data-slot": "toast-title", "class": "min-w-0 break-words font-medium", "{t.title}" }
                            }
                            if !t.description.is_empty() {
                                p { "data-slot": "toast-description", "class": "text-muted-foreground text-sm", "{t.description}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
