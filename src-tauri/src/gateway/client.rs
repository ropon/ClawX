// Gateway WebSocket Client — Actor pattern
// Maintains an independent WS connection to OpenClaw Gateway for RPC calls
// and push event forwarding to the Tauri frontend via app.emit().

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio_tungstenite::tungstenite::Message;

use crate::device_identity::{self, DeviceIdentity};
use crate::gateway::protocol::{self, DeviceAuthInfo};
use crate::settings_store;

// ─── Constants ──────────────────────────────────────────

const MAX_RECONNECT_ATTEMPTS: u32 = 10;
const RECONNECT_BASE_DELAY_SECS: u64 = 3;
const RECONNECT_MAX_DELAY_SECS: u64 = 30;
/// How long to wait for the connect.challenge event (seconds)
const CHALLENGE_TIMEOUT_SECS: u64 = 3;

// ─── Actor Command ──────────────────────────────────────

#[allow(dead_code)]
enum GatewayCommand {
    Connect {
        port: u16,
        token: String,
        data_dir: PathBuf,
        reply: oneshot::Sender<Result<(), String>>,
    },
    Reconnect {
        port: u16,
        token: String,
        data_dir: PathBuf,
    },
    Rpc {
        method: String,
        params: Option<Value>,
        timeout_ms: u64,
        reply: oneshot::Sender<Result<Value, String>>,
    },
    GetStatus {
        reply: oneshot::Sender<Value>,
    },
    Disconnect,
}

// ─── Actor State ────────────────────────────────────────

struct ActorState {
    connected: bool,
    connected_at: Option<i64>,
    port: Option<u16>,
    write_tx: Option<mpsc::Sender<Message>>,
    pending: HashMap<String, oneshot::Sender<Result<Value, String>>>,
    // Reconnection state
    last_token: Option<String>,
    last_data_dir: Option<PathBuf>,
    reconnect_attempts: u32,
    // Channel for sending reconnect commands back to the actor loop
    cmd_tx: Option<mpsc::Sender<GatewayCommand>>,
}

impl ActorState {
    fn new() -> Self {
        Self {
            connected: false,
            connected_at: None,
            port: None,
            write_tx: None,
            pending: HashMap::new(),
            last_token: None,
            last_data_dir: None,
            reconnect_attempts: 0,
            cmd_tx: None,
        }
    }
}

// ─── Public Client Handle ───────────────────────────────

pub struct GatewayClient {
    cmd_tx: mpsc::Sender<GatewayCommand>,
    data_dir: PathBuf,
    actor_state: Arc<Mutex<ActorState>>,
    app_handle: Arc<Mutex<Option<AppHandle>>>,
}

impl GatewayClient {
    pub fn new(data_dir: PathBuf) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel(64);
        let actor_state = Arc::new(Mutex::new(ActorState::new()));
        let app_handle: Arc<Mutex<Option<AppHandle>>> = Arc::new(Mutex::new(None));

        // Store cmd_tx in actor state so read tasks can request reconnection
        {
            let state = actor_state.clone();
            let tx = cmd_tx.clone();
            tauri::async_runtime::spawn(async move {
                state.lock().await.cmd_tx = Some(tx);
            });
        }

        let state_clone = actor_state.clone();
        let app_clone = app_handle.clone();

        // Spawn actor task
        tauri::async_runtime::spawn(actor_loop(cmd_rx, state_clone, app_clone));

