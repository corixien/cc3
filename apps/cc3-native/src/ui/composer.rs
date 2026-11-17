//! The composer (message box) in its two states. Static markup from the Chromium golden snapshots until the editor is ported.
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ComposerMode {
    /// The agent waits on a question: the banner sits on top of the composer.
    Question,
    Normal,
}

/// One selectable answer of a pending question.
#[derive(Clone, PartialEq)]
pub struct QuestionOption {
    pub label: String,
    pub description: String,
    pub value: String,
}

#[component]
pub fn ComposerQuestion(#[props(default)] onaction: EventHandler<String>, #[props(default)] header: String, #[props(default)] question: String, #[props(default)] options: Vec<QuestionOption>, #[props(default)] onanswer: Option<EventHandler<String>>) -> Element {
    let header = if header.is_empty() { "Choice".to_string() } else { header };
    let question = if question.is_empty() { "Which approach should I take?".to_string() } else { question };
    let options = if options.is_empty() { vec![QuestionOption { label: "Continue".into(), description: "Resume work".into(), value: "continue".into() }] } else { options };
    let mut selected = use_signal(|| None::<usize>);
    rsx! {
            div {
                "data-slot": "composer-shell",
                "class": "@container/composer-surface group/composer-surface relative isolate mx-auto w-full max-w-(--chat-content-max-width) [--chat-composer-drawer-inset:1.375rem] [--chat-composer-glass-surface:var(--card)] [--chat-composer-outline:rgb(0_0_0/8%)] dark:[--chat-composer-glass-surface:var(--surface-raised)] dark:[--chat-composer-highlight:rgb(255_255_255/3%)] dark:[--chat-composer-outline:color-mix(in_srgb,var(--color-white)_5%,transparent)] [html[data-theme-id]_&]:[--chat-composer-glass-surface:var(--app-theme-surface-raised)] [html[data-theme-id]_&]:[--chat-composer-outline:var(--app-theme-toolbar-border)] dark:[html[data-theme-id]:not([data-theme-id=t3-chat])_&]:[--chat-composer-highlight:color-mix(in_srgb,var(--app-theme-input)_12%,transparent)] dark:[html[data-theme-id]:not([data-theme-id=t3-chat])_&]:[--chat-composer-outline:color-mix(in_srgb,var(--app-theme-input)_30%,var(--background))] dark:[html[data-theme-id=t3-chat]_&]:[--chat-composer-highlight:color-mix(in_srgb,#432d48_12%,transparent)] dark:[html[data-theme-id=t3-chat]_&]:[--chat-composer-outline:#241e28] before:pointer-events-none before:absolute before:inset-0 before:z-0 before:rounded-2xl before:bg-(--chat-composer-glass-surface)/(--glass-opacity) before:backdrop-blur-(--glass-blur) before:backdrop-saturate-(--glass-saturation) not-supports-[((backdrop-filter:blur(1px))_or_(-webkit-backdrop-filter:blur(1px)))]:before:bg-(--chat-composer-glass-surface) has-data-[composer-banner-surface=attached]:before:hidden data-context-row:before:hidden",
                div {
                    "data-slot": "composer-host",
                    "class": "relative z-10 w-full rounded-2xl shadow-composer after:z-1 dark:shadow-none group-data-context-row/composer-surface:before:pointer-events-none group-data-context-row/composer-surface:before:absolute group-data-context-row/composer-surface:before:inset-0 group-data-context-row/composer-surface:before:-z-1 group-data-context-row/composer-surface:before:rounded-2xl group-data-context-row/composer-surface:before:bg-(--chat-composer-glass-surface)/(--glass-opacity) group-data-context-row/composer-surface:before:backdrop-blur-(--glass-blur) group-data-context-row/composer-surface:before:backdrop-saturate-(--glass-saturation) after:pointer-events-none after:absolute after:inset-0 after:rounded-[inherit] after:border after:border-(--chat-composer-outline) dark:after:inset-shadow-2xs dark:after:inset-shadow-(color:--chat-composer-highlight) group-has-data-[composer-banner-surface=attached]/composer-surface:shadow-none group-has-data-[composer-banner-surface=attached]/composer-surface:before:hidden group-has-data-[composer-banner-surface=attached]/composer-surface:after:hidden",
                    "aria-busy": "false",
                    div {
                        "class": "relative z-10",
                        form {
                            "class": "mx-auto w-full min-w-0 max-w-(--chat-content-max-width)",
                            "data-chat-composer-form": "true",
                            "data-thread-context-drop": "true",
                            div {
                                "data-slot": "composer-banner-attachment",
                                "class": "mx-auto mb-2 w-[calc(100%-2*var(--chat-composer-drawer-inset))] flex items-end gap-1 not-has-data-[composer-banner-surface=attached]:hidden",
                                div {
                                    "class": "flex min-w-0 flex-1 flex-col empty:hidden [&>[data-slot=composer-banner-attachment]]:w-full [&>[data-slot=composer-banner-attachment]:last-child]:mb-0",
                                    div {
                                        "data-slot": "composer-banner-attachment",
                                        "class": "mx-auto mb-2 w-[calc(100%-2*var(--chat-composer-drawer-inset))]",
                                        div {
                                            "data-composer-banner-surface": "attached",
                                            "data-variant": "info",
                                            "class": "[--chat-composer-attached-surface:var(--chat-composer-glass-surface,var(--card))] dark:[--chat-composer-attached-surface:var(--chat-composer-glass-surface,var(--surface-raised))] [html[data-theme-id]_&]:[--chat-composer-attached-surface:var(--app-theme-surface-raised)] relative isolate border-0 bg-transparent shadow-none [--chat-composer-attached-tint:transparent] [--chat-composer-attached-outline:var(--chat-composer-outline,color-mix(in_srgb,var(--contrast-foreground)_8%,transparent))] dark:[--chat-composer-attached-outline:var(--chat-composer-outline,color-mix(in_srgb,var(--color-white)_5%,transparent))] [html[data-theme-id]_&]:[--chat-composer-attached-outline:var(--chat-composer-outline,var(--app-theme-toolbar-border))] dark:[html[data-theme-id]:not([data-theme-id=t3-chat])_&]:[--chat-composer-attached-outline:var(--chat-composer-outline,color-mix(in_srgb,var(--app-theme-input)_30%,var(--background)))] dark:[html[data-theme-id=t3-chat]_&]:[--chat-composer-attached-outline:#241e28] [--chat-composer-attachment-overlap:0px] before:rounded-2xl before:pointer-events-none before:absolute before:inset-0 before:-z-1 before:border before:border-(--chat-composer-attached-outline) before:bg-(--chat-composer-attached-surface)/(--glass-opacity) before:bg-linear-to-b before:from-(--chat-composer-attached-tint) before:to-(--chat-composer-attached-tint) before:backdrop-blur-(--glass-blur) before:backdrop-saturate-(--glass-saturation) before:shadow-composer dark:before:shadow-composer-dark not-supports-[((backdrop-filter:blur(1px))_or_(-webkit-backdrop-filter:blur(1px)))]:before:bg-(--chat-composer-attached-surface) min-w-0 px-1 py-(--composer-banner-padding-block) after:block after:h-(--chat-composer-attachment-overlap) text-xs/4 [--composer-banner-corner-top:var(--radius-xl)] [--composer-banner-icon-column:--spacing(7)] [--composer-banner-padding-block:--spacing(1)] sm:[--composer-banner-icon-column:--spacing(6)] @container [&>[data-composer-banner-row]:first-child>[data-slot=composer-banner-actions]>[data-slot=button]:last-child]:rounded-se-(--composer-banner-corner-top) [&>[data-composer-banner-row]:first-child>[data-slot=composer-banner-actions]>:last-child>[data-slot=button]:last-of-type]:rounded-se-(--composer-banner-corner-top) [&>:first-child>[data-composer-banner-row]:first-child>[data-slot=composer-banner-actions]>[data-slot=button]:last-child]:rounded-se-(--composer-banner-corner-top) data-[composer-banner-surface]:[&>[data-composer-banner-row]:only-child>[data-slot=composer-banner-actions]>[data-slot=button]:last-child]:rounded-ee-xl data-[composer-banner-surface]:[&>[data-composer-banner-row]:only-child>[data-slot=composer-banner-actions]>:last-child>[data-slot=button]:last-of-type]:rounded-ee-xl",
                                            "data-slot": "composer-banner",
                                            "data-composer-banner-width": "fill",
                                            "data-chat-composer-top-drawer": "true",
                                            div {
                                                "data-open": "",
                                                "data-slot": "collapsible",
                                                "dir": "ltr",
                                                button {
                                                    onclick: move |e| { e.prevent_default(); onaction.call("question:dismiss".to_string()); },
                                                    "type": "button",
                                                    "class": "group/banner-row grid min-h-(--composer-banner-icon-column) w-full min-w-0 grid-cols-[var(--composer-banner-icon-column)_minmax(0,1fr)_auto] items-center gap-x-1 text-start not-has-[>[data-slot=composer-banner-actions]]:grid-cols-[var(--composer-banner-icon-column)_minmax(0,1fr)] [&:is(button)]:cursor-pointer [&:is(button)]:rounded-xl [&:is(button)]:hover:bg-foreground/5 [&:is(button)]:focus-visible:outline-2 [&:is(button)]:focus-visible:-outline-offset-2 [&:is(button)]:focus-visible:outline-ring cursor-pointer px-4 py-1.5",
                                                    "data-composer-banner-row": "true",
                                                    "data-composer-banner-layout": "inline",
                                                    "data-panel-open": "",
                                                    "tabindex": "0",
                                                    "aria-controls": "base-ui-_r_26_",
                                                    "aria-expanded": "true",
                                                    "data-slot": "collapsible-trigger",
                                                    "title": "Hide the question and its options",
                                                    "data-pending-user-input-toggle": "expanded",
                                                    span {
                                                        "data-slot": "composer-banner-content",
                                                        "class": "col-start-2 row-start-1 flex min-w-0 items-center gap-1 *:data-[slot=composer-banner-separator]:mx-0 @max-[560px]:group-data-[composer-banner-layout=approval]/banner-row:col-end-4 group-not-has-[>[data-slot=composer-banner-icon]]/banner-row:col-[1/3] group-not-has-[>[data-slot=composer-banner-icon]]/banner-row:ps-2 sm:group-not-has-[>[data-slot=composer-banner-icon]]/banner-row:ps-1.5 group-not-has-[>[data-slot=composer-banner-icon],>[data-slot=composer-banner-actions]]/banner-row:pe-2 sm:group-not-has-[>[data-slot=composer-banner-icon],>[data-slot=composer-banner-actions]]/banner-row:pe-1.5",
                                                        span {
                                                            "class": "min-w-0 truncate text-muted-foreground",
                                                            "dir": "ltr",
                                                            "{header}"
                                                        }
                                                    }
                                                    span {
                                                        "data-slot": "composer-banner-actions",
                                                        "class": "col-start-3 row-start-1 flex flex-wrap items-center justify-end group-data-[composer-banner-layout=approval]/banner-row:self-center group-data-[composer-banner-layout=approval]/banner-row:gap-1.5 @max-[560px]:group-data-[composer-banner-layout=approval]/banner-row:col-start-2 @max-[560px]:group-data-[composer-banner-layout=approval]/banner-row:col-end-4 @max-[560px]:group-data-[composer-banner-layout=approval]/banner-row:row-start-2 group-data-[composer-banner-layout=wrap-actions]/banner-row:ms-auto gap-0",
                                                        span {
                                                            "aria-hidden": "true",
                                                            "class": "[&_svg]:-mx-0.5 relative inline-flex shrink-0 cursor-pointer items-center justify-center gap-2 whitespace-nowrap rounded-[var(--control-radius)] border font-medium text-base outline-none transition-[box-shadow,scale] [&:not([aria-haspopup]):not(:disabled):not([aria-disabled=true])]:will-change-[scale] [&:active:not([aria-haspopup])]:scale-[0.97] before:pointer-events-none before:absolute before:inset-0 before:rounded-[calc(var(--control-radius)-1px)] pointer-coarse:after:absolute pointer-coarse:after:size-full pointer-coarse:after:min-h-11 pointer-coarse:after:min-w-11 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-64 aria-disabled:cursor-not-allowed aria-disabled:opacity-64 sm:text-sm [&_svg:not([class*='text-'])]:text-[var(--control-icon-color)] [&_svg:not([class*='size-'])]:size-4.5 sm:[&_svg:not([class*='size-'])]:size-4 [&_svg]:pointer-events-none [&_svg]:shrink-0 size-7 sm:size-6 not-in-data-[slot=input-group]:[&_svg:not([class*='size-'])]:size-4 sm:not-in-data-[slot=input-group]:[&_svg:not([class*='size-'])]:size-3.5 [--control-icon-color:var(--contrast-muted-foreground)] border-transparent text-foreground data-pressed:bg-accent [:hover,[data-pressed]]:bg-accent pointer-events-none",
                                                            "data-slot": "button",
                                                            "tabindex": "-1",
                                                            svg {
                                                                "width": "24",
                                                                "height": "24",
                                                                "viewBox": "0 0 24 24",
                                                                "fill": "none",
                                                                "stroke": "currentColor",
                                                                "stroke-width": "2",
                                                                "stroke-linecap": "round",
                                                                "stroke-linejoin": "round",
                                                                "class": "lucide lucide-chevron-down size-3.5",
                                                                "aria-hidden": "true",
                                                                path {
                                                                    "d": "m6 9 6 6 6-6",
                                                                }
                                                            }
                                                        }
                                                        span {
                                                            "role": "button",
                                                            "tabindex": "0",
                                                            "class": "[&_svg]:-mx-0.5 relative inline-flex shrink-0 cursor-pointer items-center justify-center gap-2 whitespace-nowrap rounded-[var(--control-radius)] border font-medium text-base outline-none transition-[box-shadow,scale] [&:not([aria-haspopup]):not(:disabled):not([aria-disabled=true])]:will-change-[scale] [&:active:not([aria-haspopup])]:scale-[0.97] before:pointer-events-none before:absolute before:inset-0 before:rounded-[calc(var(--control-radius)-1px)] pointer-coarse:after:absolute pointer-coarse:after:size-full pointer-coarse:after:min-h-11 pointer-coarse:after:min-w-11 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-64 aria-disabled:cursor-not-allowed aria-disabled:opacity-64 sm:text-sm [&_svg:not([class*='text-'])]:text-[var(--control-icon-color)] [&_svg:not([class*='size-'])]:size-4.5 sm:[&_svg:not([class*='size-'])]:size-4 [&_svg]:pointer-events-none [&_svg]:shrink-0 size-7 sm:size-6 not-in-data-[slot=input-group]:[&_svg:not([class*='size-'])]:size-4 sm:not-in-data-[slot=input-group]:[&_svg:not([class*='size-'])]:size-3.5 [--control-icon-color:var(--contrast-muted-foreground)] border-transparent text-foreground data-pressed:bg-accent [:hover,[data-pressed]]:bg-accent",
                                                            "data-slot": "button",
                                                            "aria-label": "Dismiss question without answering",
                                                            "title": "Dismiss question without answering",
                                                            "data-pending-user-input-dismiss": "true",
                                                            svg {
                                                                "width": "24",
                                                                "height": "24",
                                                                "viewBox": "0 0 24 24",
                                                                "fill": "none",
                                                                "stroke": "currentColor",
                                                                "stroke-width": "2",
                                                                "stroke-linecap": "round",
                                                                "stroke-linejoin": "round",
                                                                "class": "lucide lucide-x size-3.5",
                                                                "aria-hidden": "true",
                                                                path {
                                                                    "d": "M18 6 6 18",
                                                                }
                                                                path {
                                                                    "d": "m6 6 12 12",
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                div {
                                                    "data-open": "",
                                                    "id": "base-ui-_r_26_",
                                                    "data-slot": "collapsible-panel",
                                                    "class": "overflow-hidden h-(--collapsible-panel-height) transition-[height] duration-200 motion-reduce:transition-none data-ending-style:h-0 data-starting-style:h-0 data-open:data-ending-style:[height:var(--collapsible-panel-height)]",
                                                    "style": "--collapsible-panel-height: auto; --collapsible-panel-width: auto; animation-name: none;",
                                                    div {
                                                        "role": "presentation",
                                                        "class": "relative size-full min-h-0 overflow-hidden rounded-none h-auto max-h-[min(24rem,40dvh)]",
                                                        "style": "position: relative; --scroll-area-corner-height: 0px; --scroll-area-corner-width: 0px;",
                                                        div {
                                                            "data-id": "base-ui-_r_27_-viewport",
                                                            "tabindex": "-1",
                                                            "class": "h-full max-h-[inherit] overflow-auto overscroll-contain rounded-[inherit] outline-none transition-shadow focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background data-has-overflow-x:overscroll-x-contain mask-t-from-[calc(100%-min(var(--fade-size),var(--scroll-area-overflow-y-start)))] mask-b-from-[calc(100%-min(var(--fade-size),var(--scroll-area-overflow-y-end)))] mask-l-from-[calc(100%-min(var(--fade-size),var(--scroll-area-overflow-x-start)))] mask-r-from-[calc(100%-min(var(--fade-size),var(--scroll-area-overflow-x-end)))] focus-visible:mask-none [--fade-size:1.5rem] scroll-p-[var(--fade-size)] base-ui-disable-scrollbar",
                                                            "data-slot": "scroll-area-viewport",
                                                            "style": "overflow: scroll; --scroll-area-overflow-x-start: 0px; --scroll-area-overflow-x-end: 0px; --scroll-area-overflow-y-start: 0px; --scroll-area-overflow-y-end: 0px;",
                                                            div {
                                                                "class": "[[data-has-overflow-y]>&]:pe-2",
                                                                div {
                                                                    "class": "min-w-0 ps-6 pe-4 pb-1 wrap-anywhere sm:ps-5.5",
                                                                    div {
                                                                        "class": "chat-markdown w-full min-w-0 [overflow-wrap:anywhere] [word-break:break-word] text-sm leading-relaxed pt-0.5 text-foreground/85 text-left",
                                                                        p { "dir": "ltr", "{question}" }
                                                                    }
                                                                    div {
                                                                        "class": "-ms-2 mt-1.5 sm:-ms-1.5",
for (i, opt) in options.iter().enumerate() {
                                                                        div {
                                                                            button {
                                                                                "type": "button",
                                                                                "aria-pressed": "{selected() == Some(i)}",
                                                                                onclick: move |e| { e.prevent_default(); selected.set(Some(i)); },
                                                                                "class": format!("{} {} cursor-pointer", "text-left group flex w-full items-start gap-2.5 rounded-md bg-transparent py-1.5 ps-2 sm:ps-1.5 outline-none transition-colors duration-150 focus-visible:ring-1 focus-visible:ring-inset focus-visible:ring-primary/25", if selected() == Some(i) { "text-foreground bg-muted/30" } else { "text-foreground/70 hover:bg-muted/30" }),
                                                                                span {
                                                                                    "class": "pointer-events-none flex h-5 shrink-0 items-center",
                                                                                    span {
                                                                                        "aria-hidden": "true",
                                                                                        "class": "relative inline-flex size-4.5 shrink-0 items-center justify-center rounded-full border border-foreground/55 bg-background not-dark:bg-clip-padding sm:size-4 dark:bg-input/32 shadow-xs/5",
                                                                                    }
                                                                                }
                                                                                div {
                                                                                    "class": "min-w-0 flex-1 flex flex-col gap-0.5",
                                                                                    span {
                                                                                        "class": "text-sm font-medium",
                                                                                        "dir": "ltr",
                                                                                        "{opt.label}"
                                                                                    }
                                                                                    span {
                                                                                        "class": "text-secondary-label text-2xs",
                                                                                        "dir": "ltr",
                                                                                        "{opt.description}"
                                                                                    }
                                                                                }
                                                                                span {
                                                                                    "class": "flex h-5 w-7 shrink-0 items-center justify-center sm:w-6",
                                                                                    kbd {
                                                                                        "class": "pointer-events-none inline-flex h-5 min-w-5 select-none items-center justify-center gap-1 rounded px-1 font-medium font-sans text-muted-foreground text-xs [&_svg:not([class*='size-'])]:size-3 bg-foreground/[0.07]",
                                                                                        "data-slot": "kbd",
                                                                                        "{i + 1}"
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
                            div {
                                "class": "relative",
                                div {
                                    "data-chat-composer-main-surface": "true",
                                    "class": "group relative z-10 rounded-2xl p-px after:pointer-events-none after:absolute after:inset-0 after:rounded-[inherit] after:border after:border-(--chat-composer-outline) dark:after:inset-shadow-2xs dark:after:inset-shadow-(color:--chat-composer-highlight) after:z-20 after:hidden group-has-data-[composer-banner-surface=attached]/composer-surface:after:block group-has-data-[composer-banner-surface=attached]/composer-surface:bg-(--chat-composer-glass-surface)/(--glass-opacity) group-has-data-[composer-banner-surface=attached]/composer-surface:backdrop-blur-(--glass-blur) group-has-data-[composer-banner-surface=attached]/composer-surface:backdrop-saturate-(--glass-saturation) group-has-data-[composer-banner-surface=attached]/composer-surface:shadow-composer dark:group-has-data-[composer-banner-surface=attached]/composer-surface:shadow-none not-supports-[((backdrop-filter:blur(1px))_or_(-webkit-backdrop-filter:blur(1px)))]:group-has-data-[composer-banner-surface=attached]/composer-surface:bg-(--chat-composer-glass-surface) group-has-data-[composer-banner-surface=attached]/composer-surface:**:data-[chat-composer-mobile-collapsed=true]:min-h-[calc(1rem+1px)]",
                                    "data-inline-resting-controls": "true",
                                    div {
                                        "data-chat-composer-surface": "true",
                                        "data-chat-composer-mobile-collapsed": "false",
                                        "class": "rounded-2xl transition-[background-color] duration-200 in-data-[thread-context-over]:bg-accent/45 in-data-[thread-context-over]:ring-1 in-data-[thread-context-over]:ring-inset in-data-[thread-context-over]:ring-primary/70",
                                        div {
                                            "data-chat-composer-body": "true",
                                            "class": "relative px-4 pb-2 pt-3.5 sm:pt-4",
                                            div {
                                                "role": "status",
                                                "aria-atomic": "true",
                                                "class": "sr-only",
                                            }
                                            div {
                                                "class": "relative",
                                                div {
                                                    "class": "relative flow-root font-(family-name:--font-composer,var(--font-sans)) text-(length:--font-size-prompt,var(--text-sm)) max-sm:pointer-coarse:text-(length:--font-size-prompt-touch)",
                                                    div {
                                                        div {
                                                            "contenteditable": "false",
                                                            "role": "textbox",
                                                            "data-testid": "composer-editor",
                                                            "data-composer-rich-text": "true",
                                                            "aria-multiline": "true",
                                                            "aria-label": "Message Thread thread-golden-question",
                                                            "aria-readonly": "true",
                                                            "aria-placeholder": "Choose an option above",
                                                            "translate": "no",
                                                            "class": "tiptap ProseMirror composer-tiptap -m-1 block max-h-52 min-h-[calc(2lh+--spacing(2))] overflow-y-auto p-1 whitespace-pre-wrap wrap-break-word bg-transparent leading-relaxed text-foreground focus:outline-none",
                                                            p {
                                                                br {
                                                                    "class": "ProseMirror-trailingBreak",
                                                                }
                                                            }
                                                        }
                                                    }
                                                    div { "class": "pointer-events-none absolute inset-0 leading-relaxed text-placeholder", "Choose an option above" }
                                                }
                                            }
                                        }
                                        div {
                                            "data-chat-composer-footer": "true",
                                            "class": "flex min-w-0 flex-nowrap items-center overflow-visible px-2 pb-2 pt-2",
                                            div {
                                                "data-chat-composer-actions": "right",
                                                "data-chat-composer-transition-actions": "true",
                                                "data-chat-composer-primary-actions-compact": "true",
                                                "class": "flex min-w-0 flex-1 flex-nowrap items-center justify-end gap-2",
                                                div {
                                                    "data-chat-composer-controls": "footer",
                                                    "data-chat-composer-footer-controls": "true",
                                                    "class": "relative flex min-w-0 flex-1 items-center justify-end gap-0.5",
                                                    button {
                                                        "type": "button",
                                                        "class": "relative inline-flex cursor-pointer items-center rounded-(--control-radius) border border-transparent text-base outline-none hover:bg-accent data-pressed:bg-accent focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-64 data-disabled:pointer-events-none data-disabled:opacity-64 pointer-coarse:after:absolute pointer-coarse:after:size-full pointer-coarse:after:min-h-11 pointer-coarse:after:min-w-11 [&:active:not([aria-haspopup])]:scale-[0.97] [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg]:-mx-0.5 [&_svg[data-composer-control-icon]]:mx-0 [&_:is([data-composer-control-label],[data-chat-provider-model-picker-label])]:[text-box:trim-both_cap_alphabetic] supports-[text-box:trim-both_cap_alphabetic]:[&_:is([data-composer-control-label],[data-chat-provider-model-picker-label])]:py-1.5 h-7 gap-1.5 px-2 font-medium text-foreground/80 [&_svg:not([class*='text-'])]:text-muted-foreground hover:text-foreground sm:text-sm [&_svg:not([class*='size-'])]:size-4.5 sm:[&_svg:not([class*='size-'])]:size-4 aria-pressed:bg-accent aria-pressed:text-accent-foreground aria-pressed:hover:bg-accent/80 shrink justify-between whitespace-nowrap group/model-trigger data-popup-open:bg-accent data-popup-open:text-foreground min-w-13",
                                                        "aria-label": "GPT-5.4",
                                                        "data-chat-provider-model-picker": "true",
                                                        "data-composer-shortcut": "composer.effort composer.mode",
                                                        "tabindex": "0",
                                                        "data-base-ui-click-trigger": "",
                                                        "id": "base-ui-_r_29_",
                                                        "aria-haspopup": "dialog",
                                                        "aria-expanded": "false",
                                                        "data-slot": "popover-trigger",
                                                        span {
                                                            "class": "flex min-w-0 flex-1 items-center gap-1.5",
                                                            span {
                                                                "data-chat-provider-model-picker-icon": "true",
                                                                "class": "contents",
                                                                span {
                                                                    "class": "relative isolate z-30 inline-flex shrink-0 items-center justify-center overflow-visible size-4",
                                                                    svg {
                                                                        "aria-hidden": "true",
                                                                        "preserveAspectRatio": "xMidYMid",
                                                                        "viewBox": "100 100 411 411",
                                                                        "class": "fill-current shrink-0 size-4",
                                                                        path {
                                                                            "fill-rule": "evenodd",
                                                                            "clip-rule": "evenodd",
                                                                            "d": "M252.794 108.802C289.191 99.0484 326.265 110.305 351.148 135.135C385.113 126.072 422.85 134.862 449.492 161.505C476.136 188.149 484.925 225.888 475.862 259.85V259.854C500.696 284.735 511.95 321.81 502.198 358.207C492.447 394.602 464.161 421.084 430.215 430.217C421.083 464.162 394.603 492.448 358.206 502.199C321.812 511.951 284.734 500.693 259.852 475.864C225.887 484.927 188.15 476.137 161.507 449.495C134.864 422.851 126.073 385.111 135.136 351.149C110.304 326.266 99.0496 289.192 108.801 252.795C118.552 216.4 146.84 189.918 180.784 180.785C189.917 146.841 216.396 118.553 252.794 108.802ZM374.292 407.145C374.292 411.271 372.092 415.086 368.517 417.148L283.723 466.102C302.487 480.585 327.555 486.459 352.217 479.852C386.997 470.532 410.068 439.312 410.555 405.006V317.717C410.555 315.08 409.125 312.621 406.843 311.303L374.292 292.509V407.145ZM251.868 415.897C248.296 417.959 243.893 417.959 240.317 415.897L155.526 366.942C152.366 390.436 159.811 415.08 177.866 433.136H177.863C203.325 458.594 241.896 462.962 271.85 446.232L347.449 402.586C349.735 401.268 351.148 398.8 351.148 396.163V358.579L251.868 415.897ZM368.602 220.628C366.319 219.309 363.474 219.318 361.191 220.637L328.641 239.431L427.921 296.749C431.496 298.811 433.697 302.627 433.697 306.752V404.661C455.622 395.654 473.244 376.881 479.851 352.218C489.169 317.442 473.668 281.85 444.201 264.274L368.602 220.628ZM177.303 206.34C155.377 215.348 137.756 234.122 131.148 258.783C121.832 293.561 137.331 329.153 166.799 346.727L242.398 390.373C244.68 391.692 247.525 391.684 249.807 390.366L282.357 371.572L183.078 314.253C179.504 312.189 177.303 308.375 177.303 304.251V206.34ZM259.849 279.145V331.858L305.5 358.213L351.15 331.858V279.145L305.5 252.789L259.849 279.145ZM327.276 144.9C308.512 130.418 283.445 124.543 258.782 131.15C224.002 140.471 200.931 171.691 200.445 205.995V293.286C200.445 295.923 201.875 298.381 204.158 299.7L236.707 318.493V203.856C236.707 199.731 238.909 195.916 242.483 193.853L327.276 144.9ZM433.137 177.867C407.675 152.407 369.103 148.038 339.149 164.769L263.55 208.415C261.265 209.734 259.852 212.202 259.852 214.838V252.423L359.132 195.105C362.703 193.041 367.108 193.041 370.682 195.105L455.473 244.06C458.635 220.567 451.189 195.922 433.135 177.867H433.137Z",
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            span {
                                                                "class": "min-w-0 flex-1 overflow-hidden truncate max-w-56",
                                                                "data-chat-provider-model-picker-label": "true",
                                                                "GPT-5.4"
                                                            }
                                                        }
                                                    }
                                                    div {
                                                        "data-resting-controls-overflow": "true",
                                                        "aria-hidden": "true",
                                                        "inert": "",
                                                        "class": "min-w-0 shrink-0 pointer-events-none invisible absolute",
                                                        button {
                                                            onclick: move |e| { e.prevent_default(); onaction.call("unavailable:Composer options (attachments, plan mode)".to_string()); },
                                                            "type": "button",
                                                            "class": "relative inline-flex cursor-pointer items-center justify-center whitespace-nowrap rounded-(--control-radius) border border-transparent text-base outline-none hover:bg-accent data-pressed:bg-accent focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-64 data-disabled:pointer-events-none data-disabled:opacity-64 pointer-coarse:after:absolute pointer-coarse:after:size-full pointer-coarse:after:min-h-11 pointer-coarse:after:min-w-11 [&:active:not([aria-haspopup])]:scale-[0.97] [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg]:-mx-0.5 [&_svg[data-composer-control-icon]]:mx-0 [&_:is([data-composer-control-label],[data-chat-provider-model-picker-label])]:[text-box:trim-both_cap_alphabetic] supports-[text-box:trim-both_cap_alphabetic]:[&_:is([data-composer-control-label],[data-chat-provider-model-picker-label])]:py-1.5 h-7 gap-1.5 px-2 font-medium text-foreground/80 [&_svg:not([class*='text-'])]:text-muted-foreground hover:text-foreground sm:text-sm [&_svg:not([class*='size-'])]:size-4.5 sm:[&_svg:not([class*='size-'])]:size-4 aria-pressed:bg-accent aria-pressed:text-accent-foreground aria-pressed:hover:bg-accent/80 shrink-0",
                                                            "aria-label": "More composer controls",
                                                            "tabindex": "0",
                                                            "aria-haspopup": "menu",
                                                            "aria-expanded": "false",
                                                            "id": "base-ui-_r_2d_",
                                                            "data-slot": "menu-trigger",
                                                            svg {
                                                                "width": "24",
                                                                "height": "24",
                                                                "viewBox": "0 0 24 24",
                                                                "fill": "none",
                                                                "stroke": "currentColor",
                                                                "stroke-width": "2",
                                                                "stroke-linecap": "round",
                                                                "stroke-linejoin": "round",
                                                                "class": "lucide lucide-ellipsis shrink-0 size-4",
                                                                "aria-hidden": "true",
                                                                "data-composer-control-icon": "true",
                                                                circle {
                                                                    "cx": "12",
                                                                    "cy": "12",
                                                                    "r": "1",
                                                                }
                                                                circle {
                                                                    "cx": "19",
                                                                    "cy": "12",
                                                                    "r": "1",
                                                                }
                                                                circle {
                                                                    "cx": "5",
                                                                    "cy": "12",
                                                                    "r": "1",
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                div {
                                                    "class": "flex items-center justify-end gap-1.5",
                                                    button {
                                                        onclick: move |e| { e.prevent_default(); onaction.call("unavailable:Voice input".to_string()); },
                                                        "type": "button",
                                                        "class": "[&_svg]:-mx-0.5 relative inline-flex cursor-pointer items-center justify-center gap-2 whitespace-nowrap rounded-[var(--control-radius)] border font-medium text-base outline-none transition-[box-shadow,scale] [&:not([aria-haspopup]):not(:disabled):not([aria-disabled=true])]:will-change-[scale] [&:active:not([aria-haspopup])]:scale-[0.97] before:pointer-events-none before:absolute before:inset-0 before:rounded-[calc(var(--control-radius)-1px)] pointer-coarse:after:absolute pointer-coarse:after:size-full pointer-coarse:after:min-h-11 pointer-coarse:after:min-w-11 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-64 aria-disabled:cursor-not-allowed aria-disabled:opacity-64 sm:text-sm [&_svg:not([class*='text-'])]:text-[var(--control-icon-color)] [&_svg:not([class*='size-'])]:size-4.5 sm:[&_svg:not([class*='size-'])]:size-4 [&_svg]:pointer-events-none [&_svg]:shrink-0 size-8 sm:size-7 [--control-icon-color:var(--contrast-muted-foreground)] border-transparent text-foreground data-pressed:bg-accent [:hover,[data-pressed]]:bg-accent shrink-0",
                                                        "data-slot": "tooltip-trigger",
                                                        "aria-label": "Start voice input",
                                                        "disabled": "",
                                                        "id": "base-ui-_r_2g_",
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
                                                            "class": "lucide lucide-mic",
                                                            "aria-hidden": "true",
                                                            path {
                                                                "d": "M12 19v3",
                                                            }
                                                            path {
                                                                "d": "M19 10v2a7 7 0 0 1-14 0v-2",
                                                            }
                                                            rect {
                                                                "x": "9",
                                                                "y": "2",
                                                                "width": "6",
                                                                "height": "13",
                                                                "rx": "3",
                                                            }
                                                        }
                                                    }
                                                    button {
                                                        "type": "submit",
                                                        "class": "inline-flex shrink-0 cursor-pointer items-center justify-center gap-1.5 whitespace-nowrap rounded-full bg-message-action font-medium text-base text-message-action-foreground shadow-xs shadow-message-action/24 outline-none hover:bg-message-action-hover focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-64 disabled:shadow-none sm:text-sm h-8 sm:h-7 px-3",
                                                        disabled: selected().is_none(),
                                                        onclick: {
                                                            let options = options.clone();
                                                            move |e| {
                                                                e.prevent_default();
                                                                if let (Some(i), Some(answer)) = (selected(), onanswer) {
                                                                    answer.call(options[i].value.clone());
                                                                }
                                                            }
                                                        },
                                                        "Submit"
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
                div {
                    "class": "min-h-0",
                    div {
                        "class": "relative z-0",
                    }
                }
            }
    }
}

#[component]
pub fn ComposerNormal(#[props(default)] onsend: Option<EventHandler<String>>, #[props(default)] onstop: Option<EventHandler<()>>, #[props(default)] running: bool, #[props(default)] onaction: EventHandler<String>, #[props(default)] model_label: String, #[props(default)] onmodel: Option<EventHandler<()>>) -> Element {
    let label = if model_label.is_empty() { "GPT-5.4".to_string() } else { model_label };
    let mut text = use_signal(String::new);
    let live = onsend.is_some();
    let empty = text().trim().is_empty();
    let lines = text().lines().count().max(1) + usize::from(text().ends_with('\n'));
    let rows = lines.clamp(2, 8) as i64;
    rsx! {
            div {
                "data-slot": "composer-shell",
                "class": "@container/composer-surface group/composer-surface relative isolate mx-auto w-full max-w-(--chat-content-max-width) [--chat-composer-drawer-inset:1.375rem] [--chat-composer-glass-surface:var(--card)] [--chat-composer-outline:rgb(0_0_0/8%)] dark:[--chat-composer-glass-surface:var(--surface-raised)] dark:[--chat-composer-highlight:rgb(255_255_255/3%)] dark:[--chat-composer-outline:color-mix(in_srgb,var(--color-white)_5%,transparent)] [html[data-theme-id]_&]:[--chat-composer-glass-surface:var(--app-theme-surface-raised)] [html[data-theme-id]_&]:[--chat-composer-outline:var(--app-theme-toolbar-border)] dark:[html[data-theme-id]:not([data-theme-id=t3-chat])_&]:[--chat-composer-highlight:color-mix(in_srgb,var(--app-theme-input)_12%,transparent)] dark:[html[data-theme-id]:not([data-theme-id=t3-chat])_&]:[--chat-composer-outline:color-mix(in_srgb,var(--app-theme-input)_30%,var(--background))] dark:[html[data-theme-id=t3-chat]_&]:[--chat-composer-highlight:color-mix(in_srgb,#432d48_12%,transparent)] dark:[html[data-theme-id=t3-chat]_&]:[--chat-composer-outline:#241e28] before:pointer-events-none before:absolute before:inset-0 before:z-0 before:rounded-2xl before:bg-(--chat-composer-glass-surface)/(--glass-opacity) before:backdrop-blur-(--glass-blur) before:backdrop-saturate-(--glass-saturation) not-supports-[((backdrop-filter:blur(1px))_or_(-webkit-backdrop-filter:blur(1px)))]:before:bg-(--chat-composer-glass-surface) has-data-[composer-banner-surface=attached]:before:hidden data-context-row:before:hidden",
                div {
                    "data-slot": "composer-host",
                    "class": "relative z-10 w-full rounded-2xl shadow-composer after:z-1 dark:shadow-none group-data-context-row/composer-surface:before:pointer-events-none group-data-context-row/composer-surface:before:absolute group-data-context-row/composer-surface:before:inset-0 group-data-context-row/composer-surface:before:-z-1 group-data-context-row/composer-surface:before:rounded-2xl group-data-context-row/composer-surface:before:bg-(--chat-composer-glass-surface)/(--glass-opacity) group-data-context-row/composer-surface:before:backdrop-blur-(--glass-blur) group-data-context-row/composer-surface:before:backdrop-saturate-(--glass-saturation) after:pointer-events-none after:absolute after:inset-0 after:rounded-[inherit] after:border after:border-(--chat-composer-outline) dark:after:inset-shadow-2xs dark:after:inset-shadow-(color:--chat-composer-highlight) group-has-data-[composer-banner-surface=attached]/composer-surface:shadow-none group-has-data-[composer-banner-surface=attached]/composer-surface:before:hidden group-has-data-[composer-banner-surface=attached]/composer-surface:after:hidden",
                    "aria-busy": "false",
                    div {
                        "class": "relative z-10",
                        form {
                            "class": "mx-auto w-full min-w-0 max-w-(--chat-content-max-width)",
                            "data-chat-composer-form": "true",
                            "data-thread-context-drop": "true",
                            div {
                                "data-slot": "composer-banner-attachment",
                                "class": "mx-auto w-[calc(100%-2*var(--chat-composer-drawer-inset))] flex items-end gap-1 not-has-data-[composer-banner-surface=attached]:hidden",
                                div {
                                    "class": "flex min-w-0 flex-1 flex-col empty:hidden [&>[data-slot=composer-banner-attachment]]:w-full [&>[data-slot=composer-banner-attachment]:last-child]:mb-0",
                                }
                            }
                            div {
                                "class": "relative",
                                div {
                                    "data-chat-composer-main-surface": "true",
                                    "class": "group relative z-10 rounded-2xl p-px after:pointer-events-none after:absolute after:inset-0 after:rounded-[inherit] after:border after:border-(--chat-composer-outline) dark:after:inset-shadow-2xs dark:after:inset-shadow-(color:--chat-composer-highlight) after:z-20 after:hidden group-has-data-[composer-banner-surface=attached]/composer-surface:after:block group-has-data-[composer-banner-surface=attached]/composer-surface:bg-(--chat-composer-glass-surface)/(--glass-opacity) group-has-data-[composer-banner-surface=attached]/composer-surface:backdrop-blur-(--glass-blur) group-has-data-[composer-banner-surface=attached]/composer-surface:backdrop-saturate-(--glass-saturation) group-has-data-[composer-banner-surface=attached]/composer-surface:shadow-composer dark:group-has-data-[composer-banner-surface=attached]/composer-surface:shadow-none not-supports-[((backdrop-filter:blur(1px))_or_(-webkit-backdrop-filter:blur(1px)))]:group-has-data-[composer-banner-surface=attached]/composer-surface:bg-(--chat-composer-glass-surface) group-has-data-[composer-banner-surface=attached]/composer-surface:**:data-[chat-composer-mobile-collapsed=true]:min-h-[calc(1rem+1px)]",
                                    "data-inline-resting-controls": "true",
                                    div {
                                        "data-chat-composer-surface": "true",
                                        "data-chat-composer-mobile-collapsed": "false",
                                        "class": "rounded-2xl transition-[background-color] duration-200 in-data-[thread-context-over]:bg-accent/45 in-data-[thread-context-over]:ring-1 in-data-[thread-context-over]:ring-inset in-data-[thread-context-over]:ring-primary/70",
                                        div {
                                            "data-chat-composer-body": "true",
                                            "class": "relative px-4 pb-2 pt-3.5 sm:pt-4",
                                            div {
                                                "role": "status",
                                                "aria-atomic": "true",
                                                "class": "sr-only",
                                            }
                                            div {
                                                "class": "relative",
                                                div {
                                                    "class": "relative flow-root font-(family-name:--font-composer,var(--font-sans)) text-(length:--font-size-prompt,var(--text-sm)) max-sm:pointer-coarse:text-(length:--font-size-prompt-touch)",
                                                    div {
                                                        if live {
                                                            textarea {
                                                                "data-testid": "composer-editor",
                                                                "aria-label": "Message Thread thread-golden",
                                                                "class": "composer-tiptap -m-1 block max-h-52 min-h-[calc(2lh+--spacing(2))] w-full resize-none overflow-y-auto p-1 whitespace-pre-wrap wrap-break-word border-0 bg-transparent leading-relaxed text-foreground focus:outline-none",
                                                                rows: rows,
                                                                value: "{text}",
                                                                onmounted: move |e| {
                                                                    if live {
                                                                        spawn(async move {
                                                                            tokio::time::sleep(std::time::Duration::from_millis(40)).await;
                                                                            let _ = e.set_focus(true).await;
                                                                        });
                                                                    }
                                                                },
                                                                oninput: move |e| text.set(e.value()),
                                                                onkeydown: move |e| {
                                                                    if e.key().to_string() == "Enter" && !e.modifiers().shift() {
                                                                        e.prevent_default();
                                                                        let message = text();
                                                                        if !message.trim().is_empty() {
                                                                            if let Some(send) = onsend {
                                                                                send.call(message);
                                                                            }
                                                                            text.set(String::new());
                                                                        }
                                                                    }
                                                                },
                                                            }
                                                        } else {
                                                            div {
                                                                "contenteditable": "true",
                                                                "role": "textbox",
                                                                "data-testid": "composer-editor",
                                                                "data-composer-rich-text": "true",
                                                                "aria-multiline": "true",
                                                                "aria-label": "Message Thread thread-golden",
                                                                "aria-autocomplete": "list",
                                                                "aria-haspopup": "listbox",
                                                                "aria-placeholder": "Ask anything. @ files, $ skills, / commands",
                                                                "translate": "no",
                                                                "class": "tiptap ProseMirror composer-tiptap -m-1 block max-h-52 min-h-[calc(2lh+--spacing(2))] overflow-y-auto p-1 whitespace-pre-wrap wrap-break-word bg-transparent leading-relaxed text-foreground focus:outline-none ProseMirror-focused",
                                                                "tabindex": "0",
                                                                p {
                                                                    br {
                                                                        "class": "ProseMirror-trailingBreak",
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        }
                                                    if !live || text().is_empty() {
                                                        div { "class": "pointer-events-none absolute inset-0 leading-relaxed text-placeholder", "Ask anything. @ files, $ skills, / commands" }
                                                    }
                                                }
                                            }
                                        }
                                        div {
                                            "data-chat-composer-footer": "true",
                                            "class": "flex min-w-0 flex-nowrap items-center overflow-visible px-2 pb-2",
                                            div {
                                                "data-chat-composer-actions": "right",
                                                "data-chat-composer-transition-actions": "true",
                                                "data-chat-composer-primary-actions-compact": "false",
                                                "class": "flex min-w-0 flex-1 flex-nowrap items-center justify-end gap-2",
                                                div {
                                                    "data-chat-composer-controls": "footer",
                                                    "data-chat-composer-footer-controls": "true",
                                                    "class": "relative flex min-w-0 flex-1 items-center justify-end gap-0.5",
                                                    button {
                                                        "type": "button",
                                                        "class": "relative inline-flex cursor-pointer items-center rounded-(--control-radius) border border-transparent text-base outline-none hover:bg-accent data-pressed:bg-accent focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-64 data-disabled:pointer-events-none data-disabled:opacity-64 pointer-coarse:after:absolute pointer-coarse:after:size-full pointer-coarse:after:min-h-11 pointer-coarse:after:min-w-11 [&:active:not([aria-haspopup])]:scale-[0.97] [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg]:-mx-0.5 [&_svg[data-composer-control-icon]]:mx-0 [&_:is([data-composer-control-label],[data-chat-provider-model-picker-label])]:[text-box:trim-both_cap_alphabetic] supports-[text-box:trim-both_cap_alphabetic]:[&_:is([data-composer-control-label],[data-chat-provider-model-picker-label])]:py-1.5 h-7 gap-1.5 px-2 font-medium text-foreground/80 [&_svg:not([class*='text-'])]:text-muted-foreground hover:text-foreground sm:text-sm [&_svg:not([class*='size-'])]:size-4.5 sm:[&_svg:not([class*='size-'])]:size-4 aria-pressed:bg-accent aria-pressed:text-accent-foreground aria-pressed:hover:bg-accent/80 shrink justify-between whitespace-nowrap group/model-trigger data-popup-open:bg-accent data-popup-open:text-foreground min-w-13",
                                                        "aria-label": "{label}",
                                                        onclick: move |e| { e.prevent_default(); if let Some(open) = onmodel { open.call(()); } },
                                                        "data-chat-provider-model-picker": "true",
                                                        "data-composer-shortcut": "composer.effort composer.mode",
                                                        "tabindex": "0",
                                                        "data-base-ui-click-trigger": "",
                                                        "id": "base-ui-_r_2b_",
                                                        "aria-haspopup": "dialog",
                                                        "aria-expanded": "false",
                                                        "data-slot": "popover-trigger",
                                                        span {
                                                            "class": "flex min-w-0 flex-1 items-center gap-1.5",
                                                            span {
                                                                "data-chat-provider-model-picker-icon": "true",
                                                                "class": "contents",
                                                                span {
                                                                    "class": "relative isolate z-30 inline-flex shrink-0 items-center justify-center overflow-visible size-4",
                                                                    svg {
                                                                        "aria-hidden": "true",
                                                                        "preserveAspectRatio": "xMidYMid",
                                                                        "viewBox": "100 100 411 411",
                                                                        "class": "fill-current shrink-0 size-4",
                                                                        path {
                                                                            "fill-rule": "evenodd",
                                                                            "clip-rule": "evenodd",
                                                                            "d": "M252.794 108.802C289.191 99.0484 326.265 110.305 351.148 135.135C385.113 126.072 422.85 134.862 449.492 161.505C476.136 188.149 484.925 225.888 475.862 259.85V259.854C500.696 284.735 511.95 321.81 502.198 358.207C492.447 394.602 464.161 421.084 430.215 430.217C421.083 464.162 394.603 492.448 358.206 502.199C321.812 511.951 284.734 500.693 259.852 475.864C225.887 484.927 188.15 476.137 161.507 449.495C134.864 422.851 126.073 385.111 135.136 351.149C110.304 326.266 99.0496 289.192 108.801 252.795C118.552 216.4 146.84 189.918 180.784 180.785C189.917 146.841 216.396 118.553 252.794 108.802ZM374.292 407.145C374.292 411.271 372.092 415.086 368.517 417.148L283.723 466.102C302.487 480.585 327.555 486.459 352.217 479.852C386.997 470.532 410.068 439.312 410.555 405.006V317.717C410.555 315.08 409.125 312.621 406.843 311.303L374.292 292.509V407.145ZM251.868 415.897C248.296 417.959 243.893 417.959 240.317 415.897L155.526 366.942C152.366 390.436 159.811 415.08 177.866 433.136H177.863C203.325 458.594 241.896 462.962 271.85 446.232L347.449 402.586C349.735 401.268 351.148 398.8 351.148 396.163V358.579L251.868 415.897ZM368.602 220.628C366.319 219.309 363.474 219.318 361.191 220.637L328.641 239.431L427.921 296.749C431.496 298.811 433.697 302.627 433.697 306.752V404.661C455.622 395.654 473.244 376.881 479.851 352.218C489.169 317.442 473.668 281.85 444.201 264.274L368.602 220.628ZM177.303 206.34C155.377 215.348 137.756 234.122 131.148 258.783C121.832 293.561 137.331 329.153 166.799 346.727L242.398 390.373C244.68 391.692 247.525 391.684 249.807 390.366L282.357 371.572L183.078 314.253C179.504 312.189 177.303 308.375 177.303 304.251V206.34ZM259.849 279.145V331.858L305.5 358.213L351.15 331.858V279.145L305.5 252.789L259.849 279.145ZM327.276 144.9C308.512 130.418 283.445 124.543 258.782 131.15C224.002 140.471 200.931 171.691 200.445 205.995V293.286C200.445 295.923 201.875 298.381 204.158 299.7L236.707 318.493V203.856C236.707 199.731 238.909 195.916 242.483 193.853L327.276 144.9ZM433.137 177.867C407.675 152.407 369.103 148.038 339.149 164.769L263.55 208.415C261.265 209.734 259.852 212.202 259.852 214.838V252.423L359.132 195.105C362.703 193.041 367.108 193.041 370.682 195.105L455.473 244.06C458.635 220.567 451.189 195.922 433.135 177.867H433.137Z",
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            span {
                                                                "class": "min-w-0 flex-1 overflow-hidden truncate max-w-56",
                                                                "data-chat-provider-model-picker-label": "true",
                                                                "{label}"
                                                            }
                                                        }
                                                    }
                                                    div {
                                                        "data-resting-controls-overflow": "true",
                                                        "aria-hidden": "true",
                                                        "inert": "",
                                                        "class": "min-w-0 shrink-0 pointer-events-none invisible absolute",
                                                        button {
                                                            onclick: move |e| { e.prevent_default(); onaction.call("unavailable:Composer options (attachments, plan mode)".to_string()); },
                                                            "type": "button",
                                                            "class": "relative inline-flex cursor-pointer items-center justify-center whitespace-nowrap rounded-(--control-radius) border border-transparent text-base outline-none hover:bg-accent data-pressed:bg-accent focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-64 data-disabled:pointer-events-none data-disabled:opacity-64 pointer-coarse:after:absolute pointer-coarse:after:size-full pointer-coarse:after:min-h-11 pointer-coarse:after:min-w-11 [&:active:not([aria-haspopup])]:scale-[0.97] [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg]:-mx-0.5 [&_svg[data-composer-control-icon]]:mx-0 [&_:is([data-composer-control-label],[data-chat-provider-model-picker-label])]:[text-box:trim-both_cap_alphabetic] supports-[text-box:trim-both_cap_alphabetic]:[&_:is([data-composer-control-label],[data-chat-provider-model-picker-label])]:py-1.5 h-7 gap-1.5 px-2 font-medium text-foreground/80 [&_svg:not([class*='text-'])]:text-muted-foreground hover:text-foreground sm:text-sm [&_svg:not([class*='size-'])]:size-4.5 sm:[&_svg:not([class*='size-'])]:size-4 aria-pressed:bg-accent aria-pressed:text-accent-foreground aria-pressed:hover:bg-accent/80 shrink-0",
                                                            "aria-label": "More composer controls",
                                                            "tabindex": "0",
                                                            "aria-haspopup": "menu",
                                                            "aria-expanded": "false",
                                                            "id": "base-ui-_r_2f_",
                                                            "data-slot": "menu-trigger",
                                                            svg {
                                                                "width": "24",
                                                                "height": "24",
                                                                "viewBox": "0 0 24 24",
                                                                "fill": "none",
                                                                "stroke": "currentColor",
                                                                "stroke-width": "2",
                                                                "stroke-linecap": "round",
                                                                "stroke-linejoin": "round",
                                                                "class": "lucide lucide-ellipsis shrink-0 size-4",
                                                                "aria-hidden": "true",
                                                                "data-composer-control-icon": "true",
                                                                circle {
                                                                    "cx": "12",
                                                                    "cy": "12",
                                                                    "r": "1",
                                                                }
                                                                circle {
                                                                    "cx": "19",
                                                                    "cy": "12",
                                                                    "r": "1",
                                                                }
                                                                circle {
                                                                    "cx": "5",
                                                                    "cy": "12",
                                                                    "r": "1",
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                button {
                                                    onclick: move |e| { e.prevent_default(); onaction.call("unavailable:Voice input".to_string()); },
                                                    "type": "button",
                                                    "class": "[&_svg]:-mx-0.5 relative inline-flex cursor-pointer items-center justify-center gap-2 whitespace-nowrap rounded-[var(--control-radius)] border font-medium text-base outline-none transition-[box-shadow,scale] [&:not([aria-haspopup]):not(:disabled):not([aria-disabled=true])]:will-change-[scale] [&:active:not([aria-haspopup])]:scale-[0.97] before:pointer-events-none before:absolute before:inset-0 before:rounded-[calc(var(--control-radius)-1px)] pointer-coarse:after:absolute pointer-coarse:after:size-full pointer-coarse:after:min-h-11 pointer-coarse:after:min-w-11 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-64 aria-disabled:cursor-not-allowed aria-disabled:opacity-64 sm:text-sm [&_svg:not([class*='text-'])]:text-[var(--control-icon-color)] [&_svg:not([class*='size-'])]:size-4.5 sm:[&_svg:not([class*='size-'])]:size-4 [&_svg]:pointer-events-none [&_svg]:shrink-0 size-8 sm:size-7 [--control-icon-color:var(--contrast-muted-foreground)] border-transparent text-foreground data-pressed:bg-accent [:hover,[data-pressed]]:bg-accent shrink-0",
                                                    "data-slot": "tooltip-trigger",
                                                    "aria-label": "Start voice input",
                                                    "id": "base-ui-_r_2i_",
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
                                                        "class": "lucide lucide-mic",
                                                        "aria-hidden": "true",
                                                        path {
                                                            "d": "M12 19v3",
                                                        }
                                                        path {
                                                            "d": "M19 10v2a7 7 0 0 1-14 0v-2",
                                                        }
                                                        rect {
                                                            "x": "9",
                                                            "y": "2",
                                                            "width": "6",
                                                            "height": "13",
                                                            "rx": "3",
                                                        }
                                                    }
                                                }
                                                span {
                                                    "class": "inline-flex",
                                                    "id": "base-ui-_r_2l_",
                                                    "data-base-ui-tooltip-trigger": "",
                                                    "data-slot": "tooltip-trigger",
                                                    button {
                                                        "type": "submit",
                                                        "class": "relative isolate flex h-9 w-9 items-center justify-center overflow-hidden rounded-full shadow-xs transition-all duration-150 enabled:cursor-pointer enabled:inset-shadow-control-highlight hover:scale-105 active:inset-shadow-control-pressed active:shadow-none disabled:pointer-events-none disabled:shadow-none disabled:hover:scale-100 sm:h-8 sm:w-8 [&_svg]:pointer-events-none bg-transparent text-white enabled:shadow-black/24 enabled:hover:brightness-110 disabled:opacity-64",
                                                        disabled: !live || (empty && !running),
                                                        "aria-label": if running && empty { "Stop generation" } else { "Submit message" },
                                                        onclick: move |e| {
                                                            e.prevent_default();
                                                            if running && text().trim().is_empty() {
                                                                if let Some(stop) = onstop {
                                                                    stop.call(());
                                                                }
                                                                return;
                                                            }
                                                            let message = text();
                                                            if let Some(send) = onsend {
                                                                if !message.trim().is_empty() {
                                                                    send.call(message);
                                                                    text.set(String::new());
                                                                }
                                                            }
                                                        },
                                                        if running && empty {
                                                            span {
                                                                "class": "absolute inset-0 z-10 flex items-center justify-center rounded-full bg-destructive/90 text-white",
                                                                svg { "width": "12", "height": "12", "viewBox": "0 0 12 12", "fill": "currentColor", "aria-hidden": "true", rect { "x": "2", "y": "2", "width": "8", "height": "8", "rx": "1.5" } }
                                                            }
                                                        }
                                                        span {
                                                            "class": "pointer-events-none absolute inset-0 -z-10",
                                                            "aria-hidden": "true",
                                                            svg {
                                                                "data-stage-art": "blueprint",
                                                                "class": "h-full w-full",
                                                                "fill": "none",
                                                                "preserveAspectRatio": "xMinYMin slice",
                                                                "viewBox": "64 0 8192 96",
                                                                defs {
                                                                    linearGradient {
                                                                        "id": "_r_2m_-stage-bp-paper",
                                                                        "x1": "60",
                                                                        "y1": "0",
                                                                        "x2": "220",
                                                                        "y2": "96",
                                                                        "gradientUnits": "userSpaceOnUse",
                                                                        "spreadMethod": "reflect",
                                                                        stop {
                                                                            "style": "stop-color: #101f6e;",
                                                                        }
                                                                        stop {
                                                                            "offset": "0.5",
                                                                            "style": "stop-color: #2050ae;",
                                                                        }
                                                                        stop {
                                                                            "offset": "1",
                                                                            "style": "stop-color: #3a7ad1;",
                                                                        }
                                                                    }
                                                                    radialGradient {
                                                                        "id": "_r_2m_-stage-bp-glow",
                                                                        "cx": "0",
                                                                        "cy": "0",
                                                                        "r": "1",
                                                                        "gradientTransform": "translate(216 14) rotate(137) scale(120 84)",
                                                                        "gradientUnits": "userSpaceOnUse",
                                                                        stop {
                                                                            "stop-opacity": "0.4",
                                                                            "style": "stop-color: #d4f6ff;",
                                                                        }
                                                                        stop {
                                                                            "offset": "0.52",
                                                                            "stop-opacity": "0.16",
                                                                            "style": "stop-color: #65c8ff;",
                                                                        }
                                                                        stop {
                                                                            "offset": "1",
                                                                            "stop-opacity": "0",
                                                                            "style": "stop-color: #101f6e;",
                                                                        }
                                                                    }
                                                                    radialGradient {
                                                                        "id": "_r_2m_-stage-bp-glow-celeste",
                                                                        "cx": "0",
                                                                        "cy": "0",
                                                                        "r": "1",
                                                                        "gradientTransform": "translate(474 44) rotate(166) scale(156 92)",
                                                                        "gradientUnits": "userSpaceOnUse",
                                                                        stop {
                                                                            "stop-opacity": "0.34",
                                                                            "style": "stop-color: #d2ffff;",
                                                                        }
                                                                        stop {
                                                                            "offset": "0.5",
                                                                            "stop-opacity": "0.18",
                                                                            "style": "stop-color: #48dcf5;",
                                                                        }
                                                                        stop {
                                                                            "offset": "1",
                                                                            "stop-opacity": "0",
                                                                            "style": "stop-color: #101f6e;",
                                                                        }
                                                                    }
                                                                    radialGradient {
                                                                        "id": "_r_2m_-stage-bp-glow-violet",
                                                                        "cx": "0",
                                                                        "cy": "0",
                                                                        "r": "1",
                                                                        "gradientTransform": "translate(704 18) rotate(145) scale(132 88)",
                                                                        "gradientUnits": "userSpaceOnUse",
                                                                        stop {
                                                                            "stop-opacity": "0.3",
                                                                            "style": "stop-color: #d9d8ff;",
                                                                        }
                                                                        stop {
                                                                            "offset": "0.52",
                                                                            "stop-opacity": "0.14",
                                                                            "style": "stop-color: #7c8bff;",
                                                                        }
                                                                        stop {
                                                                            "offset": "1",
                                                                            "stop-opacity": "0",
                                                                            "style": "stop-color: #101f6e;",
                                                                        }
                                                                    }





                                                                }
                                                                rect {
                                                                    "width": "100%",
                                                                    "height": "96",
                                                                    "fill": "url(#_r_2m_-stage-bp-paper)",
                                                                }
                                                                g {
                                                                        rect {
                                                                            "fill": "url(#_r_2m_-stage-bp-glow)",
                                                                        }
                                                                        rect {
                                                                            "fill": "url(#_r_2m_-stage-bp-glow-celeste)",
                                                                        }
                                                                        rect {
                                                                            "fill": "url(#_r_2m_-stage-bp-glow-violet)",
                                                                        }
                                                                    }
                                                                
                                                                
                                                                
                                                                g {
                                                                        g {
                                                                            "stroke-linecap": "round",
                                                                            "stroke-opacity": "0.6",
                                                                            "stroke-width": "0.7",
                                                                            "style": "stroke: #ddf7ff;",
                                                                            path {
                                                                                "d": "M180 64H264",
                                                                                "stroke-dasharray": "5 4",
                                                                            }
                                                                            path {
                                                                                "d": "M180 61V67M264 61V67",
                                                                            }
                                                                            path {
                                                                                "d": "M276 10V44",
                                                                                "stroke-dasharray": "4 4",
                                                                                "stroke-opacity": "0.5",
                                                                            }
                                                                            path {
                                                                                "d": "M273 10H279M273 44H279",
                                                                                "stroke-opacity": "0.5",
                                                                            }
                                                                            path {
                                                                                "d": "M348 30H428",
                                                                                "stroke-dasharray": "3.5 5",
                                                                                "stroke-opacity": "0.5",
                                                                            }
                                                                            path {
                                                                                "d": "M348 27V33M428 27V33",
                                                                                "stroke-opacity": "0.5",
                                                                            }
                                                                            path {
                                                                                "d": "M512 48V80",
                                                                                "stroke-dasharray": "5 3",
                                                                                "stroke-opacity": "0.45",
                                                                            }
                                                                            path {
                                                                                "d": "M509 48H515M509 80H515",
                                                                                "stroke-opacity": "0.45",
                                                                            }
                                                                            path {
                                                                                "d": "M590 70H724",
                                                                                "stroke-dasharray": "7 4",
                                                                                "stroke-opacity": "0.55",
                                                                            }
                                                                            path {
                                                                                "d": "M590 67V73M724 67V73",
                                                                                "stroke-opacity": "0.55",
                                                                            }
                                                                        }
                                                                        g {
                                                                            "stroke-linecap": "round",
                                                                            "stroke-opacity": "0.55",
                                                                            "stroke-width": "0.6",
                                                                            "style": "stroke: #ddf7ff;",
                                                                            g {
                                                                                path {
                                                                                    "d": "M34 60L38 64M38 60L34 64",
                                                                                }
                                                                            }
                                                                            g {
                                                                                path {
                                                                                    "d": "M228 26H234M231 23V29",
                                                                                }
                                                                            }
                                                                            g {
                                                                                path {
                                                                                    "d": "M143 51H149M146 48V54",
                                                                                }
                                                                            }
                                                                            g {
                                                                                path {
                                                                                    "d": "M316 16L322 22M322 16L316 22",
                                                                                }
                                                                            }
                                                                            g {
                                                                                path {
                                                                                    "d": "M468 70H476M472 66V74",
                                                                                }
                                                                            }
                                                                            g {
                                                                                path {
                                                                                    "d": "M558 28L564 34M564 28L558 34",
                                                                                }
                                                                            }
                                                                            g {
                                                                                path {
                                                                                    "d": "M742 44H750M746 40V48",
                                                                                }
                                                                            }
                                                                        }
                                                                        g {
                                                                            "stroke-opacity": "0.35",
                                                                            "stroke-width": "0.6",
                                                                            "style": "stroke: #ddf7ff;",
                                                                            circle {
                                                                                "cx": "196",
                                                                                "cy": "38",
                                                                                "r": "13",
                                                                                "stroke-dasharray": "3.5 4",
                                                                            }
                                                                            path {
                                                                                "d": "M196 33V43M191 38H201",
                                                                                "stroke-opacity": "0.6",
                                                                                "stroke-width": "0.4",
                                                                            }
                                                                            circle {
                                                                                "cx": "414",
                                                                                "cy": "64",
                                                                                "r": "10",
                                                                                "stroke-dasharray": "2.5 3.5",
                                                                            }
                                                                            path {
                                                                                "d": "M414 60V68M410 64H418",
                                                                                "stroke-opacity": "0.6",
                                                                                "stroke-width": "0.4",
                                                                            }
                                                                            circle {
                                                                                "cx": "648",
                                                                                "cy": "32",
                                                                                "r": "15",
                                                                                "stroke-dasharray": "4 5",
                                                                            }
                                                                            path {
                                                                                "d": "M648 26V38M642 32H654",
                                                                                "stroke-opacity": "0.6",
                                                                                "stroke-width": "0.4",
                                                                            }
                                                                        }
                                                                    }
                                                            }
                                                        }
                                                        svg {
                                                            "width": "14",
                                                            "height": "14",
                                                            "viewBox": "0 0 14 14",
                                                            "fill": "none",
                                                            "aria-hidden": "true",
                                                            path {
                                                                "d": "M7 11.5V2.5M7 2.5L3 6.5M7 2.5L11 6.5",
                                                                "stroke": "currentColor",
                                                                "stroke-width": "1.8",
                                                                "stroke-linecap": "round",
                                                                "stroke-linejoin": "round",
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
                div {
                    "class": "min-h-0",
                    div {
                        "class": "relative z-0",
                    }
                }
            }
    }
}
