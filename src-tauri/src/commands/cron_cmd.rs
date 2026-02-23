// Tauri commands for cron:* channels
// 6 commands — thin wrappers around Gateway RPC
// Ported from ipc-handlers.ts:267-376

use crate::AppState;
use serde_json::{json, Value};
use tauri::State;

/// Transform a Gateway CronJob to the frontend CronJob format
/// Ported from ipc-handlers.ts:219-258
fn transform_cron_job(job: &Value) -> Value {
    let message = job
        .get("payload")
        .and_then(|p| {
            p.get("message")
                .or_else(|| p.get("text"))
                .and_then(|v| v.as_str())
        })
        .unwrap_or("");

    let channel_type = job
        .get("delivery")
        .and_then(|d| d.get("channel"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let target = json!({
        "channelType": channel_type,
        "channelId": channel_type,
        "channelName": channel_type,
    });

    let last_run = job
        .get("state")
        .and_then(|s| s.get("lastRunAtMs"))
        .and_then(|v| v.as_i64())
        .map(|ms| {
            let last_status = job
                .get("state")
                .and_then(|s| s.get("lastStatus"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let last_error = job
                .get("state")
                .and_then(|s| s.get("lastError"))
                .and_then(|v| v.as_str());
            let last_duration = job
                .get("state")
                .and_then(|s| s.get("lastDurationMs"))
                .and_then(|v| v.as_u64());

            let mut lr = json!({
                "time": ms_to_iso(ms),
                "success": last_status == "ok",
            });
            if let Some(err) = last_error {
                lr["error"] = json!(err);
            }
            if let Some(dur) = last_duration {
                lr["duration"] = json!(dur);
            }
            lr
        });

    let next_run = job
        .get("state")
        .and_then(|s| s.get("nextRunAtMs"))
        .and_then(|v| v.as_i64())
        .map(ms_to_iso);

    let created_at = job
        .get("createdAtMs")
        .and_then(|v| v.as_i64())
        .map(ms_to_iso)
        .unwrap_or_default();

    let updated_at = job
        .get("updatedAtMs")
        .and_then(|v| v.as_i64())
        .map(ms_to_iso)
        .unwrap_or_default();

    let mut result = json!({
        "id": job.get("id"),
        "name": job.get("name"),
        "message": message,
        "schedule": job.get("schedule"),
        "target": target,
        "enabled": job.get("enabled"),
        "createdAt": created_at,
        "updatedAt": updated_at,
    });

    if let Some(lr) = last_run {
        result["lastRun"] = lr;
    }
    if let Some(nr) = next_run {
        result["nextRun"] = json!(nr);
    }

    result
}

fn ms_to_iso(ms: i64) -> String {
    chrono::DateTime::from_timestamp_millis(ms)
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default()
}

/// cron:list — List all cron jobs
#[tauri::command]
pub async fn cron_list(
    _args: Vec<Value>,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let result = state
        .gateway_client
        .rpc("cron.list", Some(json!({"includeDisabled": true})), 30000)
        .await?;

    let jobs = result
        .get("jobs")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let transformed: Vec<Value> = jobs.iter().map(transform_cron_job).collect();
    Ok(json!(transformed))
}

/// cron:create — Create a new cron job
/// _args: [input] where input = { name, message, schedule, target: { channelType, channelId, channelName }, enabled? }
#[tauri::command]
pub async fn cron_create(
    _args: Vec<Value>,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let input = _args.first().ok_or("Missing input")?;

    let name = input.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let message = input.get("message").and_then(|v| v.as_str()).unwrap_or("");
    let schedule = input.get("schedule").and_then(|v| v.as_str()).unwrap_or("");
    let enabled = input.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);

    let channel_type = input
        .get("target")
        .and_then(|t| t.get("channelType"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let channel_id = input
        .get("target")
        .and_then(|t| t.get("channelId"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Discord recipient must be prefixed with "channel:"
    let delivery_to = if channel_type == "discord" && !channel_id.is_empty() {
        format!("channel:{}", channel_id)
    } else {
        channel_id.to_string()
    };

    let gateway_input = json!({
        "name": name,
        "schedule": { "kind": "cron", "expr": schedule },
        "payload": { "kind": "agentTurn", "message": message },
        "enabled": enabled,
        "wakeMode": "next-heartbeat",
        "sessionTarget": "isolated",
        "delivery": {
            "mode": "announce",
            "channel": channel_type,
            "to": delivery_to,
        },
    });

    let result = state
        .gateway_client
        .rpc("cron.add", Some(gateway_input), 30000)
        .await?;

    Ok(transform_cron_job(&result))
}

/// cron:update — Update an existing cron job
/// _args: [id, input]
#[tauri::command]
pub async fn cron_update(
    _args: Vec<Value>,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let id = _args.first().and_then(|v| v.as_str()).ok_or("Missing id")?;
    let input = _args.get(1).cloned().unwrap_or(json!({}));

    let mut patch = input.clone();

    // Transform schedule string → CronSchedule object
    if let Some(sched) = patch.get("schedule").and_then(|v| v.as_str()) {
        let expr = sched.to_string();
        patch["schedule"] = json!({ "kind": "cron", "expr": expr });
    }

    // Transform message → payload
    if let Some(msg) = patch.get("message").and_then(|v| v.as_str()) {
        let msg_str = msg.to_string();
        patch["payload"] = json!({ "kind": "agentTurn", "message": msg_str });
        if let Some(obj) = patch.as_object_mut() {
            obj.remove("message");
        }
    }

    let result = state
        .gateway_client
        .rpc("cron.update", Some(json!({ "id": id, "patch": patch })), 30000)
        .await?;

    Ok(result)
}

/// cron:delete — Delete a cron job
/// _args: [id]
#[tauri::command]
pub async fn cron_delete(
    _args: Vec<Value>,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let id = _args.first().and_then(|v| v.as_str()).ok_or("Missing id")?;
    let result = state
        .gateway_client
        .rpc("cron.remove", Some(json!({ "id": id })), 30000)
        .await?;
    Ok(result)
}

/// cron:toggle — Toggle cron job enabled/disabled
/// _args: [id, enabled]
#[tauri::command]
pub async fn cron_toggle(
    _args: Vec<Value>,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let id = _args.first().and_then(|v| v.as_str()).ok_or("Missing id")?;
    let enabled = _args
        .get(1)
        .and_then(|v| v.as_bool())
        .ok_or("Missing enabled")?;

    let result = state
        .gateway_client
        .rpc(
            "cron.update",
            Some(json!({ "id": id, "patch": { "enabled": enabled } })),
            30000,
        )
        .await?;
    Ok(result)
}

/// cron:trigger — Trigger a cron job manually
/// _args: [id]
#[tauri::command]
pub async fn cron_trigger(
    _args: Vec<Value>,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let id = _args.first().and_then(|v| v.as_str()).ok_or("Missing id")?;
    let result = state
        .gateway_client
        .rpc("cron.run", Some(json!({ "id": id, "mode": "force" })), 30000)
        .await?;
    Ok(result)
}