        Self {
            cmd_tx,
            data_dir,
            actor_state,
            app_handle,
        }
    }

    /// Set the AppHandle for event forwarding
    pub async fn set_app_handle(&self, app: AppHandle) {
        let mut handle = self.app_handle.lock().await;
        *handle = Some(app);
    }

    /// Connect to the Gateway WebSocket
    pub async fn connect(&self, port: u16, token: &str) -> Result<(), String> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.cmd_tx
            .send(GatewayCommand::Connect {
                port,
                token: token.to_string(),
                data_dir: self.data_dir.clone(),
                reply: reply_tx,
            })
            .await
            .map_err(|_| "Actor channel closed".to_string())?;
        reply_rx
            .await
            .map_err(|_| "Reply channel closed".to_string())?
    }

    /// Make an RPC call. Lazily connects if not connected.
    pub async fn rpc(
        &self,
        method: &str,
        params: Option<Value>,
        timeout_ms: u64,
    ) -> Result<Value, String> {
        // Lazy connect: if not connected, try to connect first
        {
            let state = self.actor_state.lock().await;
            if !state.connected {
                drop(state);
                self.lazy_connect().await?;
            }
        }

        let (reply_tx, reply_rx) = oneshot::channel();
        self.cmd_tx
            .send(GatewayCommand::Rpc {
                method: method.to_string(),
                params,
                timeout_ms,
                reply: reply_tx,
            })
            .await
            .map_err(|_| "Actor channel closed".to_string())?;
        reply_rx
            .await
            .map_err(|_| "Reply channel closed".to_string())?
    }

    /// Check connection status
    pub async fn is_connected(&self) -> bool {
        self.actor_state.lock().await.connected
    }

    /// Get status as JSON
    pub async fn status(&self) -> Value {
        let (reply_tx, reply_rx) = oneshot::channel();
        if self
            .cmd_tx
            .send(GatewayCommand::GetStatus { reply: reply_tx })
            .await
            .is_err()
        {
            return json!({ "connected": false });
        }
        reply_rx.await.unwrap_or(json!({ "connected": false }))
    }

    /// Lazy connect: read token from settings and connect
    async fn lazy_connect(&self) -> Result<(), String> {
        let token = settings_store::get_gateway_token(&self.data_dir)?;
        let port = settings_store::get_gateway_port();
        self.connect(port, &token).await
    }
}

// ─── Actor Loop ─────────────────────────────────────────

async fn actor_loop(
    mut cmd_rx: mpsc::Receiver<GatewayCommand>,
    state: Arc<Mutex<ActorState>>,
    app_handle: Arc<Mutex<Option<AppHandle>>>,
) {
    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            GatewayCommand::Connect {
                port,
                token,
                data_dir,
                reply,
            } => {
                let result =
                    do_connect(port, &token, &data_dir, state.clone(), app_handle.clone()).await;
                let _ = reply.send(result);
            }
            GatewayCommand::Reconnect {
                port,
                token,
                data_dir,
            } => {
                // Silent reconnect — no reply channel, just attempt to reconnect
                match do_connect(port, &token, &data_dir, state.clone(), app_handle.clone()).await {
                    Ok(()) => {
                        // Success — do_connect already emits "running" status
                    }
                    Err(e) => {
                        let attempts = state.lock().await.reconnect_attempts;
                        if let Some(app) = app_handle.lock().await.as_ref() {
                            let gw_port = settings_store::get_gateway_port();
                            if attempts >= MAX_RECONNECT_ATTEMPTS {
                                let _ = app.emit(
                                    "gateway_statusChanged",
                                    json!({
                                        "state": "error",
                                        "port": gw_port,
                                        "error": format!("Reconnect failed after {} attempts: {}", MAX_RECONNECT_ATTEMPTS, e)
                                    }),
                                );
                            }
                        }
                    }
                }
            }
            GatewayCommand::Rpc {
                method,
                params,
                timeout_ms,
                reply,
            } => {
                let result = do_rpc(&method, params, timeout_ms, state.clone()).await;
                let _ = reply.send(result);
            }
            GatewayCommand::GetStatus { reply } => {
                let s = state.lock().await;
                let status = json!({
                    "connected": s.connected,
                    "port": s.port,
                    "connectedAt": s.connected_at,
                });
                let _ = reply.send(status);
            }
            GatewayCommand::Disconnect => {
                let mut s = state.lock().await;
                s.connected = false;
                s.connected_at = None;
                s.write_tx = None;
                s.reconnect_attempts = MAX_RECONNECT_ATTEMPTS; // prevent auto-reconnect
                for (_, reply) in s.pending.drain() {
                    let _ = reply.send(Err("Disconnected".to_string()));
                }
            }
        }
    }
}

