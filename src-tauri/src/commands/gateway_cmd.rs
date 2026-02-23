// Tauri commands for gateway:* channels
// 5 commands for Gateway WebSocket interaction + 3 lifecycle commands (Week 7)

use crate::settings_store;
use crate::AppState;
use serde_json::{json, Value};
use tauri::{AppHandle, State};

/// gateway:status — Return GatewayStatus matching frontend type
/// Returns { state: "stopped"|"starting"|"running"|"error"|"reconnecting", port, error? }
#[tauri::command]
pub async fn gateway_status(
    _args: Vec<Value>,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let port = settings_store::get_gateway_port();
    let manager = state.gateway_process.lock().await;
    let state_str = manager.status_string();
    let error = manager.error_message().map(|e| e.to_string());

    // If process manager says "running", verify WS is actually connected
    let final_state = if state_str == "running" && !state.gateway_client.is_connected().await {
        "reconnecting"
    } else {
        state_str
    };

    let mut result = json!({ "state": final_state, "port": port });
    if let Some(err) = error {
        result["error"] = json!(err);
    }
    Ok(result)
}

/// gateway:isConnected — Check if WS is open
#[tauri::command]
pub async fn gateway_is_connected(
    _args: Vec<Value>,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let connected = state.gateway_client.is_connected().await;
    Ok(json!(connected))
}

/// gateway:rpc — Generic RPC call through Rust WS
/// _args: [method, params?, timeoutMs?]
#[tauri::command]
pub async fn gateway_rpc(
    _args: Vec<Value>,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let method = _args
        .first()
        .and_then(|v| v.as_str())
        .ok_or("Missing method")?;
    let params = _args.get(1).cloned().filter(|v| !v.is_null());
    let timeout_ms = _args
        .get(2)
        .and_then(|v| v.as_u64())
        .unwrap_or(30000);

    match state.gateway_client.rpc(method, params, timeout_ms).await {
        Ok(result) => Ok(json!({ "success": true, "result": result })),
        Err(e) => Ok(json!({ "success": false, "error": e })),
    }
}

/// gateway:getControlUiUrl — Build the Control UI URL with auth token
#[tauri::command]
pub async fn gateway_get_control_ui_url(
    _args: Vec<Value>,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    match settings_store::get_gateway_token(&state.data_dir) {
        Ok(token) => {
            let port = settings_store::get_gateway_port();
            let url = format!(
                "http://127.0.0.1:{}/?token={}",
                port,
                urlencoding(&token)
            );
            Ok(json!({ "success": true, "url": url, "port": port, "token": token }))
        }
        Err(e) => Ok(json!({ "success": false, "error": e })),
    }
}

/// gateway:health — Check WS connection + uptime
#[tauri::command]
pub async fn gateway_health(
    _args: Vec<Value>,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let status = state.gateway_client.status().await;
    let connected = status
        .get("connected")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if connected {
        let connected_at = status.get("connectedAt").and_then(|v| v.as_i64());
        let uptime = connected_at.map(|at| {
            (chrono::Utc::now().timestamp_millis() - at) / 1000
        });
        Ok(json!({ "success": true, "ok": true, "uptime": uptime }))
    } else {
        Ok(json!({ "success": true, "ok": false, "error": "WebSocket not connected" }))
    }
}

// ── Week 7: Lifecycle commands ──

/// gateway:start — Start the Gateway process (non-blocking)
/// Spawns the startup in a background task and returns immediately.
/// Progress is reported via `gateway_startupProgress` events.
/// _args: []
#[tauri::command]
pub async fn gateway_start(
    app: AppHandle,
    _args: Vec<Value>,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    // Quick check: already running or starting?
    {
        let manager = state.gateway_process.lock().await;
        let status = manager.status_string();
        if status == "running" {
            return Ok(json!({ "success": true, "state": "running", "message": "Gateway already running" }));
        }
        if status == "starting" {
            return Ok(json!({ "success": true, "state": "starting", "message": "Gateway is already starting" }));
        }
    }

    // Spawn background startup task — use app.state() to access managed state
    let app_clone = app.clone();

    tokio::spawn(async move {
        use tauri::Manager;
        let state = app_clone.state::<AppState>();
        let mut manager = state.gateway_process.lock().await;
        if let Err(e) = manager.start_with_progress(&app_clone, &state).await {
            eprintln!("[ClawX] Background gateway start failed: {}", e);
        }
    });

    Ok(json!({ "success": true, "state": "starting", "message": "Gateway startup initiated" }))
}

/// gateway:stop — Stop the Gateway process
/// _args: []
#[tauri::command]
pub async fn gateway_stop(
    _args: Vec<Value>,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let mut manager = state.gateway_process.lock().await;
    manager.stop(&state).await
}

/// gateway:restart — Restart the Gateway process (non-blocking)
/// _args: []
#[tauri::command]
pub async fn gateway_restart(
    app: AppHandle,
    _args: Vec<Value>,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    // Stop synchronously, then spawn the re-start in background
    {
        let mut manager = state.gateway_process.lock().await;
        let _ = manager.stop(&state).await;
    }

    let app_clone = app.clone();

    tokio::spawn(async move {
        // Brief pause before restarting
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        use tauri::Manager;
        let state = app_clone.state::<AppState>();
        let mut manager = state.gateway_process.lock().await;
        if let Err(e) = manager.start_with_progress(&app_clone, &state).await {
            eprintln!("[ClawX] Background gateway restart failed: {}", e);
        }
    });

    Ok(json!({ "success": true, "state": "starting", "message": "Gateway restart initiated" }))
}

/// Simple percent-encoding for URL query parameter
fn urlencoding(s: &str) -> String {
    let mut result = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}
