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
pub fn WorkSummaryRow(run_id: String, label: String, time: String, uid: String) -> Element {
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
                                    "aria-expanded": "false",
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
                                        "class": "lucide lucide-chevron-right size-3.5",
                                        "aria-hidden": "true",
                                        path {
                                            "d": "m9 18 6-6-6-6",
                                        }
                                    }
                                }
                                span {
                                    "class": "pointer-events-none absolute me-1 shrink-0 whitespace-nowrap rounded-md text-muted-foreground text-xs tabular-nums opacity-0 group-hover/timeline-row:pointer-events-auto group-hover/timeline-row:static group-hover/timeline-row:opacity-100 group-focus-within/timeline-row:pointer-events-auto group-focus-within/timeline-row:static group-focus-within/timeline-row:opacity-100 ms-auto",
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