/// Perform WebSocket connection + OpenClaw v3 handshake with Ed25519 device auth
async fn do_connect(
    port: u16,
    token: &str,
    data_dir: &std::path::Path,
    state: Arc<Mutex<ActorState>>,
    app_handle: Arc<Mutex<Option<AppHandle>>>,
) -> Result<(), String> {
    // Load or create device identity for Ed25519 signing
    let identity = DeviceIdentity::load_or_create(data_dir)?;

    // Use 127.0.0.1 (not localhost) so Gateway sees a loopback connection
    let url = format!("ws://127.0.0.1:{}/ws", port);

    let (ws_stream, _) = tokio_tungstenite::connect_async(&url)
        .await
        .map_err(|e| format!("WebSocket connect failed: {}", e))?;

    let (mut write, mut read) = ws_stream.split();

    // Phase 1: Wait for connect.challenge event to get nonce
    // Gateway sends this immediately after WS upgrade.
    // For local connections the nonce is optional, so timeout is non-fatal.
    let nonce = wait_for_challenge(&mut read).await;

    // Phase 2: Build device auth payload and sign it
    let signed_at_ms = chrono::Utc::now().timestamp_millis();
    let payload = device_identity::build_device_auth_payload(
        &identity.device_id,
        protocol::CLIENT_ID,
        protocol::CLIENT_MODE,
        protocol::CLIENT_ROLE,
        protocol::CLIENT_SCOPES,
        signed_at_ms,
        token,
        nonce.as_deref(),
    );
    let signature = identity.sign_payload(&payload)?;

    let device_auth = DeviceAuthInfo {
        device_id: identity.device_id.clone(),
        public_key: identity.public_key.clone(),
        signature,
        signed_at_ms,
        nonce: nonce.clone(),
    };

    // Phase 3: Send connect handshake with device auth
    let connect_frame = protocol::build_connect_frame(token, Some(device_auth));
    let connect_id = connect_frame.id.clone();
    let frame_json = serde_json::to_string(&connect_frame)
        .map_err(|e| format!("Serialize connect frame: {}", e))?;
    write
        .send(Message::Text(frame_json))
        .await
        .map_err(|e| format!("Send connect frame: {}", e))?;

    // Phase 4: Wait for handshake response (up to 10s)
    let handshake_result = tokio::time::timeout(
        Duration::from_secs(10),
        wait_for_handshake(&mut read, &connect_id),
    )
    .await
    .map_err(|_| "Connect handshake timeout".to_string())?;

    handshake_result?;

    // Set up writer channel
    let (write_tx, mut write_rx) = mpsc::channel::<Message>(64);

    // Spawn write task
    tokio::spawn(async move {
        while let Some(msg) = write_rx.recv().await {
            if write.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Update state — connection successful, reset reconnect counter
    let cmd_tx_for_read = {
        let mut s = state.lock().await;
        s.connected = true;
        s.connected_at = Some(chrono::Utc::now().timestamp_millis());
        s.port = Some(port);
        s.write_tx = Some(write_tx);
        s.last_token = Some(token.to_string());
        s.last_data_dir = Some(data_dir.to_path_buf());
        s.reconnect_attempts = 0;
        s.cmd_tx.clone()
    };

    // Emit "running" status to frontend
    if let Some(app) = app_handle.lock().await.as_ref() {
        let _ = app.emit(
            "gateway_statusChanged",
            json!({ "state": "running", "port": port }),
        );
    }

    // Spawn read task
    let state_clone = state.clone();
    let app_clone = app_handle.clone();
    tokio::spawn(async move {
        while let Some(msg_result) = read.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    handle_incoming_message(&text, &state_clone, &app_clone).await;
                }
                Ok(Message::Close(_)) | Err(_) => {
                    // Connection closed — clean up pending requests
                    let (reconnect_port, reconnect_token, reconnect_dir, attempts) = {
                        let mut s = state_clone.lock().await;
                        s.connected = false;
                        s.connected_at = None;
                        s.write_tx = None;
                        for (_, reply) in s.pending.drain() {
                            let _ = reply.send(Err("Connection closed".to_string()));
                        }
                        (
                            s.port,
                            s.last_token.clone(),
                            s.last_data_dir.clone(),
                            s.reconnect_attempts,
                        )
                    };

                    // Emit disconnection event to frontend
                    if let Some(app) = app_clone.lock().await.as_ref() {
                        let _ = app.emit("gateway_error", "Connection closed");
                        let gw_port = settings_store::get_gateway_port();
                        let _ = app.emit(
                            "gateway_statusChanged",
                            json!({ "state": "reconnecting", "port": gw_port }),
                        );
                    }

                    // Auto-reconnect via actor command channel (avoids Send issue)
                    if attempts < MAX_RECONNECT_ATTEMPTS {
                        if let (Some(r_port), Some(r_token), Some(r_dir), Some(tx)) = (
                            reconnect_port,
                            reconnect_token,
                            reconnect_dir,
                            cmd_tx_for_read.clone(),
                        ) {
                            // Increment attempt counter
                            state_clone.lock().await.reconnect_attempts = attempts + 1;

                            // Spawn a delay task that sends Reconnect command back to actor loop
                            tokio::spawn(async move {
                                let delay_secs = (RECONNECT_BASE_DELAY_SECS
                                    * 2u64.pow(attempts))
                                .min(RECONNECT_MAX_DELAY_SECS);
                                tokio::time::sleep(Duration::from_secs(delay_secs)).await;

                                let _ = tx
                                    .send(GatewayCommand::Reconnect {
                                        port: r_port,
                                        token: r_token,
                                        data_dir: r_dir,
                                    })
                                    .await;
                            });
                        }
                    }

                    break;
                }
                _ => {} // Ping/Pong/Binary ignored
            }
        }
    });

    Ok(())
}

