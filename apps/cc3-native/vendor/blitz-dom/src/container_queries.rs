//! CC3: container queries by attribute emulation. The app rewrites `@container NAME (width>=W)` rules into
//! `[data-cq-NAME~="ge-W"]` selectors (see `cc3_native::ui::cq`). After layout this pass measures every
//! container (an element whose class list has `@container` or `@container/NAME`) and sets its
//! `data-cq-NAME` attribute to the matching tokens (`ge-W` when the width is >= W, `lt-W` otherwise).
//! A changed attribute restyles the subtree and `resolve` runs again.

use std::sync::RwLock;

use markup5ever::{LocalName, QualName, ns};

use crate::BaseDocument;

static THRESHOLDS: RwLock<Vec<(String, u32)>> = RwLock::new(Vec::new());

/// Sets the `(container name, px)` thresholds the stylesheet uses; the unnamed container uses the name "".
pub fn set_container_thresholds(thresholds: Vec<(String, u32)>) {
    *THRESHOLDS.write().unwrap() = thresholds;
}

impl BaseDocument {
    /// Returns true when an attribute changed and the document needs another resolve.
    pub(crate) fn apply_container_queries(&mut self) -> bool {
        let thresholds = THRESHOLDS.read().unwrap();
        if thresholds.is_empty() {
            return false;
        }
        let mut updates: Vec<(blitz_traits::node_id::NodeId, QualName, String)> = Vec::new();
        for (id, node) in self.nodes.iter() {
            let Some(el) = node.element_data() else { continue };
            let Some(class) = el.attr(LocalName::from("class")) else { continue };
            if !class.contains("@container") {
                continue;
            }
            let width = node.final_layout().content_box_width();
            // `@container/NAME` carries the name; a bare `@container` is the unnamed container.
            let mut names: Vec<&str> = Vec::new();
            for token in class.split_whitespace() {
                if token == "@container" {
                    names.push("");
                } else if let Some(name) = token.strip_prefix("@container/") {
                    names.push(name);
                }
            }
            for name in names {
                // The unnamed query applies to any container, so every container also carries `data-cq`.
                for target in [name, ""] {
                    let value = thresholds
                        .iter()
                        .filter(|(n, _)| n == target)
                        .map(|(_, px)| if width >= *px as f32 { format!("ge-{px}") } else { format!("lt-{px}") })
                        .collect::<Vec<_>>()
                        .join(" ");
                    let attr = if target.is_empty() { "data-cq".to_string() } else { format!("data-cq-{target}") };
                    let qual = QualName::new(None, ns!(), LocalName::from(attr.as_str()));
                    if el.attr(qual.local.clone()) != Some(value.as_str()) {
                        updates.push((id, qual, value));
                    }
                }
            }
        }
        drop(thresholds);
        if updates.is_empty() {
            return false;
        }
        let mut mutator = self.mutate();
        for (id, name, value) in &updates {
            mutator.set_attribute(*id, name.clone(), value);
        }
        true
    }
}
