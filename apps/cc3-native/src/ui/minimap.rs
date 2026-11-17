//! Timeline minimap (`timeline-minimap`): one strip per user message at the left edge of the chat column.
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct Strip {
    /// Position in percent of the minimap height.
    pub top: f32,
    pub in_view: bool,
}

/// Strips for `count` user messages, spread evenly; the last half is marked in view (the timeline sits at the end).
pub fn strips(count: usize) -> Vec<Strip> {
    (0..count)
        .map(|i| Strip { top: if count > 1 { i as f32 / (count - 1) as f32 * 100.0 } else { 0.0 }, in_view: i * 2 >= count })
        .collect()
}

#[component]
pub fn Minimap(strips: Vec<Strip>, #[props(default)] onnav: EventHandler<String>) -> Element {
    rsx! {
    div {
        "class": "group/minimap pointer-events-none absolute inset-y-0 left-0 z-40 w-18 opacity-100",
        "data-testid": "timeline-minimap",
        "data-persistent-gutter": "true",
        div {
            "class": "relative h-full w-full select-none",
            div {
                "class": "absolute top-1/2 left-3 -translate-y-1/2 pointer-events-auto",
                "style": "height: min(40px, max(4rem, 100% - 14.75rem)); width: 40px;",
                span {
                    "class": "absolute left-1 z-10 inline-flex -translate-x-1/2 opacity-0 transition-opacity duration-150 hover:opacity-100 focus-within:opacity-100 pointer-events-auto bottom-[calc(100%+6px)]",
                    "id": "base-ui-_r_26_",
                    "data-base-ui-tooltip-trigger": "",
                    "data-slot": "tooltip-trigger",
                    button {
                        "type": "button",
                        "class": "[&_svg]:-mx-0.5 relative inline-flex shrink-0 cursor-pointer items-center justify-center gap-2 whitespace-nowrap border font-medium text-base outline-none transition-[box-shadow,scale] [&:not([aria-haspopup]):not(:disabled):not([aria-disabled=true])]:will-change-[scale] [&:active:not([aria-haspopup])]:scale-[0.97] before:pointer-events-none before:absolute before:inset-0 pointer-coarse:after:absolute pointer-coarse:after:size-full pointer-coarse:after:min-h-11 pointer-coarse:after:min-w-11 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-64 aria-disabled:cursor-not-allowed aria-disabled:opacity-64 sm:text-sm [&_svg:not([class*='text-'])]:text-[var(--control-icon-color)] sm:[&_svg:not([class*='size-'])]:size-4 [&_svg]:pointer-events-none [&_svg]:shrink-0 size-5 rounded-sm p-0 before:rounded-[calc(var(--radius-sm)-1px)] [&_svg:not([class*='size-'])]:size-3 [--control-icon-color:currentColor] border-transparent text-muted-foreground data-pressed:bg-accent [:hover,[data-pressed]]:bg-accent [:hover,[data-pressed]]:text-foreground",
                        "data-slot": "button",
                        "aria-label": "Previous turn",
                        onclick: move |_| onnav.call("prev".to_string()),
                        svg {
                            "width": "24",
                            "height": "24",
                            "viewBox": "0 0 24 24",
                            "fill": "none",
                            "stroke": "currentColor",
                            "stroke-width": "2",
                            "stroke-linecap": "round",
                            "stroke-linejoin": "round",
                            "class": "lucide lucide-chevron-up size-4 text-foreground/90",
                            "aria-hidden": "true",
                            path {
                                "d": "m18 15-6-6-6 6",
                            }
                        }
                    }
                }
                button {
                    "aria-label": "Jump to message: User message",
                    onclick: move |e| onnav.call(format!("frac:{}", (e.element_coordinates().y / 40.0).clamp(0.0, 1.0))),
                    "class": "absolute inset-y-0 left-0 w-full cursor-pointer bg-transparent before:absolute before:inset-x-0 before:-inset-y-1 before:content-[''] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring/70",
                    "type": "button",
                    div {
                        "class": "absolute top-0 left-3 h-full w-px bg-border/15",
                    }
                    for (index, strip) in strips.iter().enumerate() {
                        span {
                            key: "{index}",
                            "aria-hidden": "true",
                            "class": "pointer-events-none absolute left-0 h-0.5 w-6 origin-left -translate-y-1/2 rounded-full transition-transform duration-150 after:absolute after:inset-0 after:rounded-full after:bg-foreground/55 after:opacity-0 after:transition-opacity after:duration-150 data-[in-view=true]:after:opacity-100 bg-muted-foreground/25 scale-x-[0.333]",
                            "data-in-view": if strip.in_view { "true" } else { "false" },
                            "data-minimap-strip": "true",
                            "style": "top: {strip.top}%;",
                        }
                    }
                }
                span {
                    "class": "absolute left-1 z-10 inline-flex -translate-x-1/2 opacity-0 transition-opacity duration-150 hover:opacity-100 focus-within:opacity-100 pointer-events-auto top-[calc(100%+6px)]",
                    "id": "base-ui-_r_28_",
                    "data-base-ui-tooltip-trigger": "",
                    "data-slot": "tooltip-trigger",
                    button {
                        "type": "button",
                        "class": "[&_svg]:-mx-0.5 relative inline-flex shrink-0 cursor-pointer items-center justify-center gap-2 whitespace-nowrap border font-medium text-base outline-none transition-[box-shadow,scale] [&:not([aria-haspopup]):not(:disabled):not([aria-disabled=true])]:will-change-[scale] [&:active:not([aria-haspopup])]:scale-[0.97] before:pointer-events-none before:absolute before:inset-0 pointer-coarse:after:absolute pointer-coarse:after:size-full pointer-coarse:after:min-h-11 pointer-coarse:after:min-w-11 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-64 aria-disabled:cursor-not-allowed aria-disabled:opacity-64 sm:text-sm [&_svg:not([class*='text-'])]:text-[var(--control-icon-color)] sm:[&_svg:not([class*='size-'])]:size-4 [&_svg]:pointer-events-none [&_svg]:shrink-0 size-5 rounded-sm p-0 before:rounded-[calc(var(--radius-sm)-1px)] [&_svg:not([class*='size-'])]:size-3 [--control-icon-color:currentColor] border-transparent text-muted-foreground data-pressed:bg-accent [:hover,[data-pressed]]:bg-accent [:hover,[data-pressed]]:text-foreground",
                        "data-slot": "button",
                        "aria-label": "Next turn",
                        onclick: move |_| onnav.call("next".to_string()),
                        svg {
                            "width": "24",
                            "height": "24",
                            "viewBox": "0 0 24 24",
                            "fill": "none",
                            "stroke": "currentColor",
                            "stroke-width": "2",
                            "stroke-linecap": "round",
                            "stroke-linejoin": "round",
                            "class": "lucide lucide-chevron-down size-4 text-foreground/90",
                            "aria-hidden": "true",
                            path {
                                "d": "m6 9 6 6 6-6",
                            }
                        }
                    }
                }
            }
        }
    }
    }
}