/// Wait for the connect.challenge event sent by Gateway after WS upgrade.
/// Returns the nonce UUID if received, None on timeout or unexpected message.
/// For local connections the nonce is optional — v1 payload works without it.
async fn wait_for_challenge(
    read: &mut (impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>>
             + Unpin),
) -> Option<String> {
    match tokio::time::timeout(Duration::from_secs(CHALLENGE_TIMEOUT_SECS), read.next()).await {
        Ok(Some(Ok(Message::Text(text)))) => {
            if let Ok(json) = serde_json::from_str::<Value>(&text) {
                if json.get("type").and_then(|v| v.as_str()) == Some("event")
                    && json.get("event").and_then(|v| v.as_str()) == Some("connect.challenge")
                {
                    return json
                        .get("payload")
                        .and_then(|p| p.get("nonce"))
                        .and_then(|n| n.as_str())
                        .map(|s| s.to_string());
                }
            }
            None
        }
        _ => None,
    }
}

/// Wait for handshake response matching connect_id
async fn wait_for_handshake(
    read: &mut (impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>>
             + Unpin),
    connect_id: &str,
) -> Result<(), String> {
    while let Some(msg_result) = read.next().await {
        match msg_result {
            Ok(Message::Text(text)) => {
                if let Ok(json) = serde_json::from_str::<Value>(&text) {
                    if json.get("type").and_then(|v| v.as_str()) == Some("res")
                        && json.get("id").and_then(|v| v.as_str()) == Some(connect_id)
                    {
                        if json.get("ok") == Some(&Value::Bool(false)) {
                            let err = json
                                .get("error")
                                .map(|e| e.to_string())
                                .unwrap_or_else(|| "Handshake rejected".to_string());
                            return Err(err);
                        }
                        return Ok(());
                    }
                }
            }
            Ok(Message::Close(_)) | Err(_) => {
                return Err("Connection closed during handshake".to_string());
            }
            _ => {}
        }
    }
    Err("Stream ended during handshake".to_string())
}

