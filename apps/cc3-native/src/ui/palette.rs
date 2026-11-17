//! Command palette (Ctrl+K): the golden dialog frame as RSX, rows from the golden row templates
//! (`assets/palette/rows.json`, made by tools/extract_palette.py), filtered by the query.
use dioxus::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Action {
    text: String,
    html: String,
}

#[derive(Deserialize)]
struct Index {
    actions: Vec<Action>,
    thread: String,
    group: String,
    settings: String,
}

fn index() -> &'static Index {
    static INDEX: std::sync::OnceLock<Index> = std::sync::OnceLock::new();
    INDEX.get_or_init(|| serde_json::from_str(include_str!("../../assets/palette/rows.json")).expect("palette rows"))
}

/// A thread the palette can open.
#[derive(Clone, PartialEq)]
pub struct PaletteThread {
    pub id: String,
    pub title: String,
    pub project: String,
    pub monogram: String,
    pub branch: String,
    pub age: String,
}

#[derive(Clone, PartialEq)]
pub struct PaletteRow {
    /// Action to run when picked (`page:...`, `thread:<id>`); empty for headers and inert rows.
    pub key: String,
    pub html: String,
    /// Selectable (not a group label).
    pub selectable: bool,
}

/// Result of an action row: what the app does when it is picked.
fn action_key(i: usize) -> &'static str {
    match i {
        0 | 1 => "page:new-chat",
        7 => "dialog:add-project",
        11 => "page:route-pull-requests",
        13 => "page:route-usage",
        16 => "page:settings-general",
        17 => "page:settings-projects",
        _ => "close",
    }
}

fn highlight(html: &str) -> String {
    let Some(end) = html.find('>') else { return html.to_string() };
    let (tag, rest) = html.split_at(end);
    let tag = tag.replacen("data-slot=\"command-item\"", "data-slot=\"command-item\" data-highlighted=\"\" data-active=\"true\"", 1);
    let tag = match tag.find("class=\"") {
        Some(c) => match tag[c + 7..].find('"') {
            Some(e) => format!("{} bg-accent! text-accent-foreground!{}", &tag[..c + 7 + e], &tag[c + 7 + e..]),
            None => tag,
        },
        None => tag,
    };
    format!("{tag}{rest}")
}

fn thread_html(t: &PaletteThread) -> String {
    index()
        .thread
        .replace("Thread thread-other", &t.title)
        .replace(">render<", &format!(">{}<", t.project))
        .replace(">RR<", &format!(">{}<", t.monogram))
        .replace(">main<", &format!(">{}<", t.branch))
        .replace("13h ago", &format!("{} ago", t.age))
}

/// Rows for a query: actions, then threads ("Recent Threads" without a query).
pub fn rows_for(query: &str, threads: &[PaletteThread], active: usize) -> Vec<PaletteRow> {
    let q = query.trim().to_lowercase();
    let idx = index();
    let label = |name: &str| idx.group.replace("Actions", name);
    let mut rows: Vec<PaletteRow> = Vec::new();
    let actions: Vec<(usize, &Action)> = idx.actions.iter().enumerate().filter(|(_, a)| q.is_empty() || a.text.to_lowercase().contains(&q)).collect();
    if !actions.is_empty() {
        rows.push(PaletteRow { key: String::new(), html: label("Actions"), selectable: false });
        rows.extend(actions.iter().map(|(i, a)| PaletteRow { key: action_key(*i).into(), html: a.html.clone(), selectable: true }));
    }
    let shown: Vec<&PaletteThread> = threads.iter().filter(|t| q.is_empty() || t.title.to_lowercase().contains(&q)).take(if q.is_empty() { 8 } else { 20 }).collect();
    if !shown.is_empty() {
        rows.push(PaletteRow { key: String::new(), html: label(if q.is_empty() { "Recent Threads" } else { "Threads" }), selectable: false });
        rows.extend(shown.iter().map(|t| PaletteRow { key: format!("thread:{}", t.id), html: thread_html(t), selectable: true }));
    }
    // Highlight the `active`-th selectable row.
    let mut n = 0;
    for row in rows.iter_mut().filter(|r| r.selectable) {
        if n == active {
            row.html = highlight(&row.html);
        }
        n += 1;
    }
    rows
}

