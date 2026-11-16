//! Compares native renders with the Chromium golden images and geometry exports.
use std::path::Path;

use blitz_traits::shell::ColorScheme;

use crate::render::render;

pub struct StateResult {
    pub name: String,
    pub pixel_diff_pct: f64,
    pub mean_abs: f64,
    pub geometry_ok_pct: f64,
    pub geometry_n: usize,
    pub tag_mismatch: bool,
}

pub fn compare_pixels(a: &[u8], b: &[u8]) -> (f64, f64) {
    let n = a.len() / 4;
    let (mut bad, mut sum) = (0usize, 0u64);
    for i in 0..n {
        let d: Vec<u8> = (0..3).map(|c| a[i * 4 + c].abs_diff(b[i * 4 + c])).collect();
        if d.iter().any(|&v| v > 8) {
            bad += 1;
        }
        sum += d.iter().map(|&v| v as u64).sum::<u64>();
    }
    (bad as f64 / n as f64 * 100.0, sum as f64 / (3 * n) as f64)
}

pub fn run(snapshots: &Path, goldens: &Path, out: &Path) -> Vec<StateResult> {
    std::fs::create_dir_all(out).ok();
    let mut names: Vec<String> = std::fs::read_dir(snapshots)
        .expect("snapshots dir")
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().to_str().and_then(|n| n.strip_suffix(".html")).map(String::from))
        .collect();
    names.sort();
    let mut results = Vec::new();
    for name in names {
        let html = std::fs::read_to_string(snapshots.join(format!("{name}.html"))).unwrap();
        let scheme = if name.ends_with("-dark") { ColorScheme::Dark } else { ColorScheme::Light };
        let golden = image::open(goldens.join(format!("{name}.png"))).map(|i| i.to_rgba8());
        let Ok(golden) = golden else { continue };
        let (w, h) = golden.dimensions();
        let r = render(&html, w, h, scheme);
        image::save_buffer(out.join(format!("{name}.png")), &r.rgba, w, h, image::ColorType::Rgba8).ok();
        let (pct, mean) = compare_pixels(&r.rgba, golden.as_raw());
        let (mut ok, mut n, mut mismatch) = (0usize, 0usize, false);
        if let Ok(text) = std::fs::read_to_string(snapshots.join(format!("{name}.geometry.json"))) {
            if let Ok(serde_json::Value::Array(rows)) = serde_json::from_str::<serde_json::Value>(&text) {
                mismatch = rows.len() != r.boxes.len();
                for (row, mine) in rows.iter().zip(r.boxes.iter()) {
                    let a = row.as_array().unwrap();
                    // Inline boxes live inside the text layout, not in the box tree.
                    if matches!(a[5].as_str(), Some("inline" | "contents" | "none")) {
                        continue;
                    }
                    let (cw, ch) = (a[3].as_f64().unwrap() as f32, a[4].as_f64().unwrap() as f32);
                    if cw == 0.0 && ch == 0.0 {
                        continue;
                    }
                    n += 1;
                    let (cx, cy) = (a[1].as_f64().unwrap() as f32, a[2].as_f64().unwrap() as f32);
                    if std::env::var("DEBUG_GEOM").is_ok_and(|v| name.contains(&v)) && !(a[0].as_str() == Some(&mine.0) && (cx - mine.1).abs() <= 0.5 && (cy - mine.2).abs() <= 0.5 && (cw - mine.3).abs() <= 0.5 && (ch - mine.4).abs() <= 0.5) {
                        eprintln!("#{n} {} chromium=({cx:.1},{cy:.1},{cw:.1},{ch:.1}) native={} ({:.1},{:.1},{:.1},{:.1})", a[0].as_str().unwrap_or("?"), mine.0, mine.1, mine.2, mine.3, mine.4);
                    }
                    if a[0].as_str() == Some(&mine.0)
                        && (cx - mine.1).abs() <= 0.5
                        && (cy - mine.2).abs() <= 0.5
                        && (cw - mine.3).abs() <= 0.5
                        && (ch - mine.4).abs() <= 0.5
                    {
                        ok += 1;
                    }
                }
            }
        }
        results.push(StateResult {
            name,
            pixel_diff_pct: pct,
            mean_abs: mean,
            geometry_ok_pct: if n == 0 { 0.0 } else { ok as f64 / n as f64 * 100.0 },
            geometry_n: n,
            tag_mismatch: mismatch,
        });
    }
    results
}

pub fn markdown(results: &[StateResult]) -> String {
    let mut s = String::from("| Zustand | Pixel abw. % | mittl. Abw. | Geometrie ok % | Elemente | DOM-Reihenfolge |\n|---|---|---|---|---|---|\n");
    for r in results {
        s += &format!(
            "| {} | {:.2} | {:.2} | {:.1} | {} | {} |\n",
            r.name, r.pixel_diff_pct, r.mean_abs, r.geometry_ok_pct, r.geometry_n, if r.tag_mismatch { "abweichend" } else { "gleich" }
        );
    }
    let n = results.len().max(1) as f64;
    s += &format!(
        "\nMittel: Pixel {:.2} %, Geometrie {:.1} % ({} Zustaende)\\n",
        results.iter().map(|r| r.pixel_diff_pct).sum::<f64>() / n,
        results.iter().map(|r| r.geometry_ok_pct).sum::<f64>() / n,
        results.len()
    );
    s
}
