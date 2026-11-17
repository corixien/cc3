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
    sidebar_rows(store, open, false)
}

/// Rows of the settled shelf.
pub fn settled_rows_from_store(store: &Store, open: Option<String>) -> Vec<ThreadRowData> {
    sidebar_rows(store, open, true)
}

fn sidebar_rows(store: &Store, open: Option<String>, settled: bool) -> Vec<ThreadRowData> {
    let shell = store.shell.lock().unwrap();
    let environment_id = store.environment_id.lock().unwrap().clone();
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    (if settled { shell.settled_threads() } else { shell.visible_threads() })
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let project = shell.project(&t.project_id).map(|p| p.title.clone()).unwrap_or_default();
            let monogram: String = project.chars().filter(|c| c.is_alphanumeric()).take(2).collect::<String>().to_uppercase();
            ThreadRowData {
                thread_id: t.id.clone(),
                thread_key: format!("{environment_id}:{}", t.id),
                aria_label: format!("{}, {project}", t.title),
                ui_id: format!("_r_{}{i}_", if settled { "s" } else { "" }),
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
    let mut dark = use_signal(|| crate::ui::settings::SWITCHES.read().get("__theme_dark").copied().unwrap_or(true));
    let mut diff_open = use_signal(|| false);
    let mut settled = use_signal(Vec::<ThreadRowData>::new);
    let mut shelf_open = use_signal(|| false);
    let mut sidebar_hidden = use_signal(|| false);
    let mut details_hidden = use_signal(|| false);
    let mut diff_wide = use_signal(|| false);
    let mut project_filter = use_signal(|| None::<String>);
    let mut popup = use_signal(|| None::<(String, i32, i32, Vec<(String, String)>)>);
    let mut prompt = use_signal(|| None::<String>);
    let mut prompt_value = use_signal(String::new);
    let mut menu = use_signal(|| None::<(String, i32, i32)>);
    let mut toasts = use_signal(Vec::<crate::ui::toast::ToastView>::new);
    let mut terminal_open = use_signal(|| false);
    let mut sidebar_query = use_signal(String::new);
    let mut model_open = use_signal(|| false);
    let mut model_query = use_signal(String::new);
    let mut add_project_open = use_signal(|| false);
    let mut add_project_path = use_signal(String::new);
    let mut add_project_error = use_signal(String::new);

    // CC3_AUTOTOGGLE=1: flips provider switches on the providers page (exercises re-rendering in the real window).
    use_future(move || async move {
        if std::env::var("CC3_AUTOTOGGLE").is_err() {
            return;
        }
        let mut on = false;
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(400)).await;
            on = !on;
            for label in ["Enable Codex", "Enable Claude", "Show GPT-5.4 in the model picker", "Show GPT-5.5 in the model picker"] {
                crate::ui::settings::set_switch(label, on, |_, _| {});
            }
        }
    });

    // The store is fed from other threads; rows and the open thread follow its version.
    use_future({
        move || {
            async move {
                let mut seen = u64::MAX;
                let mut last_alert: Option<String> = None;
                loop {
                    for (kind, title, description) in crate::ui::toast::drain_notices() {
                        store.notify(&kind, &title, &description);
                    }
                    if store.prune_toasts(None, std::time::Duration::from_secs(6)) {
                        toasts.set(store.toasts_view());
                    }
                    let version = store.version();
                    if version != seen {
                        seen = version;
                        toasts.set(store.toasts_view());
                        let settings = store.config.lock().unwrap()["settings"].clone();
                        crate::ui::settings::sync_from_server(&settings);
                        rows.set(rows_from_store(store, selected()));
                        settled.set(settled_rows_from_store(store, selected()));
                        let mut view = if draft() { crate::model::ThreadView::default() } else { store.thread_view().unwrap_or_default() };
                        view.model_label = store.model_label();
                        // Needs-input card (only in the real app: it starts a process of the executable).
                        if frame {
                            let key = view.pending_question.as_ref().map(|q| q.request_id.clone());
                            if key.is_some() && key != last_alert {
                                if let Some(q) = &view.pending_question {
                                    crate::shell::alert::spawn(1, "Input needed", &q.question);
                                }
                            }
                            last_alert = key;
                        }
                        if let Some((project, branch)) = store.current_project_info() {
                            view.monogram = project.chars().filter(|c| c.is_alphanumeric()).take(2).collect::<String>().to_uppercase();
                            view.project_name = project;
                            view.branch = branch;
                        }
                        thread.set(view);
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                }
            }
        }
    });

    // First run: no project yet, show the welcome page until one exists. (Locks are taken here, not inside
    // the props below, which hold the environment id guard.)
    let no_projects = store.shell.lock().unwrap().projects.is_empty() && !store.environment_id.lock().unwrap().is_empty();
    let content = rsx! {
                AppLayout {
                    rows: {
                        let q = sidebar_query().to_lowercase();
                        let project = project_filter().and_then(|id| store.projects().into_iter().find(|p| p.0 == id).map(|p| p.1));
                        rows().into_iter().filter(|r| (q.is_empty() || r.title.to_lowercase().contains(&q) || r.project.to_lowercase().contains(&q)) && project.as_ref().is_none_or(|p| &r.project == p)).collect::<Vec<_>>()
                    },
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
                    settled: settled(),
                    shelf_open: shelf_open(),
                    page: page().or_else(|| no_projects.then(|| "route-welcome".to_string())),
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
                        } else if action == "ui:shelf" {
                            shelf_open.set(!shelf_open());
                        } else if action == "question:dismiss" {
                            store.dismiss_question();
                        } else if action == "ui:sidebar" {
                            sidebar_hidden.set(!sidebar_hidden());
                        } else if action == "ui:details" {
                            details_hidden.set(!details_hidden());
                        } else if action == "ui:panel-wide" {
                            diff_wide.set(!diff_wide());
                        } else if action == "editor:open" {
                            store.open_in_editor(None);
                        } else if action == "prompt:commit" {
                            prompt_value.set(String::new());
                            prompt.set(Some("commit".into()));
                        } else if let Some(what) = action.strip_prefix("unavailable:") {
                            store.notify("error", what, "This is not available in the native app yet");
                        } else if let Some(rest) = action.strip_prefix("popup:") {
                            // popup:<kind>:<x>:<y>
                            let mut parts = rest.splitn(3, ':');
                            if let (Some(kind), Some(x), Some(y)) = (parts.next(), parts.next().and_then(|v| v.parse::<i32>().ok()), parts.next().and_then(|v| v.parse::<i32>().ok())) {
                                let items: Vec<(String, String)> = match kind {
                                    "projects" => std::iter::once(("".to_string(), "All projects".to_string())).chain(store.projects().into_iter()).collect(),
                                    "editors" => store.editors().into_iter().map(|e| (e.clone(), e)).collect(),
                                    "branches" => store.list_branches().into_iter().map(|(n, cur)| (n.clone(), if cur { format!("{n}  (current)") } else { n })).collect(),
                                    "git" => vec![("commit".into(), "Commit".into()), ("push".into(), "Push".into()), ("commit_push".into(), "Commit and push".into())],
                                    "thread" => crate::ui::context_menu::ITEMS.iter().map(|(k, l)| (k.to_string(), l.to_string())).collect(),
                                    _ => Vec::new(),
                                };
                                popup.set(Some((kind.to_string(), x, y, items)));
                            }
                        } else if action == "panel:terminal" {
                            if !terminal_open() {
                                store.terminal_attach();
                            }
                            terminal_open.set(!terminal_open());
                        } else if action == "panel:diff" {
                            if !diff_open() {
                                store.refresh_diff();
                            }
                            diff_open.set(!diff_open());
                        } else if let Some(q) = action.strip_prefix("search:") {
                            sidebar_query.set(q.to_string());
                        } else if action == "dialog:model-picker" {
                            model_query.set(String::new());
                            model_open.set(true);
                        } else if action == "dialog:add-project" {
                            add_project_path.set(String::new());
                            add_project_error.set(String::new());
                            add_project_open.set(true);
                        } else if let Some(rest) = action.strip_prefix("row:menu:") {
                            // row:menu:<x>:<y>:<thread id>
                            let mut parts = rest.splitn(3, ':');
                            if let (Some(x), Some(y), Some(id)) = (parts.next().and_then(|v| v.parse().ok()), parts.next().and_then(|v| v.parse().ok()), parts.next()) {
                                menu.set(Some((id.to_string(), x, y)));
                            }
                        } else if let Some(rest) = action.strip_prefix("row:") {
                            if let Some((what, id)) = rest.split_once(':') {
                                store.thread_action(id, what);
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
                if let Some((kind, x, y, items)) = popup() {
                    crate::ui::context_menu::ListMenu {
                        x,
                        y,
                        items,
                        onpick: {
                            let kind = kind.clone();
                            move |key: String| {
                                popup.set(None);
                                match kind.as_str() {
                                    "projects" => project_filter.set(if key.is_empty() { None } else { Some(key) }),
                                    "editors" => store.open_in_editor(Some(key)),
                                    "branches" => store.switch_branch(key),
                                    "git" if key == "commit" || key == "commit_push" => {
                                        prompt_value.set(String::new());
                                        prompt.set(Some(key));
                                    }
                                    "git" => store.git_action(&key, None),
                                    "thread" => {
                                        if let Some(id) = selected() {
                                            store.thread_action(&id, &key);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        },
                        onclose: move |_| popup.set(None),
                    }
                }
                if let Some(action) = prompt() {
                    crate::ui::add_project::AddProjectDialog {
                        title: "Commit message".to_string(),
                        hint: "Describe the changes. Leave empty to let the server write one.".to_string(),
                        placeholder: "Commit message".to_string(),
                        confirm: "Commit".to_string(),
                        value: prompt_value(),
                        error: String::new(),
                        oninput: move |v: String| prompt_value.set(v),
                        onsubmit: {
                            let action = action.clone();
                            move |_| {
                                let action = if action == "commit" || action == "commit_push" { action.clone() } else { "commit".into() };
                                store.git_action(&action, Some(prompt_value()));
                                prompt.set(None);
                            }
                        },
                        onclose: move |_| prompt.set(None),
                    }
                }
                if let Some((id, x, y)) = menu() {
                    crate::ui::context_menu::ThreadMenu {
                        x,
                        y,
                        onpick: {
                            let id = id.clone();
                            move |what: String| {
                                store.thread_action(&id, &what);
                                menu.set(None);
                            }
                        },
                        onclose: move |_| menu.set(None),
                    }
                }
                if !toasts().is_empty() {
                    crate::ui::toast::ToastStack {
                        items: toasts(),
                        ondismiss: move |id: u64| {
                            store.prune_toasts(Some(id), std::time::Duration::from_secs(3600));
                            toasts.set(store.toasts_view());
                        },
                    }
                }
                if terminal_open() {
                    {
                        let (rows, status) = match &*store.terminal.lock().unwrap() {
                            Some(t) => (crate::ui::terminal_panel::screen_runs(t.parser.screen()), t.status.clone()),
                            None => (Vec::new(), "Open a thread to start a shell".to_string()),
                        };
                        rsx! {
                            crate::ui::terminal_panel::TerminalPanel {
                                rows,
                                status,
                                onkey: move |(key, ctrl, alt): (String, bool, bool)| {
                                    if let Some(bytes) = crate::ui::terminal_panel::key_bytes(&key, ctrl, alt) {
                                        store.terminal_write(bytes);
                                    }
                                },
                            }
                        }
                    }
                }
                if diff_open() {
                    {
                        let (files, loading, error) = match &*store.diff.lock().unwrap() {
                            None => (Vec::new(), true, String::new()),
                            Some(Ok(preview)) => (crate::ui::diff_panel::files_from_preview(preview), false, String::new()),
                            Some(Err(e)) => (Vec::new(), false, e.clone()),
                        };
                        rsx! {
                            crate::ui::diff_panel::DiffPanel { files, loading, error, wide: diff_wide(), onrefresh: move |_| store.refresh_diff() }
                        }
                    }
                }
                if model_open() {
                    crate::ui::model_picker::ModelPicker {
                        options: store.model_options(),
                        current: store.model_label(),
                        query: model_query(),
                        oninput: move |q: String| model_query.set(q),
                        onpick: move |(instance, slug): (String, String)| {
                            store.select_model(&instance, &slug);
                            thread.write().model_label = store.model_label();
                            model_open.set(false);
                        },
                        onclose: move |_| model_open.set(false),
                    }
                }
                if add_project_open() {
                    crate::ui::add_project::AddProjectDialog {
                        value: add_project_path(),
                        error: add_project_error(),
                        oninput: move |v: String| add_project_path.set(v),
                        onsubmit: move |_| match store.add_project(&add_project_path()) {
                            Ok(()) => add_project_open.set(false),
                            Err(e) => add_project_error.set(e),
                        },
                        onclose: move |_| add_project_open.set(false),
                    }
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
                                } else if key == "action:toggle-theme" {
                                    let next = !dark();
                                    dark.set(next);
                                    crate::ui::settings::set_switch("__theme_dark", next, |_, _| {});
                                } else if key == "dialog:add-project" {
                                    add_project_path.set(String::new());
                                    add_project_error.set(String::new());
                                    add_project_open.set(true);
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
            class: format!("cc3-root{}{}{}", if dark() { " dark" } else { "" }, if sidebar_hidden() { " cc3-sidebar-hidden" } else { "" }, if details_hidden() { " cc3-details-hidden" } else { "" }),
            tabindex: "0",
            onmounted: move |e| {
                // Hotkeys need a focused element to be routed to (children that autofocus do so later).
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
                } else if (m.ctrl() || m.meta()) && m.shift() && key.eq_ignore_ascii_case("n") {
                    page.set(None);
                    draft.set(true);
                    selected.set(None);
                    thread.set(crate::model::ThreadView::default());
                } else if (m.ctrl() || m.meta()) && !m.shift() && key.eq_ignore_ascii_case("u") {
                    page.set(Some("route-usage".to_string()));
                } else if (m.ctrl() || m.meta()) && key == "," {
                    page.set(Some("settings-general".to_string()));
                } else if key == "Escape" && (popup().is_some() || menu().is_some() || prompt().is_some() || model_open() || add_project_open()) {
                    popup.set(None);
                    menu.set(None);
                    prompt.set(None);
                    model_open.set(false);
                    add_project_open.set(false);
                } else if key == "Escape" && palette_open() {
                    palette_open.set(false);
                } else if key == "Escape" && page().is_some() {
                    page.set(None);
                }
            },
            style: "width:100vw;height:100vh;--workspace-topbar-height:40px;--native-titlebar-height:40px;--workspace-controls-top:0px;--workspace-controls-left:0.75rem;--workspace-controls-right:calc(138px + 0.75rem);--workspace-native-controls-inset:calc(138px + 0.75rem);",
            if frame {
                WindowFrame {
                    dark: dark(),
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
