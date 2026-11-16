//! Offscreen rendering of a standalone HTML snapshot with Blitz (Vello CPU), plus layout boxes.
use anyrender::render_to_buffer;
use anyrender_vello_cpu::VelloCpuImageRenderer;
use blitz_dom::{DocumentConfig, NodeId};
use blitz_html::HtmlDocument;
use blitz_paint::paint_scene;
use blitz_traits::shell::{ColorScheme, Viewport};

pub struct Rendered {
    pub rgba: Vec<u8>,
    /// [tag, x, y, width, height] for every element below `<body>`, in document order.
    pub boxes: Vec<(String, f32, f32, f32, f32)>,
}

pub fn render(html: &str, width: u32, height: u32, scheme: ColorScheme) -> Rendered {
    let mut doc = HtmlDocument::from_html(
        html,
        DocumentConfig {
            viewport: Some(Viewport::new(width, height, 1.0, scheme)),
            ..Default::default()
        },
    );
    doc.resolve(0.0);
    let boxes = collect_boxes(&doc);
    let rgba = render_to_buffer::<VelloCpuImageRenderer, _>(
        |scene| paint_scene(scene, &mut doc, 1.0, width, height, 0, 0),
        width,
        height,
    );
    Rendered { rgba, boxes }
}

fn collect_boxes(doc: &HtmlDocument) -> Vec<(String, f32, f32, f32, f32)> {
    let mut out = Vec::new();
    let Some(body) = find_body(doc, doc.root_element().id) else { return out };
    let mut stack: Vec<NodeId> = doc.get_node(body).map(|n| n.children.iter().rev().copied().collect()).unwrap_or_default();
    while let Some(id) = stack.pop() {
        let Some(node) = doc.get_node(id) else { continue };
        if let Some(data) = node.element_data() {
            // Children of <svg> are not CSS boxes; the exporter skips them too.
            if &*data.name.local == "svg" {
                let p = node.absolute_position(0.0, 0.0);
                let s = node.final_layout().size;
                out.push(("svg".to_string(), p.x, p.y, s.width, s.height));
                continue;
            }
            let p = node.absolute_position(0.0, 0.0);
            let s = node.final_layout().size;
            out.push((data.name.local.to_string(), p.x, p.y, s.width, s.height));
            stack.extend(node.children.iter().rev().copied());
        }
    }
    out
}

fn find_body(doc: &HtmlDocument, from: NodeId) -> Option<NodeId> {
    let node = doc.get_node(from)?;
    if node.element_data().is_some_and(|d| &*d.name.local == "body") {
        return Some(from);
    }
    node.children.iter().find_map(|c| find_body(doc, *c))
}
