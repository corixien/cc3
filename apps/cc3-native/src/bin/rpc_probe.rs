//! Starts a server in CC3_HOME, connects over HTTP and WebSocket and prints what the first requests return.
//! Used to verify the protocol port: `CC3_HOME=/tmp/x cargo run --release --bin rpc_probe`.

use cc3_native::net;
use serde_json::json;
use std::time::Duration;

fn main() {
    let server = net::server::start().expect("server");
    println!("server on {}", server.http_base());
    let bearer = net::auth::bootstrap(&server.http_base(), &server.token).expect("bearer");
    let ticket = net::auth::websocket_ticket(&server.http_base(), &bearer).expect("ticket");
    let client = net::RpcClient::connect(
        &server.ws_base(),
        &ticket,
        &net::rpc::ConnectParams { app_version: "0.1.9", device_id: "cc3-native-probe" },
    )
    .expect("websocket");
    let config = client.request(
        "subscribeServerConfig",
        json!({"environmentThemes": true, "usageLimitSources": true, "usageLimitsCommand": true, "workspaceSnapshotsOnDemand": true}),
    );
    match config.recv_timeout(Duration::from_secs(10)) {
        Ok(net::rpc::StreamEvent::Chunk(values)) => {
            let v = &values[0];
            println!("config chunk: type={} serverVersion={} label={}", v["type"], v["config"]["environment"]["serverVersion"], v["config"]["environment"]["label"]);
        }
        other => println!("config: {other:?}"),
    }
    let shell = client.request(
        "orchestration.subscribeShell",
        json!({"afterSequence": 0, "requestCompletionMarker": true, "omitIdleSubagents": true, "omitOlderSettled": true}),
    );
    for _ in 0..3 {
        match shell.recv_timeout(Duration::from_secs(10)) {
            Ok(net::rpc::StreamEvent::Chunk(values)) => println!("shell chunk: {}", values.iter().map(|v| v.to_string().chars().take(100).collect::<String>()).collect::<Vec<_>>().join(" | ")),
            other => {
                println!("shell: {other:?}");
                break;
            }
        }
    }
    println!("snapshot via HTTP: {}", net::auth::get_json(&server.http_base(), &bearer, "/api/orchestration/shell?omitIdleSubagents=1&omitOlderSettled=1").map(|v| v.to_string().chars().take(160).collect::<String>()).unwrap_or_else(|e| e));
}
