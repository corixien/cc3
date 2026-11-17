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
    let (x, y) = h.center_of("aria-label", "gpt-5.4").expect("model button");
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
