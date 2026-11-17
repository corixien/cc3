//! Timeline rows of the chat column. Markup and classes come from the Chromium golden snapshot
//! (tools/html2rsx.py); ids, text and times are props, message bodies are rendered markdown.
use dioxus::prelude::*;

#[component]
pub fn UserMessageRow(message_id: String, time: String, uid: String, body: Element) -> Element {
    rsx! {
            div {
                "class": "messages-timeline-row-frame",
                div {
                    "class": "chat-content-lane overflow-x-clip",
                    "data-timeline-root": "true",
                    div {
                        "class": "pb-4",
                        "data-timeline-row-id": "{message_id}",
                        "data-timeline-row-kind": "message",
                        "data-message-id": "{message_id}",
                        "data-message-role": "user",
                        div {
                            "class": "group @container flex flex-col items-end gap-1",
                            div {
                                "class": "relative min-w-0 max-w-[80%] rounded-2xl text-message-foreground [&_[data-slot=context-chip]]:[--context-chip-ink:var(--context-chip-message,var(--context-chip-accent))] bg-message p-3",
                                "data-user-message-bubble": "",
                                h3 { "class": "sr-only select-none", "You" }
                                div {
                                    "dir": "ltr",
                                    "class": "",
                                    div {
                                        div {
                                            div {
                                                "class": "relative",
                                                "data-user-message-body": "true",
                                                "data-thread-find-text": "true",
                                                "data-user-message-collapsed": "false",
                                                "data-user-message-collapsible": "false",
                                                "data-user-message-fade": "false",
                                                div {
                                                    div {
                                                        "class": "chat-markdown w-full min-w-0 [overflow-wrap:anywhere] [word-break:break-word] text-sm leading-relaxed text-message-foreground user-message-markdown",
                                                        {body}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            div {
                                "class": "flex w-full max-w-[80%] justify-end pe-1 items-center text-xs tabular-nums opacity-0 transition-opacity duration-200 pointer-coarse:opacity-100 focus-within:opacity-100 group-hover:opacity-100",
                                div {
                                    "class": "flex items-center justify-end gap-x-2 gap-y-0.5 shrink-0",
                                    p {
                                        "class": "text-muted-foreground text-xs tabular-nums",
                                        "id": "base-ui-{uid}-2n",
                                        "data-base-ui-tooltip-trigger": "",
                                        "data-slot": "tooltip-trigger",
                                        "{time}"
                                    }
                                    div {
                                        "class": "flex items-center gap-0.5",
                                        button {
                                            "type": "button",
                                            "class": "[&_svg]:-mx-0.5 relative inline-flex shrink-0 cursor-pointer items-center justify-center whitespace-nowrap rounded-[var(--control-radius)] border font-medium outline-none transition-[box-shadow,scale] [&:not([aria-haspopup]):not(:disabled):not([aria-disabled=true])]:will-change-[scale] [&:active:not([aria-haspopup])]:scale-[0.97] before:pointer-events-none before:absolute before:inset-0 before:rounded-[calc(var(--control-radius)-1px)] pointer-coarse:after:absolute pointer-coarse:after:size-full pointer-coarse:after:min-h-11 pointer-coarse:after:min-w-11 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-64 aria-disabled:cursor-not-allowed aria-disabled:opacity-64 [&_svg:not([class*='text-'])]:text-[var(--control-icon-color)] [&_svg]:pointer-events-none [&_svg]:shrink-0 h-7 gap-1 px-[calc(--spacing(2)-1px)] text-sm sm:h-6 sm:text-xs [&_svg:not([class*='size-'])]:size-4 sm:[&_svg:not([class*='size-'])]:size-3.5 [--control-icon-color:currentColor] border-transparent text-muted-foreground data-pressed:bg-accent [:hover,[data-pressed]]:bg-accent [:hover,[data-pressed]]:text-foreground",
                                            "data-slot": "tooltip-trigger",
                                            "aria-label": "Copy message",
                                            "id": "base-ui-{uid}-2p",
                                            "data-base-ui-tooltip-trigger": "",
                                            svg {
                                                "width": "24",
                                                "height": "24",
                                                "viewBox": "0 0 24 24",
                                                "fill": "none",
                                                "stroke": "currentColor",
                                                "stroke-width": "2",
                                                "stroke-linecap": "round",
                                                "stroke-linejoin": "round",
                                                "class": "lucide lucide-copy size-3",
                                                "aria-hidden": "true",
                                                rect {
                                                    "width": "14",
                                                    "height": "14",
                                                    "x": "8",
                                                    "y": "8",
                                                    "rx": "2",
                                                    "ry": "2",
                                                }
                                                path {
                                                    "d": "M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2",
                                                }
                                            }
                                        }
                                        button {
                                            "type": "button",
                                            "class": "[&_svg]:-mx-0.5 relative inline-flex shrink-0 cursor-pointer items-center justify-center whitespace-nowrap rounded-[var(--control-radius)] border font-medium outline-none transition-[box-shadow,scale] [&:not([aria-haspopup]):not(:disabled):not([aria-disabled=true])]:will-change-[scale] [&:active:not([aria-haspopup])]:scale-[0.97] before:pointer-events-none before:absolute before:inset-0 before:rounded-[calc(var(--control-radius)-1px)] pointer-coarse:after:absolute pointer-coarse:after:size-full pointer-coarse:after:min-h-11 pointer-coarse:after:min-w-11 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-64 aria-disabled:cursor-not-allowed aria-disabled:opacity-64 [&_svg:not([class*='text-'])]:text-[var(--control-icon-color)] [&_svg]:pointer-events-none [&_svg]:shrink-0 h-7 gap-1 px-[calc(--spacing(2)-1px)] text-sm sm:h-6 sm:text-xs [&_svg:not([class*='size-'])]:size-4 sm:[&_svg:not([class*='size-'])]:size-3.5 [--control-icon-color:currentColor] border-transparent text-muted-foreground data-pressed:bg-accent [:hover,[data-pressed]]:bg-accent [:hover,[data-pressed]]:text-foreground",
                                            "data-slot": "tooltip-trigger",
                                            "aria-label": "Copy prompt link",
                                            "id": "base-ui-{uid}-2r",
                                            "data-base-ui-tooltip-trigger": "",
                                            svg {
                                                "width": "24",
                                                "height": "24",
                                                "viewBox": "0 0 24 24",
                                                "fill": "none",
                                                "stroke": "currentColor",
                                                "stroke-width": "2",
                                                "stroke-linecap": "round",
                                                "stroke-linejoin": "round",
                                                "class": "lucide lucide-link size-3",
                                                "aria-hidden": "true",
                                                path {
                                                    "d": "M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71",
                                                }
                                                path {
                                                    "d": "M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71",
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
    }
}

#[component]
pub fn AssistantMessageRow(message_id: String, environment_id: String, thread_id: String, time: String, uid: String, body: Element) -> Element {
    rsx! {
            div {
                "class": "messages-timeline-row-frame",
                div {
                    "class": "chat-content-lane overflow-x-clip",
                    "data-timeline-root": "true",
                    div {
                        "class": "pb-4 group/assistant",
                        "data-timeline-row-id": "{message_id}",
                        "data-timeline-row-kind": "message",
                        "data-message-id": "{message_id}",
                        "data-message-role": "assistant",
                        div {
                            "class": "relative min-w-0 px-1 py-0.5",
                            h3 { "class": "sr-only select-none", "T3 Code" }
                            div {
                                "data-thread-find-text": "true",
                                div {
                                    "data-assistant-citation-source": "{message_id}",
                                    "data-assistant-citation-environment": "{environment_id}",
                                    "data-assistant-citation-thread": "{thread_id}",
                                    div {
                                        "class": "chat-markdown w-full min-w-0 [overflow-wrap:anywhere] [word-break:break-word] text-sm leading-relaxed text-foreground/[calc(90%+var(--appearance-contrast-boost)/10)]",
                                        {body}
                                    }
                                }
                            }
                            div {
                                "class": "flex items-center gap-2 text-xs tabular-nums transition-opacity duration-200 opacity-0 pointer-coarse:opacity-100 focus-within:opacity-100 group-hover/assistant:opacity-100 mt-1.5",
                                button {
                                    "type": "button",
                                    "class": "[&_svg]:-mx-0.5 relative inline-flex shrink-0 cursor-pointer items-center justify-center whitespace-nowrap rounded-[var(--control-radius)] border font-medium outline-none transition-[box-shadow,scale] [&:not([aria-haspopup]):not(:disabled):not([aria-disabled=true])]:will-change-[scale] [&:active:not([aria-haspopup])]:scale-[0.97] before:pointer-events-none before:absolute before:inset-0 before:rounded-[calc(var(--control-radius)-1px)] pointer-coarse:after:absolute pointer-coarse:after:size-full pointer-coarse:after:min-h-11 pointer-coarse:after:min-w-11 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-64 aria-disabled:cursor-not-allowed aria-disabled:opacity-64 [&_svg:not([class*='text-'])]:text-[var(--control-icon-color)] [&_svg]:pointer-events-none [&_svg]:shrink-0 h-7 gap-1 px-[calc(--spacing(2)-1px)] text-sm sm:h-6 sm:text-xs [&_svg:not([class*='size-'])]:size-4 sm:[&_svg:not([class*='size-'])]:size-3.5 [--control-icon-color:currentColor] border-transparent text-muted-foreground data-pressed:bg-accent [:hover,[data-pressed]]:bg-accent [:hover,[data-pressed]]:text-foreground",
                                    "data-slot": "tooltip-trigger",
                                    "aria-label": "Copy message",
                                    "id": "base-ui-{uid}-2v",
                                    "data-base-ui-tooltip-trigger": "",
                                    svg {
                                        "width": "24",
                                        "height": "24",
                                        "viewBox": "0 0 24 24",
                                        "fill": "none",
                                        "stroke": "currentColor",
                                        "stroke-width": "2",
                                        "stroke-linecap": "round",
                                        "stroke-linejoin": "round",
                                        "class": "lucide lucide-copy size-3",
                                        "aria-hidden": "true",
                                        rect {
                                            "width": "14",
                                            "height": "14",
                                            "x": "8",
                                            "y": "8",
                                            "rx": "2",
                                            "ry": "2",
                                        }
                                        path {
                                            "d": "M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2",
                                        }
                                    }
                                }
                                button {
                                    "type": "button",
                                    "class": "[&_svg]:-mx-0.5 relative inline-flex shrink-0 cursor-pointer items-center justify-center whitespace-nowrap rounded-[var(--control-radius)] border font-medium outline-none transition-[box-shadow,scale] [&:not([aria-haspopup]):not(:disabled):not([aria-disabled=true])]:will-change-[scale] [&:active:not([aria-haspopup])]:scale-[0.97] before:pointer-events-none before:absolute before:inset-0 before:rounded-[calc(var(--control-radius)-1px)] pointer-coarse:after:absolute pointer-coarse:after:size-full pointer-coarse:after:min-h-11 pointer-coarse:after:min-w-11 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-64 aria-disabled:cursor-not-allowed aria-disabled:opacity-64 [&_svg:not([class*='text-'])]:text-[var(--control-icon-color)] [&_svg]:pointer-events-none [&_svg]:shrink-0 h-7 gap-1 px-[calc(--spacing(2)-1px)] text-sm sm:h-6 sm:text-xs [&_svg:not([class*='size-'])]:size-4 sm:[&_svg:not([class*='size-'])]:size-3.5 [--control-icon-color:var(--contrast-muted-foreground)] border-transparent text-foreground data-pressed:bg-accent [:hover,[data-pressed]]:bg-accent",
                                    "data-slot": "tooltip-trigger",
                                    "aria-label": "Fork from this response",
                                    "id": "base-ui-{uid}-31",
                                    "data-base-ui-tooltip-trigger": "",
                                    svg {
                                        "width": "24",
                                        "height": "24",
                                        "viewBox": "0 0 24 24",
                                        "fill": "none",
                                        "stroke": "currentColor",
                                        "stroke-width": "2",
                                        "stroke-linecap": "round",
                                        "stroke-linejoin": "round",
                                        "class": "lucide lucide-git-fork size-3",
                                        "aria-hidden": "true",
                                        circle {
                                            "cx": "12",
                                            "cy": "18",
                                            "r": "3",
                                        }
                                        circle {
                                            "cx": "6",
                                            "cy": "6",
                                            "r": "3",
                                        }
                                        circle {
                                            "cx": "18",
                                            "cy": "6",
                                            "r": "3",
                                        }
                                        path {
                                            "d": "M18 9v2c0 .6-.4 1-1 1H7c-.6 0-1-.4-1-1V9",
                                        }
                                        path {
                                            "d": "M12 12v3",
                                        }
                                    }
                                }
                                button {
                                    "type": "button",
                                    "class": "[&_svg]:-mx-0.5 relative inline-flex shrink-0 cursor-pointer items-center justify-center whitespace-nowrap rounded-[var(--control-radius)] border font-medium outline-none transition-[box-shadow,scale] [&:not([aria-haspopup]):not(:disabled):not([aria-disabled=true])]:will-change-[scale] [&:active:not([aria-haspopup])]:scale-[0.97] before:pointer-events-none before:absolute before:inset-0 before:rounded-[calc(var(--control-radius)-1px)] pointer-coarse:after:absolute pointer-coarse:after:size-full pointer-coarse:after:min-h-11 pointer-coarse:after:min-w-11 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-64 aria-disabled:cursor-not-allowed aria-disabled:opacity-64 [&_svg:not([class*='text-'])]:text-[var(--control-icon-color)] [&_svg]:pointer-events-none [&_svg]:shrink-0 h-7 gap-1 px-[calc(--spacing(2)-1px)] text-sm sm:h-6 sm:text-xs [&_svg:not([class*='size-'])]:size-4 sm:[&_svg:not([class*='size-'])]:size-3.5 [--control-icon-color:currentColor] border-transparent text-muted-foreground data-pressed:bg-accent [:hover,[data-pressed]]:bg-accent [:hover,[data-pressed]]:text-foreground",
                                    "data-slot": "tooltip-trigger",
                                    "aria-label": "Reply to this response",
                                    "id": "base-ui-{uid}-33",
                                    "data-base-ui-tooltip-trigger": "",
                                    svg {
                                        "width": "24",
                                        "height": "24",
                                        "viewBox": "0 0 24 24",
                                        "fill": "none",
                                        "stroke": "currentColor",
                                        "stroke-width": "2",
                                        "stroke-linecap": "round",
                                        "stroke-linejoin": "round",
                                        "class": "lucide lucide-reply size-3",
                                        "aria-hidden": "true",
                                        path {
                                            "d": "M20 18v-2a4 4 0 0 0-4-4H4",
                                        }
                                        path {
                                            "d": "m9 17-5-5 5-5",
                                        }
                                    }
                                }
                                p {
                                    "class": "text-muted-foreground text-xs tabular-nums",
                                    "id": "base-ui-{uid}-35",
                                    "data-base-ui-tooltip-trigger": "",
                                    "data-slot": "tooltip-trigger",
                                    "{time}"
                                }
                            }
                        }
                    }
                }
            }
    }
}

#[component]
pub fn WorkSummaryRow(run_id: String, label: String, time: String, uid: String, expanded: bool, ontoggle: EventHandler<()>) -> Element {
    let ms_auto = if expanded { "ms-auto" } else { "" };
    rsx! {
            div {
                "class": "messages-timeline-row-frame",
                div {
                    "class": "chat-content-lane overflow-x-clip",
                    "data-timeline-root": "true",
                    div {
                        "class": "pb-1.5",
                        "data-timeline-row-id": "turn-fold:{run_id}",
                        "data-timeline-row-kind": "turn-fold",
                        div {
                            "class": "group/timeline-row border-b border-border/60 pb-2 pe-0.5 pt-1",
                            div {
                                "class": "relative flex items-center gap-1",
                                button {
                                    "type": "button",
                                    "aria-expanded": "{expanded}",
                                    onclick: move |_| ontoggle.call(()),
                                    "data-scroll-anchor-ignore": "true",
                                    "aria-busy": "false",
                                    "class": "flex cursor-pointer select-none items-center gap-1 rounded-md px-1 text-sm leading-relaxed text-muted-foreground tabular-nums transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring/70",
                                    span { "{label}" }
                                    svg {
                                        "width": "24",
                                        "height": "24",
                                        "viewBox": "0 0 24 24",
                                        "fill": "none",
                                        "stroke": "currentColor",
                                        "stroke-width": "2",
                                        "stroke-linecap": "round",
                                        "stroke-linejoin": "round",
                                        "class": if expanded { "lucide lucide-chevron-down size-3.5" } else { "lucide lucide-chevron-right size-3.5" },
                                        "aria-hidden": "true",
                                        path {
                                            "d": if expanded { "m6 9 6 6 6-6" } else { "m9 18 6-6-6-6" },
                                        }
                                    }
                                }
                                span {
                                    "class": "pointer-events-none absolute me-1 shrink-0 whitespace-nowrap rounded-md text-muted-foreground text-xs tabular-nums opacity-0 group-hover/timeline-row:pointer-events-auto group-hover/timeline-row:static group-hover/timeline-row:opacity-100 group-focus-within/timeline-row:pointer-events-auto group-focus-within/timeline-row:static group-focus-within/timeline-row:opacity-100 {ms_auto}",
                                    "id": "base-ui-{uid}-2t",
                                    "data-base-ui-tooltip-trigger": "",
                                    "data-slot": "tooltip-trigger",
                                    "{time}"
                                }
                            }
                        }
                    }
                }
            }
    }
}

/// Collapsed group of a run's work entries ("Ran 1 command, changed 1 file, ...").
#[component]
pub fn WorkToggleRow(first_item_id: String, label: String, time: String, uid: String, open: bool, ontoggle: EventHandler<()>) -> Element {
    let ms = if open { " ms-auto" } else { "" };
    rsx! {
        div {
            "class": "messages-timeline-row-frame",
            div {
                "class": "chat-content-lane overflow-x-clip",
                "data-timeline-root": "true",
                div {
                    "class": "",
                    "data-timeline-row-id": "work-toggle:{first_item_id}",
                    "data-timeline-row-kind": "work-toggle",
                    div {
                        "class": if open { "pb-0" } else { "pb-2" },
                        button {
                            "aria-label": "{label}",
                            "aria-expanded": "{open}",
                            onclick: move |_| ontoggle.call(()),
                            "type": "button",
                            "class": "group/timeline-row relative w-full min-w-0 rounded-md px-0.5 text-left transition-colors py-0.5 cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring/70",
                            div {
                                "class": "flex min-h-6 min-w-0 gap-1.5 text-sm leading-relaxed select-none [&_*]:select-none items-center",
                                div {
                                    "class": "flex min-w-0 flex-1 items-baseline gap-1.5",
                                    span {
                                        "class": "relative w-6 shrink-0 leading-0",
                                        span {
                                            "class": "inline-block size-6 overflow-hidden align-[calc(0.5cap_-_0.75rem)]",
                                            span {
                                                "class": "flex size-full items-center justify-center",
                                                svg {
                                                    "width": "24", "height": "24", "viewBox": "0 0 24 24", "fill": "none", "stroke": "currentColor", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round",
                                                    "class": "lucide lucide-hammer size-4 shrink-0 stroke-2 text-icon-muted opacity-70 light:brightness-60",
                                                    "aria-hidden": "true",
                                                    path { "d": "m15 12-9.373 9.373a1 1 0 0 1-3.001-3L12 9" }
                                                    path { "d": "m18 15 4-4" }
                                                    path { "d": "m21.5 11.5-1.914-1.914A2 2 0 0 1 19 8.172v-.344a2 2 0 0 0-.586-1.414l-1.657-1.657A6 6 0 0 0 12.516 3H9l1.243 1.243A6 6 0 0 1 12 8.485V10l2 2h1.172a2 2 0 0 1 1.414.586L18.5 14.5" }
                                                }
                                            }
                                        }
                                    }
                                    div {
                                        "class": "min-w-0 flex-1 text-secondary-label truncate [&_*]:whitespace-nowrap",
                                        span { "class": "block truncate", "{label}" }
                                    }
                                }
                                span {
                                    "class": "pointer-events-none absolute me-1 shrink-0 whitespace-nowrap rounded-md text-muted-foreground text-xs tabular-nums opacity-0 group-hover/timeline-row:pointer-events-auto group-hover/timeline-row:static group-hover/timeline-row:opacity-100 group-focus-within/timeline-row:pointer-events-auto group-focus-within/timeline-row:static group-focus-within/timeline-row:opacity-100{ms}",
                                    "id": "base-ui-{uid}-wt",
                                    "data-base-ui-tooltip-trigger": "",
                                    "data-slot": "tooltip-trigger",
                                    "{time}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn entry_icon(kind: &str) -> Element {
    let class = |n: &str| format!("lucide lucide-{n} block size-4 shrink-0 stroke-2 opacity-70 light:brightness-60");
    match kind {
        "reasoning" => rsx! { svg { "width": "24", "height": "24", "viewBox": "0 0 24 24", "fill": "none", "stroke": "currentColor", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round", "class": class("brain"), "aria-hidden": "true",
            path { "d": "M12 18V5" } path { "d": "M15 13a4.17 4.17 0 0 1-3-4 4.17 4.17 0 0 1-3 4" } path { "d": "M17.598 6.5A3 3 0 1 0 12 5a3 3 0 1 0-5.598 1.5" } path { "d": "M17.997 5.125a4 4 0 0 1 2.526 5.77" } path { "d": "M18 18a4 4 0 0 0 2-7.464" } path { "d": "M19.967 17.483A4 4 0 1 1 12 18a4 4 0 1 1-7.967-.517" } path { "d": "M6 18a4 4 0 0 1-2-7.464" } path { "d": "M6.003 5.125a4 4 0 0 0-2.526 5.77" } } },
        "command_execution" => rsx! { svg { "width": "24", "height": "24", "viewBox": "0 0 24 24", "fill": "none", "stroke": "currentColor", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round", "class": class("terminal"), "aria-hidden": "true",
            path { "d": "M12 19h8" } path { "d": "m4 17 6-6-6-6" } } },
        "file_change" => rsx! { svg { "width": "24", "height": "24", "viewBox": "0 0 24 24", "fill": "none", "stroke": "currentColor", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round", "class": class("square-pen"), "aria-hidden": "true",
            path { "d": "M12 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" } path { "d": "M18.375 2.625a1 1 0 0 1 3 3l-9.013 9.014a2 2 0 0 1-.853.505l-2.873.84a.5.5 0 0 1-.62-.62l.84-2.873a2 2 0 0 1 .506-.852z" } } },
        _ => rsx! { svg { "width": "24", "height": "24", "viewBox": "0 0 24 24", "fill": "none", "stroke": "currentColor", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round", "class": class("globe"), "aria-hidden": "true",
            circle { "cx": "12", "cy": "12", "r": "10" } path { "d": "M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20" } path { "d": "M2 12h20" } } },
    }
}

/// The expanded group: a scrollable list of the run's tool calls and reasoning lines.
#[component]
pub fn WorkDetailsRow(first_item_id: String, entries: Vec<crate::model::WorkEntry>, workspace: String, times: Vec<String>, uid: String) -> Element {
    let height = entries.len() * 24;
    rsx! {
        div {
            "class": "messages-timeline-row-frame",
            div {
                "class": "chat-content-lane overflow-x-clip",
                "data-timeline-root": "true",
                div {
                    "class": "",
                    "data-timeline-row-id": "work-group:{first_item_id}:details",
                    "data-timeline-row-kind": "work",
                    div {
                        "class": "pb-1",
                        div {
                            "class": "flex min-w-0 flex-col",
                            div {
                                "class": "scrollbar-gutter-stable scroll-py-6 overflow-x-hidden rounded-md focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring/70",
                                "tabindex": "0",
                                "role": "region",
                                "aria-label": "Tool calls",
                                "data-tool-group-scroll": "true",
                                "style": "overflow: auto; max-height: calc(0px + min(18rem, 50dvh)); overflow-anchor: none;",
                                div {
                                    "class": "legend-list-content-container",
                                    "style": "display: block; min-height: 100%; box-sizing: border-box; overflow-anchor: none;",
                                    div {
                                        "style": "height: {height}px; opacity: 1; position: relative;",
                                        for (i, e) in entries.iter().enumerate() {
                                            div {
                                                key: "{e.id}",
                                                "style": "contain: layout style paint; left: 0px; position: absolute; right: 0px; top: {i * 24}px;",
                                                {
                                                    let label = if e.kind == "file_change" && !workspace.is_empty() { format!("{workspace}/{}", e.label) } else { e.label.clone() };
                                                    let time = times.get(i).cloned().unwrap_or_default();
                                                    rsx! {
                                                        div {
                                                            "data-work-entry-id": "{e.id}",
                                                            "data-v2-item-type": "{e.kind}",
                                                            "data-v2-item-visibility": "local",
                                                            "role": "button",
                                                            "tabindex": "0",
                                                            "aria-label": "{label}",
                                                            "aria-expanded": "false",
                                                            "class": "group/timeline-row relative w-full min-w-0 rounded-md px-0.5 text-left transition-colors py-0 cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring/70",
                                                            div {
                                                                "class": "flex min-h-6 min-w-0 gap-1.5 text-sm leading-relaxed select-none [&_*]:select-none items-center",
                                                                div {
                                                                    "class": "flex min-w-0 flex-1 items-baseline gap-1.5",
                                                                    span {
                                                                        "class": "relative w-6 shrink-0 leading-0",
                                                                        span {
                                                                            "class": "inline-block size-6 overflow-hidden align-[calc(0.5cap_-_0.75rem)]",
                                                                            span {
                                                                                "class": "flex size-full items-center justify-center",
                                                                                span { "class": "flex size-4 items-center justify-center text-icon-muted", {entry_icon(&e.kind)} }
                                                                            }
                                                                        }
                                                                    }
                                                                    div {
                                                                        "class": "min-w-0 flex-1 text-secondary-label truncate [&_*]:whitespace-nowrap",
                                                                        div {
                                                                            "class": "min-w-0 flex-1 overflow-hidden",
                                                                            p {
                                                                                "class": "flex min-w-0 w-full items-baseline gap-1.5 text-sm leading-relaxed",
                                                                                span {
                                                                                    "class": "min-w-0 flex-1 truncate text-secondary-label",
                                                                                    if e.kind == "reasoning" {
                                                                                        span { "class": "chat-markdown w-full min-w-0 [overflow-wrap:anywhere] [word-break:break-word]", "{label}" }
                                                                                    } else {
                                                                                        "{label}"
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                                span {
                                                                    "class": "flex size-4 shrink-0 items-center justify-center",
                                                                    "aria-hidden": "true",
                                                                    svg { "width": "24", "height": "24", "viewBox": "0 0 24 24", "fill": "none", "stroke": "currentColor", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round", "class": "lucide lucide-chevron-right size-3 shrink-0 text-icon-muted opacity-70 transition-transform duration-200", "aria-hidden": "true", path { "d": "m9 18 6-6-6-6" } }
                                                                }
                                                                span {
                                                                    "class": "pointer-events-none absolute me-1 shrink-0 whitespace-nowrap rounded-md text-muted-foreground text-xs tabular-nums opacity-0 group-hover/timeline-row:pointer-events-auto group-hover/timeline-row:static group-hover/timeline-row:opacity-100 group-focus-within/timeline-row:pointer-events-auto group-focus-within/timeline-row:static group-focus-within/timeline-row:opacity-100",
                                                                    "id": "base-ui-{uid}-e{i}",
                                                                    "data-base-ui-tooltip-trigger": "",
                                                                    "data-slot": "tooltip-trigger",
                                                                    "{time}"
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
