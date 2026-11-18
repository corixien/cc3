//! Shared client state fed by the server's streams on a background thread and read by the UI.
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::model::{Shell, ThreadState, ThreadView};
use crate::net::rpc::StreamEvent;
use crate::net::RpcClient;

#[derive(Default)]
pub struct Store {
    pub shell: Mutex<Shell>,
    pub environment_id: Mutex<String>,
    /// Latest `server.subscribeConfig` snapshot (providers, models, settings).
    pub config: Mutex<serde_json::Value>,
    /// Model chosen in the picker; used for new messages and threads instead of the thread's own.
    selected_model: Mutex<Option<serde_json::Value>>,
    /// Latest `review.getDiffPreview` result for the open thread's workspace, or an error text.
    pub diff: Mutex<Option<Result<serde_json::Value, String>>>,
    /// Archived threads (id, title, project title), loaded when the archived page opens.
    pub archived: Mutex<Vec<(String, String, String)>>,
    /// Notifications shown as toasts: (id, kind, title, description, created).
    pub toasts: Mutex<Vec<(u64, String, String, String, std::time::Instant)>>,
    /// The terminal drawer's screen, fed by `terminal.attach`.
    pub terminal: Mutex<Option<TerminalState>>,
    /// Bumped on every change so the UI only rebuilds when needed.
    pub version: AtomicU64,
    client: Mutex<Option<Arc<RpcClient>>>,
    /// The open thread: its id, the stream request id and the state built from the stream.
    open: Mutex<Option<(String, u64)>>,
    pub thread: Mutex<Option<ThreadState>>,
}

pub struct TerminalState {
    pub parser: vt100::Parser,
    pub thread_id: String,
    pub status: String,
}

impl Store {
    pub fn version(&self) -> u64 {
        self.version.load(Ordering::Acquire)
    }
    fn bump(&self) {
        self.version.fetch_add(1, Ordering::AcqRel);
    }
}

impl Store {
    /// Sends a user message to the open thread (`message.dispatch`). Runs on a background thread.
    pub fn send_message(self: &Arc<Self>, text: &str) {
        let text = text.trim();
        let Some(client) = self.client.lock().unwrap().clone() else { return };
        let Some((thread_id, thread_model, active)) = self
            .thread
            .lock()
            .unwrap()
            .as_ref()
            .map(|t| (t.thread_id.clone(), t.model_selection().clone(), t.has_active_run()))
        else {
            return;
        };
        let model = self.selected_model.lock().unwrap().clone().unwrap_or(thread_model);
        if text.is_empty() {
            return;
        }
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        let dispatch_mode = if active { "queue_after_active" } else { "start_immediately" };
        let command = serde_json::json!({
            "type": "message.dispatch",
            "createdBy": "user",
            "creationSource": "web",
            "commandId": format!("cc3-cmd-{seed:x}"),
            "threadId": thread_id,
            "messageId": format!("cc3-msg-{seed:x}"),
            "text": text,
            "attachments": [],
            "modelSelection": model,
            "dispatchMode": {"type": dispatch_mode},
        });
        std::thread::spawn(move || {
            let _ = client.call("orchestration.dispatchCommand", command, std::time::Duration::from_secs(30));
        });
    }

