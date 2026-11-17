//! Client-side model of the orchestration shell (projects and threads), parsed from the server's JSON.
//! Only the fields the UI uses are typed; unknown fields are ignored so server updates do not break it.
use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Project {
    pub id: String,
    pub title: String,
    pub workspace_root: String,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ThreadShell {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub branch: Option<String>,
    pub status: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct Shell {
    pub projects: Vec<Project>,
    pub threads: Vec<ThreadShell>,
}

impl Shell {
    pub fn from_json(value: &serde_json::Value) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }

    pub fn project(&self, id: &str) -> Option<&Project> {
        self.projects.iter().find(|p| p.id == id)
    }

    /// Threads that appear in the sidebar, most recently updated first.
    pub fn visible_threads(&self) -> Vec<&ThreadShell> {
        let mut threads: Vec<&ThreadShell> = self.threads.iter().filter(|t| t.archived_at.is_none() && t.deleted_at.is_none()).collect();
        threads.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        threads
    }
}

impl Shell {
    /// Applies one item of `orchestration.subscribeShell`. Returns whether the model changed.
    pub fn apply(&mut self, item: &serde_json::Value) -> bool {
        match item["kind"].as_str() {
            Some("snapshot") => {
                let snapshot = &item["snapshot"];
                self.projects = serde_json::from_value(snapshot["projects"].clone()).unwrap_or_default();
                self.threads = serde_json::from_value(snapshot["threads"].clone()).unwrap_or_default();
                true
            }
            Some("project.updated") => {
                let Ok(project) = serde_json::from_value::<Project>(item["project"].clone()) else { return false };
                match self.projects.iter_mut().find(|p| p.id == project.id) {
                    Some(existing) => *existing = project,
                    None => self.projects.push(project),
                }
                true
            }
            Some("project.removed") => {
                let id = item["projectId"].as_str().unwrap_or_default();
                let before = self.projects.len();
                self.projects.retain(|p| p.id != id);
                self.projects.len() != before
            }
            Some("thread.updated") => {
                let Ok(thread) = serde_json::from_value::<ThreadShell>(item["thread"].clone()) else { return false };
                // Archived threads leave the active list (they stay available through the archive).
                if item["location"].as_str() == Some("archive") {
                    self.threads.retain(|t| t.id != thread.id);
                    return true;
                }
                match self.threads.iter_mut().find(|t| t.id == thread.id) {
                    Some(existing) => *existing = thread,
                    None => self.threads.push(thread),
                }
                true
            }
            Some("thread.removed") => {
                let id = item["threadId"].as_str().unwrap_or_default();
                let before = self.threads.len();
                self.threads.retain(|t| t.id != id);
                self.threads.len() != before
            }
            _ => false,
        }
    }
}

/// Short relative age like the sidebar shows ("13h", "5m", "2d").
pub fn relative_age(iso: &str, now_unix: i64) -> String {
    let Some(then) = parse_iso_unix(iso) else { return String::new() };
    let secs = (now_unix - then).max(0);
    match secs {
        0..=59 => "now".into(),
        60..=3599 => format!("{}m", secs / 60),
        3600..=86_399 => format!("{}h", secs / 3600),
        _ => format!("{}d", secs / 86_400),
    }
}

/// `2026-03-17T19:12:28.000Z` to unix seconds (UTC), without a date library.
pub fn parse_iso_unix(iso: &str) -> Option<i64> {
    let b = iso.as_bytes();
    if b.len() < 19 {
        return None;
    }
    let num = |a: usize, z: usize| iso.get(a..z)?.parse::<i64>().ok();
    let (y, m, d, hh, mm, ss) = (num(0, 4)?, num(5, 7)?, num(8, 10)?, num(11, 13)?, num(14, 16)?, num(17, 19)?);
    // days from civil (Howard Hinnant)
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Some(days * 86_400 + hh * 3600 + mm * 60 + ss)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_roundtrip() {
        let t = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_773_824_430);
        assert_eq!(iso_from_system_time(t), "2026-03-18T09:00:30.000Z");
        assert_eq!(parse_iso_unix(&iso_from_system_time(t)), Some(1_773_824_430));
    }

    #[test]
    fn pending_question_from_projection() {
        let p: serde_json::Value = serde_json::from_str(include_str!("../fixtures/thread-golden-question.projection.json")).unwrap();
        let q = ThreadView::from_projection(&p).pending_question.expect("pending question");
        assert_eq!(q.question, "Which approach should I take?");
        assert_eq!(q.options[0].2, "continue");
    }

    #[test]
    fn iso_and_age() {
        assert_eq!(parse_iso_unix("1970-01-02T00:00:00.000Z"), Some(86_400));
        assert_eq!(relative_age("2026-03-17T19:12:28.000Z", parse_iso_unix("2026-03-18T08:12:28.000Z").unwrap()), "13h");
    }

    #[test]
    fn shell_stream_items_update_the_model() {
        let mut shell = Shell::default();
        assert!(shell.apply(&serde_json::json!({"kind":"snapshot","snapshot":{"projects":[{"id":"p","title":"render"}],"threads":[{"id":"a","projectId":"p","title":"A"}]}})));
        assert!(shell.apply(&serde_json::json!({"kind":"thread.updated","location":"active","thread":{"id":"b","projectId":"p","title":"B"}})));
        assert!(shell.apply(&serde_json::json!({"kind":"thread.updated","location":"active","thread":{"id":"a","projectId":"p","title":"A2"}})));
        assert_eq!(shell.threads.iter().map(|t| t.title.as_str()).collect::<Vec<_>>(), ["A2", "B"]);
        assert!(shell.apply(&serde_json::json!({"kind":"thread.removed","threadId":"a"})));
        assert!(!shell.apply(&serde_json::json!({"kind":"synchronized"})));
        assert_eq!(shell.threads.len(), 1);
    }

    #[test]
    fn thread_view_from_golden_projection() {
        let json: serde_json::Value = serde_json::from_str(include_str!("../fixtures/thread-golden-question.projection.json")).unwrap();
        let view = ThreadView::from_projection(&json);
        assert_eq!(view.rows.len(), 3);
        assert!(matches!(view.rows[0], TimelineRow::User { .. }));
        assert!(matches!(view.rows[1], TimelineRow::Work { duration_ms: 0, .. }));
        assert!(matches!(view.rows[2], TimelineRow::Assistant { .. }));
    }

    #[test]
    fn thread_events_update_the_timeline() {
        let json: serde_json::Value = serde_json::from_str(include_str!("../fixtures/thread-golden-question.projection.json")).unwrap();
        let mut state = ThreadState::from_projection(&json);
        let mut item = json["visibleTurnItems"][1]["item"].clone();
        item["id"] = "new-assistant".into();
        item["messageId"] = "new-assistant".into();
        item["ordinal"] = 9.into();
        item["text"] = "streamed".into();
        assert!(state.apply_event(&serde_json::json!({"type": "turn-item.updated", "payload": item})));
        let view = state.view();
        assert!(matches!(view.rows.last(), Some(TimelineRow::Assistant { text, .. }) if text == "streamed"));
    }

    #[test]
    fn shell_parses_and_filters() {
        let v = serde_json::json!({"projects":[{"id":"p","title":"render"}],"threads":[
            {"id":"a","projectId":"p","title":"A","updatedAt":"2026-01-02T00:00:00Z"},
            {"id":"b","projectId":"p","title":"B","updatedAt":"2026-01-03T00:00:00Z"},
            {"id":"c","projectId":"p","title":"C","updatedAt":"2026-01-04T00:00:00Z","archivedAt":"2026-01-05T00:00:00Z"}]});
        let shell = Shell::from_json(&v);
        let titles: Vec<_> = shell.visible_threads().iter().map(|t| t.title.clone()).collect();
        assert_eq!(titles, ["B", "A"]);
    }
}

