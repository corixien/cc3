//! Pages taken from the golden markup (`fragments.rs`) and the click routing for them.
//! A fragment is rendered as real DOM nodes; clicks are resolved by hit-testing the document and
//! reading the attributes of the element under the pointer (and its ancestors).
use dioxus::prelude::*;
use dioxus_native_dom::NodeHandle;

/// What a click on a fragment means.
pub fn action_at(doc: &blitz_dom::BaseDocument, x: f32, y: f32) -> Option<String> {
    let hit = doc.hit(x, y)?;
    let mut id = Some(hit.node_id);
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
            if attr("class").is_some_and(|c| c.split_whitespace().any(|c| c == "dialog-backdrop" || c == "command-dialog-backdrop")) {
                return Some("close".into());
            }
            if attr("aria-label").is_some_and(|l| l == "Close" || l == "Close settings") {
                return Some("close".into());
            }
        }
        id = node.parent;
    }
    None
}

#[component]
pub fn Fragment(html: &'static str, onaction: EventHandler<String>, #[props(default)] overlay: bool) -> Element {
    let mut handle = use_signal(|| None::<std::rc::Rc<MountedData>>);
    rsx! {
        div {
            class: if overlay { "" } else { "contents" },
            style: "",
            "data-cc3-fragment": "",
            dangerous_inner_html: html,
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