    /// Creates a thread in `project_id` (or the open thread's project, else the first project), opens it and
    /// sends `text` as its first message. Returns the new thread id; the work runs on a background thread.
    pub fn start_thread(self: &Arc<Self>, text: &str, project_id: Option<String>) -> Option<String> {
        let text = text.trim().to_string();
        let client = self.client.lock().unwrap().clone()?;
        if text.is_empty() {
            return None;
        }
        let (project_id, model) = {
            let shell = self.shell.lock().unwrap();
            let open_project = self.thread.lock().unwrap().as_ref().and_then(|t| shell.threads.iter().find(|s| s.id == t.thread_id).map(|s| s.project_id.clone()));
            let project = project_id.or(open_project).or_else(|| shell.projects.first().map(|p| p.id.clone()))?;
            let model = self.current_model().unwrap_or_else(|| serde_json::json!({"instanceId": "codex", "model": "gpt-5.4"}));
            (project, model)
        };
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        let thread_id = format!("cc3-thread-{seed:x}");
        let title: String = text.lines().next().unwrap_or("New thread").chars().take(60).collect();
        let create = serde_json::json!({
            "type": "thread.create", "createdBy": "user", "creationSource": "web", "commandId": format!("cc3-cmd-c-{seed:x}"),
            "threadId": thread_id, "projectId": project_id, "title": title, "modelSelection": model,
            "runtimeMode": "full-access", "interactionMode": "default", "branch": null, "worktreePath": null
        });
        let store = self.clone();
        let id = thread_id.clone();
        std::thread::spawn(move || {
            if client.call("orchestration.dispatchCommand", create, std::time::Duration::from_secs(30)).is_ok() {
                store.open_thread(&id);
                // The snapshot arrives asynchronously; send_message needs the opened thread state.
                for _ in 0..50 {
                    if store.thread.lock().unwrap().as_ref().is_some_and(|t| t.thread_id == id) {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                store.send_message(&text);
            }
        });
        Some(thread_id)
    }

    /// Answers a pending question (`runtime-request.respond`).
    pub fn answer_question(self: &Arc<Self>, request_id: &str, value: &str) {
        let Some(client) = self.client.lock().unwrap().clone() else { return };
        let Some((thread_id, question_id)) = self.thread.lock().unwrap().as_ref().map(|t| (t.thread_id.clone(), t.question_id(request_id))) else { return };
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        let mut command = serde_json::json!({
            "type": "runtime-request.respond", "commandId": format!("cc3-cmd-a-{seed:x}"), "threadId": thread_id,
            "requestId": request_id,
        });
        match value.strip_prefix("decision:") {
            Some(decision) => command["decision"] = decision.into(),
            None => command["answers"] = serde_json::json!({ question_id: value }),
        }
        std::thread::spawn(move || {
            let _ = client.call("orchestration.dispatchCommand", command, std::time::Duration::from_secs(30));
        });
    }

    /// Registers a workspace folder as a project (`projects.mutate` / `project.create`). Blocks until the
    /// server answers so the dialog can show the error.
    pub fn add_project(self: &Arc<Self>, path: &str) -> Result<(), String> {
        let client = self.client.lock().unwrap().clone().ok_or("Not connected")?;
        let path = path.trim();
        let path = match path.strip_prefix("~/") {
            Some(rest) => format!("{}/{rest}", std::env::var("HOME").unwrap_or_default()),
            None => path.to_string(),
        };
        if !std::path::Path::new(&path).is_absolute() {
            return Err("Enter an absolute folder path".into());
        }
        if !std::path::Path::new(&path).is_dir() {
            return Err("That folder does not exist".into());
        }
        let title = std::path::Path::new(&path).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| path.clone());
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        client
            .call(
                "projects.mutate",
                serde_json::json!({"type": "project.create", "commandId": format!("cc3-cmd-p-{seed:x}"), "projectId": format!("cc3-project-{seed:x}"), "title": title, "workspaceRoot": path}),
                std::time::Duration::from_secs(20),
            )
            .map(|_| ())
            .map_err(|e| format!("{e:?}"))
    }

    /// Settles, snoozes, pins, archives or deletes a thread from the sidebar.
    pub fn thread_action(self: &Arc<Self>, thread_id: &str, action: &str) {
        let Some(client) = self.client.lock().unwrap().clone() else { return };
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        let mut command = serde_json::json!({"commandId": format!("cc3-cmd-t-{seed:x}"), "threadId": thread_id});
        let kind = match action {
            "settle" => "thread.settle",
            "snooze" => "thread.snooze",
            "pin" => "thread.pin",
            "unpin" => "thread.unpin",
            "archive" => "thread.archive",
            "unarchive" => "thread.unarchive",
            "delete" => "thread.delete",
            _ => return,
        };
        command["type"] = kind.into();
        if action == "snooze" {
            // Until this time tomorrow.
            let until = std::time::SystemTime::now() + std::time::Duration::from_secs(24 * 3600);
            command["snoozedUntil"] = crate::model::iso_from_system_time(until).into();
        }
        std::thread::spawn(move || {
            let _ = client.call("orchestration.dispatchCommand", command, std::time::Duration::from_secs(30));
        });
    }

    /// Models of enabled providers: (instance id, provider name, slug, model name).
    pub fn model_options(&self) -> Vec<(String, String, String, String)> {
        let config = self.config.lock().unwrap();
        let mut out = Vec::new();
        for p in config["providers"].as_array().into_iter().flatten() {
            if p["enabled"].as_bool() != Some(true) {
                continue;
            }
            let instance = p["instanceId"].as_str().unwrap_or_default();
            let provider = p["displayName"].as_str().unwrap_or(instance);
            for m in p["models"].as_array().into_iter().flatten() {
                let slug = m["slug"].as_str().unwrap_or_default();
                out.push((instance.to_string(), provider.to_string(), slug.to_string(), m["name"].as_str().unwrap_or(slug).to_string()));
            }
        }
        out
    }

    pub fn select_model(&self, instance_id: &str, slug: &str) {
        *self.selected_model.lock().unwrap() = Some(serde_json::json!({"instanceId": instance_id, "model": slug}));
        self.bump();
    }

    /// The model used for the next message: the picked one, else the open thread's.
    pub fn current_model(&self) -> Option<serde_json::Value> {
        self.selected_model.lock().unwrap().clone().or_else(|| self.thread.lock().unwrap().as_ref().map(|t| t.model_selection().clone()).filter(|m| !m.is_null()))
    }

    pub fn model_label(&self) -> String {
        let Some(current) = self.current_model() else { return String::new() };
        let (instance, slug) = (current["instanceId"].as_str().unwrap_or_default(), current["model"].as_str().unwrap_or_default());
        self.model_options().into_iter().find(|(i, _, s, _)| i == instance && s == slug).map(|(_, _, _, name)| name).unwrap_or_else(|| if slug.starts_with("gpt") { slug.to_uppercase() } else { slug.to_string() })
    }

    /// Shows a toast (`success` or `error`); the UI drops it after a few seconds.
    pub fn notify(&self, kind: &str, title: &str, description: &str) {
        let id = self.version.load(Ordering::Relaxed) * 1000 + self.toasts.lock().unwrap().len() as u64 + 1;
        self.toasts.lock().unwrap().push((id, kind.into(), title.into(), description.into(), std::time::Instant::now()));
        self.bump();
    }

    pub fn toasts_view(&self) -> Vec<crate::ui::toast::ToastView> {
        self.toasts.lock().unwrap().iter().map(|(id, kind, title, description, _)| crate::ui::toast::ToastView { id: *id, kind: kind.clone(), title: title.clone(), description: description.clone() }).collect()
    }

    /// Drops toasts older than `max_age` (or the given one); returns whether any went.
    pub fn prune_toasts(&self, dismiss: Option<u64>, max_age: std::time::Duration) -> bool {
        let mut toasts = self.toasts.lock().unwrap();
        let before = toasts.len();
        toasts.retain(|t| Some(t.0) != dismiss && t.4.elapsed() < max_age);
        toasts.len() != before
    }

    /// Workspace folder of the open thread's project.
    pub fn current_workspace(&self) -> Option<String> {
        let thread_id = self.thread.lock().unwrap().as_ref().map(|t| t.thread_id.clone())?;
        let shell = self.shell.lock().unwrap();
        let project_id = shell.threads.iter().find(|t| t.id == thread_id).map(|t| t.project_id.clone())?;
        shell.project(&project_id).map(|p| p.workspace_root.clone())
    }

    /// (project title, branch) of the open thread.
    pub fn current_project_info(&self) -> Option<(String, String)> {
        let thread_id = self.thread.lock().unwrap().as_ref().map(|t| t.thread_id.clone())?;
        let shell = self.shell.lock().unwrap();
        let thread = shell.threads.iter().find(|t| t.id == thread_id)?;
        let project = shell.project(&thread.project_id)?.title.clone();
        Some((project, thread.branch.clone().unwrap_or_default()))
    }

    /// Loads the working-tree diff of the open thread's workspace in the background.
    pub fn refresh_diff(self: &Arc<Self>) {
        let Some(client) = self.client.lock().unwrap().clone() else { return };
        let Some(cwd) = self.current_workspace() else {
            *self.diff.lock().unwrap() = Some(Err("Open a thread to see its changes".into()));
            self.bump();
            return;
        };
        *self.diff.lock().unwrap() = None;
        let store = self.clone();
        std::thread::spawn(move || {
            let result = client.call("review.getDiffPreview", serde_json::json!({"cwd": cwd}), std::time::Duration::from_secs(30)).map_err(|e| format!("{e:?}"));
            *store.diff.lock().unwrap() = Some(result);
            store.bump();
        });
    }

    /// Attaches the drawer terminal of the open thread (shell in the project folder). Streams on its own thread.
    pub fn terminal_attach(self: &Arc<Self>) {
        let Some(client) = self.client.lock().unwrap().clone() else { return };
        let Some(thread_id) = self.thread.lock().unwrap().as_ref().map(|t| t.thread_id.clone()) else { return };
        let Some(cwd) = self.current_workspace() else { return };
        if self.terminal.lock().unwrap().as_ref().is_some_and(|t| t.thread_id == thread_id) {
            return;
        }
        let (cols, rows) = (crate::ui::terminal_panel::COLS, crate::ui::terminal_panel::ROWS);
        *self.terminal.lock().unwrap() = Some(TerminalState { parser: vt100::Parser::new(rows, cols, 0), thread_id: thread_id.clone(), status: "Starting...".into() });
        self.bump();
        let stream = client.request(
            "terminal.attach",
            serde_json::json!({"threadId": thread_id, "terminalId": "cc3-drawer", "cwd": cwd, "cols": cols, "rows": rows, "restartIfNotRunning": true}),
        );
        let store = self.clone();
        std::thread::spawn(move || {
            while let Ok(event) = stream.recv() {
                let StreamEvent::Chunk(values) = event else { break };
                for v in &values {
                    let mut guard = store.terminal.lock().unwrap();
                    let Some(t) = guard.as_mut() else { return };
                    if t.thread_id != thread_id {
                        return;
                    }
                    match v["type"].as_str() {
                        Some("snapshot") => {
                            t.parser.process(v["snapshot"]["history"].as_str().unwrap_or_default().as_bytes());
                            t.status = v["snapshot"]["status"].as_str().unwrap_or("running").to_string();
                        }
                        Some("output") => t.parser.process(v["data"].as_str().unwrap_or_default().as_bytes()),
                        Some("exited") => t.status = "exited".into(),
                        Some("error") => t.status = v["message"].as_str().unwrap_or("error").to_string(),
                        _ => continue,
                    }
                    drop(guard);
                    store.bump();
                }
            }
        });
    }

    /// Sends keyboard input to the drawer terminal.
    pub fn terminal_write(self: &Arc<Self>, data: String) {
        let Some(client) = self.client.lock().unwrap().clone() else { return };
        let Some(thread_id) = self.terminal.lock().unwrap().as_ref().map(|t| t.thread_id.clone()) else { return };
        std::thread::spawn(move || {
            let _ = client.call("terminal.write", serde_json::json!({"threadId": thread_id, "terminalId": "cc3-drawer", "data": data}), std::time::Duration::from_secs(10));
        });
    }

    /// Dismisses the pending question (`thread.user-input.dismiss`).
    pub fn dismiss_question(self: &Arc<Self>) {
        let Some(client) = self.client.lock().unwrap().clone() else { return };
        let Some((thread_id, request_id)) = self.thread.lock().unwrap().as_ref().and_then(|t| t.view().pending_question.map(|q| (t.thread_id.clone(), q.request_id))) else { return };
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        let command = serde_json::json!({"type": "thread.user-input.dismiss", "commandId": format!("cc3-cmd-d-{seed:x}"), "threadId": thread_id, "requestId": request_id});
        std::thread::spawn(move || {
            let _ = client.call("orchestration.dispatchCommand", command, std::time::Duration::from_secs(30));
        });
    }

    /// Loads the archived threads in the background.
    pub fn refresh_archived(self: &Arc<Self>) {
        let Some(client) = self.client.lock().unwrap().clone() else { return };
        let store = self.clone();
        std::thread::spawn(move || {
            if let Ok(snapshot) = client.call("orchestration.getArchivedShellSnapshot", serde_json::json!({}), std::time::Duration::from_secs(20)) {
                let project = |id: &str| snapshot["projects"].as_array().and_then(|p| p.iter().find(|p| p["id"] == id)).and_then(|p| p["title"].as_str()).unwrap_or_default().to_string();
                let list = snapshot["threads"].as_array().map(|t| t.iter().map(|t| (t["id"].as_str().unwrap_or_default().to_string(), t["title"].as_str().unwrap_or_default().to_string(), project(t["projectId"].as_str().unwrap_or_default()))).collect()).unwrap_or_default();
                *store.archived.lock().unwrap() = list;
                store.bump();
            }
        });
    }

    /// Editor ids the server can launch.
    pub fn editors(&self) -> Vec<String> {
        self.config.lock().unwrap()["availableEditors"].as_array().map(|a| a.iter().filter_map(|e| e.as_str().map(String::from)).collect()).unwrap_or_default()
    }

    /// Opens the workspace in an editor (`shell.openInEditor`); toasts on failure.
    pub fn open_in_editor(self: &Arc<Self>, editor: Option<String>) {
        let Some(client) = self.client.lock().unwrap().clone() else { return };
        let Some(cwd) = self.current_workspace() else {
            self.notify("error", "No workspace", "Open a thread first");
            return;
        };
        let Some(editor) = editor.or_else(|| self.editors().into_iter().next()) else {
            self.notify("error", "No editor available", "The server found no installed editor");
            return;
        };
        let store = self.clone();
        std::thread::spawn(move || {
            if let Err(e) = client.call("shell.openInEditor", serde_json::json!({"cwd": cwd, "editor": editor}), std::time::Duration::from_secs(20)) {
                store.notify("error", "Could not open the editor", &format!("{e:?}").chars().take(160).collect::<String>());
            }
        });
    }

    /// Local branches of the open thread's workspace: (name, is current).
    pub fn list_branches(&self) -> Vec<(String, bool)> {
        let Some(client) = self.client.lock().unwrap().clone() else { return Vec::new() };
        let Some(cwd) = self.current_workspace() else { return Vec::new() };
        let Ok(result) = client.call("vcs.listRefs", serde_json::json!({"cwd": cwd, "refKind": "local"}), std::time::Duration::from_secs(10)) else { return Vec::new() };
        result["refs"].as_array().map(|r| r.iter().filter_map(|b| Some((b["name"].as_str()?.to_string(), b["current"].as_bool().unwrap_or(false)))).collect()).unwrap_or_default()
    }

    pub fn switch_branch(self: &Arc<Self>, name: String) {
        let Some(client) = self.client.lock().unwrap().clone() else { return };
        let Some(cwd) = self.current_workspace() else { return };
        let store = self.clone();
        std::thread::spawn(move || match client.call("vcs.switchRef", serde_json::json!({"cwd": cwd, "refName": name}), std::time::Duration::from_secs(30)) {
            Ok(_) => store.notify("success", "Switched branch", &name),
            Err(e) => store.notify("error", "Could not switch branch", &format!("{e:?}").chars().take(160).collect::<String>()),
        });
    }

    /// Commit / push through `git.runStackedAction`.
    pub fn git_action(self: &Arc<Self>, action: &str, message: Option<String>) {
        let Some(client) = self.client.lock().unwrap().clone() else { return };
        let Some(cwd) = self.current_workspace() else { return };
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        let mut payload = serde_json::json!({"actionId": format!("cc3-git-{seed:x}"), "cwd": cwd, "action": action});
        if let Some(m) = message.filter(|m| !m.trim().is_empty()) {
            payload["commitMessage"] = m.into();
        }
        let label = action.replace('_', " ");
        let store = self.clone();
        std::thread::spawn(move || match client.call("git.runStackedAction", payload, std::time::Duration::from_secs(120)) {
            Ok(_) => store.notify("success", "Done", &label),
            Err(e) => store.notify("error", &format!("Git {label} failed"), &format!("{e:?}").chars().take(160).collect::<String>()),
        });
    }

    /// Projects: (id, title).
    pub fn projects(&self) -> Vec<(String, String)> {
        self.shell.lock().unwrap().projects.iter().map(|p| (p.id.clone(), p.title.clone())).collect()
    }

    /// Stops the open thread's run (`thread.stop`).
    pub fn stop_thread(self: &Arc<Self>) {
        let Some(client) = self.client.lock().unwrap().clone() else { return };
        let Some(thread_id) = self.thread.lock().unwrap().as_ref().map(|t| t.thread_id.clone()) else { return };
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        let command = serde_json::json!({"type": "thread.stop", "commandId": format!("cc3-cmd-s-{seed:x}"), "threadId": thread_id});
        std::thread::spawn(move || {
            let _ = client.call("orchestration.dispatchCommand", command, std::time::Duration::from_secs(30));
        });
    }

    /// Enables or disables a provider instance (`providerInstanceMutation` upsert).
    pub fn set_provider_enabled(self: &Arc<Self>, display_name: &str, enabled: bool) {
        let Some(client) = self.client.lock().unwrap().clone() else { return };
        let config = self.config.lock().unwrap().clone();
        let Some(provider) = config["providers"].as_array().and_then(|p| p.iter().find(|p| p["displayName"] == display_name)).cloned() else { return };
        let instance_id = provider["instanceId"].as_str().unwrap_or_default().to_string();
        let driver = provider["driver"].as_str().unwrap_or(&instance_id).to_string();
        let mut instance = config["settings"]["providerInstances"][&instance_id].clone();
        if !instance.is_object() {
            instance = serde_json::json!({"driver": driver});
        }
        instance["enabled"] = enabled.into();
        self.provider_mutation(client, instance_id, instance);
    }

    /// Edits one text field of a provider instance (display name or a driver setting).
    pub fn set_provider_field(self: &Arc<Self>, control_id: &str, value: &str) {
        let Some(client) = self.client.lock().unwrap().clone() else { return };
        // control id: provider-instance-<instance>-<field>
        let rest = control_id.trim_start_matches("provider-instance-");
        let Some((instance_id, field)) = rest.rsplit_once('-') else { return };
        let config = self.config.lock().unwrap().clone();
        let driver = config["providers"].as_array().and_then(|p| p.iter().find(|p| p["instanceId"] == instance_id)).and_then(|p| p["driver"].as_str()).unwrap_or(instance_id).to_string();
        let mut instance = config["settings"]["providerInstances"][instance_id].clone();
        if !instance.is_object() {
            instance = serde_json::json!({"driver": driver});
        }
        match field {
            "name" => instance["displayName"] = value.into(),
            other => {
                if !instance["config"].is_object() {
                    instance["config"] = serde_json::json!({});
                }
                instance["config"][other] = value.into();
            }
        }
        self.provider_mutation(client, instance_id.to_string(), instance);
    }

    fn provider_mutation(self: &Arc<Self>, client: Arc<RpcClient>, instance_id: String, instance: serde_json::Value) {
        let store = self.clone();
        let payload = serde_json::json!({"patch": {}, "providerInstanceMutation": {"operation": "upsert", "instanceId": instance_id, "instance": instance}});
        std::thread::spawn(move || {
            if let Err(e) = client.call("server.updateSettings", payload, std::time::Duration::from_secs(30)) {
                store.notify("error", "Provider setting not saved", &format!("{e:?}").chars().take(160).collect::<String>());
            }
        });
    }

    /// Writes one server-owned setting (`server.updateSettings`) on a background thread.
    pub fn update_server_setting(self: &Arc<Self>, key: &str, value: serde_json::Value) {
        if let Some(control) = key.strip_prefix("provider:") {
            self.set_provider_field(control, value.as_str().unwrap_or_default());
            return;
        }
        let Some(client) = self.client.lock().unwrap().clone() else { return };
        let payload = serde_json::json!({ "patch": { key: value } });
        std::thread::spawn(move || {
            let _ = client.call("server.updateSettings", payload, std::time::Duration::from_secs(30));
        });
    }

    pub fn thread_view(&self) -> Option<ThreadView> {
        self.thread.lock().unwrap().as_ref().map(ThreadState::view)
    }

    /// Opens a thread: ends the previous thread stream and subscribes to the new one.
    pub fn open_thread(self: &Arc<Self>, thread_id: &str) {
        let Some(client) = self.client.lock().unwrap().clone() else { return };
        if let Some((current, request_id)) = self.open.lock().unwrap().take() {
            if current == thread_id {
                *self.open.lock().unwrap() = Some((current, request_id));
                return;
            }
            client.interrupt(request_id);
        }
        *self.thread.lock().unwrap() = None;
        let (request_id, stream) = client.request_with_id(
            "orchestration.subscribeThread",
            serde_json::json!({"threadId": thread_id, "afterSequence": 0, "requestCompletionMarker": true}),
        );
        *self.open.lock().unwrap() = Some((thread_id.to_string(), request_id));
        let store = self.clone();
        let id = thread_id.to_string();
        std::thread::spawn(move || {
            while let Ok(event) = stream.recv() {
                let StreamEvent::Chunk(values) = event else { break };
                for item in &values {
                    let mut guard = store.thread.lock().unwrap();
                    match item["kind"].as_str() {
                        Some("snapshot") => *guard = Some(ThreadState::from_projection(&item["projection"])),
                        Some("event") => {
                            if let Some(state) = guard.as_mut() {
                                state.apply_event(&item["event"]);
                            }
                        }
                        _ => continue,
                    }
                    drop(guard);
                    if store.open.lock().unwrap().as_ref().map(|(current, _)| current != &id).unwrap_or(true) {
                        return;
                    }
                    store.bump();
                }
            }
        });
    }
}

/// Subscribes to server config and the shell stream; the threads end when the connection closes.
pub fn spawn_streams(client: Arc<RpcClient>, store: Arc<Store>) {
    *store.client.lock().unwrap() = Some(client.clone());
    let config = client.request(
        "subscribeServerConfig",
        serde_json::json!({"environmentThemes": true, "usageLimitSources": true, "usageLimitsCommand": true, "workspaceSnapshotsOnDemand": true}),
    );
    let s = store.clone();
    std::thread::spawn(move || {
        while let Ok(event) = config.recv() {
            match event {
                StreamEvent::Chunk(values) => {
                    if let Some(config) = values.first().map(|v| &v["config"]).filter(|c| c.is_object()) {
                        *s.config.lock().unwrap() = config.clone();
                    }
                    if let Some(id) = values.first().and_then(|v| v["config"]["environment"]["environmentId"].as_str()) {
                        *s.environment_id.lock().unwrap() = id.to_string();
                        s.bump();
                    }
                }
                StreamEvent::End(_) => break,
            }
        }
    });
    let shell = client.request(
        "orchestration.subscribeShell",
        serde_json::json!({"afterSequence": 0, "requestCompletionMarker": true, "omitIdleSubagents": true, "omitOlderSettled": true}),
    );
    std::thread::spawn(move || {
        while let Ok(event) = shell.recv() {
            match event {
                StreamEvent::Chunk(values) => {
                    let mut changed = false;
                    {
                        let mut model = store.shell.lock().unwrap();
                        for item in &values {
                            changed |= model.apply(item);
                        }
                    }
                    if changed {
                        store.bump();
                    }
                }
                StreamEvent::End(_) => break,
            }
        }
    });
}
