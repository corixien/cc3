//! Effect RPC over WebSocket (JSON). Messages, as captured from the web client:
//!   client -> server  {"_tag":"Request","id":N,"tag":"<method>","payload":{..},"traceId":..,"spanId":..,"sampled":true,"headers":[]}
//!                     {"_tag":"Ack","requestId":N}   after every Chunk        {"_tag":"Ping"}   {"_tag":"Interrupt","requestId":N}
//!   server -> client  {"_tag":"Chunk","requestId":N,"values":[..]}            {"_tag":"Exit","requestId":N,"exit":{"_tag":"Success","value":..}|{"_tag":"Failure","cause":..}}
//!                     {"_tag":"Pong"}   {"_tag":"Ping"}
//! The `frameParts` query flag is not sent, so the server never splits large messages.
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::Message;

#[derive(Debug, Clone, PartialEq)]
pub enum RpcError {
    Closed,
    Timeout,
    /// The server answered with a failure exit; the cause is kept as JSON.
    Failure(Value),
}

#[derive(Debug, Clone)]
pub enum StreamEvent {
    Chunk(Vec<Value>),
    End(Result<Value, RpcError>),
}

type Pending = Arc<Mutex<HashMap<u64, Sender<StreamEvent>>>>;

pub struct RpcClient {
    outgoing: Sender<String>,
    pending: Pending,
    next_id: AtomicU64,
}

/// Query parameters of the socket URL besides the ticket.
pub struct ConnectParams<'a> {
    pub app_version: &'a str,
    pub device_id: &'a str,
}

fn trace_id() -> String {
    let mut buf = [0u8; 16];
    if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
        let _ = std::io::Read::read_exact(&mut f, &mut buf);
    }
    buf.iter().map(|b| format!("{b:02x}")).collect()
}

impl RpcClient {
    pub fn connect(ws_base: &str, ticket: &str, params: &ConnectParams) -> Result<Self, String> {
        let url = format!(
            "{ws_base}/ws?wsTicket={ticket}&clientSurface=desktop&clientAppVersion={}&clientDeviceType=desktop&clientOs=Linux&connectionMethod=direct&clientDevice={}&orchestrationProtocol=2&clientCapabilities=settingsSecretReferences",
            params.app_version, params.device_id
        );
        let (mut socket, _) = tungstenite::connect(url).map_err(|e| format!("websocket connect failed: {e}"))?;
        if let MaybeTlsStream::Plain(stream) = socket.get_mut() {
            let _ = stream.set_read_timeout(Some(Duration::from_millis(40)));
        }
        let (outgoing, queue) = mpsc::channel::<String>();
        let pending: Pending = Arc::default();
        let reader_pending = pending.clone();
        std::thread::spawn(move || {
            let mut last_ping = Instant::now();
            loop {
                while let Ok(text) = queue.try_recv() {
                    if socket.send(Message::text(text)).is_err() {
                        return finish(&reader_pending);
                    }
                }
                if last_ping.elapsed() > Duration::from_secs(5) {
                    last_ping = Instant::now();
                    if socket.send(Message::text(r#"{"_tag":"Ping"}"#)).is_err() {
                        return finish(&reader_pending);
                    }
                }
                match socket.read() {
                    Ok(Message::Text(text)) => handle(text.as_str().trim_start_matches('\u{1e}'), &reader_pending, &mut socket),
                    Ok(Message::Close(_)) => return finish(&reader_pending),
                    Ok(_) => {}
                    Err(tungstenite::Error::Io(e)) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => {}
                    Err(_) => return finish(&reader_pending),
                }
            }
        });
        Ok(Self { outgoing, pending, next_id: AtomicU64::new(0) })
    }

    /// Sends a request and returns its event stream (chunks, then the exit).
    pub fn request(&self, tag: &str, payload: Value) -> Receiver<StreamEvent> {
        self.request_with_id(tag, payload).1
    }

    /// Like `request`, also returning the request id (needed to `interrupt` a stream).
    pub fn request_with_id(&self, tag: &str, payload: Value) -> (u64, Receiver<StreamEvent>) {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = mpsc::channel();
        self.pending.lock().unwrap().insert(id, tx);
        let message = json!({
            "_tag": "Request", "id": id, "tag": tag, "payload": payload,
            "traceId": trace_id(), "spanId": &trace_id()[..16], "sampled": true, "headers": [],
        });
        if self.outgoing.send(message.to_string()).is_err() {
            if let Some(tx) = self.pending.lock().unwrap().remove(&id) {
                let _ = tx.send(StreamEvent::End(Err(RpcError::Closed)));
            }
        }
        (id, rx)
    }

    /// Ends a stream on the server and forgets it locally.
    pub fn interrupt(&self, id: u64) {
        self.pending.lock().unwrap().remove(&id);
        let _ = self.outgoing.send(json!({"_tag": "Interrupt", "requestId": id}).to_string());
    }

    /// Unary call: waits for the exit value.
    pub fn call(&self, tag: &str, payload: Value, timeout: Duration) -> Result<Value, RpcError> {
        let rx = self.request(tag, payload);
        let deadline = Instant::now() + timeout;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            match rx.recv_timeout(left) {
                Ok(StreamEvent::Chunk(_)) => {}
                Ok(StreamEvent::End(result)) => return result,
                Err(_) => return Err(RpcError::Timeout),
            }
        }
    }
}

fn finish(pending: &Pending) {
    for (_, tx) in pending.lock().unwrap().drain() {
        let _ = tx.send(StreamEvent::End(Err(RpcError::Closed)));
    }
}

fn handle(text: &str, pending: &Pending, socket: &mut tungstenite::WebSocket<MaybeTlsStream<std::net::TcpStream>>) {
    let Ok(message) = serde_json::from_str::<Value>(text) else { return };
    let request_id = message["requestId"].as_u64();
    match message["_tag"].as_str() {
        Some("Chunk") => {
            let Some(id) = request_id else { return };
            let values = message["values"].as_array().cloned().unwrap_or_default();
            if let Some(tx) = pending.lock().unwrap().get(&id) {
                let _ = tx.send(StreamEvent::Chunk(values));
            }
            let _ = socket.send(Message::text(json!({"_tag": "Ack", "requestId": id}).to_string()));
        }
        Some("Exit") => {
            let Some(id) = request_id else { return };
            let exit = &message["exit"];
            let result = if exit["_tag"] == "Success" { Ok(exit["value"].clone()) } else { Err(RpcError::Failure(exit["cause"].clone())) };
            if let Some(tx) = pending.lock().unwrap().remove(&id) {
                let _ = tx.send(StreamEvent::End(result));
            }
        }
        Some("Ping") => {
            let _ = socket.send(Message::text(r#"{"_tag":"Pong"}"#));
        }
        _ => {}
    }
}
