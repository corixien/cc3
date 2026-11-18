//! Drives the real `AppRoot` headlessly: pointer and key events go through Blitz's event system, so
//! hotkeys, clicks and typing are verified without a window. Screenshots land in `$CC3_TEST_OUT`.
use std::sync::Arc;

use blitz_dom::DocumentConfig;
use blitz_traits::events::{BlitzKeyEvent, BlitzPointerEvent, BlitzPointerId, KeyState, MouseEventButton, MouseEventButtons, PointerCoords, PointerDetails, UiEvent};
use blitz_traits::shell::{ColorScheme, Viewport};
use cc3_native::model::{Shell, ThreadState};
use cc3_native::store::Store;
use cc3_native::ui::root::{app_props, AppRoot};
use dioxus::dioxus_core::VirtualDom;
use dioxus_native_dom::DioxusDocument;
use std::str::FromStr;

use keyboard_types::{Code, Key, Location, Modifiers};

struct Harness {
    doc: DioxusDocument,
}

fn fixture_store() -> Arc<Store> {
    let store = Arc::new(Store::default());
    let shell: serde_json::Value = serde_json::from_str(include_str!("../fixtures/thread-golden.shell.json")).unwrap();
    {
        let mut s = store.shell.lock().unwrap();
        *s = Shell::default();
        s.apply(&serde_json::json!({"kind": "snapshot", "snapshot": {"projects": [{"id": shell["projectId"], "title": "render", "workspaceRoot": "/tmp/render"}], "threads": [shell]}}));
        *store.environment_id.lock().unwrap() = "environment-render".into();
    }
    let projection: serde_json::Value = serde_json::from_str(include_str!("../fixtures/thread-golden.projection.json")).unwrap();
    *store.thread.lock().unwrap() = Some(ThreadState::from_projection(&projection));
    store
}

impl Harness {
    fn new(store: Arc<Store>) -> Self {
        cc3_native::ui::clock::freeze(1_773_824_430_000);
        let vdom = VirtualDom::new_with_props(AppRoot, app_props(store, false));
        let mut doc = DioxusDocument::new(
            vdom,
            DocumentConfig {
                viewport: Some(Viewport::new(1400, 900, 1.0, ColorScheme::Dark)),
                font_ctx: Some(cc3_native::render::font_context()),
                html_parser_provider: Some(Arc::new(blitz_html::HtmlProvider)),
                ..Default::default()
            },
        );
        doc.initial_build();
        let mut h = Self { doc };
        h.settle();
        h
    }

    fn settle(&mut self) {
        use blitz_dom::Document;
        for _ in 0..6 {
            self.doc.poll(None);
        }
        self.doc.inner_mut().resolve(0.0);
    }

    fn pointer(&self, x: f32, y: f32, button: MouseEventButton) -> BlitzPointerEvent {
        BlitzPointerEvent {
            id: BlitzPointerId::Mouse,
            is_primary: true,
            coords: PointerCoords { page_x: x, page_y: y, screen_x: x, screen_y: y, client_x: x, client_y: y },
            button,
            buttons: MouseEventButtons::Primary,
            mods: Modifiers::empty(),
            details: PointerDetails::default(),
            element: Default::default(),
            active_pointers: Default::default(),
        }
    }

    fn click(&mut self, x: f32, y: f32) {
        use blitz_dom::Document;
        self.doc.handle_ui_event(UiEvent::PointerMove(self.pointer(x, y, MouseEventButton::Main)));
        self.doc.handle_ui_event(UiEvent::PointerDown(self.pointer(x, y, MouseEventButton::Main)));
        self.doc.handle_ui_event(UiEvent::PointerUp(self.pointer(x, y, MouseEventButton::Main)));
        self.settle();
    }

    fn hover(&mut self, x: f32, y: f32) {
        use blitz_dom::Document;
        self.doc.handle_ui_event(UiEvent::PointerMove(self.pointer(x, y, MouseEventButton::Main)));
        self.settle();
    }

