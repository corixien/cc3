//! Bearer bootstrap and WebSocket ticket (same calls as the web client and bridge.js).
use serde_json::Value;

/// Exchanges the desktop bootstrap token for an access token.
pub fn bootstrap(http_base: &str, bootstrap_token: &str) -> Result<String, String> {
    let mut response = ureq::post(format!("{http_base}/oauth/token"))
        .send_form([
            ("grant_type", "urn:ietf:params:oauth:grant-type:token-exchange"),
            ("subject_token", bootstrap_token),
            ("subject_token_type", "urn:t3:params:oauth:token-type:environment-bootstrap"),
            ("requested_token_type", "urn:ietf:params:oauth:token-type:access_token"),
            ("client_label", "CC3"),
            ("client_device_type", "desktop"),
        ])
        .map_err(|e| format!("bearer bootstrap failed: {e}"))?;
    let body: Value = response.body_mut().read_json().map_err(|e| e.to_string())?;
    body["access_token"].as_str().map(String::from).ok_or_else(|| "no access_token in response".into())
}

/// One-time ticket for the RPC socket.
pub fn websocket_ticket(http_base: &str, bearer: &str) -> Result<String, String> {
    let mut response = ureq::post(format!("{http_base}/api/auth/websocket-ticket/rpc"))
        .header("authorization", format!("Bearer {bearer}"))
        .send_empty()
        .map_err(|e| format!("websocket ticket failed: {e}"))?;
    let body: Value = response.body_mut().read_json().map_err(|e| e.to_string())?;
    body["ticket"].as_str().map(String::from).ok_or_else(|| "no ticket in response".into())
}

/// Authenticated GET returning JSON (orchestration shell snapshot and similar).
pub fn get_json(http_base: &str, bearer: &str, path: &str) -> Result<Value, String> {
    let mut response = ureq::get(format!("{http_base}{path}"))
        .header("authorization", format!("Bearer {bearer}"))
        .header("x-t3-orchestration-protocol", "2")
        .call()
        .map_err(|e| format!("GET {path} failed: {e}"))?;
    response.body_mut().read_json().map_err(|e| e.to_string())
}
