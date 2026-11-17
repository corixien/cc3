//! Pages taken from the golden markup (`fragments.rs`) and the click routing for them.
//! A fragment is rendered as real DOM nodes; clicks are resolved by hit-testing the document and
//! reading the attributes of the element under the pointer (and its ancestors).
use dioxus::prelude::*;
use dioxus_native_dom::NodeHandle;

/// What a click on a fragment means.
pub fn action_at(doc: &blitz_dom::BaseDocument, x: f32, y: f32) -> Option<String> {
    let hit = doc.hit(x, y)?;
    let mut id = Some(hit.node_id);
    let mut first = true;
    while let Some(current) = id {
        let node = doc.get_node(current)?;
        if let Some(el) = node.element_data() {
            let attr = |name: &str| el.attrs().iter().find(|a| &*a.name.local == name).map(|a| a.value.to_string());
            if let Some(section) = attr("data-settings-section") {
                return Some(format!("section:{section}"));
            }
            if let Some(action) = attr("data-cc3-action") {
                return Some(action);
            }
            if attr("role").as_deref() == Some("switch") {
                let on = attr("aria-checked").as_deref() == Some("true");
                return Some(format!("switch:{}:{}", !on, attr("aria-label").unwrap_or_default()));
            }
            // The click landed outside the dialog popup: on the backdrop or the full-screen popup frame.
            let outside = |c: &str| c.split_whitespace().any(|c| c == "dialog-backdrop" || c == "command-dialog-backdrop");
            let frame = |c: &str| first && c.split_whitespace().any(|c| c == "fixed") && c.split_whitespace().any(|c| c == "inset-0");
            if attr("class").is_some_and(|c| outside(&c) || frame(&c)) {
                return Some("close".into());
            }
            if attr("aria-label").is_some_and(|l| l == "Close" || l == "Close settings") {
                return Some("close".into());
            }
        }
        first = false;
        id = node.parent;
    }
    None
}

#[component]
pub fn Fragment(html: &'static str, onaction: EventHandler<String>, #[props(default)] overlay: bool) -> Element {
    let mut handle = use_signal(|| None::<std::rc::Rc<MountedData>>);
    let mut patched = crate::ui::settings::patch_switches(html, &crate::ui::settings::SWITCHES.read()).into_owned();
    // The welcome page's Start button adds the first project.
    if patched.contains("Welcome to T3 Code") {
        let start = patched.match_indices("Start").map(|(i, _)| i).find(|&i| patched[..i].trim_end().ends_with('>') && patched[i..].trim_start_matches("Start").trim_start().starts_with("</button"));
        if let Some(button) = start.and_then(|i| patched[..i].rfind("<button")) {
            patched.insert_str(button + "<button".len(), " data-cc3-action=\"dialog:add-project\"");
        }
    }
    rsx! {
        div {
            class: if overlay { "" } else { "contents" },
            style: "",
            "data-cc3-fragment": "",
            dangerous_inner_html: patched,
            onmounted: move |e| handle.set(Some(e.data())),
            onclick: move |e| {
                let Some(mounted) = handle() else { return };
                let Some(node) = mounted.downcast::<NodeHandle>() else { return };
                let p = e.client_coordinates();
                let action = action_at(&node.doc(), p.x as f32, p.y as f32);
                if let Some(action) = action {
                    onaction.call(action);
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::screens::{render_html, screens};

    #[test]
    fn clicks_resolve_to_actions() {
        let screen = screens().into_iter().find(|s| s.name == "settings-general").unwrap();
        let html = render_html(&screen, true);
        let doc = crate::render::load(&html, 1400, 900, blitz_traits::shell::ColorScheme::Dark);
        // "Appearance" in the settings navigation, then the close button, then the backdrop corner.
        assert_eq!(action_at(&doc, 150.0, 141.0).as_deref(), Some("section:/settings/appearance"));
        assert_eq!(action_at(&doc, 1317.0, 65.0).as_deref(), Some("close"));
        assert_eq!(action_at(&doc, 20.0, 450.0).as_deref(), Some("close"));
        assert_eq!(action_at(&doc, 700.0, 300.0), None);
    }
}
