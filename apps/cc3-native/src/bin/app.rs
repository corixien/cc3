//! cc3: the native CC3 application (Dioxus components rendered by Blitz). Work in progress, see docs/native/README.md.
//!   cc3 [t3code:... | https://...]        start, or hand the links to the running instance
//!   cc3 --alert <id> <title> <body>       the large notification card (spawned by the app itself)
use dioxus::native::Config;
use dioxus::prelude::*;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::window::{WindowAttributes, WindowLevel};

use cc3_native::shell::{self, alert, badge, instance, ShellEvent};
use cc3_native::ui::app_layout::AppLayout;
use cc3_native::ui::sidebar_list::ThreadRowData;
use cc3_native::ui::window_frame::WindowFrame;

/// The CC3 icon is also the taskbar and Alt-Tab icon.
fn window_icon() -> Option<winit::icon::Icon> {
    let rgba = image::load_from_memory(include_bytes!("../../assets/icon-128.png")).ok()?.to_rgba8();
    let (w, h) = rgba.dimensions();
    winit::icon::RgbaIcon::new(rgba.into_raw(), w, h).ok().map(winit::icon::Icon::from)
}

fn main() {
    // Chromium at 1x on Linux: whole-pixel glyph advances.
    parley_engine::ROUND_ADVANCES.store(true, std::sync::atomic::Ordering::Relaxed);
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.first().map(String::as_str) == Some("--alert") {
        let id = args.get(1).and_then(|v| v.parse().ok()).unwrap_or(0);
        ALERT.set((id, args.get(2).cloned().unwrap_or_default(), args.get(3).cloned().unwrap_or_default())).ok();
        let attributes = WindowAttributes::default()
            .with_title(alert::TITLE)
            .with_decorations(false)
            .with_transparent(true)
            .with_resizable(false)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_surface_size(LogicalSize::new(alert::WIDTH, alert::HEIGHT))
            .with_position(LogicalPosition::new(680.0, 72.0));
        alert::keep_above();
        dioxus::native::launch_cfg(AlertApp, vec![], vec![Box::new(Config::new().with_window_attributes(attributes))]);
        return;
    }

    if matches!(instance::acquire(&instance::launch_message(args.iter().cloned())), instance::Role::Secondary) {
        return;
    }
    for arg in &args {
        if shell::is_link(arg) {
            shell::push_event(ShellEvent::OpenLink(arg.clone()));
        }
    }
    let attributes = WindowAttributes::default()
        .with_window_icon(window_icon())
        .with_title("CC3")
        .with_decorations(false)
        .with_transparent(true)
        .with_surface_size(LogicalSize::new(1400.0, 900.0))
        .with_min_surface_size(LogicalSize::new(640.0, 480.0));
    dioxus::native::launch_cfg(App, vec![], vec![Box::new(Config::new().with_window_attributes(attributes))]);
}

static ALERT: std::sync::OnceLock<(u64, String, String)> = std::sync::OnceLock::new();

#[component]
fn AlertApp() -> Element {
    let (id, title, body) = ALERT.get().cloned().unwrap_or_default();
    rsx! { alert::AlertCard { id, title, body } }
}

static STORE: std::sync::LazyLock<std::sync::Arc<cc3_native::store::Store>> = std::sync::LazyLock::new(Default::default);

/// Starts the server, authenticates and subscribes to the streams. Runs on its own thread.
fn connect() -> Result<(), String> {
    let (http, token, port) = cc3_native::net::server::start_global()?;
    let bearer = cc3_native::net::auth::bootstrap(&http, &token)?;
    let ticket = cc3_native::net::auth::websocket_ticket(&http, &bearer)?;
    let client = cc3_native::net::RpcClient::connect(
        &format!("ws://127.0.0.1:{port}"),
        &ticket,
        &cc3_native::net::rpc::ConnectParams { app_version: env!("CARGO_PKG_VERSION"), device_id: "cc3-native" },
    )?;
    let client = std::sync::Arc::new(client);
    cc3_native::store::spawn_streams(client.clone(), STORE.clone());
    // The RPC client stays alive for the app's lifetime.
    std::mem::forget(client);
    Ok(())
}

/// Sidebar rows from the shell model.
fn rows_from_store(store: &cc3_native::store::Store, open: Option<String>) -> Vec<ThreadRowData> {
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
                age: cc3_native::model::relative_age(&t.updated_at, now),
                branch: t.branch.clone().unwrap_or_default(),
                active: open.as_deref() == Some(t.id.as_str()),
            }
        })
        .collect()
}

#[component]
fn App() -> Element {
    let window = dioxus::native::use_window();
    let mut last_event = use_signal(String::new);
    let mut rows = use_signal(Vec::<ThreadRowData>::new);
    let mut status = use_signal(|| "Connecting...".to_string());
    let mut selected = use_signal(|| None::<String>);
    let mut thread = use_signal(cc3_native::model::ThreadView::default);

    use_hook(|| {
        std::thread::spawn(|| {
            if let Err(error) = connect() {
                eprintln!("connect failed: {error}");
            }
        });
    });

    // Links, activation and notification clicks arrive from other threads and processes; so do rows.
    use_future({
        let window = window.clone();
        move || {
            let window = window.clone();
            async move {
                let mut seen = u64::MAX;
                loop {
                    for event in shell::drain_events() {
                        window.set_minimized(false);
                        window.focus_window();
                        last_event.set(format!("{event:?}"));
                    }
                    let version = STORE.version();
                    if version != seen {
                        seen = version;
                        status.set(String::new());
                        rows.set(rows_from_store(&STORE, selected()));
                        thread.set(STORE.thread_view().unwrap_or_default());
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                }
            }
        }
    });

    // CC3_SELFTEST=1: exercises window controls, badge and the notification card without input.
    use_future({
        let window = window.clone();
        move || {
            let window = window.clone();
            async move {
                if std::env::var("CC3_SELFTEST").is_err() {
                    return;
                }
                let step = std::time::Duration::from_secs(4);
                tokio::time::sleep(step).await;
                window.set_maximized(true);
                tokio::time::sleep(step).await;
                window.set_maximized(false);
                badge::set(3);
                alert::spawn(7, "Input needed", "Which approach should I take?");
            }
        }
    });

    let _ = (&last_event, &status);
    rsx! {
        style { {cc3_native::ui::app_css()} }
        div {
            class: "cc3-root dark",
            style: "width:100vw;height:100vh;--workspace-topbar-height:40px;--native-titlebar-height:40px;--workspace-controls-top:0px;--workspace-controls-left:0.75rem;--workspace-controls-right:calc(138px + 0.75rem);--workspace-native-controls-inset:calc(138px + 0.75rem);",
            WindowFrame {
                dark: true,
                background: "var(--background)".to_string(),
                title_content: rsx! { div {} },
                AppLayout {
                    rows: rows(),
                    thread: thread(),
                    environment_id: STORE.environment_id.lock().unwrap().clone(),
                    composer: cc3_native::ui::composer::ComposerMode::Normal,
                    onselect: move |id: String| {
                        selected.set(Some(id.clone()));
                        rows.set(rows_from_store(&STORE, Some(id.clone())));
                        STORE.open_thread(&id);
                    },
                }
            }
        }
    }
}
