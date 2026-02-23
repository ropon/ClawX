// Agent storage — CRUD operations for AgentConfig

use crate::config;
use crate::storage::JsonStore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub avatar: String,
    pub description: String,
    pub system_prompt: String,
    pub provider_id: Option<String>,
    pub model: String,
    pub temperature: f64,
    pub max_tokens: Option<u32>,
    pub skill_ids: Vec<String>,
    pub knowledge_base_ids: Vec<String>,
    pub channel_bindings: Vec<String>,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AgentStoreData {
    pub agents: HashMap<String, AgentConfig>,
    pub active_agent_id: Option<String>,
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn make_default_agent() -> AgentConfig {
    let now = now_iso();
    AgentConfig {
        id: config::DEFAULT_AGENT_ID.to_string(),
        name: config::DEFAULT_AGENT_NAME.to_string(),
        avatar: config::DEFAULT_AGENT_AVATAR.to_string(),
        description: config::DEFAULT_AGENT_DESCRIPTION.to_string(),
        system_prompt: String::new(),
        provider_id: None,
        model: String::new(),
        temperature: config::DEFAULT_TEMPERATURE,
        max_tokens: None,
        skill_ids: vec![],
        knowledge_base_ids: vec![],
        channel_bindings: vec![],
        is_default: true,
        created_at: now.clone(),
        updated_at: now,
    }
}

/// Ensure at least one default agent exists
async fn ensure_default_agent(store: &JsonStore<AgentStoreData>) -> Result<AgentStoreData, String> {
    let mut data = store.load().await?;
    if data.agents.is_empty() {
        let agent = make_default_agent();
        data.active_agent_id = Some(agent.id.clone());
        data.agents.insert(agent.id.clone(), agent);
        store.save(&data).await?;
    }
    Ok(data)
}

pub async fn get_all_agents(store: &JsonStore<AgentStoreData>) -> Result<Vec<AgentConfig>, String> {
    let data = ensure_default_agent(store).await?;
    Ok(data.agents.into_values().collect())
}

pub async fn get_agent(store: &JsonStore<AgentStoreData>, id: &str) -> Result<Option<AgentConfig>, String> {
    let data = store.load().await?;
    Ok(data.agents.get(id).cloned())
}

pub async fn create_agent(
    store: &JsonStore<AgentStoreData>,
    config: serde_json::Value,
) -> Result<AgentConfig, String> {
    let mut data = store.load().await?;
    let now = now_iso();
    let id = format!("agent-{}", uuid::Uuid::new_v4());

    let agent = AgentConfig {
        id: id.clone(),
        name: config.get("name").and_then(|v| v.as_str()).unwrap_or("New Agent").to_string(),
        avatar: config.get("avatar").and_then(|v| v.as_str()).unwrap_or(config::DEFAULT_AGENT_AVATAR).to_string(),
        description: config.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        system_prompt: config.get("systemPrompt").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        provider_id: config.get("providerId").and_then(|v| v.as_str().map(|s| s.to_string())),
        model: config.get("model").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        temperature: config.get("temperature").and_then(|v| v.as_f64()).unwrap_or(config::DEFAULT_TEMPERATURE),
        max_tokens: config.get("maxTokens").and_then(|v| v.as_u64().map(|n| n as u32)),
        skill_ids: json_str_array(&config, "skillIds"),
        knowledge_base_ids: json_str_array(&config, "knowledgeBaseIds"),
        channel_bindings: json_str_array(&config, "channelBindings"),
        is_default: config.get("isDefault").and_then(|v| v.as_bool()).unwrap_or(false),
        created_at: now.clone(),
        updated_at: now,
    };

    data.agents.insert(id, agent.clone());
    store.save(&data).await?;
    Ok(agent)
}

pub async fn update_agent(
    store: &JsonStore<AgentStoreData>,
    id: &str,
    updates: serde_json::Value,
) -> Result<Option<AgentConfig>, String> {
    let mut data = store.load().await?;
    let agent = match data.agents.get(id) {
        Some(a) => a.clone(),
        None => return Ok(None),
    };

    let updated = AgentConfig {
        id: id.to_string(),
        name: str_or(&updates, "name", &agent.name),
        avatar: str_or(&updates, "avatar", &agent.avatar),
        description: str_or(&updates, "description", &agent.description),
        system_prompt: str_or(&updates, "systemPrompt", &agent.system_prompt),
        provider_id: if updates.get("providerId").is_some() {
            updates.get("providerId").and_then(|v| v.as_str().map(|s| s.to_string()))
        } else {
            agent.provider_id
        },
        model: str_or(&updates, "model", &agent.model),
        temperature: updates.get("temperature").and_then(|v| v.as_f64()).unwrap_or(agent.temperature),
        max_tokens: if updates.get("maxTokens").is_some() {
            updates.get("maxTokens").and_then(|v| v.as_u64().map(|n| n as u32))
        } else {
            agent.max_tokens
        },
        skill_ids: if updates.get("skillIds").is_some() {
            json_str_array(&updates, "skillIds")
        } else {
            agent.skill_ids
        },
        knowledge_base_ids: if updates.get("knowledgeBaseIds").is_some() {
            json_str_array(&updates, "knowledgeBaseIds")
        } else {
            agent.knowledge_base_ids
        },
        channel_bindings: if updates.get("channelBindings").is_some() {
            json_str_array(&updates, "channelBindings")
        } else {
            agent.channel_bindings
        },
        is_default: updates.get("isDefault").and_then(|v| v.as_bool()).unwrap_or(agent.is_default),
        created_at: agent.created_at,
        updated_at: now_iso(),
    };

    data.agents.insert(id.to_string(), updated.clone());
    store.save(&data).await?;
    Ok(Some(updated))
}

pub async fn delete_agent(store: &JsonStore<AgentStoreData>, id: &str) -> Result<bool, String> {
    let mut data = store.load().await?;

    match data.agents.get(id) {
        Some(a) if a.is_default => return Ok(false), // Cannot delete default agent
        None => return Ok(false),
        _ => {}
    }

    data.agents.remove(id);

    // If active agent was deleted, switch to default
    if data.active_agent_id.as_deref() == Some(id) {
        data.active_agent_id = data.agents.values().find(|a| a.is_default).map(|a| a.id.clone());
    }

    store.save(&data).await?;
    Ok(true)
}

pub async fn get_active_agent_id(store: &JsonStore<AgentStoreData>) -> Result<Option<String>, String> {
    let data = ensure_default_agent(store).await?;
    Ok(data.active_agent_id)
}

pub async fn set_active_agent_id(store: &JsonStore<AgentStoreData>, id: &str) -> Result<(), String> {
    let mut data = store.load().await?;
    data.active_agent_id = Some(id.to_string());
    store.save(&data).await
}

pub async fn export_agent(store: &JsonStore<AgentStoreData>, id: &str) -> Result<Option<String>, String> {
    let data = store.load().await?;
    let agent = match data.agents.get(id) {
        Some(a) => a,
        None => return Ok(None),
    };

    // Export without id, createdAt, updatedAt
    let mut map = serde_json::to_value(agent).map_err(|e| e.to_string())?;
    if let Some(obj) = map.as_object_mut() {
        obj.remove("id");
        obj.remove("createdAt");
        obj.remove("updatedAt");
    }
    let json = serde_json::to_string_pretty(&map).map_err(|e| e.to_string())?;
    Ok(Some(json))
}

pub async fn import_agent(
    store: &JsonStore<AgentStoreData>,
    json: &str,
) -> Result<AgentConfig, String> {
    let data: serde_json::Value = serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {}", e))?;

    // Force isDefault to false for imports
    let mut config = data.clone();
    if let Some(obj) = config.as_object_mut() {
        obj.remove("id");
        obj.remove("createdAt");
        obj.remove("updatedAt");
        obj.insert("isDefault".to_string(), serde_json::Value::Bool(false));
    }

    create_agent(store, config).await
}

// Helpers

fn str_or(val: &serde_json::Value, key: &str, default: &str) -> String {
    val.get(key).and_then(|v| v.as_str()).unwrap_or(default).to_string()
}

fn json_str_array(val: &serde_json::Value, key: &str) -> Vec<String> {
    val.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn test_store() -> (TempDir, JsonStore<AgentStoreData>) {
        let tmp = TempDir::new().unwrap();
        let store = JsonStore::new(tmp.path(), "test-agents");
        (tmp, store)
    }

    #[tokio::test]
    async fn test_ensure_default_agent() {
        let (_tmp, store) = test_store().await;
        let data = ensure_default_agent(&store).await.unwrap();
        assert_eq!(data.agents.len(), 1);
        assert!(data.agents.contains_key("default"));
        assert_eq!(data.active_agent_id, Some("default".to_string()));
    }

    #[tokio::test]
    async fn test_crud_agent() {
        let (_tmp, store) = test_store().await;

        // Ensure default agent is created first
        let all = get_all_agents(&store).await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, "default");

        // Create
        let config = serde_json::json!({
            "name": "Test Agent",
            "avatar": "🧪",
            "description": "A test agent",
            "systemPrompt": "You are a test agent.",
        });
        let agent = create_agent(&store, config).await.unwrap();
        assert!(agent.id.starts_with("agent-"));
        assert_eq!(agent.name, "Test Agent");

        // Get
        let fetched = get_agent(&store, &agent.id).await.unwrap().unwrap();
        assert_eq!(fetched.name, "Test Agent");

        // Update
        let updates = serde_json::json!({ "name": "Updated Agent" });
        let updated = update_agent(&store, &agent.id, updates).await.unwrap().unwrap();
        assert_eq!(updated.name, "Updated Agent");

        // List: default + created
        let all = get_all_agents(&store).await.unwrap();
        assert_eq!(all.len(), 2);

        // Delete
        let deleted = delete_agent(&store, &agent.id).await.unwrap();
        assert!(deleted);

        // Cannot delete default
        let deleted = delete_agent(&store, "default").await.unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_export_import() {
        let (_tmp, store) = test_store().await;

        let config = serde_json::json!({
            "name": "Export Test",
            "avatar": "📤",
        });
        let agent = create_agent(&store, config).await.unwrap();

        let json = export_agent(&store, &agent.id).await.unwrap().unwrap();
        assert!(!json.contains(&agent.id));

        let imported = import_agent(&store, &json).await.unwrap();
        assert_ne!(imported.id, agent.id);
        assert_eq!(imported.name, "Export Test");
        assert!(!imported.is_default);
    }
}