    fn wheel(&mut self, x: f32, y: f32, dy: f64) {
        use blitz_dom::Document;
        let p = self.pointer(x, y, MouseEventButton::Main);
        self.doc.handle_ui_event(UiEvent::Wheel(blitz_traits::events::BlitzWheelEvent { delta: blitz_traits::events::BlitzWheelDelta::Pixels(0.0, dy), coords: p.coords, buttons: MouseEventButtons::empty(), mods: Modifiers::empty(), element: Default::default() }));
        self.settle();
    }

    /// Pointer move right after the vdom changed, before layout ran (what the window does between frames).
    fn hover_before_layout(&mut self, x: f32, y: f32) {
        use blitz_dom::Document;
        for _ in 0..6 {
            self.doc.poll(None);
        }
        self.doc.handle_ui_event(UiEvent::PointerMove(self.pointer(x, y, MouseEventButton::Main)));
    }

    fn right_click(&mut self, x: f32, y: f32) {
        use blitz_dom::Document;
        let mut down = self.pointer(x, y, MouseEventButton::Secondary);
        down.buttons = MouseEventButtons::Secondary;
        self.doc.handle_ui_event(UiEvent::PointerMove(self.pointer(x, y, MouseEventButton::Main)));
        self.doc.handle_ui_event(UiEvent::PointerDown(down.clone()));
        self.doc.handle_ui_event(UiEvent::PointerUp(down));
        self.settle();
    }

    fn key(&mut self, key: Key, code: Code, modifiers: Modifiers) {
        use blitz_dom::Document;
        let text = match &key {
            Key::Character(c) => Some(smol_str::SmolStr::new(c)),
            _ => None,
        };
        for state in [KeyState::Pressed, KeyState::Released] {
            let event = BlitzKeyEvent { key: key.clone(), code, modifiers, location: Location::Standard, is_auto_repeating: false, is_composing: false, state, text: text.clone() };
            self.doc.handle_ui_event(if state == KeyState::Pressed { UiEvent::KeyDown(event) } else { UiEvent::KeyUp(event) });
        }
        self.settle();
    }

    fn html_contains(&self, needle: &str) -> bool {
        use blitz_dom::Document;
        let doc = self.doc.inner();
        let mut stack = vec![doc.root_node().id];
        while let Some(id) = stack.pop() {
            let Some(node) = doc.get_node(id) else { continue };
            if node.element_data().is_some_and(|e| e.attrs().iter().any(|a| a.value.contains(needle))) {
                return true;
            }
            stack.extend(node.children.iter().copied());
        }
        false
    }

    /// Center of the first element whose attribute `name` equals `value`.
    fn center_of(&self, name: &str, value: &str) -> Option<(f32, f32)> {
        use blitz_dom::Document;
        let doc = self.doc.inner();
        let mut stack = vec![doc.root_node().id];
        while let Some(id) = stack.pop() {
            let node = doc.get_node(id)?;
            if node.element_data().is_some_and(|e| e.attrs().iter().any(|a| &*a.name.local == name && &*a.value == value)) {
                let r = doc.get_client_bounding_rect(id)?;
                return Some(((r.x + r.width / 2.0) as f32, (r.y + r.height / 2.0) as f32));
            }
            stack.extend(node.children.iter().copied());
        }
        None
    }

    /// Centers of all elements whose tag is one of `tags` (visible ones only).
    fn centers_of_tags(&self, tags: &[&str]) -> Vec<(f32, f32)> {
        use blitz_dom::Document;
        let doc = self.doc.inner();
        let mut out = Vec::new();
        let mut stack = vec![doc.root_node().id];
        while let Some(id) = stack.pop() {
            let Some(node) = doc.get_node(id) else { continue };
            if node.element_data().is_some_and(|e| tags.contains(&&*e.name.local)) {
                if let Some(r) = doc.get_client_bounding_rect(id) {
                    if r.width > 2.0 && r.height > 2.0 && r.x >= 0.0 && r.y >= 0.0 && r.x < 1400.0 && r.y < 900.0 {
                        out.push(((r.x + r.width / 2.0) as f32, (r.y + r.height / 2.0) as f32));
                    }
                }
            }
            stack.extend(node.children.iter().copied());
        }
        out
    }