/// A question the agent is waiting on (`user_input_request` with a pending runtime request).
#[derive(Debug, Clone, PartialEq)]
pub struct PendingQuestion {
    pub request_id: String,
    pub header: String,
    pub question: String,
    /// (label, description, value)
    pub options: Vec<(String, String, String)>,
}

/// One tool call / reasoning line inside the expanded work group.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkEntry {
    pub id: String,
    /// Item type: reasoning, command_execution, file_change, web_search, ...
    pub kind: String,
    pub label: String,
    pub at: String,
}

/// One row of the chat timeline, as shown to the user.
#[derive(Debug, Clone, PartialEq)]
pub enum TimelineRow {
    User { id: String, text: String, at: String },
    /// Collapsed work of a run ("Worked for 4s").
    Work { run_id: String, duration_ms: i64, at: String, first_item_id: String, summary: String, entries: Vec<WorkEntry> },
    Assistant { id: String, text: String, at: String },
}

/// Live state of one open thread: the projection's runs and visible items, kept up to date by events.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ThreadState {
    pub thread_id: String,
    pub title: String,
    runs: Vec<serde_json::Value>,
    items: Vec<serde_json::Value>,
    requests: Vec<serde_json::Value>,
    model_selection: serde_json::Value,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ThreadView {
    /// Display name of the model messages are sent with (empty: default label).
    pub model_label: String,
    pub pending_question: Option<PendingQuestion>,
    /// A run is active: the composer offers Stop.
    pub running: bool,
    /// Workspace folder name; file paths in tool-call rows are shown below it.
    pub workspace_name: String,
    pub thread_id: String,
    pub title: String,
    pub rows: Vec<TimelineRow>,
}

