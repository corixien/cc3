//! Creates a project and thread in CC3_HOME, sends a message through `Store::send_message` and prints the resulting timeline.
//! `CC3_HOME=/tmp/x cargo run --release --bin send_probe`.

use cc3_native::net;
use cc3_native::store::{spawn_streams, Store};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

fn main() {
    let server = net::server::start().expect("server");
    let bearer = net::auth::bootstrap(&server.http_base(), &server.token).expect("bearer");
    let ticket = net::auth::websocket_ticket(&server.http_base(), &bearer).expect("ticket");
    let client = Arc::new(
        net::RpcClient::connect(&server.ws_base(), &ticket, &net::rpc::ConnectParams { app_version: "0.1.9", device_id: "cc3-send-probe" }).expect("websocket"),
    );
    let store = Arc::new(Store::default());
    spawn_streams(client.clone(), store.clone());
    let root = std::env::temp_dir().join("cc3-send-probe-ws");
    let _ = std::fs::create_dir_all(&root);
    let t = Duration::from_secs(20);
    let r = client.call(
        "projects.mutate",
        json!({"type": "project.create", "commandId": "probe-c1", "projectId": "probe-p1", "title": "Probe", "workspaceRoot": root.to_string_lossy()}),
        t,
    );
    println!("project.create: {r:?}");
    let r = client.call(
        "orchestration.dispatchCommand",
        json!({
            "type": "thread.create", "createdBy": "user", "creationSource": "web", "commandId": "probe-c2", "threadId": "probe-t1", "projectId": "probe-p1",
            "title": "Probe thread", "modelSelection": {"instanceId": "codex", "model": "gpt-5.4"}, "runtimeMode": "full-access",
            "interactionMode": "default", "branch": null, "worktreePath": null
        }),
        t,
    );
    println!("thread.create: {r:?}");
    store.open_thread("probe-t1");
    std::thread::sleep(Duration::from_secs(2));
    store.send_message("Hello from the native composer");
    std::thread::sleep(Duration::from_secs(5));
    let view = store.thread_view();
    println!("view: {:?}", view.map(|v| (v.title, v.rows.len(), format!("{:?}", v.rows).chars().take(300).collect::<String>())));
    let id = store.start_thread("Second thread from draft", None);
    std::thread::sleep(Duration::from_secs(6));
    let view = store.thread_view();
    println!("start_thread: {id:?} -> {:?}", view.map(|v| (v.thread_id, v.title, v.rows.len())));
    let config = store.config.lock().unwrap().clone();
    if let Some(providers) = config["providers"].as_array() {
        for p in providers {
            println!("provider {} keys={:?} model0={}", p["instanceId"], p.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()), p["models"][0]);
        }
    } else {
        println!("config keys: {:?}", config.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()));
    }
    let r = client.call("review.getDiffPreview", json!({"cwd": root.to_string_lossy()}), Duration::from_secs(20));
    println!("diff: {}", format!("{r:?}").chars().take(1500).collect::<String>());
    store.open_thread("probe-t1");
    std::thread::sleep(Duration::from_secs(2));
    store.terminal_attach();
    std::thread::sleep(Duration::from_secs(2));
    store.terminal_write("echo cc3-term-ok\r".to_string());
    std::thread::sleep(Duration::from_secs(2));
    let guard = store.terminal.lock().unwrap();
    if let Some(t) = guard.as_ref() {
        let text = t.parser.screen().contents();
        println!("terminal[{}]: {}", t.status, text.lines().filter(|l| !l.trim().is_empty()).collect::<Vec<_>>().join(" | "));
    }
    drop(guard);
    println!("config keys: {:?}", config.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()));
    println!("settings: {}", config["settings"].to_string().chars().take(300).collect::<String>());
    if let Some(o) = config["settings"].as_object() {
        println!("boolkeys: {:?}", o.iter().filter(|(_, v)| v.is_boolean()).map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>());
    }
}
