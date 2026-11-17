//! One thread row of the sidebar. Markup and classes come from the Chromium golden snapshot
//! (tools/html2rsx.py); the dynamic parts are props. `active` marks the open thread.
use dioxus::prelude::*;

#[component]
pub fn ThreadRow(
    thread_key: String,
    aria_label: String,
    ui_id: String,
    title: String,
    project: String,
    monogram: String,
    age: String,
    branch: String,
    active: bool,
    onselect: EventHandler<()>,
) -> Element {
    rsx! {
        li {
            "data-thread-item": "{thread_key}",
            "class": "list-none py-0.5 [content-visibility:auto] [contain-intrinsic-size:auto_78px]",
            div {
                "role": "button",
                    onclick: move |_| onselect.call(()),
                        "aria-current": active.then_some("page"),
                "tabindex": "0",
                "data-sidebar-row": "{thread_key}",
                "aria-label": "{aria_label}",
                "data-testid": "sidebar-row-card",
                "class": if active { "group/sidebar-row relative w-full cursor-pointer overflow-hidden rounded-md text-left outline-none select-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring bg-sidebar-row-active text-sidebar-foreground light:ring-1 light:ring-inset light:ring-sidebar-border/70" } else { "group/sidebar-row relative w-full cursor-pointer overflow-hidden rounded-md text-left outline-none select-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring text-sidebar-muted-foreground/75 hover:bg-sidebar-row-hover hover:text-sidebar-foreground" },
                "id": "base-ui-{ui_id}",
                "data-base-ui-tooltip-trigger": "",
                "data-slot": "tooltip-trigger",
                div {
                    "class": "relative z-10 h-[4.875rem] px-(--sidebar-row-content-inset) py-(--sidebar-content-inset)",
                    div {
                        "class": "flex h-5 min-w-0 items-center gap-1.5",
                        span {
                            "aria-hidden": "true",
                            "data-slot": "project-monogram",
                            "class": "inline-flex items-center justify-center size-4 shrink-0",
                            svg {
                                "viewBox": "0 0 16 16",
                                "class": "size-full overflow-hidden rounded-[25%] font-mono select-none text-sky-600 dark:text-sky-400",
                                "style": "background-color: color-mix(in srgb, currentcolor 14%, transparent 86%);",
                                text {
                                    "x": "8",
                                    "y": "10.8",
                                    "text-anchor": "middle",
                                    "fill": "currentColor",
                                    "class": "font-mono",
                                    "font-size": "8.25",
                                    "font-weight": "700",
                                    "textLength": "12",
                                    "lengthAdjust": "spacingAndGlyphs",
                                    "text-rendering": "geometricPrecision",
                                    "{monogram}"
                                }
                            }
                        }
                        span { "class": "min-w-0 flex-1 truncate font-normal text-secondary-label text-xs", "{project}" }
                        span {
                            "class": "group/sidebar-status-slot relative ml-auto flex h-5 min-w-8 shrink-0 items-stretch justify-end text-xs",
                            span { "class": "pointer-events-none group-has-[:focus-visible]/sidebar-status-slot:absolute group-has-[:focus-visible]/sidebar-status-slot:right-0 group-has-[:focus-visible]/sidebar-status-slot:opacity-0 group-any-hover/sidebar-row:absolute group-any-hover/sidebar-row:right-0 group-any-hover/sidebar-row:opacity-0 flex items-center self-center justify-self-end tabular-nums text-secondary-label", "{age}" }
                            span {
                                "class": "pointer-events-none absolute inset-y-0 right-0 flex items-stretch opacity-0 has-[:focus-visible]:pointer-events-auto has-[:focus-visible]:static has-[:focus-visible]:opacity-100 group-any-hover/sidebar-row:pointer-events-auto group-any-hover/sidebar-row:static group-any-hover/sidebar-row:opacity-100",
                                button {
                                    "type": "button",
                                    "aria-label": "Snooze thread",
                                    "class": "inline-flex h-full cursor-pointer items-center gap-0.5 rounded-md bg-transparent px-1.5 text-xs text-muted-foreground hover:text-foreground",
                                    "tabindex": "0",
                                    "aria-haspopup": "menu",
                                    "aria-expanded": "false",
                                    "id": "base-ui-{ui_id}-snooze",
                                    "data-slot": "tooltip-trigger",
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
                                        "class": "lucide lucide-clock size-3",
                                        "aria-hidden": "true",
                                        path {
                                            "d": "M12 6v6l4 2",
                                        }
                                        circle {
                                            "cx": "12",
                                            "cy": "12",
                                            "r": "10",
                                        }
                                    }
                                }
                                button {
                                    "type": "button",
                                    "aria-label": "Settle thread",
                                    "class": "-mr-1 inline-flex cursor-pointer items-center gap-1 rounded-md bg-transparent px-1.5 text-xs text-muted-foreground hover:text-foreground",
                                    "id": "base-ui-{ui_id}-settle",
                                    "data-base-ui-tooltip-trigger": "",
                                    "data-slot": "tooltip-trigger",
                                    svg {
                                        "width": "24",
                                        "height": "24",
                                        "viewBox": "0 0 24 24",
                                        "fill": "none",
                                        "stroke": "currentColor",
                                        "stroke-width": "2",
                                        "stroke-linecap": "round",
                                        "stroke-linejoin": "round",
                                        "class": "lucide lucide-check size-3.5",
                                        "aria-hidden": "true",
                                        path {
                                            "d": "M20 6 9 17l-5-5",
                                        }
                                    }
                                    "Settle"
                                }
                            }
                        }
                    }
                    div {
                        "class": "mt-1 flex min-w-0 items-center gap-1.5",
                        span {
                            "class": if active { "min-w-0 flex-1 text-start text-sm transition-opacity motion-reduce:transition-none font-medium truncate text-foreground" } else { "min-w-0 flex-1 text-start text-sm transition-opacity motion-reduce:transition-none font-normal truncate text-foreground/75 group-hover/sidebar-row:text-foreground group-focus-visible/sidebar-row:text-foreground" },
                            "dir": "ltr",
                            "{title}"
                        }
                    }
                    div {
                        "class": "mt-0.5 flex min-w-0 items-center gap-1.5 text-secondary-label text-xs",
                        span {
                            "class": "flex min-w-0 flex-1",
                            span {
                                "class": "inline-flex min-w-0 max-w-full overflow-hidden whitespace-nowrap",
                                span { "class": "min-w-0 truncate", "{branch}" }
                            }
                        }
                        span {
                            "aria-hidden": "true",
                            "class": "pointer-events-none ml-auto inline-flex shrink-0 items-center gap-1",
                            span {
                                "class": "inline-flex shrink-0 items-center",
                                span {
                                    "class": "relative isolate z-30 inline-flex shrink-0 items-center justify-center overflow-visible",
                                    svg {
                                        "aria-hidden": "true",
                                        "preserveAspectRatio": "xMidYMid",
                                        "viewBox": "100 100 411 411",
                                        "class": "fill-current shrink-0 size-3.5 opacity-60",
                                        path {
                                            "fill-rule": "evenodd",
                                            "clip-rule": "evenodd",
                                            "d": "M252.794 108.802C289.191 99.0484 326.265 110.305 351.148 135.135C385.113 126.072 422.85 134.862 449.492 161.505C476.136 188.149 484.925 225.888 475.862 259.85V259.854C500.696 284.735 511.95 321.81 502.198 358.207C492.447 394.602 464.161 421.084 430.215 430.217C421.083 464.162 394.603 492.448 358.206 502.199C321.812 511.951 284.734 500.693 259.852 475.864C225.887 484.927 188.15 476.137 161.507 449.495C134.864 422.851 126.073 385.111 135.136 351.149C110.304 326.266 99.0496 289.192 108.801 252.795C118.552 216.4 146.84 189.918 180.784 180.785C189.917 146.841 216.396 118.553 252.794 108.802ZM374.292 407.145C374.292 411.271 372.092 415.086 368.517 417.148L283.723 466.102C302.487 480.585 327.555 486.459 352.217 479.852C386.997 470.532 410.068 439.312 410.555 405.006V317.717C410.555 315.08 409.125 312.621 406.843 311.303L374.292 292.509V407.145ZM251.868 415.897C248.296 417.959 243.893 417.959 240.317 415.897L155.526 366.942C152.366 390.436 159.811 415.08 177.866 433.136H177.863C203.325 458.594 241.896 462.962 271.85 446.232L347.449 402.586C349.735 401.268 351.148 398.8 351.148 396.163V358.579L251.868 415.897ZM368.602 220.628C366.319 219.309 363.474 219.318 361.191 220.637L328.641 239.431L427.921 296.749C431.496 298.811 433.697 302.627 433.697 306.752V404.661C455.622 395.654 473.244 376.881 479.851 352.218C489.169 317.442 473.668 281.85 444.201 264.274L368.602 220.628ZM177.303 206.34C155.377 215.348 137.756 234.122 131.148 258.783C121.832 293.561 137.331 329.153 166.799 346.727L242.398 390.373C244.68 391.692 247.525 391.684 249.807 390.366L282.357 371.572L183.078 314.253C179.504 312.189 177.303 308.375 177.303 304.251V206.34ZM259.849 279.145V331.858L305.5 358.213L351.15 331.858V279.145L305.5 252.789L259.849 279.145ZM327.276 144.9C308.512 130.418 283.445 124.543 258.782 131.15C224.002 140.471 200.931 171.691 200.445 205.995V293.286C200.445 295.923 201.875 298.381 204.158 299.7L236.707 318.493V203.856C236.707 199.731 238.909 195.916 242.483 193.853L327.276 144.9ZM433.137 177.867C407.675 152.407 369.103 148.038 339.149 164.769L263.55 208.415C261.265 209.734 259.852 212.202 259.852 214.838V252.423L359.132 195.105C362.703 193.041 367.108 193.041 370.682 195.105L455.473 244.06C458.635 220.567 451.189 195.922 433.135 177.867H433.137Z",
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
