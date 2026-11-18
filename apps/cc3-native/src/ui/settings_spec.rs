//! What each settings control offers: select options and the server setting a control mirrors.
//! Controls are identified by their `aria-label` (the golden pages label every select and input).

pub type Options = &'static [(&'static str, &'static str)];

pub struct SelectSpec {
    pub aria: &'static str,
    /// (value, label shown)
    pub options: Options,
    /// `ServerSettings` key the value mirrors, if the server owns it.
    pub server: Option<&'static str>,
}

const fn sel(aria: &'static str, options: Options, server: Option<&'static str>) -> SelectSpec {
    SelectSpec { aria, options, server }
}

const SEARCH_ENGINES: Options = &[("duckduckgo", "DuckDuckGo"), ("google", "Google"), ("bing", "Bing"), ("brave", "Brave"), ("kagi", "Kagi"), ("ecosia", "Ecosia"), ("custom", "Custom")];
const LINK_TARGET: Options = &[("system", "Your default browser"), ("app", "CC3")];
const FPS: Options = &[("15", "15 fps"), ("24", "24 fps"), ("30", "30 fps"), ("60", "60 fps")];
const LANGUAGES: Options = &[("system", "System language"), ("en", "English"), ("de", "Deutsch"), ("fr", "Francais"), ("es", "Espanol"), ("pt", "Portugues"), ("ja", "Japanese"), ("zh", "Chinese")];
const TIME_FORMAT: Options = &[("locale", "System default"), ("12-hour", "12-hour"), ("24-hour", "24-hour")];
const WHEN: Options = &[("in", "In"), ("today", "Today at"), ("tomorrow", "Tomorrow at"), ("monday", "Monday at"), ("friday", "Friday at")];
const UNITS: Options = &[("minutes", "minutes"), ("hours", "hours"), ("days", "days")];
const FONT_SIZES: Options = &[("12", "12 px"), ("13", "13 px"), ("14", "14 px"), ("15", "15 px"), ("16", "16 px"), ("17", "17 px"), ("18", "18 px"), ("20", "20 px")];
const CODE_SIZES: Options = &[("11", "11 px"), ("12", "12 px"), ("13", "13 px"), ("14", "14 px"), ("15", "15 px"), ("16", "16 px")];

pub static SELECTS: &[SelectSpec] = &[
    sel("Chat width", &[("comfortable", "Comfortable"), ("wide", "Wide"), ("full", "Full")], None),
    sel("Effort display", &[("label", "Word"), ("bars", "Bars")], None),
    sel("Interface font size", FONT_SIZES, None),
    sel("Code font size", CODE_SIZES, None),
    sel("Open links in", LINK_TARGET, None),
    sel("Search engine", SEARCH_ENGINES, None),
    sel("Browser recording frame rate", FPS, None),
    sel("Send shortcut", &[("enter", "Enter"), ("mod-enter-multiline", "Ctrl+Enter (Enter adds a line)"), ("mod-enter", "Ctrl+Enter")], None),
    sel("Follow-up behavior", &[("queue", "Queue"), ("steer", "Steer")], None),
    sel("Tool calls in chat", &[("showAll", "Show all"), ("hideSuccessful", "Hide successful"), ("hideAll", "Hide all"), ("zen", "Zen")], None),
    sel("Screen reader announcements", &[("status", "Status only"), ("messages", "Messages"), ("everything", "Everything")], None),
    sel("Diff layout", &[("stacked", "Stacked"), ("split", "Split")], None),
    sel("Default diff file state", &[("expanded", "Expanded"), ("collapsed", "Collapsed")], None),
    sel("Diff colors", &[("red-green", "Red & green"), ("blue-orange", "Blue & orange")], None),
    sel("Thread notifications", &[("off", "Off"), ("notifications", "Notifications"), ("sound", "Sound"), ("notifications-and-sound", "Notifications and sound")], None),
    sel("Project order", &[("updated_at", "Last user message"), ("created_at", "Creation date"), ("manual", "Manual")], None),
    sel("Language", LANGUAGES, None),
    sel("Timestamp format", TIME_FORMAT, None),
    sel("Background activity profile", &[("balanced", "Balanced"), ("performance", "Performance"), ("battery-saver", "Battery saver"), ("advanced", "Advanced")], Some("backgroundActivityProfile")),
    sel("Compact idle threads", &[("off", "Never"), ("on-return", "When you return"), ("always", "While idle")], None),
    sel("Default pull request merge method", &[("last", "Last selected"), ("merge", "Merge commit"), ("squash", "Squash"), ("rebase", "Rebase")], Some("pullRequestMergeMethod")),
    sel("Commit and pull request writing", &[("repo_conventions", "Repository conventions"), ("conventional_commits", "Conventional Commits"), ("custom", "Custom")], None),
    sel("Default permissions", &[("full-access", "Full access"), ("approval-required", "Approval required"), ("auto-accept-edits", "Auto-accept edits")], Some("defaultRuntimeMode")),
    sel("Default workspace", &[("local", "Current checkout"), ("worktree", "New worktree")], Some("defaultThreadEnvMode")),
    sel("When", WHEN, None),
    sel("Unit", UNITS, None),
    sel("Dictation shortcut mode", &[("auto", "Auto"), ("hold", "Hold"), ("toggle", "Toggle")], None),
    sel("Transcribe with", &[("off", "Off"), ("local", "This computer"), ("openai", "OpenAI"), ("xai", "xAI")], None),
    sel("Worktree branch naming", &[("static", "Static prefix"), ("semantic", "Semantic"), ("custom", "Custom")], Some("branchNamingMode")),
];

pub fn select_spec(aria: &str) -> Option<&'static SelectSpec> {
    SELECTS.iter().find(|s| s.aria == aria)
}

/// Text and number inputs that mirror a server setting: (aria-label or id, server key, numeric).
pub static TEXTS: &[(&str, &str, bool)] = &[
    ("Branch prefix", "branchNamePrefix", false),
    ("Add project base directory", "addProjectBaseDirectory", false),
    ("Days of inactivity before auto-settle", "sidebarAutoSettleAfterDays", true),
];

/// Spinner buttons: (button aria-label, input aria-label, step, min, max).
pub static STEPPERS: &[(&str, &str, i64, i64, i64)] = &[
    ("Decrease provider health check interval", "Provider health check interval in seconds", -30, 30, 3600),
    ("Increase provider health check interval", "Provider health check interval in seconds", 30, 30, 3600),
    ("Decrease", "Amount", -1, 1, 999),
    ("Increase", "Amount", 1, 1, 999),
];