impl ThreadState {
    pub fn model_selection(&self) -> &serde_json::Value {
        &self.model_selection
    }

    /// True while a run has not finished: new messages queue behind it.
    pub fn has_active_run(&self) -> bool {
        self.runs.iter().any(|run| {
            matches!(run["status"].as_str(), Some("preparing" | "queued" | "starting" | "running" | "waiting"))
        })
    }

    /// From an `OrchestrationV2ThreadProjection` (the `snapshot` item of `orchestration.subscribeThread`).
    pub fn from_projection(projection: &serde_json::Value) -> Self {
        let mut visible: Vec<&serde_json::Value> = projection["visibleTurnItems"].as_array().map(|a| a.iter().collect()).unwrap_or_default();
        visible.sort_by_key(|v| v["position"].as_i64().unwrap_or(0));
        Self {
            thread_id: projection["thread"]["id"].as_str().unwrap_or_default().into(),
            title: projection["thread"]["title"].as_str().unwrap_or_default().into(),
            model_selection: projection["thread"]["modelSelection"].clone(),
            runs: projection["runs"].as_array().cloned().unwrap_or_default(),
            requests: projection["runtimeRequests"].as_array().cloned().unwrap_or_default(),
            items: visible.iter().map(|v| v["item"].clone()).collect(),
        }
    }

    fn upsert(list: &mut Vec<serde_json::Value>, value: &serde_json::Value) {
        match list.iter_mut().find(|v| v["id"] == value["id"]) {
            Some(existing) => *existing = value.clone(),
            None => list.push(value.clone()),
        }
    }

    /// Applies one `event` item of the thread stream. Returns whether the timeline may have changed.
    pub fn apply_event(&mut self, event: &serde_json::Value) -> bool {
        let payload = &event["payload"];
        match event["type"].as_str() {
            Some("turn-item.updated") => {
                Self::upsert(&mut self.items, payload);
                self.items.sort_by_key(|v| v["ordinal"].as_i64().unwrap_or(0));
                true
            }
            Some("runtime-request.updated") => {
                Self::upsert(&mut self.requests, payload);
                true
            }
            Some("run.created" | "run.updated") => {
                Self::upsert(&mut self.runs, payload);
                true
            }
            _ => false,
        }
    }

    /// Id of the first question of a request (the key of its answer).
    pub fn question_id(&self, request_id: &str) -> String {
        self.items.iter().find(|i| i["requestId"] == request_id).and_then(|i| i["questions"][0]["id"].as_str()).unwrap_or("choice").into()
    }

    fn pending_question(&self) -> Option<PendingQuestion> {
        self.items.iter().rev().find_map(|i| {
            if i["type"].as_str() != Some("user_input_request") {
                return None;
            }
            let request_id = i["requestId"].as_str()?;
            let pending = self.requests.iter().any(|r| r["id"] == request_id && r["status"].as_str() == Some("pending"));
            let q = i["questions"].get(0)?;
            pending.then(|| PendingQuestion {
                request_id: request_id.into(),
                header: q["header"].as_str().unwrap_or_default().into(),
                question: q["question"].as_str().unwrap_or_default().into(),
                options: q["options"].as_array().map(|o| o.iter().map(|o| (o["label"].as_str().unwrap_or_default().into(), o["description"].as_str().unwrap_or_default().into(), o["value"].as_str().or(o["label"].as_str()).unwrap_or_default().into())).collect()).unwrap_or_default(),
            })
        })
    }