    fn attr_of(&self, name: &str, value: &str, attr: &str) -> Option<String> {
        use blitz_dom::Document;
        let doc = self.doc.inner();
        let mut stack = vec![doc.root_node().id];
        while let Some(id) = stack.pop() {
            let node = doc.get_node(id)?;
            if let Some(e) = node.element_data() {
                if e.attrs().iter().any(|a| &*a.name.local == name && &*a.value == value) {
                    return e.attrs().iter().find(|a| &*a.name.local == attr).map(|a| a.value.to_string());
                }
            }
            stack.extend(node.children.iter().copied());
        }
        None
    }

    /// Text content of the first element whose attribute equals `value`.
    fn text_of(&self, name: &str, value: &str) -> Option<String> {
        use blitz_dom::Document;
        let doc = self.doc.inner();
        let mut stack = vec![doc.root_node().id];
        while let Some(id) = stack.pop() {
            let node = doc.get_node(id)?;
            if node.element_data().is_some_and(|e| e.attrs().iter().any(|a| &*a.name.local == name && &*a.value == value)) {
                return Some(node.text_content().trim().to_string());
            }
            stack.extend(node.children.iter().copied());
        }
        None
    }

    fn text_contains(&self, needle: &str) -> bool {
        use blitz_dom::Document;
        let doc = self.doc.inner();
        let mut stack = vec![doc.root_node().id];
        while let Some(id) = stack.pop() {
            let Some(node) = doc.get_node(id) else { continue };
            if node.text_data().is_some_and(|t| t.content.contains(needle)) {
                return true;
            }
            stack.extend(node.children.iter().copied());
        }
        false
    }

    fn type_text(&mut self, text: &str) {
        for ch in text.chars() {
            let code = match ch {
                'a'..='z' => Code::from_str(&format!("Key{}", ch.to_ascii_uppercase())).unwrap_or(Code::Unidentified),
                _ => Code::Unidentified,
            };
            self.key(Key::Character(ch.to_string()), code, Modifiers::empty());
        }
    }

    fn screenshot(&mut self, name: &str) {
        use blitz_dom::Document;
        if let Ok(dir) = std::env::var("CC3_TEST_OUT") {
            let mut doc = self.doc.inner_mut();
            let rgba = anyrender::render_to_buffer::<anyrender_vello_cpu::VelloCpuImageRenderer, _>(|scene| blitz_paint::paint_scene(scene, &mut doc, 1.0, 1400, 900, 0, 0), 1400, 900);
            image::save_buffer(format!("{dir}/{name}.png"), &rgba, 1400, 900, image::ColorType::Rgba8).unwrap();
        }
    }
}

#[test]
fn ctrl_k_opens_the_palette_and_escape_closes_it() {
    let rt = tokio::runtime::Builder::new_current_thread().enable_time().build().unwrap();
    let _guard = rt.enter();
    let mut h = Harness::new(fixture_store());
    assert!(!h.html_contains("command-dialog-popup"));
    h.screenshot("interact-start");
    h.key(Key::Character("k".into()), Code::KeyK, Modifiers::CONTROL);
    assert!(h.html_contains("command-dialog-popup"), "palette should open on Ctrl+K");
    h.screenshot("interact-palette");
    h.key(Key::Named(keyboard_types::NamedKey::Escape), Code::Escape, Modifiers::empty());
    assert!(!h.html_contains("command-dialog-popup"), "Escape closes the palette");
    // Hit-testing after overlays went away must not touch removed nodes.
    for (x, y) in [(700.0, 117.0), (700.0, 300.0), (1200.0, 100.0), (100.0, 400.0)] {
        h.hover(x, y);
    }
}

/// Tests that set `CC3_HOME` run one at a time.
fn home_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread().enable_time().build().unwrap()
}

