//! Renders a standalone HTML snapshot (apps/web golden DOM export) with Blitz on the CPU and writes a PNG.
//! Usage: cc3-native <snapshot.html> <out.png> [width] [height]
use anyrender::render_to_buffer;
use anyrender_vello_cpu::VelloCpuImageRenderer;
use blitz_dom::DocumentConfig;
use blitz_html::HtmlDocument;
use blitz_paint::paint_scene;
use blitz_traits::shell::{ColorScheme, Viewport};

fn main() {
    let mut args = std::env::args().skip(1);
    let input = args.next().expect("snapshot.html");
    let output = args.next().expect("out.png");
    let width: u32 = args.next().and_then(|v| v.parse().ok()).unwrap_or(1400);
    let height: u32 = args.next().and_then(|v| v.parse().ok()).unwrap_or(900);

    let html = std::fs::read_to_string(&input).expect("read snapshot");
    let mut doc = HtmlDocument::from_html(
        &html,
        DocumentConfig {
            viewport: Some(Viewport::new(width, height, 1.0, ColorScheme::Dark)),
            ..Default::default()
        },
    );
    doc.resolve(0.0);
    let buffer = render_to_buffer::<VelloCpuImageRenderer, _>(
        |scene| paint_scene(scene, &mut doc, 1.0, width, height, 0, 0),
        width,
        height,
    );
    image::save_buffer(&output, &buffer, width, height, image::ColorType::Rgba8).expect("write png");
    println!("wrote {output}");
}