    /// Id of the run's first work item and the collapsed group label ("Ran 1 command, changed 1 file, ...").
    fn work_summary(&self, run_id: &str) -> (String, String, Vec<WorkEntry>) {
        let work: Vec<&serde_json::Value> = self.items.iter().filter(|i| i["runId"] == run_id && !matches!(i["type"].as_str(), Some("user_message" | "assistant_message"))).collect();
        let count = |f: &dyn Fn(&str) -> bool| work.iter().filter(|i| i["type"].as_str().is_some_and(f)).count();
        let commands = count(&|t| t == "command_execution");
        let files = count(&|t| t == "file_change");
        let other = count(&|t| !matches!(t, "command_execution" | "file_change" | "reasoning"));
        let plural = |n: usize, one: &str, many: &str| format!("{n} {}", if n == 1 { one } else { many });
        let mut parts = Vec::new();
        if commands > 0 { parts.push(format!("Ran {}", plural(commands, "command", "commands"))); }
        if files > 0 { parts.push(format!("{} {}", if parts.is_empty() { "Changed" } else { "changed" }, plural(files, "file", "files"))); }
        if other > 0 { parts.push(format!("{} {}", if parts.is_empty() { "Performed" } else { "performed" }, plural(other, "other action", "other actions"))); }
        let label = match parts.len() {
            0 => "Reasoned".to_string(),
            1 => parts.remove(0),
            2 => format!("{} and {}", parts[0], parts[1]),
            _ => { let last = parts.pop().unwrap(); format!("{}, and {last}", parts.join(", ")) }
        };
        let entries = work
            .iter()
            .map(|i| {
                let kind = i["type"].as_str().unwrap_or_default();
                let label = match kind {
                    "reasoning" => i["text"].as_str(),
                    "command_execution" => i["input"].as_str(),
                    "file_change" => i["fileName"].as_str(),
                    "web_search" => i["patterns"][0].as_str(),
                    _ => i["title"].as_str(),
                };
                WorkEntry {
                    id: i["id"].as_str().unwrap_or_default().into(),
                    kind: kind.into(),
                    label: label.unwrap_or(kind).into(),
                    at: i["completedAt"].as_str().or(i["startedAt"].as_str()).unwrap_or_default().into(),
                }
            })
            .collect();
        (work.first().and_then(|i| i["id"].as_str()).unwrap_or_default().into(), label, entries)
    }

    pub fn view(&self) -> ThreadView {
        let run = |id: &str| self.runs.iter().find(|r| r["id"] == id);
        let mut rows = Vec::new();
        let mut folded: Vec<String> = Vec::new();
        for item in &self.items {
            let at = item["completedAt"].as_str().or(item["startedAt"].as_str()).unwrap_or_default().to_string();
            match item["type"].as_str() {
                Some("user_message") => rows.push(TimelineRow::User { id: item["messageId"].as_str().unwrap_or_default().into(), text: item["text"].as_str().unwrap_or_default().into(), at }),
                Some("assistant_message") => {
                    let run_id = item["runId"].as_str().unwrap_or_default().to_string();
                    // A turn folds into "Worked for ..." only when it did more than exchange messages.
                    let did_work = self.items.iter().any(|i| i["runId"] == run_id.as_str() && !matches!(i["type"].as_str(), Some("user_message" | "assistant_message")));
                    if did_work && !folded.contains(&run_id) {
                        if let Some(run) = run(&run_id) {
                            if let (Some(start), Some(end)) = (run["startedAt"].as_str().and_then(parse_iso_ms), run["completedAt"].as_str().and_then(parse_iso_ms)) {
                                let (first_item_id, summary, entries) = self.work_summary(&run_id);
                                rows.push(TimelineRow::Work { run_id: run_id.clone(), duration_ms: end - start, at: run["completedAt"].as_str().unwrap_or_default().into(), first_item_id, summary, entries });
                            }
                        }
                        folded.push(run_id);
                    }
                    rows.push(TimelineRow::Assistant { id: item["messageId"].as_str().unwrap_or_default().into(), text: item["text"].as_str().unwrap_or_default().into(), at });
                }
                _ => {}
            }
        }
        ThreadView { model_label: String::new(), pending_question: self.pending_question(), running: self.has_active_run(), workspace_name: String::new(), thread_id: self.thread_id.clone(), title: self.title.clone(), rows }
    }
}

impl ThreadView {
    /// Builds the timeline from a projection (used by fixtures and tests).
    pub fn from_projection(projection: &serde_json::Value) -> Self {
        ThreadState::from_projection(projection).view()
    }
}

/// ISO timestamp with optional fraction to unix milliseconds.
pub fn parse_iso_ms(iso: &str) -> Option<i64> {
    let secs = parse_iso_unix(iso)?;
    let ms = iso.find('.').and_then(|i| iso.get(i + 1..i + 4)).and_then(|f| f.parse::<i64>().ok()).unwrap_or(0);
    Some(secs * 1000 + ms)
}

/// `2026-03-17T19:12:28.000Z` for a system time (UTC), without a date library.
pub fn iso_from_system_time(t: std::time::SystemTime) -> String {
    let secs = t.duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // civil from days (Howard Hinnant)
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}.000Z", rem / 3600, rem % 3600 / 60, rem % 60)
}