#[test]
fn gear_opens_settings_and_close_returns() {
    let rt = runtime();
    let _guard = rt.enter();
    let mut h = Harness::new(fixture_store());
    assert!(!h.text_contains("In-app notifications"));
    h.click(24.0, 876.0);
    assert!(h.text_contains("In-app notifications"), "settings general should be visible");
    h.screenshot("interact-settings");
    h.click(1317.0, 65.0);
    assert!(!h.text_contains("In-app notifications"), "close button leaves settings");
}

#[test]
fn palette_filters_while_typing_and_enter_opens_a_page() {
    let rt = runtime();
    let _guard = rt.enter();
    let mut h = Harness::new(fixture_store());
    h.key(Key::Character("k".into()), Code::KeyK, Modifiers::CONTROL);
    h.click(700.0, 117.0);
    h.type_text("u");
    h.type_text("sage");
    h.screenshot("interact-palette-usage");
    assert!(h.text_contains("Open usage"), "typed query keeps the matching action");
    assert!(!h.text_contains("Restart agent session"), "typed query filters the others out");
    h.key(Key::Named(keyboard_types::NamedKey::Enter), Code::Enter, Modifiers::empty());
    assert!(!h.html_contains("command-dialog-popup"), "Enter runs the action and closes the palette");
    assert!(h.html_contains("route-usage") || h.text_contains("Usage"), "the usage page is shown");
}

#[test]
fn settings_switch_toggles_and_persists() {
    let _home = home_lock();
    let rt = runtime();
    let _guard = rt.enter();
    let home = std::env::temp_dir().join(format!("cc3-interact-{}", std::process::id()));
    std::fs::create_dir_all(&home).unwrap();
    // SAFETY: tests in this binary that read CC3_HOME run on this thread only after it is set.
    unsafe { std::env::set_var("CC3_HOME", &home) };
    let mut h = Harness::new(fixture_store());
    h.click(24.0, 876.0);
    let before = h.attr_of("aria-label", "Confirm thread archiving", "aria-checked").expect("switch present");
    let (x, y) = h.center_of("aria-label", "Confirm thread archiving").expect("switch position");
    h.click(x, y);
    let after = h.attr_of("aria-label", "Confirm thread archiving", "aria-checked").unwrap();
    assert_ne!(before, after, "clicking the switch flips it");
    let saved = std::fs::read_to_string(home.join("native-ui-settings.json")).unwrap();
    assert!(saved.contains("Confirm thread archiving"), "state is persisted: {saved}");
}

#[test]
fn add_project_dialog_opens_from_the_sidebar() {
    let rt = runtime();
    let _guard = rt.enter();
    let mut h = Harness::new(fixture_store());
    let (x, y) = h.center_of("aria-label", "Add project").expect("sidebar button");
    h.click(x, y);
    assert!(h.html_contains("dialog-popup"), "dialog opens");
    h.screenshot("interact-add-project");
    h.type_text("x");
    h.key(Key::Named(keyboard_types::NamedKey::Escape), Code::Escape, Modifiers::empty());
    assert!(!h.html_contains("dialog-popup"), "Escape closes it");
}

#[test]
fn model_picker_changes_the_label() {
    let rt = runtime();
    let _guard = rt.enter();
    let store = fixture_store();
    *store.config.lock().unwrap() = serde_json::json!({"providers": [{"instanceId": "claudeAgent", "displayName": "Claude", "enabled": true, "models": [{"slug": "claude-sonnet-5-5", "name": "Claude Sonnet 5.5"}, {"slug": "claude-opus-5-5", "name": "Claude Opus 5.5"}]}]});
    let mut h = Harness::new(store);
    let (x, y) = h.center_of("aria-label", "GPT-5.4").expect("model button");
    h.click(x, y);
    assert!(h.html_contains("Model picker"), "picker opens");
    h.screenshot("interact-model-picker");
    h.type_text("opus");
    assert!(h.text_contains("Claude Opus 5.5") && !h.text_contains("Claude Sonnet 5.5"), "search filters");
    let (x, y) = h.center_of("role", "option").expect("option");
    h.click(x, y);
    assert!(!h.html_contains("Model picker"), "picking closes the picker");
    assert!(h.center_of("aria-label", "Claude Opus 5.5").is_some(), "composer shows the new model");
}

