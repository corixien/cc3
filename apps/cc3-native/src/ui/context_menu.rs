//! Right-click menu of a sidebar thread row.
use dioxus::prelude::*;

pub const ITEMS: &[(&str, &str)] = &[("pin", "Pin thread"), ("settle", "Settle thread"), ("snooze", "Snooze for a day"), ("archive", "Archive thread"), ("delete", "Delete thread")];

#[component]
pub fn ThreadMenu(x: i32, y: i32, onpick: EventHandler<String>, onclose: EventHandler<()>) -> Element {
    // Deleting asks for a second click.
    let mut armed = use_signal(|| false);
    rsx! {
        div {
            "data-base-ui-portal": "",
            div { "class": "fixed inset-0 z-50", onclick: move |_| onclose.call(()) }
            div {
                "role": "menu",
                "data-slot": "menu-popup",
                "class": "dropdown-glass fixed z-50 flex min-w-44 flex-col rounded-lg border p-1 text-sm text-popover-foreground shadow-lg",
                "style": "left: {x}px; top: {y}px;",
                for (key, label) in ITEMS.iter() {
                    div {
                        key: "{key}",
                        "role": "menuitem",
                        "class": "flex min-h-7 cursor-pointer items-center rounded-sm px-2 py-1 font-normal hover:bg-accent",
                        onclick: { let key = key.to_string(); move |_| if key == "delete" && !armed() { armed.set(true) } else { onpick.call(key.clone()) } },
                        if *key == "delete" && armed() { "Click again to delete" } else { "{label}" }
                    }
                }
            }
        }
    }
}
