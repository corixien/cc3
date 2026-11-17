//! cc3-native: native renderer for CC3. See docs/native/README.md.
//!   cc3-native render <snapshot.html> <out.png> [width] [height] [dark|light]
//!   cc3-native report <snapshots-dir> <goldens-dir> <out-dir>
use cc3_native::{render, report};

use blitz_traits::shell::ColorScheme;
use std::path::PathBuf;

fn main() {
    // Chromium at 1x on Linux: whole-pixel glyph advances.
    parley_engine::ROUND_ADVANCES.store(true, std::sync::atomic::Ordering::Relaxed);
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("render") => {
            let (input, output) = (&args[1], &args[2]);
            let width = args.get(3).and_then(|v| v.parse().ok()).unwrap_or(1400);
            let height = args.get(4).and_then(|v| v.parse().ok()).unwrap_or(900);
            let scheme = if args.get(5).map(String::as_str) == Some("light") { ColorScheme::Light } else { ColorScheme::Dark };
            let html = std::fs::read_to_string(input).expect("read snapshot");
            let r = render::render(&html, width, height, scheme);
            image::save_buffer(output, &r.rgba, width, height, image::ColorType::Rgba8).expect("write png");
        }
        Some("boxes") => {
            let html = std::fs::read_to_string(&args[1]).expect("read html");
            let r = render::render(&html, 1400, 900, ColorScheme::Light);
            for (tag, x, y, w, h) in r.boxes {
                println!("{tag} {x:.2} {y:.2} {w:.2} {h:.2}");
            }
        }
        Some("ui-report") => {
            let results = report::run_ui(&PathBuf::from(&args[1]), &PathBuf::from(&args[2]), &PathBuf::from(&args[3]));
            let md = report::markdown(&results);
            std::fs::write(PathBuf::from(&args[3]).join("report.md"), &md).ok();
            print!("{md}");
        }
        Some("report") => {
            let results = report::run(&PathBuf::from(&args[1]), &PathBuf::from(&args[2]), &PathBuf::from(&args[3]));
            let md = report::markdown(&results);
            std::fs::write(PathBuf::from(&args[3]).join("report.md"), &md).ok();
            print!("{md}");
        }
        _ => eprintln!("usage: cc3-native render|report ..."),
    }
}