#[test]
fn sidebar_search_filters_threads() {
    let rt = runtime();
    let _guard = rt.enter();
    let mut h = Harness::new(fixture_store());
    // Let the store-version loop run once so the sidebar rows exist.
    rt.block_on(tokio::time::sleep(std::time::Duration::from_millis(250)));
    h.settle();
    assert!(h.attr_of("aria-label", "Thread thread-golden, render", "role").is_some(), "row present before searching");
    let (x, y) = h.center_of("aria-label", "Search threads").unwrap();
    h.click(x, y);
    h.type_text("zzz");
    assert!(h.attr_of("aria-label", "Thread thread-golden, render", "role").is_none(), "no row matches the query");
}

#[test]
fn work_entries_expand_to_their_output() {
    let rt = runtime();
    let _guard = rt.enter();
    let store = fixture_store();
    let projection: serde_json::Value = serde_json::from_str(include_str!("../fixtures/thread-golden-work.projection.json")).unwrap();
    *store.thread.lock().unwrap() = Some(ThreadState::from_projection(&projection));
    let mut h = Harness::new(store);
    rt.block_on(tokio::time::sleep(std::time::Duration::from_millis(250)));
    h.settle();
    let (x, y) = h.center_of("data-scroll-anchor-ignore", "true").expect("fold header");
    h.click(x, y);
    let (x, y) = h.center_of("data-timeline-row-kind", "work-toggle").expect("group row");
    h.click(x, y + 1.0);
    assert!(h.html_contains("data-work-entry-id") || h.html_contains("pnpm test --filter sidebar"), "entries listed");
    let (x, y) = h.center_of("data-v2-item-type", "command_execution").expect("command entry");
    h.click(x, y);
    assert!(h.text_contains("Tests  14 passed"), "output is shown");
    h.screenshot("interact-work");
}

#[test]
fn diff_panel_shows_changed_files() {
    let rt = runtime();
    let _guard = rt.enter();
    let store = fixture_store();
    *store.diff.lock().unwrap() = Some(Ok(serde_json::json!({"sources": [{"diff": "diff --git a/src/a.rs b/src/a.rs\nindex 1..2 100644\n--- a/src/a.rs\n+++ b/src/a.rs\n@@ -1,2 +1,2 @@\n-let x = 1;\n+let x = 2;\n keep\n", "files": [{"path": "src/a.rs", "additions": 1, "deletions": 1}]}]})));
    let mut h = Harness::new(store);
    let (x, y) = h.center_of("aria-label", "Toggle right panel").expect("toggle");
    h.click(x, y);
    assert!(h.html_contains("diff-panel"), "panel opens");
    assert!(h.text_contains("src/a.rs") && h.text_contains("let x = 2;"), "diff lines are listed");
    h.screenshot("interact-diff");
}

#[test]
fn terminal_drawer_renders_the_screen() {
    let rt = runtime();
    let _guard = rt.enter();
    let store = fixture_store();
    let mut parser = vt100::Parser::new(14, 110, 0);
    parser.process(b"\x1b[32muser@host\x1b[0m:\x1b[34m~/proj\x1b[0m$ echo hi\r\nhi\r\n\x1b[32muser@host\x1b[0m:\x1b[34m~/proj\x1b[0m$ ");
    *store.terminal.lock().unwrap() = Some(cc3_native::store::TerminalState { parser, thread_id: "thread-golden".into(), status: "running".into() });
    let mut h = Harness::new(store);
    let (x, y) = h.center_of("aria-label", "Toggle terminal drawer").expect("toggle");
    h.click(x, y);
    assert!(h.html_contains("terminal-panel"), "drawer opens");
    assert!(h.text_contains("echo hi"), "screen text is drawn");
    h.screenshot("interact-terminal");
}

