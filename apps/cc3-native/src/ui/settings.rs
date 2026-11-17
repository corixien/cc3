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
