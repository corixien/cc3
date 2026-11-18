//! Live state for the settings pages. The pages are golden markup (`fragments.rs`); switches are
//! patched into that markup from `SWITCHES`, which is persisted next to the server data and, for the
//! settings the server owns, written through `server.updateSettings`.
use std::borrow::Cow;
use std::collections::HashMap;

use dioxus::prelude::*;

/// Switch label -> server setting key (`ServerSettings` in packages/contracts).
const SERVER_KEYS: &[(&str, &str)] = &[
    ("Check provider versions", "enableProviderUpdateChecks"),
    ("Continue threads after restarts", "continueThreadsAfterServerUpdate"),
    ("Keep awake while agents work", "keepAwakeWhileAgentsWork"),
    ("Agent browser access", "enableAgentBrowserAccess"),
    ("Guest links", "guestSharingEnabled"),
    ("Linked environments", "linkedEnvironmentsEnabled"),
    ("Suggest next prompt", "promptSuggestionsEnabled"),
    ("Simlock device hosts", "simlockDeviceHostsEnabled"),
    ("Agent device access", "enableAgentDeviceAccess"),
    ("Device hub", "enableDeviceSupport"),
    ("Resume after usage limits", "autoResumeLimitedThreads"),
    ("Snooze until limits reset", "snoozeLimitedThreads"),
    ("Auto-settle merged threads", "sidebarAutoSettleOnMerge"),
    ("Default automatic pull", "defaultAutoPull"),
    ("Remove agent credits when merging", "removeAgentCreditsOnMerge"),
    ("Review guides", "reviewGuidesEnabled"),
    ("Start new worktrees from remote by default", "newWorktreesStartFromOrigin"),
    ("Clone into host folders", "ghqCloneLayout"),
];

pub static SWITCHES: GlobalSignal<HashMap<String, bool>> = Signal::global(load);

fn path() -> std::path::PathBuf {
    crate::net::server::home().join("native-ui-settings.json")
}

fn load() -> HashMap<String, bool> {
    std::fs::read_to_string(path()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

fn save(map: &HashMap<String, bool>) {
    if let Ok(text) = serde_json::to_string_pretty(map) {
        let _ = std::fs::write(path(), text);
    }
}

/// Takes the server-owned switches from the server's settings (the server is the source of truth).
pub fn sync_from_server(settings: &serde_json::Value) {
    for (label, key) in SERVER_KEYS {
        let Some(value) = settings[*key].as_bool() else { continue };
        if SWITCHES.read().get(*label) != Some(&value) {
            SWITCHES.write().insert((*label).to_string(), value);
        }
    }
}

/// Provider switches ("Enable Codex") follow the providers the server reports.
pub fn sync_providers(providers: &serde_json::Value) {
    for p in providers.as_array().into_iter().flatten() {
        let (Some(name), Some(enabled)) = (p["displayName"].as_str(), p["enabled"].as_bool()) else { continue };
        let label = format!("Enable {name}");
        if SWITCHES.read().get(&label) != Some(&enabled) {
            SWITCHES.write().insert(label, enabled);
        }
    }
}

/// Sets a switch, persists it and forwards server-owned settings.
pub fn set_switch(label: &str, value: bool, send: impl FnOnce(&str, bool)) {
    let mut map = SWITCHES.write();
    map.insert(label.to_string(), value);
    save(&map);
    if let Some((_, key)) = SERVER_KEYS.iter().find(|(l, _)| *l == label) {
        send(key, value);
    }
}

/// Rewrites `data-checked`/`data-unchecked` and `aria-checked` of every switch that has an override.
pub fn patch_switches<'a>(html: &'a str, overrides: &HashMap<String, bool>) -> Cow<'a, str> {
    if overrides.is_empty() || !html.contains("role=\"switch\"") {
        return Cow::Borrowed(html);
    }
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(at) = rest.find("data-slot=\"switch\"") {
        // The switch element starts at the last `<span` before this attribute and ends after its thumb.
        let Some(start) = rest[..at].rfind("<span") else { break };
        let Some(thumb_end) = rest[at..].find("</span") .and_then(|a| rest[at + a + 6..].find("</span").map(|b| at + a + 6 + b + 6)) else { break };
        out.push_str(&rest[..start]);
        let element = &rest[start..thumb_end];
        let label = element.split("aria-label=\"").nth(1).and_then(|s| s.split('"').next());
        match label.and_then(|l| overrides.get(l)) {
            Some(&on) => {
                let (from, to) = if on { ("data-unchecked=\"\"", "data-checked=\"\"") } else { ("data-checked=\"\"", "data-unchecked=\"\"") };
                out.push_str(&element.replace(from, to).replace("aria-checked=\"true\"", &format!("aria-checked=\"{on}\"")).replace("aria-checked=\"false\"", &format!("aria-checked=\"{on}\"")));
            }
            None => out.push_str(element),
        }
        rest = &rest[thumb_end..];
    }
    out.push_str(rest);
    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patches_switch_state() {
        let html = r#"<span data-unchecked="" role="switch" aria-checked="false" data-slot="switch" aria-label="A"><span data-unchecked="" data-slot="switch-thumb"></span></span><input>"#;
        let mut m = HashMap::new();
        m.insert("A".to_string(), true);
        let out = patch_switches(html, &m);
        assert!(out.contains("data-checked=\"\" role=\"switch\" aria-checked=\"true\""));
        assert!(out.contains("<span data-checked=\"\" data-slot=\"switch-thumb\">"));
        assert!(!out.contains("data-unchecked"));
    }
}

