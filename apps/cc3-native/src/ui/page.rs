//! Pages taken from the golden markup (`fragments.rs`) and the click routing for them.
//! A fragment is rendered as real DOM nodes; clicks are resolved by hit-testing the document and
//! reading the attributes of the element under the pointer (and its ancestors).
use dioxus::prelude::*;
use dioxus_native_dom::NodeHandle;

fn node_attr(doc: &blitz_dom::BaseDocument, id: blitz_traits::node_id::NodeId, name: &str) -> Option<String> {
    doc.get_node(id)?.element_data()?.attrs().iter().find(|a| &*a.name.local == name).map(|a| a.value.to_string())
}

/// Key of a settings control: its label (aria-label, else id), numbered when several share it.
pub fn control_key_of(doc: &blitz_dom::BaseDocument, id: blitz_traits::node_id::NodeId) -> Option<String> {
    let label = node_attr(doc, id, "aria-label").or_else(|| node_attr(doc, id, "id")).filter(|l| !l.is_empty())?;
    let by_label = node_attr(doc, id, "aria-label").is_some();
    let name = if by_label { "aria-label" } else { "id" };
    let tag = doc.get_node(id)?.element_data()?.name.local.to_string();
    let mut n = 0;
    let mut stack = vec![doc.root_node().id];
    while let Some(cur) = stack.pop() {
        if cur == id {
            break;
        }
        let Some(node) = doc.get_node(cur) else { continue };
        if let Some(el) = node.element_data() {
            if *el.name.local == *tag && el.attrs().iter().any(|a| &*a.name.local == name && *a.value == *label) {
                n += 1;
            }
        }
        // Depth-first in document order: push children reversed.
        stack.extend(node.children.iter().rev().copied());
    }
    Some(crate::ui::settings::control_key(&label, n))
}

/// What a click on a fragment means.
pub fn action_at(doc: &blitz_dom::BaseDocument, x: f32, y: f32) -> Option<String> {
    let hit = doc.hit(x, y)?;
    let mut id = Some(hit.node_id);
    let mut first = true;
    let mut dead_button: Option<String> = None;
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
            if attr("data-slot").as_deref() == Some("select-trigger") {
                if let Some(key) = control_key_of(doc, current) {
                    return Some(format!("select:{key}:{}:{}", x as i32, y as i32));
                }
            }
            if &*el.name.local == "input" && attr("type").as_deref() == Some("range") {
                if let (Some(key), Some(rect)) = (control_key_of(doc, current), doc.get_client_bounding_rect(current)) {
                    let num = |n: &str, d: f64| attr(n).and_then(|v| v.parse::<f64>().ok()).unwrap_or(d);
                    let (min, max, step) = (num("min", 0.0), num("max", 100.0), num("step", 1.0));
                    let frac = ((x as f64 - rect.x) / rect.width.max(1.0)).clamp(0.0, 1.0);
                    let value = (min + (max - min) * frac) / step;
                    let value = value.round() * step;
                    return Some(format!("range:{key}:{value}"));
                }
            }
            if &*el.name.local == "button" {
                if let Some(label) = attr("aria-label") {
                    if crate::ui::settings_spec::STEPPERS.iter().any(|s| s.0 == label) {
                        let mut n = 0;
                        let mut stack = vec![doc.root_node().id];
                        while let Some(cur) = stack.pop() {
                            if cur == current {
                                break;
                            }
                            if let Some(nd) = doc.get_node(cur) {
                                if nd.element_data().is_some_and(|e| &*e.name.local == "button" && e.attrs().iter().any(|a| &*a.name.local == "aria-label" && *a.value == *label)) {
                                    n += 1;
                                }
                                stack.extend(nd.children.iter().rev().copied());
                            }
                        }
                        return Some(format!("step:{label}:{n}"));
                    }
                }
            }
            if dead_button.is_none() && (&*el.name.local == "button" || attr("role").as_deref() == Some("button")) {
                dead_button = Some(attr("aria-label").or_else(|| attr("title")).unwrap_or_else(|| "This button".into()));
            }
        }
        first = false;
        id = node.parent;
    }
    dead_button.map(|label| format!("unavailable:{label}"))
}

#[component]
pub fn Fragment(html: &'static str, onaction: EventHandler<String>, #[props(default)] overlay: bool) -> Element {
    let mut handle = use_signal(|| None::<std::rc::Rc<MountedData>>);
    let mut patched = crate::ui::settings::patch_switches(html, &crate::ui::settings::SWITCHES.read()).into_owned();
    patched = crate::ui::settings::patch_controls(&patched, &crate::ui::settings::VALUES.read()).into_owned();
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
            oninput: move |_| {
                let Some(mounted) = handle() else { return };
                let Some(node) = mounted.downcast::<NodeHandle>() else { return };
                let Some(doc) = node.try_doc() else { return };
                let Some(focused) = doc.get_focussed_node_id() else { return };
                let (Some(key), Some(value)) = (control_key_of(&doc, focused), doc.get_node(focused).and_then(|n| n.element_data().and_then(|e| e.text_input_data().map(|t| t.editor.text().to_string())))) else { return };
                crate::ui::settings::remember_typed(&key, &value);
            },
            onkeydown: move |e| {
                if e.key().to_string() == "Enter" {
                    onaction.call("flush".to_string());
                }
            },
            onclick: move |e| {
                let Some(mounted) = handle() else { return };
                let Some(node) = mounted.downcast::<NodeHandle>() else { return };
                let p = e.client_coordinates();
                let Some(doc) = node.try_doc() else { return };
                let action = action_at(&doc, p.x as f32, p.y as f32);
                drop(doc);
                // Typed text is committed when the user clicks elsewhere.
                onaction.call("flush".to_string());
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