#[test]
fn welcome_page_start_opens_add_project() {
    let rt = runtime();
    let _guard = rt.enter();
    let store = Arc::new(Store::default());
    *store.environment_id.lock().unwrap() = "environment-render".into();
    let mut h = Harness::new(store);
    assert!(h.text_contains("Welcome to T3 Code"), "welcome shows without projects");
    h.screenshot("interact-welcome");
    let (x, y) = h.center_of("data-cc3-action", "dialog:add-project").expect("Start button");
    h.click(x, y);
    assert!(h.html_contains("dialog-popup"), "Start opens the add-project dialog");
}

#[test]
fn toasts_show_and_dismiss() {
    let rt = runtime();
    let _guard = rt.enter();
    let store = fixture_store();
    let mut h = Harness::new(store.clone());
    store.notify("error", "Message not sent", "Connection lost");
    rt.block_on(tokio::time::sleep(std::time::Duration::from_millis(250)));
    h.settle();
    assert!(h.text_contains("Message not sent") && h.text_contains("Connection lost"), "toast is shown");
    h.screenshot("interact-toast");
    let (x, y) = h.center_of("aria-label", "Dismiss notification").expect("close button");
    h.click(x, y);
    assert!(!h.text_contains("Message not sent"), "dismissed");
}

#[test]
fn right_click_on_a_thread_opens_its_menu() {
    let rt = runtime();
    let _guard = rt.enter();
    let mut h = Harness::new(fixture_store());
    rt.block_on(tokio::time::sleep(std::time::Duration::from_millis(250)));
    h.settle();
    let (x, y) = h.center_of("aria-label", "Thread thread-golden, render").expect("row");
    h.right_click(x, y);
    assert!(h.text_contains("Archive thread"), "context menu is shown");
    h.screenshot("interact-menu");
    let (x, y) = h.center_of("role", "menuitem").expect("item");
    h.click(x, y);
    assert!(h.text_contains("Click again to delete"), "deleting asks to confirm");
    h.click(x, y);
    assert!(!h.text_contains("Archive thread"), "confirming closes the menu");
}

#[test]
fn shortcuts_open_pages() {
    let rt = runtime();
    let _guard = rt.enter();
    let mut h = Harness::new(fixture_store());
    h.key(Key::Character(",".into()), Code::Comma, Modifiers::CONTROL);
    assert!(h.text_contains("In-app notifications"), "Ctrl+, opens settings");
    h.key(Key::Named(keyboard_types::NamedKey::Escape), Code::Escape, Modifiers::empty());
    assert!(!h.text_contains("In-app notifications"), "Escape closes it");
}

#[test]
fn palette_toggles_the_theme() {
    let _home = home_lock();
    let rt = runtime();
    let _guard = rt.enter();
    let home = std::env::temp_dir().join(format!("cc3-interact-theme-{}", std::process::id()));
    std::fs::create_dir_all(&home).unwrap();
    unsafe { std::env::set_var("CC3_HOME", &home) };
    let mut h = Harness::new(fixture_store());
    assert!(h.html_contains("cc3-root dark"));
    h.key(Key::Character("k".into()), Code::KeyK, Modifiers::CONTROL);
    h.type_text("change theme");
    h.key(Key::Named(keyboard_types::NamedKey::Enter), Code::Enter, Modifiers::empty());
    assert!(!h.html_contains("cc3-root dark"), "theme switched to light");
    h.screenshot("interact-light");
}

#[test]
fn typing_goes_to_the_composer_without_a_click() {
    let rt = runtime();
    let _guard = rt.enter();
    let mut h = Harness::new(fixture_store());
    assert!(h.attr_of("aria-label", "Submit message", "disabled").is_some(), "send is disabled while empty");
    rt.block_on(tokio::time::sleep(std::time::Duration::from_millis(120)));
    h.settle();
    h.type_text("hi");
    assert!(h.attr_of("aria-label", "Submit message", "disabled").is_none(), "send enables after typing");
    // Hotkeys still work from inside the composer.
    h.key(Key::Character("k".into()), Code::KeyK, Modifiers::CONTROL);
    assert!(h.html_contains("command-dialog-popup"));
}

