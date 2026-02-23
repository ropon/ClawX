// Tauri commands for agent:* channels
// 9 commands mapping to agent CRUD operations

use crate::config;
use crate::storage::agents::{self, AgentStoreData};
use crate::storage::JsonStore;
use crate::AppState;
use serde_json::json;
use tauri::State;

fn store(state: &AppState) -> JsonStore<AgentStoreData> {
    JsonStore::new(&state.data_dir, config::AGENT_STORE_NAME)
}

#[tauri::command]
pub async fn agent_list(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let s = store(&state);
    let agents = agents::get_all_agents(&s).await?;
    Ok(json!({ "success": true, "agents": agents }))
}

#[tauri::command]
pub async fn agent_get(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing agent id")?;
    let s = store(&state);
    let agent = agents::get_agent(&s, id).await?;
    Ok(json!({ "success": true, "agent": agent }))
}

#[tauri::command]
pub async fn agent_create(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let config = _args.first()
        .cloned()
        .unwrap_or(serde_json::Value::Object(Default::default()));
    let s = store(&state);
    let agent = agents::create_agent(&s, config).await?;
    Ok(json!({ "success": true, "agent": agent }))
}

#[tauri::command]
pub async fn agent_update(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing agent id")?;
    let updates = _args.get(1)
        .cloned()
        .unwrap_or(serde_json::Value::Object(Default::default()));
    let s = store(&state);
    let agent = agents::update_agent(&s, id, updates).await?;
    Ok(json!({ "success": true, "agent": agent }))
}

#[tauri::command]
pub async fn agent_delete(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing agent id")?;
    let s = store(&state);
    let deleted = agents::delete_agent(&s, id).await?;
    Ok(json!({ "success": true, "deleted": deleted }))
}

#[tauri::command]
pub async fn agent_get_active(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let s = store(&state);
    let id = agents::get_active_agent_id(&s).await?;
    Ok(json!({ "success": true, "activeAgentId": id }))
}

#[tauri::command]
pub async fn agent_set_active(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing agent id")?;
    let s = store(&state);
    agents::set_active_agent_id(&s, id).await?;
    Ok(json!({ "success": true }))
}

#[tauri::command]
pub async fn agent_export(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing agent id")?;
    let s = store(&state);
    let json_str = agents::export_agent(&s, id).await?;
    Ok(json!({ "success": true, "json": json_str }))
}

#[tauri::command]
pub async fn agent_import(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let json_str = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing JSON string")?;
    let s = store(&state);
    let agent = agents::import_agent(&s, json_str).await?;
    Ok(json!({ "success": true, "agent": agent }))
}
