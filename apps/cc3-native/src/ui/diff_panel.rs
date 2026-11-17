//! Right-hand diff panel: changed files of the workspace (`review.getDiffPreview`) as a unified diff.
use dioxus::prelude::*;
use serde_json::Value;

#[derive(Clone, PartialEq)]
pub struct DiffFile {
    pub path: String,
    pub additions: i64,
    pub deletions: i64,
    pub lines: Vec<String>,
}

/// Splits a unified diff into per-file sections, matched to the summary `files` by path.
pub fn files_from_preview(preview: &Value) -> Vec<DiffFile> {
    let Some(source) = preview["sources"].get(0) else { return Vec::new() };
    let diff = source["diff"].as_str().unwrap_or_default();
    let mut sections: Vec<(String, Vec<String>)> = Vec::new();
    for line in diff.lines() {
        if let Some(rest) = line.strip_prefix("diff --git ") {
            let path = rest.split(" b/").last().unwrap_or(rest).to_string();
            sections.push((path, Vec::new()));
        } else if let Some((_, lines)) = sections.last_mut() {
            if !(line.starts_with("index ") || line.starts_with("--- ") || line.starts_with("+++ ")) {
                lines.push(line.to_string());
            }
        }
    }
    sections
        .into_iter()
        .map(|(path, lines)| {
            let stat = source["files"].as_array().and_then(|f| f.iter().find(|f| f["path"] == path.as_str()));
            DiffFile {
                additions: stat.and_then(|s| s["additions"].as_i64()).unwrap_or_else(|| lines.iter().filter(|l| l.starts_with('+')).count() as i64),
                deletions: stat.and_then(|s| s["deletions"].as_i64()).unwrap_or_else(|| lines.iter().filter(|l| l.starts_with('-')).count() as i64),
                path,
                lines,
            }
        })
        .collect()
}

fn line_style(line: &str) -> &'static str {
    match line.chars().next() {
        Some('+') => "background-color: rgba(46,160,67,0.16);",
        Some('-') => "background-color: rgba(248,81,73,0.16);",
        Some('@') => "color: var(--muted-foreground); background-color: rgba(120,140,255,0.08);",
        _ => "",
    }
}

#[component]
pub fn DiffPanel(files: Vec<DiffFile>, loading: bool, error: String, onrefresh: EventHandler<()>) -> Element {
    let (adds, dels): (i64, i64) = files.iter().fold((0, 0), |(a, d), f| (a + f.additions, d + f.deletions));
    rsx! {
        div {
            "data-slot": "diff-panel",
            "class": "flex min-h-0 flex-col border-l bg-background text-foreground",
            "style": "position: fixed; top: 40px; right: 0; bottom: 0; width: 540px; z-index: 30;",
            div {
                "class": "flex h-10 shrink-0 items-center gap-2 border-b px-3 text-sm",
                span { "class": "font-medium", "Diff" }
                span { "class": "text-xs font-normal text-muted-foreground", "{files.len()} files" }
                span { "class": "text-xs font-normal", "style": "color: #3fb950;", "+{adds}" }
                span { "class": "text-xs font-normal", "style": "color: #f85149;", "-{dels}" }
                button {
                    "type": "button",
                    "class": "ms-auto cursor-pointer rounded-md px-2 py-1 text-xs text-muted-foreground hover:text-foreground",
                    onclick: move |_| onrefresh.call(()),
                    "Refresh"
                }
            }
            div {
                "class": "min-h-0 flex-1 overflow-y-auto",
                "style": "overflow: auto;",
                if loading {
                    div { "class": "p-4 text-sm text-muted-foreground", "Loading changes..." }
                } else if !error.is_empty() {
                    div { "class": "p-4 text-sm text-muted-foreground", "{error}" }
                } else if files.is_empty() {
                    div { "class": "p-4 text-sm text-muted-foreground", "No changes" }
                }
                for file in files.iter() {
                    div {
                        key: "{file.path}",
                        "class": "border-b",
                        div {
                            "class": "sticky top-0 flex items-center gap-2 bg-background px-3 py-1.5 text-xs font-medium",
                            span { "class": "min-w-0 flex-1 truncate", "{file.path}" }
                            span { "style": "color: #3fb950;", "+{file.additions}" }
                            span { "style": "color: #f85149;", "-{file.deletions}" }
                        }
                        for (i, line) in file.lines.iter().enumerate() {
                            div {
                                key: "{i}",
                                "class": "whitespace-pre px-3 font-mono text-xs leading-5",
                                "style": line_style(line),
                                "{line}"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_unified_diff_by_file() {
        let preview = serde_json::json!({"sources": [{"diff": "diff --git a/f.txt b/f.txt\nindex 1..2 100644\n--- a/f.txt\n+++ b/f.txt\n@@ -1,3 +1,4 @@\n a\n-b\n+B\n c\n+d\n", "files": [{"path": "f.txt", "additions": 2, "deletions": 1}]}]});
        let files = files_from_preview(&preview);
        assert_eq!(files.len(), 1);
        assert_eq!((files[0].additions, files[0].deletions), (2, 1));
        assert_eq!(files[0].lines[0], "@@ -1,3 +1,4 @@");
        assert_eq!(files[0].lines.len(), 6);
    }
}
