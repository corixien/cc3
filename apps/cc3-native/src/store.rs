//! Shared client state fed by the server's streams on a background thread and read by the UI.
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::model::Shell;
use crate::net::rpc::StreamEvent;
use crate::net::RpcClient;

#[derive(Default)]
pub struct Store {
    pub shell: Mutex<Shell>,
    pub environment_id: Mutex<String>,
    /// Bumped on every change so the UI only rebuilds when needed.
    pub version: AtomicU64,
}

impl Store {
    pub fn version(&self) -> u64 {
        self.version.load(Ordering::Acquire)
    }
    fn bump(&self) {
        self.version.fetch_add(1, Ordering::AcqRel);
    }
}

/// Subscribes to server config and the shell stream; the threads end when the connection closes.
pub fn spawn_streams(client: Arc<RpcClient>, store: Arc<Store>) {
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