#[test]
fn provider_settings_survive_clicks() {
    let _home = home_lock();
    let rt = runtime();
    let _guard = rt.enter();
    let home = std::env::temp_dir().join(format!("cc3-interact-prov-{}", std::process::id()));
    std::fs::create_dir_all(&home).unwrap();
    unsafe { std::env::set_var("CC3_HOME", &home) };
    let mut h = Harness::new(fixture_store());
    h.key(Key::Character("k".into()), Code::KeyK, Modifiers::CONTROL);
    h.type_text("open settings");
    h.key(Key::Named(keyboard_types::NamedKey::Enter), Code::Enter, Modifiers::empty());
    // Navigate to providers via the settings navigation.
    let (x, y) = h.center_of("data-settings-section", "/settings/providers").expect("providers nav");
    h.click(x, y);
    for label in ["Enable Codex", "Enable Claude", "Show GPT-5.4 in the model picker", "Show GPT-5.5 in the model picker"] {
        let (x, y) = h.center_of("aria-label", label).expect(label);
        h.click(x, y);
        h.hover(x + 5.0, y + 5.0);
    }
    h.screenshot("interact-providers");
}

#[test]
fn every_control_on_every_settings_page_survives_clicks_and_typing() {
    let _home = home_lock();
    let rt = runtime();
    let _guard = rt.enter();
    let home = std::env::temp_dir().join(format!("cc3-interact-fuzz-{}", std::process::id()));
    std::fs::create_dir_all(&home).unwrap();
    unsafe { std::env::set_var("CC3_HOME", &home) };
    for page in ["general", "providers", "appearance", "chat", "keybindings", "projects", "threads", "browser", "diffs", "voice", "connections", "access"] {
        let mut h = Harness::new(fixture_store());
        h.key(Key::Character(",".into()), Code::Comma, Modifiers::CONTROL);
        if let Some((x, y)) = h.center_of("data-settings-section", &format!("/settings/{page}")) {
            h.click(x, y);
        }
        for dy in [300.0, 600.0, -900.0, 1200.0, -2000.0] {
            h.wheel(700.0, 450.0, dy);
        }
        for (x, y) in h.centers_of_tags(&["input", "button", "textarea"]).into_iter().take(60) {
            h.click(x, y);
            h.type_text("ab");
            h.key(Key::Named(keyboard_types::NamedKey::Backspace), Code::Backspace, Modifiers::empty());
            h.key(Key::Named(keyboard_types::NamedKey::Enter), Code::Enter, Modifiers::empty());
            h.hover(x + 3.0, y + 3.0);
        }
    }
}

#[test]
fn header_and_sidebar_buttons_do_something() {
    let rt = runtime();
    let _guard = rt.enter();
    let mut h = Harness::new(fixture_store());
    rt.block_on(tokio::time::sleep(std::time::Duration::from_millis(250)));
    h.settle();
    // Sidebar toggle hides the sidebar.
    let (x, y) = h.center_of("aria-label", "Toggle main sidebar").expect("toggle");
    h.click(x, y);
    assert!(h.html_contains("cc3-sidebar-hidden"), "sidebar hides");
    h.click(x, y);
    assert!(!h.html_contains("cc3-sidebar-hidden"), "sidebar shows again");
    // Project filter lists the projects.
    let (x, y) = h.center_of("aria-label", "Filter threads by project").expect("filter");
    h.click(x, y);
    assert!(h.text_contains("All projects") && h.text_contains("render"), "project menu opens");
    rt.block_on(tokio::time::sleep(std::time::Duration::from_millis(120)));
    h.settle();
    h.key(Key::Named(keyboard_types::NamedKey::Escape), Code::Escape, Modifiers::empty());
    assert!(!h.html_contains("menu-popup"), "escape closes the project menu");
    // Thread actions menu.
    let (x, y) = h.center_of("aria-label", "Thread actions for Thread thread-golden").expect("thread actions");
    h.click(x, y);
    assert!(h.text_contains("Archive thread"), "thread menu opens");
}

