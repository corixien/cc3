//! Model picker popup above the composer: searchable list of the enabled providers' models.
use dioxus::prelude::*;

/// (instance id, provider name, slug, model name)
pub type ModelOption = (String, String, String, String);

#[component]
pub fn ModelPicker(options: Vec<ModelOption>, current: String, query: String, oninput: EventHandler<String>, onpick: EventHandler<(String, String)>, onclose: EventHandler<()>) -> Element {
    let q = query.to_lowercase();
    let shown: Vec<&ModelOption> = options.iter().filter(|(_, p, s, n)| q.is_empty() || n.to_lowercase().contains(&q) || s.to_lowercase().contains(&q) || p.to_lowercase().contains(&q)).collect();
    rsx! {
        div {
            "data-base-ui-portal": "",
            div { "class": "fixed inset-0 z-50", onclick: move |_| onclose.call(()) }
            div {
                "role": "dialog",
                "aria-label": "Model picker",
                "data-slot": "popover-popup",
                "class": "dialog-glass fixed z-50 flex flex-col overflow-hidden rounded-xl border text-foreground",
                "style": "left: 437px; bottom: 76px; width: 340px; max-height: 360px;",
                div {
                    "class": "border-b px-3 py-2",
                    input {
                        "class": "h-7 w-full bg-transparent text-sm outline-none",
                        "autocomplete": "off",
                        "spellcheck": "false",
                        "value": "{query}",
                        onmounted: move |e| {
                            spawn(async move {
                                let _ = e.set_focus(true).await;
                            });
                        },
                        oninput: move |e| oninput.call(e.value()),
                        onkeydown: move |e| {
                            if e.key().to_string() == "Escape" {
                                onclose.call(());
                            }
                        },
                    }
                    if query.is_empty() {
                        span { "class": "pointer-events-none absolute left-3 top-2 flex h-7 items-center text-sm font-normal text-placeholder", "Search models..." }
                    }
                }
                div {
                    "class": "flex min-h-0 flex-col gap-px overflow-y-auto p-1.5",
                    "style": "overflow: auto;",
                    for (instance, provider, slug, name) in shown {
                        div {
                            key: "{instance}/{slug}",
                            "role": "option",
                            "class": "flex min-h-8 cursor-pointer items-center gap-2 rounded-sm px-2 py-1.5 text-sm hover:bg-accent",
                            onclick: { let (i, s) = (instance.clone(), slug.clone()); move |_| onpick.call((i.clone(), s.clone())) },
                            span { "class": "min-w-0 flex-1 truncate font-medium", "{name}" }
                            span { "class": "shrink-0 text-xs font-normal text-muted-foreground", "{provider}" }
                            if *name == current {
                                span { "class": "shrink-0 text-xs text-muted-foreground", "✓" }
                            }
                        }
                    }
                }
            }
        }
    }
}