/// Handle an incoming WS message — route responses to pending requests,
/// and forward push events to the frontend via Tauri emit.
async fn handle_incoming_message(
    text: &str,
    state: &Arc<Mutex<ActorState>>,
    app_handle: &Arc<Mutex<Option<AppHandle>>>,
) {
    let frame: Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => return,
    };

    let frame_type = frame.get("type").and_then(|v| v.as_str()).unwrap_or("");

    // Handle RPC responses
    if frame_type == "res" {
        let id = match frame.get("id").and_then(|v| v.as_str()) {
            Some(id) => id.to_string(),
            None => return,
        };

        let mut s = state.lock().await;
        if let Some(reply) = s.pending.remove(&id) {
            if frame.get("ok") == Some(&Value::Bool(false)) || frame.get("error").is_some() {
                let error = frame
                    .get("error")
                    .and_then(|e| {
                        e.get("message")
                            .and_then(|m| m.as_str())
                            .map(|s| s.to_string())
                            .or_else(|| Some(e.to_string()))
                    })
                    .unwrap_or_else(|| "Unknown error".to_string());
                let _ = reply.send(Err(error));
            } else {
                let payload = frame
                    .get("payload")
                    .cloned()
                    .unwrap_or_else(|| frame.clone());
                let _ = reply.send(Ok(payload));
            }
        }
        return;
    }

    // Handle push events — forward to frontend via Tauri emit
    let app_guard = app_handle.lock().await;
    let app = match app_guard.as_ref() {
        Some(app) => app,
        None => return,
    };

    let event_name = frame
        .get("event")
        .or_else(|| frame.get("method"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let payload = frame
        .get("payload")
        .or_else(|| frame.get("params"))
        .cloned()
        .unwrap_or(Value::Null);

    match event_name {
        "tick" | "" => { /* ignore heartbeat / unknown */ }
        "chat" => {
            let _ = app.emit("gateway_chatMessage", json!({ "message": payload }));
            maybe_send_notification(app, &payload);
        }
        "channel.status" => {
            let _ = app.emit("gateway_channelStatus", &payload);
        }
        "agent" => {
            let _ = app.emit(
                "gateway_notification",
                json!({ "method": "agent", "params": payload }),
            );
            maybe_send_notification(app, &payload);
        }
        other => {
            let _ = app.emit(
                "gateway_notification",
                json!({ "method": other, "params": payload }),
            );
        }
    }
}

/// Send a desktop notification for completed chat messages when the window is not focused.
fn maybe_send_notification(app: &AppHandle, payload: &Value) {
    // Only notify for "final" state messages
    let state = payload
        .get("state")
        .or_else(|| payload.get("data").and_then(|d| d.get("state")))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if state != "final" {
        return;
    }

    // Check if main window is focused — skip notification if user is looking at the app
    if let Some(window) = app.get_webview_window("main") {
        if window.is_focused().unwrap_or(false) {
            return;
        }
    }

    // Extract a preview of the message content for the notification body
    let message = payload
        .get("message")
        .or_else(|| payload.get("data").and_then(|d| d.get("message")));

    let body = extract_notification_body(message);
    if body.is_empty() {
        return;
    }

    // Send notification via tauri-plugin-notification
    use tauri_plugin_notification::NotificationExt;
    let _ = app
        .notification()
        .builder()
        .title("ClawX")
        .body(&body)
        .show();
}

/// Extract a short text preview from a chat message for notification display.
fn extract_notification_body(message: Option<&Value>) -> String {
    let msg = match message {
        Some(m) => m,
        None => return String::new(),
    };

    // Try content as string
    if let Some(text) = msg.get("content").and_then(|v| v.as_str()) {
        return truncate_text(text, 120);
    }

    // Try content as array of blocks
    if let Some(blocks) = msg.get("content").and_then(|v| v.as_array()) {
        for block in blocks {
            if block.get("type").and_then(|v| v.as_str()) == Some("text") {
                if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
                    return truncate_text(text, 120);
                }
            }
        }
    }

    // Try top-level text field
    if let Some(text) = msg.get("text").and_then(|v| v.as_str()) {
        return truncate_text(text, 120);
    }

    String::new()
}

/// Truncate text to max_len characters, adding "..." if truncated.
fn truncate_text(text: &str, max_len: usize) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= max_len {
        trimmed.to_string()
    } else {
        let truncated: String = trimmed.chars().take(max_len - 3).collect();
        format!("{}...", truncated)
    }
}

/// Send an RPC request and wait for the response
async fn do_rpc(
    method: &str,
    params: Option<Value>,
    timeout_ms: u64,
    state: Arc<Mutex<ActorState>>,
) -> Result<Value, String> {
    let frame = protocol::build_rpc_frame(method, params);
    let request_id = frame.id.clone();
    let frame_json = serde_json::to_string(&frame)
        .map_err(|e| format!("Serialize RPC frame: {}", e))?;

    let (reply_tx, reply_rx) = oneshot::channel();

    // Register pending request and send
    {
        let mut s = state.lock().await;
        if !s.connected {
            return Err("Gateway not connected".to_string());
        }
        if s.pending.len() >= 1000 {
            return Err("Too many pending RPC requests (limit: 1000)".to_string());
        }
        let write_tx = s
            .write_tx
            .as_ref()
            .ok_or("No write channel")?
            .clone();
        s.pending.insert(request_id.clone(), reply_tx);
        drop(s);

        write_tx
            .send(Message::Text(frame_json))
            .await
            .map_err(|_| "Failed to send RPC request".to_string())?;
    }

    // Wait with timeout
    match tokio::time::timeout(Duration::from_millis(timeout_ms), reply_rx).await {
        Ok(Ok(result)) => result,
        Ok(Err(_)) => {
            state.lock().await.pending.remove(&request_id);
            Err(format!("RPC reply channel closed: {}", method))
        }
        Err(_) => {
            state.lock().await.pending.remove(&request_id);
            Err(format!("RPC timeout: {}", method))
        }
    }
}
