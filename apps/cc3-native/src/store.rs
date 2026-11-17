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
    /// Bumped on every change so the UI only rebuilds when needed.
    pub version: AtomicU64,
    client: Mutex<Option<Arc<RpcClient>>>,
    /// The open thread: its id, the stream request id and the state built from the stream.
    open: Mutex<Option<(String, u64)>>,
    pub thread: Mutex<Option<ThreadState>>,
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
        let Some((thread_id, model, active)) = self
            .thread
            .lock()
            .unwrap()
            .as_ref()
            .map(|t| (t.thread_id.clone(), t.model_selection().clone(), t.has_active_run()))
        else {
            return;
        };
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

    /// Writes one server-owned setting (`server.updateSettings`) on a background thread.
    pub fn update_server_setting(self: &Arc<Self>, key: &str, value: serde_json::Value) {
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
