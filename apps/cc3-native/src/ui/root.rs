//! The application root: sidebar, chat, pages and palette wired to the `Store`. The binary wraps it in the
//! window frame; tests mount it without a window (`tests/interact.rs`).
use std::sync::Arc;

use dioxus::prelude::*;

use crate::store::Store;
use crate::ui::app_layout::AppLayout;
use crate::ui::sidebar_list::ThreadRowData;
use crate::ui::window_frame::WindowFrame;

/// Props wrapper: stores compare by identity.
#[derive(Clone)]
pub struct StoreHandle(pub Arc<Store>);

impl PartialEq for StoreHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

/// Sidebar rows from the shell model.
pub fn rows_from_store(store: &Store, open: Option<String>) -> Vec<ThreadRowData> {
    let shell = store.shell.lock().unwrap();
    let environment_id = store.environment_id.lock().unwrap().clone();
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    shell
        .visible_threads()
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let project = shell.project(&t.project_id).map(|p| p.title.clone()).unwrap_or_default();
            let monogram: String = project.chars().filter(|c| c.is_alphanumeric()).take(2).collect::<String>().to_uppercase();
            ThreadRowData {
                thread_id: t.id.clone(),
                thread_key: format!("{environment_id}:{}", t.id),
                aria_label: format!("{}, {project}", t.title),
                ui_id: format!("_r_{i}_"),
                title: t.title.clone(),
                project,
                monogram,
                age: crate::model::relative_age(&t.updated_at, now),
                branch: t.branch.clone().unwrap_or_default(),
                active: open.as_deref() == Some(t.id.as_str()),
            }
        })
        .collect()
}

