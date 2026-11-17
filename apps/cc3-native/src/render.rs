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

/// Font context with the fonts CC3 ships (Lexend Deca, Liberation Mono) plus system fallbacks.
fn font_context() -> parley::FontContext {
    use parley::fontique::Blob;
    let mut ctx = parley::FontContext::new();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.path().extension().is_some_and(|e| e == "ttf") {
                if let Ok(bytes) = std::fs::read(entry.path()) {
                    ctx.collection.register_fonts(Blob::new(std::sync::Arc::new(bytes)), None);
                }
            }
        }
    }
    ctx
}

pub fn render(html: &str, width: u32, height: u32, scheme: ColorScheme) -> Rendered {
    let mut doc = HtmlDocument::from_html(
        html,
        DocumentConfig {
            viewport: Some(Viewport::new(width, height, 1.0, scheme)),
            font_ctx: Some(font_context()),
            ..Default::default()
        },
    );
    doc.resolve(0.0);
    // The chat timeline opens scrolled to the newest message, as in the web UI.
    if let Ok(Some(scroller)) = doc.query_selector(".messages-timeline-scroll") {
        doc.scroll_to(scroller, 0.0, f64::MAX, blitz_dom::ScrollBehavior::Instant);
        doc.resolve(0.0);
    }
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
                let (x, y) = unrounded_position(doc, id);
                let s = node.unrounded_layout().size;
                out.push(("svg".to_string(), x, y, s.width, s.height));
                continue;
            }
            // Fragment hosts are layout-neutral wrappers (`display: contents`) the web DOM does not have.
            if data.attrs().iter().any(|a| &*a.name.local == "data-cc3-fragment") {
                stack.extend(node.children.iter().rev().copied());
                continue;
            }
            let (x, y) = unrounded_position(doc, id);
            let s = node.unrounded_layout().size;
            out.push((data.name.local.to_string(), x, y, s.width, s.height));
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

/// Position in the page from unrounded layout, like Chromium's fractional layout units.
fn unrounded_position(doc: &HtmlDocument, id: NodeId) -> (f32, f32) {
    let (mut x, mut y) = (0.0, 0.0);
    let mut current = Some(id);
    while let Some(cur) = current {
        let Some(node) = doc.get_node(cur) else { break };
        let l = node.unrounded_layout().location;
        let scroll = node.scroll_offset();
        x += l.x - scroll.x as f32;
        y += l.y - scroll.y as f32;
        current = node.containing_block();
    }
    (x, y)
}