#[component]
pub fn CommandPalette(query: String, rows: Vec<PaletteRow>, oninput: EventHandler<String>, onkey: EventHandler<String>, onpick: EventHandler<String>, onclose: EventHandler<()>) -> Element {
    rsx! {
        div {
            "id": "_r_6l_",
            "data-base-ui-portal": "",
            div { "role": "presentation", "data-base-ui-inert": "", "aria-hidden": "true", "style": "position: fixed; inset: 0px; user-select: none;" }
            div {
                "data-open": "",
                "role": "presentation",
                "data-slot": "command-dialog-backdrop",
                "class": "dialog-backdrop fixed inset-0 z-50 transition-all duration-200 data-ending-style:opacity-0 data-starting-style:opacity-0 data-ending-style:pointer-events-none",
                "aria-hidden": "true",
                "data-base-ui-inert": "",
                "style": "user-select: none;",
                onclick: move |_| onclose.call(()),
            }
            div {
                "data-open": "",
                "role": "presentation",
                "data-slot": "command-dialog-viewport",
                "class": "pointer-events-none fixed inset-0 z-50 flex flex-col items-center px-4 [-webkit-app-region:no-drag] pt-[max(--spacing(4),4vh,calc(var(--command-titlebar-height)+--spacing(2)))] pb-[max(--spacing(4),4vh)] sm:pt-[max(10vh,calc(var(--command-titlebar-height)+--spacing(2)))] sm:pb-[10vh]",
                "style": "--command-titlebar-height: 0px;",
                span { "data-type": "inside", "aria-hidden": "true", "tabindex": "0", "data-base-ui-focus-guard": "", "data-base-ui-inert": "", "style": "clip-path: inset(50%); overflow: hidden; white-space: nowrap; border: 0px; padding: 0px; width: 1px; height: 1px; margin: 0px; position: fixed; top: 0px; left: 0px;" }
                    div {
                        "data-open": "",
                        "id": "_r_4_",
                        "role": "dialog",
                        "tabindex": "-1",
                        "data-base-ui-focusable": "",
                        "data-slot": "command-dialog-popup",
                        "aria-label": "Command palette",
                        "data-command-palette": "true",
                        "data-palette-mode": "command",
                        "data-testid": "command-palette",
                        "class": "dialog-glass -translate-y-[calc(1.25rem*var(--nested-dialogs))] relative flex min-h-0 w-full min-w-0 scale-[calc(1-0.1*var(--nested-dialogs))] flex-col opacity-[calc(1-0.1*var(--nested-dialogs))] outline-none transition-[scale,opacity,translate] duration-200 ease-in-out will-change-transform data-nested:data-ending-style:translate-y-8 data-nested:data-starting-style:translate-y-8 data-nested-dialog-open:origin-top data-ending-style:scale-98 data-starting-style:scale-98 data-ending-style:opacity-0 data-starting-style:opacity-0 [-webkit-app-region:no-drag] rounded-2xl border pointer-events-auto max-h-105 max-w-xl text-foreground data-ending-style:pointer-events-none overflow-hidden",
                        "style": "--nested-dialogs: 0;",
                        div {
                            "class": "contents",
                            div {
                                "class": "relative",
                                div {
                                    "class": "px-[var(--command-shell-inset)] py-1.5 [&_[data-slot=autocomplete-start-addon]]:ps-[calc(var(--command-shell-inset)+0.0625rem)]",
                                    div {
                                        "class": "relative not-has-[>*.w-full]:w-fit w-full text-foreground has-disabled:opacity-64",
                                        div {
                                            "aria-hidden": "true",
                                            "class": "[&_svg]:-mx-0.5 pointer-events-none has-[button]:pointer-events-auto absolute inset-y-0 start-px z-10 flex items-center ps-[calc(--spacing(3)-1px)] opacity-80 has-[+[data-size=sm]]:ps-[calc(--spacing(2.5)-1px)] [&_svg:not([class*='size-'])]:size-4.5 sm:[&_svg:not([class*='size-'])]:size-4",
                                            "data-slot": "autocomplete-start-addon",
                                            svg {
                                                "width": "24",
                                                "height": "24",
                                                "viewBox": "0 0 24 24",
                                                "fill": "none",
                                                "stroke": "currentColor",
                                                "stroke-width": "2",
                                                "stroke-linecap": "round",
                                                "stroke-linejoin": "round",
                                                "class": "lucide lucide-search translate-x-0.5 text-icon-muted",
                                                "aria-hidden": "true",
                                                path {
                                                    "d": "m21 21-4.34-4.34",
                                                }
                                                circle {
                                                    "cx": "11",
                                                    "cy": "11",
                                                    "r": "8",
                                                }
                                            }
                                        }
                                        span {
                                            "class": "relative inline-flex w-full rounded-lg border border-input bg-background not-dark:bg-clip-padding text-base text-foreground ring-ring/24 transition-shadow before:pointer-events-none before:absolute before:inset-0 before:rounded-[calc(var(--radius-lg)-1px)] not-has-disabled:not-has-focus-visible:not-has-aria-invalid:before:shadow-[0_1px_--theme(--color-black/4%)] has-focus-visible:has-aria-invalid:border-destructive/64 has-focus-visible:has-aria-invalid:ring-destructive/16 has-aria-invalid:border-destructive/36 has-focus-visible:border-ring has-autofill:bg-foreground/4 has-disabled:opacity-64 has-[:disabled,:focus-visible,[aria-invalid]]:shadow-none has-focus-visible:ring-inset sm:text-sm dark:bg-input/32 dark:has-autofill:bg-foreground/8 dark:has-aria-invalid:ring-destructive/24 dark:not-has-disabled:not-has-focus-visible:not-has-aria-invalid:before:shadow-[0_-1px_--theme(--color-white/6%)] data-[size=sm]:*:data-[slot=autocomplete-input]:ps-[calc(--spacing(7.5)-1px)] *:data-[slot=autocomplete-input]:ps-[calc(--spacing(8.5)-1px)] sm:data-[size=sm]:*:data-[slot=autocomplete-input]:ps-[calc(--spacing(7)-1px)] sm:*:data-[slot=autocomplete-input]:ps-[calc(--spacing(8)-1px)] has-[+[data-slot=autocomplete-trigger],+[data-slot=autocomplete-clear]]:*:data-[slot=autocomplete-input]:pe-7 border-transparent! bg-transparent! shadow-none before:hidden has-focus-visible:ring-0 placeholder:text-placeholder *:data-[slot=autocomplete-input]:ps-9! sm:*:data-[slot=autocomplete-input]:ps-[calc(var(--command-shell-inset)+1.5rem)]!",
                                            "data-size": "lg",
                                            "data-slot": "input-control",
                                            input {
                                                "class": "w-full min-w-0 rounded-[inherit] px-[calc(--spacing(3)-1px)] outline-none placeholder:text-placeholder [transition:background-color_5000000s_ease-in-out_0s] h-9.5 leading-9.5 sm:h-8.5 sm:leading-8.5",
                                                "data-slot": "autocomplete-input",
                                                "data-popup-open": "",
                                                "data-pressed": "",
                                                "autocomplete": "off",
                                                "spellcheck": "false",
                                                "autocorrect": "off",
                                                "autocapitalize": "none",
                                                "role": "combobox",
                                                "aria-expanded": "true",
                                                "aria-haspopup": "listbox",
                                                "aria-autocomplete": "none",
                                                "id": "base-ui-_r_6p_",
                                                "placeholder": "Search commands, projects, and threads...",
                                                "value": "{query}",
                                                onmounted: move |e| {
                                    spawn(async move {
                                        let _ = e.set_focus(true).await;
                                    });
                                },
                                oninput: move |e| oninput.call(e.value()),
                                                onkeydown: move |e| onkey.call(e.key().to_string()),
                                                "aria-controls": "_r_6o_",
                                            }
                                        }
                                    }
                                }
                            }
                            div {
                                "class": "relative overflow-hidden rounded-t-xl not-has-[+[data-slot=command-footer]]:rounded-b-2xl bg-transparent **:data-[slot=scroll-area-scrollbar]:mt-2 [touch-action:pan-y] flex max-h-[min(28rem,70vh)] min-h-0 flex-col",
                                div {
                                    "role": "listbox",
                                    "tabindex": "-1",
                                    "id": "_r_6o_",
                                    "data-slot": "command-list",
                                    "class": "size-full min-w-0 flex max-h-[inherit] min-h-0 flex-col",
                                    "aria-activedescendant": "base-ui-_r_6n_-0",
                                    div {
                                        "class": "min-h-0 scroll-py-6 overflow-x-hidden overscroll-y-contain [&::-webkit-scrollbar-track]:my-2 virtualized-scroll-fade [--fade-size:1.5rem] [--virtualized-scroll-fade-mask:linear-gradient(to_bottom,black_calc(100%-var(--fade-size)),transparent)]",
                                        "style": "overflow: auto; padding-top: 8px; padding-bottom: 8px; overflow-anchor: none;",
                                        div {
                                            "class": "legend-list-content-container px-2",
                                            "style": "display: block; min-height: 100%; box-sizing: border-box; overflow-anchor: none;",
                                            div {
                                                "style": "display: block; position: relative;",
                                                for (i, row) in rows.iter().enumerate() {
                                                    div {
                                                        key: "{i}-{row.html.len()}-{row.key}",
                                                        onclick: { let key = row.key.clone(); move |_| if !key.is_empty() { onpick.call(key.clone()) } },
                                                        dangerous_inner_html: row.html.clone(),
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            div {
                                "class": "relative flex items-center justify-between gap-2 rounded-b-[calc(var(--radius-2xl)-1px)] border-t border-foreground/[0.07] bg-foreground/[0.025] px-[var(--command-content-inset)] py-2.5 font-medium text-sm text-muted-foreground [&_[data-slot=kbd-group]]:font-sans [&_[data-slot=kbd]]:bg-foreground/[0.08] [&_[data-slot=kbd]]:text-foreground [&_[data-slot=kbd]]:ring-0 max-sm:flex-col max-sm:items-start",
                                "data-slot": "command-footer",
                                div {
                                    "class": "flex min-w-0 flex-wrap items-center gap-x-3 gap-y-1.5",
                                    kbd {
                                        "class": "inline-flex items-center gap-1",
                                        "data-slot": "kbd-group",
                                        kbd {
                                            "class": "pointer-events-none inline-flex h-5 min-w-5 select-none items-center justify-center gap-1 rounded px-1 font-medium font-sans text-muted-foreground text-xs [&_svg:not([class*='size-'])]:size-3 bg-foreground/[0.07]",
                                            "data-slot": "kbd",
                                            svg {
                                                "width": "24",
                                                "height": "24",
                                                "viewBox": "0 0 24 24",
                                                "fill": "none",
                                                "stroke": "currentColor",
                                                "stroke-width": "2",
                                                "stroke-linecap": "round",
                                                "stroke-linejoin": "round",
                                                "class": "lucide lucide-arrow-up",
                                                "aria-hidden": "true",
                                                path {
                                                    "d": "m5 12 7-7 7 7",
                                                }
                                                path {
                                                    "d": "M12 19V5",
                                                }
                                            }
                                        }
                                        kbd {
                                            "class": "pointer-events-none inline-flex h-5 min-w-5 select-none items-center justify-center gap-1 rounded px-1 font-medium font-sans text-muted-foreground text-xs [&_svg:not([class*='size-'])]:size-3 bg-foreground/[0.07]",
                                            "data-slot": "kbd",
                                            svg {
                                                "width": "24",
                                                "height": "24",
                                                "viewBox": "0 0 24 24",
                                                "fill": "none",
                                                "stroke": "currentColor",
                                                "stroke-width": "2",
                                                "stroke-linecap": "round",
                                                "stroke-linejoin": "round",
                                                "class": "lucide lucide-arrow-down",
                                                "aria-hidden": "true",
                                                path {
                                                    "d": "M12 5v14",
                                                }
                                                path {
                                                    "d": "m19 12-7 7-7-7",
                                                }
                                            }
                                        }
                                        span { "Navigate" }
                                    }
                                    kbd {
                                        "class": "inline-flex items-center gap-1",
                                        "data-slot": "kbd-group",
                                        kbd {
                                            "class": "pointer-events-none inline-flex h-5 min-w-5 select-none items-center justify-center gap-1 rounded px-1 font-medium font-sans text-muted-foreground text-xs [&_svg:not([class*='size-'])]:size-3 bg-foreground/[0.07]",
                                            "data-slot": "kbd",
                                            "Enter"
                                        }
                                        span { "Select" }
                                    }
                                    kbd {
                                        "class": "inline-flex items-center gap-1",
                                        "data-slot": "kbd-group",
                                        kbd {
                                            "class": "pointer-events-none inline-flex h-5 min-w-5 select-none items-center justify-center gap-1 rounded px-1 font-medium font-sans text-muted-foreground text-xs [&_svg:not([class*='size-'])]:size-3 bg-foreground/[0.07]",
                                            "data-slot": "kbd",
                                            "Esc"
                                        }
                                        span { "Close" }
                                    }
                                }
                            }
                            input {
                                "id": "base-ui-_r_6n_-hidden-input",
                                "tabindex": "-1",
                                "aria-hidden": "true",
                                "value": "",
                                "style": "clip-path: inset(50%); overflow: hidden; white-space: nowrap; border: 0px; padding: 0px; width: 1px; height: 1px; margin: 0px; position: fixed; top: 0px; left: 0px;",
                            }
                        }
                    }

                span { "data-type": "inside", "aria-hidden": "true", "tabindex": "0", "data-base-ui-focus-guard": "", "data-base-ui-inert": "", "style": "clip-path: inset(50%); overflow: hidden; white-space: nowrap; border: 0px; padding: 0px; width: 1px; height: 1px; margin: 0px; position: fixed; top: 0px; left: 0px;" }
            }
        }
    }
}
