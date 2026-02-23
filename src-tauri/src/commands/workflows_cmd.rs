// Tauri commands for workflow:* channels
// 8 CRUD commands + 4 workflow run commands + 6 runtime commands (Week 7)

use crate::config;
use crate::database::workflow_runs;
use crate::storage::workflows::{self, WorkflowStoreData};
use crate::storage::JsonStore;
use crate::AppState;
use serde_json::json;
use tauri::{AppHandle, State};

fn store(state: &AppState) -> JsonStore<WorkflowStoreData> {
    JsonStore::new(&state.data_dir, config::WORKFLOW_STORE_NAME)
}

#[tauri::command]
pub async fn workflow_list(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let s = store(&state);
    let workflows = workflows::get_all_workflows(&s).await?;
    Ok(json!({ "success": true, "workflows": workflows }))
}

#[tauri::command]
pub async fn workflow_get(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing workflow id")?;
    let s = store(&state);
    let workflow = workflows::get_workflow(&s, id).await?;
    Ok(json!({ "success": true, "workflow": workflow }))
}

#[tauri::command]
pub async fn workflow_create(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let config = _args.first()
        .cloned()
        .unwrap_or(serde_json::Value::Object(Default::default()));
    let s = store(&state);
    let workflow = workflows::create_workflow(&s, config).await?;
    Ok(json!({ "success": true, "workflow": workflow }))
}

#[tauri::command]
pub async fn workflow_update(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing workflow id")?;
    let updates = _args.get(1)
        .cloned()
        .unwrap_or(serde_json::Value::Object(Default::default()));
    let s = store(&state);
    let workflow = workflows::update_workflow(&s, id, updates).await?;
    Ok(json!({ "success": true, "workflow": workflow }))
}

#[tauri::command]
pub async fn workflow_delete(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing workflow id")?;
    let s = store(&state);
    let deleted = workflows::delete_workflow(&s, id).await?;
    Ok(json!({ "success": true, "deleted": deleted }))
}

#[tauri::command]
pub async fn workflow_duplicate(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing workflow id")?;
    let s = store(&state);
    let workflow = workflows::duplicate_workflow(&s, id).await?;
    Ok(json!({ "success": true, "workflow": workflow }))
}

#[tauri::command]
pub async fn workflow_export(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing workflow id")?;
    let s = store(&state);
    let json_str = workflows::export_workflow(&s, id).await?;
    Ok(json!({ "success": true, "json": json_str }))
}

#[tauri::command]
pub async fn workflow_import(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let json_str = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing JSON string")?;
    let s = store(&state);
    let workflow = workflows::import_workflow(&s, json_str).await?;
    Ok(json!({ "success": true, "workflow": workflow }))
}

// ── Workflow Run commands (Week 4) ──

#[tauri::command]
pub async fn workflow_get_runs(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let workflow_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing workflow id")?;
    let limit = _args.get(1)
        .and_then(|v| v.as_u64())
        .unwrap_or(50) as u32;

    let runs = state.db_manager.with_runs_conn(|conn| {
        workflow_runs::get_runs_for_workflow(conn, workflow_id, limit)
    })?;
    Ok(json!({ "success": true, "runs": runs }))
}

#[tauri::command]
pub async fn workflow_get_run(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let run_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing run id")?;

    let run = state.db_manager.with_runs_conn(|conn| {
        workflow_runs::get_run(conn, run_id)
    })?;
    Ok(json!({ "success": true, "run": run }))
}

#[tauri::command]
pub async fn workflow_delete_run(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let run_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing run id")?;

    let deleted = state.db_manager.with_runs_conn(|conn| {
        workflow_runs::delete_run(conn, run_id)
    })?;
    Ok(json!({ "success": true, "deleted": deleted }))
}

#[tauri::command]
pub async fn workflow_clear_runs(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let workflow_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing workflow id")?;

    let cleared = state.db_manager.with_runs_conn(|conn| {
        workflow_runs::clear_runs_for_workflow(conn, workflow_id)
    })?;
    Ok(json!({ "success": true, "cleared": cleared }))
}

// ── Week 7: Runtime commands ──

/// workflow:execute — Execute a workflow
/// _args: [workflowId, input, triggerType?]
#[tauri::command]
pub async fn workflow_execute(
    app: AppHandle,
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let workflow_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing workflow id")?;
    let input = _args.get(1)
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let trigger_type = _args.get(2)
        .and_then(|v| v.as_str())
        .unwrap_or("manual");

    // Get workflow
    let s = store(&state);
    let workflow = workflows::get_workflow(&s, workflow_id).await?
        .ok_or("Workflow not found")?;

    // Execute
    let mut engine = state.workflow_engine.lock().await;
    engine.execute(&app, &state, &workflow, input, trigger_type).await
}

/// workflow:cancel — Cancel a running workflow
/// _args: [runId]
#[tauri::command]
pub async fn workflow_cancel(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let run_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing run id")?;

    let mut engine = state.workflow_engine.lock().await;
    engine.cancel(run_id);

    Ok(json!({ "success": true }))
}

/// workflow:registerTriggers — Register triggers for a workflow
/// _args: [workflowId]
#[tauri::command]
pub async fn workflow_register_triggers(
    app: AppHandle,
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let workflow_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing workflow id")?;

    let s = store(&state);
    let workflow = workflows::get_workflow(&s, workflow_id).await?
        .ok_or("Workflow not found")?;

    let mut triggers = state.workflow_triggers.lock().map_err(|e| e.to_string())?;
    triggers.register_triggers(&app, &workflow)?;

    Ok(json!({ "success": true }))
}

/// workflow:unregisterTriggers — Unregister triggers for a workflow
/// _args: [workflowId]
#[tauri::command]
pub async fn workflow_unregister_triggers(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let workflow_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing workflow id")?;

    let mut triggers = state.workflow_triggers.lock().map_err(|e| e.to_string())?;
    triggers.unregister_triggers(workflow_id);

    Ok(json!({ "success": true }))
}

/// workflow:getTemplates — Get workflow templates (placeholder)
/// _args: []
#[tauri::command]
pub async fn workflow_get_templates(
    _args: Vec<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    // Placeholder: return empty templates list
    Ok(json!({ "success": true, "templates": [] }))
}

/// workflow:importTemplate — Import a workflow template
/// _args: [templateJson]
#[tauri::command]
pub async fn workflow_import_template(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let json_str = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing template JSON")?;

    let s = store(&state);
    let workflow = workflows::import_workflow(&s, json_str).await?;
    Ok(json!({ "success": true, "workflow": workflow }))
}