// ---------------------------------------------------------------------------------------------
// Selects, text, number and range controls

use crate::ui::settings_spec;

/// Values of selects (option value) and inputs (text), by control key. Persisted next to the server data.
pub static VALUES: GlobalSignal<HashMap<String, String>> = Signal::global(load_values);

/// Text typed into inputs since the last flush. Not reactive: re-rendering a page while typing would
/// replace the focused input.
static TYPED: std::sync::Mutex<Vec<(String, String)>> = std::sync::Mutex::new(Vec::new());

fn values_path() -> std::path::PathBuf {
    crate::net::server::home().join("native-ui-values.json")
}

fn load_values() -> HashMap<String, String> {
    std::fs::read_to_string(values_path()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

fn save_values(map: &HashMap<String, String>) {
    if let Ok(text) = serde_json::to_string_pretty(map) {
        let _ = std::fs::write(values_path(), text);
    }
}

/// Key of the `n`-th control with this label on a page.
pub fn control_key(label: &str, n: usize) -> String {
    if n == 0 { label.to_string() } else { format!("{label}#{n}") }
}

/// Sets a control value, persists it and forwards server-owned settings through `send(key, json)`.
pub fn set_value(key: &str, value: &str, send: impl FnOnce(&str, serde_json::Value)) {
    TYPED.lock().unwrap().retain(|(k, _)| k != key);
    {
        let mut map = VALUES.write();
        map.insert(key.to_string(), value.to_string());
        save_values(&map);
    }
    let base = key.split('#').next().unwrap_or(key);
    if key.starts_with("provider-instance-") {
        send(&format!("provider:{key}"), serde_json::Value::from(value));
        return;
    }
    if let Some(server_key) = settings_spec::select_spec(base).and_then(|s| s.server).or_else(|| settings_spec::TEXTS.iter().find(|t| t.0 == base).map(|t| t.1)) {
        let numeric = settings_spec::TEXTS.iter().any(|t| t.0 == base && t.2);
        let json = if numeric {
            value.trim().parse::<i64>().map(serde_json::Value::from).unwrap_or(serde_json::Value::Null)
        } else if server_key == "pullRequestMergeMethod" && value == "last" {
            serde_json::Value::Null
        } else {
            serde_json::Value::from(value)
        };
        send(server_key, json);
    }
}

/// Remembers text typed into an input without touching reactive state.
pub fn remember_typed(key: &str, value: &str) {
    let mut typed = TYPED.lock().unwrap();
    typed.retain(|(k, _)| k != key);
    typed.push((key.to_string(), value.to_string()));
}

/// Commits everything typed so far (on Enter or when focus leaves an input).
pub fn flush_typed(send: impl Fn(&str, serde_json::Value)) {
    let pending: Vec<(String, String)> = std::mem::take(&mut *TYPED.lock().unwrap());
    for (key, value) in pending {
        set_value(&key, &value, |k, v| send(k, v));
    }
}

/// Takes server-owned values (selects and texts) from the server's settings.
pub fn sync_values_from_server(settings: &serde_json::Value) {
    let mut changes = Vec::new();
    for spec in settings_spec::SELECTS {
        let Some(key) = spec.server else { continue };
        let value = match &settings[key] {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Null if key == "pullRequestMergeMethod" => "last".to_string(),
            _ => continue,
        };
        changes.push((spec.aria.to_string(), value));
    }
    for (aria, key, numeric) in settings_spec::TEXTS {
        let value = match &settings[*key] {
            serde_json::Value::String(s) => s.clone(),
            v if *numeric && v.is_number() => v.to_string(),
            _ => continue,
        };
        changes.push((aria.to_string(), value));
    }
    let differs = changes.iter().any(|(k, v)| VALUES.read().get(k) != Some(v));
    if differs {
        let mut map = VALUES.write();
        for (k, v) in changes {
            map.insert(k, v);
        }
    }
}

fn attr_in<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!("{name}=\"");
    let start = tag.find(&key)? + key.len();
    Some(&tag[start..start + tag[start..].find('"')?])
}

fn replace_attr(tag: &str, name: &str, value: &str) -> String {
    let escaped = value.replace('&', "&amp;").replace('"', "&quot;");
    let key = format!("{name}=\"");
    match tag.find(&key) {
        Some(at) => {
            let start = at + key.len();
            let end = start + tag[start..].find('"').unwrap_or(0);
            format!("{}{escaped}{}", &tag[..start], &tag[end..])
        }
        None => format!("{} {name}=\"{escaped}\"", tag.trim_end_matches('>').trim_end_matches('/').trim_end()) + if tag.ends_with("/>") { " />" } else { ">" },
    }
}

/// Applies stored select labels, input values and slider positions to a settings page.
pub fn patch_controls<'a>(html: &'a str, values: &HashMap<String, String>) -> Cow<'a, str> {
    if values.is_empty() {
        return Cow::Borrowed(html);
    }
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    let mut seen: HashMap<String, usize> = HashMap::new();
    let typed = TYPED.lock().unwrap().clone();
    let mut value_of = |key: &str| typed.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone()).or_else(|| values.get(key).cloned());
    loop {
        let select_at = rest.find("data-slot=\"select-trigger\"");
        let input_at = rest.find("<input");
        let next = match (select_at, input_at) {
            (None, None) => break,
            (Some(a), None) => (a, true),
            (None, Some(b)) => (b, false),
            (Some(a), Some(b)) => if a < b { (a, true) } else { (b, false) },
        };
        let (pos, is_select) = next;
        if is_select {
            let Some(tag_start) = rest[..pos].rfind("<button") else { out.push_str(&rest[..pos + 1]); rest = &rest[pos + 1..]; continue };
            let tag_end = pos + rest[pos..].find('>').unwrap_or(0) + 1;
            let tag = &rest[tag_start..tag_end];
            let aria = attr_in(tag, "aria-label").unwrap_or_default().to_string();
            let n = { let c = seen.entry(aria.clone()).or_insert(0); let n = *c; *c += 1; n };
            let key = control_key(&aria, n);
            let chosen = value_of(&key).and_then(|v| settings_spec::select_spec(&aria).and_then(|s| s.options.iter().find(|o| o.0 == v).map(|o| o.1)));
            if let (Some(label), Some(v_at)) = (chosen, rest[tag_end..].find("data-slot=\"select-value\"")) {
                let v_at = tag_end + v_at;
                let open_end = v_at + rest[v_at..].find('>').unwrap_or(0) + 1;
                let close = open_end + rest[open_end..].find("</span").unwrap_or(0);
                out.push_str(&rest[..open_end]);
                out.push_str(label);
                rest = &rest[close..];
            } else {
                out.push_str(&rest[..tag_end]);
                rest = &rest[tag_end..];
            }
        } else {
            let tag_end = pos + rest[pos..].find('>').unwrap_or(rest.len() - pos - 1) + 1;
            let tag = &rest[pos..tag_end];
            let skip = tag.contains("aria-hidden=\"true\"") || attr_in(tag, "type") == Some("checkbox");
            let label = attr_in(tag, "aria-label").or_else(|| attr_in(tag, "id")).unwrap_or_default().to_string();
            if skip || label.is_empty() {
                out.push_str(&rest[..tag_end]);
                rest = &rest[tag_end..];
                continue;
            }
            let n = { let c = seen.entry(label.clone()).or_insert(0); let n = *c; *c += 1; n };
            let key = control_key(&label, n);
            out.push_str(&rest[..pos]);
            match value_of(&key) {
                Some(v) => {
                    let mut t = replace_attr(tag, "value", &v);
                    if attr_in(tag, "type") == Some("range") {
                        let num = |name: &str, d: f64| attr_in(tag, name).and_then(|x| x.parse::<f64>().ok()).unwrap_or(d);
                        let (min, max) = (num("min", 0.0), num("max", 100.0));
                        let pct = if max > min { ((v.parse::<f64>().unwrap_or(min) - min) / (max - min) * 100.0).clamp(0.0, 100.0) } else { 0.0 };
                        if let Some(sp) = t.find("--settings-slider-progress:") {
                            let end = sp + t[sp..].find(';').unwrap_or(t.len() - sp);
                            t = format!("{}--settings-slider-progress: {pct}%{}", &t[..sp], &t[end..]);
                        }
                    }
                    out.push_str(&t);
                }
                None => out.push_str(tag),
            }
            rest = &rest[tag_end..];
        }
    }
    out.push_str(rest);
    Cow::Owned(out)
}

