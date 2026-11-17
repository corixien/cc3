//! "Add project" dialog: asks for a workspace folder path. Uses the palette dialog's frame classes.
use dioxus::prelude::*;

#[component]
pub fn AddProjectDialog(value: String, error: String, oninput: EventHandler<String>, onsubmit: EventHandler<()>, onclose: EventHandler<()>) -> Element {
    rsx! {
        div {
            "data-base-ui-portal": "",
            div {
                "data-open": "",
                "role": "presentation",
                "data-slot": "dialog-backdrop",
                "class": "dialog-backdrop fixed inset-0 z-50 transition-all duration-200",
                "aria-hidden": "true",
                onclick: move |_| onclose.call(()),
            }
            div {
                "data-open": "",
                "role": "presentation",
                "data-slot": "dialog-viewport",
                "class": "pointer-events-none fixed inset-0 z-50 flex flex-col items-center px-4",
                "style": "padding-top: 12vh;",
                div {
                    "role": "dialog",
                    "aria-label": "Add project",
                    "data-slot": "dialog-popup",
                    "class": "dialog-glass relative flex min-h-0 w-full max-w-md min-w-0 flex-col gap-3 rounded-2xl border p-4 text-foreground pointer-events-auto",
                    div { "class": "text-base font-semibold", "Add project" }
                    div { "class": "text-sm font-normal text-muted-foreground", "Folder on this computer, for example /home/you/code/app" }
                    div {
                        "class": "relative flex w-full rounded-lg border border-input bg-background text-sm dark:bg-input/32",
                        input {
                            "class": "h-8.5 w-full min-w-0 rounded-[inherit] bg-transparent px-3 leading-8.5 outline-none placeholder:text-placeholder",
                            "placeholder": "/path/to/project",
                            "autocomplete": "off",
                            "spellcheck": "false",
                            "value": "{value}",
                            onmounted: move |e| {
                                spawn(async move {
                                    let _ = e.set_focus(true).await;
                                });
                            },
                            oninput: move |e| oninput.call(e.value()),
                            onkeydown: move |e| {
                                let key = e.key().to_string();
                                if key == "Enter" {
                                    onsubmit.call(());
                                } else if key == "Escape" {
                                    onclose.call(());
                                }
                            },
                        }
                    }
                    if !error.is_empty() {
                        div { "class": "text-sm text-destructive-foreground", "{error}" }
                    }
                    div {
                        "class": "flex justify-end gap-2",
                        button {
                            "type": "button",
                            "class": "inline-flex h-8 cursor-pointer items-center rounded-full border border-input px-4 text-sm text-foreground",
                            onclick: move |_| onclose.call(()),
                            "Cancel"
                        }
                        button {
                            "type": "button",
                            "class": "inline-flex h-8 cursor-pointer items-center rounded-full bg-primary px-4 text-sm font-medium text-primary-foreground",
                            "disabled": value.trim().is_empty(),
                            onclick: move |_| onsubmit.call(()),
                            "Add project"
                        }
                    }
                }
            }
        }
    }
}
