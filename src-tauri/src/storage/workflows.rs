// Workflow storage — CRUD operations for WorkflowConfig

use crate::config;
use crate::storage::JsonStore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Type definitions matching src/types/workflow.ts ──

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkflowNodeType {
    Agent,
    Condition,
    Merge,
    Input,
    Output,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionRule {
    pub id: String,
    pub handle: String,
    #[serde(rename = "type")]
    pub rule_type: String, // "keyword" | "regex" | "ai-classify"
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowNodeData {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_rules: Option<Vec<ConditionRule>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_template: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: WorkflowNodeType,
    pub position: NodePosition,
    pub data: WorkflowNodeData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_handle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_handle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TriggerType {
    Manual,
    Cron,
    FileChange,
    Clipboard,
    Shortcut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowTrigger {
    pub id: String,
    #[serde(rename = "type")]
    pub trigger_type: TriggerType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cron_expr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watch_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clipboard_pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortcut_accelerator: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
    pub triggers: Vec<WorkflowTrigger>,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStoreData {
    pub workflows: HashMap<String, WorkflowConfig>,
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

// ── CRUD Operations ──

pub async fn get_all_workflows(
    store: &JsonStore<WorkflowStoreData>,
) -> Result<Vec<WorkflowConfig>, String> {
    let data = store.load().await?;
    let mut workflows: Vec<WorkflowConfig> = data.workflows.into_values().collect();
    // Sort by updatedAt descending (matches TS behavior)
    workflows.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(workflows)
}

pub async fn get_workflow(
    store: &JsonStore<WorkflowStoreData>,
    id: &str,
) -> Result<Option<WorkflowConfig>, String> {
    let data = store.load().await?;
    Ok(data.workflows.get(id).cloned())
}

pub async fn create_workflow(
    store: &JsonStore<WorkflowStoreData>,
    config: serde_json::Value,
) -> Result<WorkflowConfig, String> {
    let mut data = store.load().await?;
    let now = now_iso();
    let id = format!("wf-{}", uuid::Uuid::new_v4());

    let nodes: Vec<WorkflowNode> = config.get("nodes")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    let edges: Vec<WorkflowEdge> = config.get("edges")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    let triggers: Vec<WorkflowTrigger> = config.get("triggers")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let workflow = WorkflowConfig {
        id: id.clone(),
        name: str_field(&config, "name", "New Workflow"),
        description: str_field(&config, "description", ""),
        icon: str_field(&config, "icon", config::DEFAULT_WORKFLOW_ICON),
        nodes,
        edges,
        triggers,
        enabled: config.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
        created_at: now.clone(),
        updated_at: now,
    };

    data.workflows.insert(id, workflow.clone());
    store.save(&data).await?;
    Ok(workflow)
}

pub async fn update_workflow(
    store: &JsonStore<WorkflowStoreData>,
    id: &str,
    updates: serde_json::Value,
) -> Result<Option<WorkflowConfig>, String> {
    let mut data = store.load().await?;
    let wf = match data.workflows.get(id) {
        Some(w) => w.clone(),
        None => return Ok(None),
    };

    let updated = WorkflowConfig {
        id: id.to_string(),
        name: str_or(&updates, "name", &wf.name),
        description: str_or(&updates, "description", &wf.description),
        icon: str_or(&updates, "icon", &wf.icon),
        nodes: updates.get("nodes")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or(wf.nodes),
        edges: updates.get("edges")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or(wf.edges),
        triggers: updates.get("triggers")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or(wf.triggers),
        enabled: updates.get("enabled").and_then(|v| v.as_bool()).unwrap_or(wf.enabled),
        created_at: wf.created_at,
        updated_at: now_iso(),
    };

    data.workflows.insert(id.to_string(), updated.clone());
    store.save(&data).await?;
    Ok(Some(updated))
}

pub async fn delete_workflow(
    store: &JsonStore<WorkflowStoreData>,
    id: &str,
) -> Result<bool, String> {
    let mut data = store.load().await?;
    if data.workflows.remove(id).is_none() {
        return Ok(false);
    }
    store.save(&data).await?;
    Ok(true)
}

pub async fn duplicate_workflow(
    store: &JsonStore<WorkflowStoreData>,
    id: &str,
) -> Result<Option<WorkflowConfig>, String> {
    let data = store.load().await?;
    let original = match data.workflows.get(id) {
        Some(w) => w.clone(),
        None => return Ok(None),
    };

    let config = serde_json::json!({
        "name": format!("{} (Copy)", original.name),
        "description": original.description,
        "icon": original.icon,
        "nodes": original.nodes,
        "edges": original.edges,
        "triggers": [],  // Don't copy triggers
    });

    let new_wf = create_workflow(store, config).await?;
    Ok(Some(new_wf))
}

pub async fn export_workflow(
    store: &JsonStore<WorkflowStoreData>,
    id: &str,
) -> Result<Option<String>, String> {
    let data = store.load().await?;
    let wf = match data.workflows.get(id) {
        Some(w) => w,
        None => return Ok(None),
    };

    let mut val = serde_json::to_value(wf).map_err(|e| e.to_string())?;
    if let Some(obj) = val.as_object_mut() {
        obj.remove("id");
        obj.remove("createdAt");
        obj.remove("updatedAt");
    }
    let json = serde_json::to_string_pretty(&val).map_err(|e| e.to_string())?;
    Ok(Some(json))
}

pub async fn import_workflow(
    store: &JsonStore<WorkflowStoreData>,
    json: &str,
) -> Result<WorkflowConfig, String> {
    let mut data: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    if let Some(obj) = data.as_object_mut() {
        obj.remove("id");
        obj.remove("createdAt");
        obj.remove("updatedAt");
    }

    create_workflow(store, data).await
}

// Helpers

fn str_field(val: &serde_json::Value, key: &str, default: &str) -> String {
    val.get(key).and_then(|v| v.as_str()).unwrap_or(default).to_string()
}

fn str_or(val: &serde_json::Value, key: &str, default: &str) -> String {
    val.get(key).and_then(|v| v.as_str()).unwrap_or(default).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn test_store() -> (TempDir, JsonStore<WorkflowStoreData>) {
        let tmp = TempDir::new().unwrap();
        let store = JsonStore::new(tmp.path(), "test-workflows");
        (tmp, store)
    }

    #[tokio::test]
    async fn test_workflow_crud() {
        let (_tmp, store) = test_store().await;

        let config = serde_json::json!({
            "name": "Test Workflow",
            "description": "A test workflow",
        });
        let wf = create_workflow(&store, config).await.unwrap();
        assert!(wf.id.starts_with("wf-"));
        assert_eq!(wf.name, "Test Workflow");
        assert!(wf.enabled);

        // Update
        let updates = serde_json::json!({ "name": "Updated Workflow", "enabled": false });
        let updated = update_workflow(&store, &wf.id, updates).await.unwrap().unwrap();
        assert_eq!(updated.name, "Updated Workflow");
        assert!(!updated.enabled);

        // Get
        let fetched = get_workflow(&store, &wf.id).await.unwrap().unwrap();
        assert_eq!(fetched.name, "Updated Workflow");

        // List
        let all = get_all_workflows(&store).await.unwrap();
        assert_eq!(all.len(), 1);

        // Delete
        let deleted = delete_workflow(&store, &wf.id).await.unwrap();
        assert!(deleted);

        let all = get_all_workflows(&store).await.unwrap();
        assert!(all.is_empty());
    }

    #[tokio::test]
    async fn test_duplicate() {
        let (_tmp, store) = test_store().await;

        let config = serde_json::json!({
            "name": "Original",
            "icon": "🎯",
        });
        let wf = create_workflow(&store, config).await.unwrap();
        let dup = duplicate_workflow(&store, &wf.id).await.unwrap().unwrap();

        assert_ne!(dup.id, wf.id);
        assert_eq!(dup.name, "Original (Copy)");
        assert_eq!(dup.icon, "🎯");
        assert!(dup.triggers.is_empty());
    }

    #[tokio::test]
    async fn test_export_import() {
        let (_tmp, store) = test_store().await;

        let config = serde_json::json!({
            "name": "Export Test",
            "description": "For export",
        });
        let wf = create_workflow(&store, config).await.unwrap();

        let json = export_workflow(&store, &wf.id).await.unwrap().unwrap();
        assert!(!json.contains(&wf.id));

        let imported = import_workflow(&store, &json).await.unwrap();
        assert_ne!(imported.id, wf.id);
        assert_eq!(imported.name, "Export Test");
    }
}