#[cfg(test)]
mod control_tests {
    use super::*;

    #[test]
    fn patches_select_label_and_input_value() {
        let html = r#"<button role="combobox" data-slot="select-trigger" aria-label="Chat width"><span data-slot="select-value" class="x">Comfortable</span><span></span></button><input aria-label="Branch prefix" value="t3" />"#;
        let mut v = HashMap::new();
        v.insert("Chat width".to_string(), "wide".to_string());
        v.insert("Branch prefix".to_string(), "me".to_string());
        let out = patch_controls(html, &v);
        assert!(out.contains(">Wide</span"), "{out}");
        assert!(out.contains(r#"value="me""#), "{out}");
    }
}

// ---------------------------------------------------------------------------------------------
// Pages that list live data (projects, archived threads)

#[derive(Clone, Default, PartialEq)]
pub struct Live {
    /// (id, title, workspace root)
    pub projects: Vec<(String, String, String)>,
    /// (id, title, project title)
    pub archived: Vec<(String, String, String)>,
}

pub static LIVE: GlobalSignal<Live> = Signal::global(Live::default);

fn esc(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Fills the projects and archived-threads pages from live data.
pub fn patch_dynamic(html: &str, live: &Live) -> String {
    let mut out = html.to_string();
    // Projects: the golden shows one project row; repeat it per project.
    if let Some(at) = out.find("/workspace/render") {
        if let (Some(start), Some(end)) = (out[..at].rfind("<button"), out[at..].find("</button>").map(|e| at + e + "</button>".len())) {
            let template = out[start..end].to_string();
            let rows: String = live
                .projects
                .iter()
                .map(|(_, title, root)| template.replace("/workspace/render", &esc(root)).replace(">render<", &format!(">{}<", esc(title))))
                .collect();
            out.replace_range(start..end, &rows);
        }
    }
    // Archived threads: replace the loading line and list the threads.
    if out.contains("Loading archived threads") {
        let heading = if live.archived.is_empty() { "No archived threads" } else { "Archived threads" };
        out = out.replace("Loading archived threads", heading);
        if !live.archived.is_empty() {
            let rows: String = live
                .archived
                .iter()
                .map(|(id, title, project)| {
                    format!(
                        "<div class=\"flex items-center gap-3 border-t px-3 py-3 sm:px-4\"><span class=\"min-w-0 flex-1\"><span class=\"block truncate text-sm font-medium\">{}</span><span class=\"block truncate text-xs text-muted-foreground\">{}</span></span><button type=\"button\" data-cc3-action=\"restore:{}\" class=\"inline-flex h-7 cursor-pointer items-center rounded-lg border border-input px-3 text-sm\">Restore</button></div>",
                        esc(title),
                        esc(project),
                        esc(id)
                    )
                })
                .collect();
            let block = format!("<div class=\"mx-3 mt-3 rounded-xl border border-border/60 bg-card/40\">{rows}</div>");
            match out.rfind("<span data-type=\"inside\"") {
                Some(at) => out.insert_str(at, &block),
                None => out.push_str(&block),
            }
        }
    }
    out
}