#[component]
pub fn AppRoot(store: StoreHandle, #[props(default)] frame: bool) -> Element {
    // Leaked once per mounted root so event closures can copy the reference.
    let store: &'static Arc<Store> = use_hook(|| &*Box::leak(Box::new(store.0.clone())));
    let mut rows = use_signal(Vec::<ThreadRowData>::new);
    let mut selected = use_signal(|| None::<String>);
    let mut page = use_signal(|| std::env::var("CC3_PAGE").ok());
    let mut thread = use_signal(crate::model::ThreadView::default);
    let mut draft = use_signal(|| false);
    let mut palette_open = use_signal(|| std::env::var("CC3_PALETTE").is_ok());
    let mut palette_query = use_signal(String::new);
    let mut palette_active = use_signal(|| 0usize);

    // The store is fed from other threads; rows and the open thread follow its version.
    use_future({
        move || {
            async move {
                let mut seen = u64::MAX;
                loop {
                    let version = store.version();
                    if version != seen {
                        seen = version;
                        rows.set(rows_from_store(store, selected()));
                        thread.set(if draft() { crate::model::ThreadView::default() } else { store.thread_view().unwrap_or_default() });
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                }
            }
        }
    });

    let content = rsx! {
                AppLayout {
                    rows: rows(),
                    thread: thread(),
                    environment_id: store.environment_id.lock().unwrap().clone(),
                    composer: crate::ui::composer::ComposerMode::Normal,
                    onsend: move |text: String| {
                        if draft() {
                            if let Some(id) = store.start_thread(&text, None) {
                                draft.set(false);
                                selected.set(Some(id));
                            }
                        } else {
                            store.send_message(&text);
                        }
                    },
                    page: page(),
                    onstop: move |_| store.stop_thread(),
                    onanswer: move |(request_id, value): (String, String)| store.answer_question(&request_id, &value),
                    onaction: move |action: String| {
                        if action == "page:new-chat" {
                            page.set(None);
                            draft.set(true);
                            selected.set(None);
                            thread.set(crate::model::ThreadView::default());
                        } else if let Some(name) = action.strip_prefix("page:") {
                            page.set(Some(name.to_string()));
                        } else if let Some(section) = action.strip_prefix("section:") {
                            let name = format!("settings-{}", section.trim_start_matches("/settings/"));
                            if crate::ui::fragments::page(&name).is_some() {
                                page.set(Some(name));
                            }
                        } else if let Some(rest) = action.strip_prefix("switch:") {
                            if let Some((value, label)) = rest.split_once(':') {
                                crate::ui::settings::set_switch(label, value == "true", |key, on| store.update_server_setting(key, serde_json::json!(on)));
                            }
                        } else if action == "close" {
                            page.set(None);
                        }
                    },
                    onselect: move |id: String| {
                        draft.set(false);
                        page.set(None);
                        selected.set(Some(id.clone()));
                        rows.set(rows_from_store(store, Some(id.clone())));
                        store.open_thread(&id);
                    },
                }
                if palette_open() {
                    {
                        let threads: Vec<crate::ui::palette::PaletteThread> = rows()
                            .iter()
                            .map(|r| crate::ui::palette::PaletteThread { id: r.thread_id.clone(), title: r.title.clone(), project: r.project.clone(), monogram: r.monogram.clone(), branch: r.branch.clone(), age: r.age.clone() })
                            .collect();
                        let items = crate::ui::palette::rows_for(&palette_query(), &threads, palette_active());
                        let selectable: Vec<String> = items.iter().filter(|r| r.selectable).map(|r| r.key.clone()).collect();
                        let run = {
                            let selectable = selectable.clone();
                            move |key: String| {
                                palette_open.set(false);
                                if let Some(id) = key.strip_prefix("thread:") {
                                    draft.set(false);
                                    page.set(None);
                                    selected.set(Some(id.to_string()));
                                    rows.set(rows_from_store(store, Some(id.to_string())));
                                    store.open_thread(id);
                                } else if key == "page:new-chat" {
                                    page.set(None);
                                    draft.set(true);
                                    selected.set(None);
                                    thread.set(crate::model::ThreadView::default());
                                } else if let Some(name) = key.strip_prefix("page:") {
                                    page.set(Some(name.to_string()));
                                }
                                let _ = &selectable;
                            }
                        };
                        rsx! {
                            crate::ui::palette::CommandPalette {
                                query: palette_query(),
                                rows: items,
                                oninput: move |q: String| { palette_query.set(q); palette_active.set(0); },
                                onkey: {
                                    let selectable = selectable.clone();
                                    let mut run = run.clone();
                                    move |key: String| match key.as_str() {
                                        "ArrowDown" => palette_active.set((palette_active() + 1).min(selectable.len().saturating_sub(1))),
                                        "ArrowUp" => palette_active.set(palette_active().saturating_sub(1)),
                                        "Enter" => if let Some(k) = selectable.get(palette_active()) { run(k.clone()) },
                                        "Escape" => palette_open.set(false),
                                        _ => {}
                                    }
                                },
                                onpick: { let mut run = run.clone(); move |k: String| run(k) },
                                onclose: move |_| palette_open.set(false),
                            }
                        }
                    }
                }
    };

    rsx! {
        style { {crate::ui::app_css()} }
        div {
            class: "cc3-root dark",
            tabindex: "0",
            onmounted: move |e| {
                // Hotkeys need a focused element to be routed to.
                spawn(async move {
                    let _ = e.set_focus(true).await;
                });
            },
            onkeydown: move |e| {
                let m = e.modifiers();
                let key = e.key().to_string();
                if (m.ctrl() || m.meta()) && key.eq_ignore_ascii_case("k") {
                    palette_query.set(String::new());
                    palette_active.set(0);
                    palette_open.set(!palette_open());
                } else if key == "Escape" && palette_open() {
                    palette_open.set(false);
                }
            },
            style: "width:100vw;height:100vh;--workspace-topbar-height:40px;--native-titlebar-height:40px;--workspace-controls-top:0px;--workspace-controls-left:0.75rem;--workspace-controls-right:calc(138px + 0.75rem);--workspace-native-controls-inset:calc(138px + 0.75rem);",
            if frame {
                WindowFrame {
                    dark: true,
                    background: "var(--background)".to_string(),
                    title_content: rsx! { div {} },
                    {content.clone()}
                }
            } else {
                {content}
            }
        }
    }
}

/// Props for mounting the root outside the binary (tests, tools).
pub fn app_props(store: Arc<Store>, frame: bool) -> AppRootProps {
    AppRootProps { store: StoreHandle(store), frame }
}
