// Gateway lifecycle — Process spawn/stop/restart management

use crate::openclaw_paths;
use crate::providers;
use crate::secure_storage;
use crate::settings_store;
use crate::AppState;
use serde_json::json;
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub struct GatewayProcessManager {
    child: Option<tokio::process::Child>,
    owns_process: bool,
    should_reconnect: bool,
    reconnect_attempts: u32,
    status: GatewayStatus,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum GatewayStatus {
    Stopped,
    Starting,
    Running,
    Error(String),
    Reconnecting,
}

impl GatewayProcessManager {
    pub fn new() -> Self {
        Self {
            child: None,
            owns_process: false,
            should_reconnect: true,
            reconnect_attempts: 0,
            status: GatewayStatus::Stopped,
        }
    }

    /// Start the Gateway process (original blocking version, kept for internal use)
    pub async fn start(
        &mut self,
        app: &AppHandle,
        state: &AppState,
    ) -> Result<serde_json::Value, String> {
        self.start_with_progress(app, state).await
    }

    /// Start the Gateway process with progress events for real-time UI feedback
    pub async fn start_with_progress(
        &mut self,
        app: &AppHandle,
        state: &AppState,
    ) -> Result<serde_json::Value, String> {
        if self.status == GatewayStatus::Running {
            emit_progress(app, "ready", "Gateway already running", 0, 100);
            return Ok(json!({ "success": true, "message": "Gateway already running" }));
        }

        let start_time = std::time::Instant::now();

        self.status = GatewayStatus::Starting;
        emit_status(app, "starting");

        let port = settings_store::get_gateway_port();

        // Phase 1: Probing — fast check if Gateway is already running
        emit_progress(app, "probing", "Checking for existing Gateway...", 0, 5);

        if let Ok(()) = ws_probe_fast(port).await {
            self.status = GatewayStatus::Running;
            self.owns_process = false;

            // Connect Rust WS client
            let token = settings_store::get_gateway_token(&state.data_dir)
                .unwrap_or_else(|_| String::new());
            let _ = state.gateway_client.connect(port, &token).await;

            let elapsed = start_time.elapsed().as_millis() as u64;
            emit_progress(app, "ready", "Connected to existing Gateway", elapsed, 100);
            emit_status(app, "running");
            return Ok(json!({
                "success": true,
                "message": "Connected to existing Gateway",
                "port": port,
            }));
        }

        // Phase 2: Spawning — prepare and launch Gateway process
        let elapsed = start_time.elapsed().as_millis() as u64;
        emit_progress(app, "spawning", "Preparing OpenClaw runtime...", elapsed, 10);

        // Ensure openclaw runtime is extracted before resolving entry path.
        // First launch / after a ClawX update will block here for ~3-5 s while
        // ~/.openclaw/runtime/openclaw is unpacked from the bundled tar.gz.
        let app_clone = app.clone();
        match tokio::task::spawn_blocking(move || {
            crate::openclaw_install::ensure_installed(&app_clone)
        }).await {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => {
                self.status = GatewayStatus::Error(format!("openclaw extract: {e}"));
                let elapsed = start_time.elapsed().as_millis() as u64;
                emit_progress(app, "failed", &format!("OpenClaw extract failed: {e}"), elapsed, 0);
                emit_status(app, "error");
                return Err(format!("OpenClaw extract failed: {e}"));
            }
            Err(e) => {
                let msg = format!("extract task: {e}");
                self.status = GatewayStatus::Error(msg.clone());
                let elapsed = start_time.elapsed().as_millis() as u64;
                emit_progress(app, "failed", &msg, elapsed, 0);
                emit_status(app, "error");
                return Err(msg);
            }
        }

        let elapsed = start_time.elapsed().as_millis() as u64;
        emit_progress(app, "spawning", "Launching Gateway process...", elapsed, 15);

        let entry_path = openclaw_paths::get_openclaw_entry_path();
        if !PathBuf::from(&entry_path).exists() {
            self.status = GatewayStatus::Error("OpenClaw not installed".to_string());
            emit_progress(app, "failed", "OpenClaw not installed", elapsed, 0);
            emit_status(app, "error");
            return Err("OpenClaw entry script not found. Please install OpenClaw first.".to_string());
        }

        // Get or generate gateway token, and persist it so the WS client can read it later
        let token = settings_store::get_gateway_token(&state.data_dir)
            .unwrap_or_else(|_| {
                let new_token = uuid::Uuid::new_v4().to_string();
                let _ = settings_store::save_gateway_token(&state.data_dir, &new_token);
                new_token
            });

        // Build env vars (inject provider API keys)
        let mut env_vars: Vec<(String, String)> = vec![
            ("OPENCLAW_GATEWAY_TOKEN".to_string(), token.clone()),
            ("OPENCLAW_NO_RESPAWN".to_string(), "1".to_string()),
            (
                "NODE_OPTIONS".to_string(),
                "--disable-warning=ExperimentalWarning".to_string(),
            ),
        ];

        // Inject provider API keys as environment variables
        if let Ok(store) = secure_storage::get_all_providers_with_key_info(&state.data_dir) {
            if let Some(providers_arr) = store.as_array() {
                for prov in providers_arr {
                    let ptype = prov.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    let pid = prov.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    let has_key = prov.get("hasKey").and_then(|v| v.as_bool()).unwrap_or(false);

                    if has_key {
                        if let Ok(Some(key)) = secure_storage::get_api_key(&state.data_dir, pid) {
                            if let Some(env_var) = providers::get_provider_env_var(ptype) {
                                env_vars.push((env_var.to_string(), key));
                            }
                        }
                    }
                }
            }
        }

        // Spawn using tokio::process::Command (bypasses Tauri shell plugin scope)
        let port_str = port.to_string();
        let mut cmd = tokio::process::Command::new("node");
        cmd.args(&[
            &entry_path,
            "gateway",
            "--port",
            &port_str,
            "--token",
            &token,
            "--dev",
            "--allow-unconfigured",
        ]);

        for (k, v) in &env_vars {
            cmd.env(k, v);
        }

        // Capture stdout/stderr for logging
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        match cmd.spawn() {
            Ok(mut child) => {
                let pid = child.id().unwrap_or(0);

                // Spawn background tasks to read stdout/stderr
                if let Some(stdout) = child.stdout.take() {
                    tokio::spawn(async move {
                        use tokio::io::{AsyncBufReadExt, BufReader};
                        let reader = BufReader::new(stdout);
                        let mut lines = reader.lines();
                        while let Ok(Some(line)) = lines.next_line().await {
                            println!("[Gateway:stdout] {}", line);
                        }
                    });
                }
                if let Some(stderr) = child.stderr.take() {
                    tokio::spawn(async move {
                        use tokio::io::{AsyncBufReadExt, BufReader};
                        let reader = BufReader::new(stderr);
                        let mut lines = reader.lines();
                        while let Ok(Some(line)) = lines.next_line().await {
                            println!("[Gateway:stderr] {}", line);
                        }
                    });
                }
                self.child = Some(child);
                self.owns_process = true;
                self.should_reconnect = true;
                self.reconnect_attempts = 0;

                let elapsed = start_time.elapsed().as_millis() as u64;
                emit_progress(app, "waiting_ready", "Waiting for Gateway to respond...", elapsed, 30);

                // Phase 3: Waiting for ready — poll with progress events
                match wait_for_ready(port, 30, Some(app)).await {
                    Ok(()) => {
                        // Phase 4: Connecting WS client
                        let elapsed = start_time.elapsed().as_millis() as u64;
                        emit_progress(app, "connecting", "Establishing WebSocket connection...", elapsed, 85);

                        match state.gateway_client.connect(port, &token).await {
                            Ok(()) => {
                                self.status = GatewayStatus::Running;
                                let elapsed = start_time.elapsed().as_millis() as u64;
                                emit_progress(app, "ready", "Gateway is ready", elapsed, 100);
                                emit_status(app, "running");
                                Ok(json!({
                                    "success": true,
                                    "message": "Gateway started",
                                    "port": port,
                                    "pid": pid,
                                }))
                            }
                            Err(e) => {
                                self.status = GatewayStatus::Error(e.clone());
                                let elapsed = start_time.elapsed().as_millis() as u64;
                                emit_progress(app, "failed", &format!("WS connect failed: {}", e), elapsed, 0);
                                emit_status(app, "error");
                                Err(format!("Gateway started but WS connect failed: {}", e))
                            }
                        }
                    }
                    Err(e) => {
                        self.status = GatewayStatus::Error(e.clone());
                        let elapsed = start_time.elapsed().as_millis() as u64;
                        emit_progress(app, "failed", &format!("Gateway not ready: {}", e), elapsed, 0);
                        emit_status(app, "error");
                        Err(format!("Gateway failed to become ready: {}", e))
                    }
                }
            }
            Err(e) => {
                self.status = GatewayStatus::Error(e.to_string());
                let elapsed = start_time.elapsed().as_millis() as u64;
                emit_progress(app, "failed", &format!("Spawn failed: {}", e), elapsed, 0);
                emit_status(app, "error");
                Err(format!("Failed to spawn Gateway process: {}", e))
            }
        }
    }

    /// Stop the Gateway process
    pub async fn stop(
        &mut self,
        state: &AppState,
    ) -> Result<serde_json::Value, String> {
        self.should_reconnect = false;

        // Try graceful shutdown via RPC if connected
        if state.gateway_client.is_connected().await {
            let _ = state
                .gateway_client
                .rpc("shutdown", None, 5000)
                .await;
        }

        // Kill the process if we own it
        if let Some(mut child) = self.child.take() {
            let _ = child.kill().await;
        }

        self.owns_process = false;
        self.status = GatewayStatus::Stopped;

        Ok(json!({ "success": true, "message": "Gateway stopped" }))
    }

    /// Restart the Gateway process
    pub async fn restart(
        &mut self,
        app: &AppHandle,
        state: &AppState,
    ) -> Result<serde_json::Value, String> {
        self.stop(state).await?;

        // Wait a moment before restarting
        tokio::time::sleep(Duration::from_secs(1)).await;

        self.start(app, state).await
    }

    /// Get the current status as a string matching frontend GatewayStatus.state
    pub fn status_string(&self) -> &str {
        match &self.status {
            GatewayStatus::Stopped => "stopped",
            GatewayStatus::Starting => "starting",
            GatewayStatus::Running => "running",
            GatewayStatus::Error(_) => "error",
            GatewayStatus::Reconnecting => "reconnecting",
        }
    }

    /// Get the error message if in error state
    pub fn error_message(&self) -> Option<&str> {
        if let GatewayStatus::Error(msg) = &self.status {
            Some(msg)
        } else {
            None
        }
    }
}

impl Default for GatewayProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Fast WebSocket probe — used for initial check when Gateway is likely not running
/// Short timeout (500ms) to avoid blocking
async fn ws_probe_fast(port: u16) -> Result<(), String> {
    let url = format!("ws://127.0.0.1:{}/ws", port);

    match tokio::time::timeout(
        Duration::from_millis(500),
        tokio_tungstenite::connect_async(&url),
    )
    .await
    {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(format!("WS probe failed: {}", e)),
        Err(_) => Err("WS probe timeout".to_string()),
    }
}

/// Poll WebSocket probe — used during wait_for_ready with moderate timeout
async fn ws_probe_poll(port: u16) -> Result<(), String> {
    let url = format!("ws://127.0.0.1:{}/ws", port);

    match tokio::time::timeout(
        Duration::from_millis(800),
        tokio_tungstenite::connect_async(&url),
    )
    .await
    {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(format!("WS probe failed: {}", e)),
        Err(_) => Err("WS probe timeout".to_string()),
    }
}

/// Wait for Gateway to be ready (poll WS endpoint)
/// Optimized: 150ms intervals, emits progress events every ~750ms
async fn wait_for_ready(
    port: u16,
    max_seconds: u64,
    app: Option<&AppHandle>,
) -> Result<(), String> {
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(max_seconds);
    let mut poll_count: u32 = 0;

    loop {
        if start.elapsed() > timeout {
            return Err(format!(
                "Gateway not ready after {} seconds",
                max_seconds
            ));
        }

        if ws_probe_poll(port).await.is_ok() {
            return Ok(());
        }

        poll_count += 1;

        // Emit progress every 5 polls (~750ms) to avoid flooding
        if let Some(app) = app {
            if poll_count % 5 == 0 {
                let elapsed = start.elapsed().as_millis() as u64;
                // Linear interpolation: 30% → 85% over the wait period
                let max_ms = max_seconds * 1000;
                let pct = 30 + ((elapsed.min(max_ms) * 55) / max_ms) as u8;
                emit_progress(app, "waiting_ready", "Waiting for Gateway to respond...", elapsed, pct);
            }
        }

        tokio::time::sleep(Duration::from_millis(150)).await;
    }
}

/// Emit gateway status event to frontend.
/// Payload matches the frontend GatewayStatus type: { state, port }
fn emit_status(app: &AppHandle, status: &str) {
    let port = crate::settings_store::get_gateway_port();
    let _ = app.emit(
        "gateway_statusChanged",
        json!({ "state": status, "port": port }),
    );
}

/// Emit startup progress event to frontend for real-time UI feedback
fn emit_progress(app: &AppHandle, phase: &str, message: &str, elapsed_ms: u64, pct: u8) {
    let _ = app.emit(
        "gateway_startupProgress",
        json!({
            "phase": phase,
            "message": message,
            "elapsedMs": elapsed_ms,
            "progressPct": pct,
        }),
    );
}