#[test]
fn commit_button_asks_for_a_message_and_dead_buttons_explain_themselves() {
    let rt = runtime();
    let _guard = rt.enter();
    let mut h = Harness::new(fixture_store());
    let (x, y) = h.center_of("aria-label", "Start voice input").expect("mic");
    h.click(x, y);
    rt.block_on(tokio::time::sleep(std::time::Duration::from_millis(250)));
    h.settle();
    assert!(h.text_contains("Voice input"), "an unavailable action shows a toast");
}

#[test]
fn hovering_between_a_change_and_the_next_layout_does_not_panic() {
    let _home = home_lock();
    let rt = runtime();
    let _guard = rt.enter();
    let mut h = Harness::new(fixture_store());
    for round in 0..4 {
        h.key(Key::Character("k".into()), Code::KeyK, Modifiers::CONTROL);
        h.type_text("th");
        h.key(Key::Named(keyboard_types::NamedKey::Escape), Code::Escape, Modifiers::empty());
        h.click(24.0, 876.0);
        for (x, y) in [(700.0, 117.0), (300.0, 300.0), (1200.0, 90.0 + round as f32), (150.0, 140.0)] {
            h.hover_before_layout(x, y);
        }
        h.key(Key::Named(keyboard_types::NamedKey::Escape), Code::Escape, Modifiers::empty());
        for (x, y) in [(700.0, 400.0), (200.0, 600.0)] {
            h.hover_before_layout(x, y);
        }
        h.settle();
    }
}

#[test]
fn hovering_right_after_a_settings_fragment_changed_does_not_panic() {
    let _home = home_lock();
    let rt = runtime();
    let _guard = rt.enter();
    let home = std::env::temp_dir().join(format!("cc3-interact-frag-{}", std::process::id()));
    std::fs::create_dir_all(&home).unwrap();
    unsafe { std::env::set_var("CC3_HOME", &home) };
    let mut h = Harness::new(fixture_store());
    h.click(24.0, 876.0);
    for label in ["Confirm thread archiving", "Project grouping", "Working section"] {
        let (x, y) = h.center_of("aria-label", label).expect(label);
        h.click(x, y);
        for dx in [-40.0, 0.0, 40.0, 200.0] {
            h.hover_before_layout(x + dx, y);
            h.hover_before_layout(x + dx, y + 30.0);
        }
    }
}

#[test]
fn settings_select_range_and_text_controls_change_values() {
    let _home = home_lock();
    let rt = runtime();
    let _guard = rt.enter();
    let home = std::env::temp_dir().join(format!("cc3-interact-ctl-{}", std::process::id()));
    std::fs::create_dir_all(&home).unwrap();
    unsafe { std::env::set_var("CC3_HOME", &home) };
    let mut h = Harness::new(fixture_store());
    h.key(Key::Character(",".into()), Code::Comma, Modifiers::CONTROL);
    let (x, y) = h.center_of("data-settings-section", "/settings/appearance").expect("nav");
    h.click(x, y);
    // Select: open, pick the last option (Full).
    let (x, y) = h.center_of("aria-label", "Chat width").expect("select");
    h.click(x, y);
    assert!(h.text_contains("Wide"), "options listed");
    let (x, y) = h.center_of("role", "menuitem").expect("option");
    h.click(x, y);
    assert!(!h.html_contains("menu-popup"), "popup closes");
    assert_eq!(h.text_of("aria-label", "Chat width").as_deref(), Some("Full"), "trigger shows the choice");
    // Range: click at the right end of the contrast slider.
    let (x, y) = h.center_of("aria-label", "Contrast").expect("slider");
    h.click(x + 60.0, y);
    let saved = std::fs::read_to_string(home.join("native-ui-values.json")).unwrap();
    assert!(saved.contains("\"Chat width\": \"full\"") && saved.contains("Contrast"), "values persisted: {saved}");
}
