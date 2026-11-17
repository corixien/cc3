//! Container queries for Blitz, which has none. `@container NAME (width>=W) { rules }` becomes
//! `[data-cq-NAME~="ge-PX"] rules`; the document sets `data-cq-NAME` on every container after layout
//! (see `blitz_dom::container_queries`), so descendants match once the container is wide enough.

/// Rewrites the container-query blocks and returns the thresholds `(name, px)` they use.
/// The unnamed form uses the name `` and attribute `data-cq`.
pub fn rewrite(css: &str) -> (String, Vec<(String, u32)>) {
    let mut out = String::with_capacity(css.len());
    let mut thresholds: Vec<(String, u32)> = Vec::new();
    let mut rest = css;
    while let Some(at) = rest.find("@container ") {
        out.push_str(&rest[..at]);
        let after = &rest[at + "@container ".len()..];
        let Some(open) = after.find('{') else { break };
        let head = after[..open].trim();
        let Some(end) = block_end(after, open) else { break };
        let body = &after[open + 1..end];
        match parse_head(head) {
            Some((name, px, negated)) => {
                if !thresholds.contains(&(name.clone(), px)) {
                    thresholds.push((name.clone(), px));
                }
                let attr = if name.is_empty() { "data-cq".to_string() } else { format!("data-cq-{name}") };
                let token = if negated { "lt" } else { "ge" };
                out.push_str(&prefix_rules(body, &format!("[{attr}~=\"{token}-{px}\"] ")));
            }
            None => out.push_str(&format!("@container {head}{{{body}}}")),
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    (out, thresholds)
}

fn parse_head(head: &str) -> Option<(String, u32, bool)> {
    let (name, cond) = match head.find('(') {
        Some(i) => (head[..i].trim(), &head[i..]),
        None => return None,
    };
    let (name, negated) = match name.strip_suffix("not") {
        Some(n) => (n.trim(), true),
        None => (name, false),
    };
    let cond = cond.trim().trim_start_matches('(').trim_end_matches(')').replace(' ', "");
    let value = cond.strip_prefix("width>=")?;
    let px = if let Some(v) = value.strip_suffix("rem") {
        v.parse::<f32>().ok()? * 16.0
    } else if let Some(v) = value.strip_suffix("px") {
        v.parse::<f32>().ok()?
    } else {
        return None;
    };
    Some((name.to_string(), px.round() as u32, negated))
}

/// Index of the `}` that closes the `{` at `open`.
fn block_end(s: &str, open: usize) -> Option<usize> {
    let mut depth = 0;
    let bytes = s.as_bytes();
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 1,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn prefix_rules(body: &str, prefix: &str) -> String {
    let mut out = String::new();
    let mut rest = body;
    while let Some(open) = rest.find('{') {
        let selectors = rest[..open].trim();
        let Some(end) = block_end(rest, open) else { break };
        let decls = &rest[open + 1..end];
        let list: Vec<String> = split_selectors(selectors).into_iter().map(|s| format!("{prefix}{}", s.trim())).collect();
        out.push_str(&format!("{}{{{decls}}}\n", list.join(",")));
        rest = &rest[end + 1..];
    }
    out
}

fn split_selectors(s: &str) -> Vec<&str> {
    let (mut parts, mut depth, mut start) = (Vec::new(), 0, 0);
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 1,
            b'(' | b'[' => depth += 1,
            b')' | b']' => depth -= 1,
            b',' if depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    parts.push(&s[start..]);
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_named_and_unnamed() {
        let (css, t) = rewrite("a{} @container settings-row (width>=32rem) { .x\\:y, .z { color: red } } @container (width>=700px) { .w { top: 0 } } b{}");
        assert!(css.contains("[data-cq-settings-row~=\"ge-512\"] .x\\:y,[data-cq-settings-row~=\"ge-512\"] .z{ color: red }"));
        assert!(css.contains("[data-cq~=\"ge-700\"] .w{ top: 0 }"));
        assert!(css.starts_with("a{} ") && css.trim_end().ends_with("b{}"));
        assert_eq!(t, vec![("settings-row".to_string(), 512), ("".to_string(), 700)]);
    }
}
