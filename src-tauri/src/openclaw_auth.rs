// OpenClaw Auth Profiles + Default Model configuration
// Writes to ~/.openclaw/agents/main/agent/auth-profiles.json and ~/.openclaw/openclaw.json

use crate::providers;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

const AUTH_STORE_VERSION: u32 = 1;
const AUTH_PROFILE_FILENAME: &str = "auth-profiles.json";

fn openclaw_config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".openclaw")
}

fn auth_profiles_path(agent_id: &str) -> PathBuf {
    openclaw_config_dir()
        .join("agents")
        .join(agent_id)
        .join("agent")
        .join(AUTH_PROFILE_FILENAME)
}

fn openclaw_config_path() -> PathBuf {
    openclaw_config_dir().join("openclaw.json")
}

fn read_auth_profiles(agent_id: &str) -> Value {
    let path = auth_profiles_path(agent_id);
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(data) = serde_json::from_str::<Value>(&content) {
                if data.get("version").is_some() && data.get("profiles").is_some() {
                    return data;
                }
            }
        }
    }
    json!({ "version": AUTH_STORE_VERSION, "profiles": {} })
}

fn write_auth_profiles(store: &Value, agent_id: &str) -> Result<(), String> {
    let path = auth_profiles_path(agent_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create auth profiles dir: {}", e))?;
    }
    let content = serde_json::to_string_pretty(store)
        .map_err(|e| format!("Failed to serialize auth profiles: {}", e))?;
    fs::write(&path, content)
        .map_err(|e| format!("Failed to write auth profiles: {}", e))
}

fn read_openclaw_config() -> Value {
    let path = openclaw_config_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(data) = serde_json::from_str::<Value>(&content) {
                return data;
            }
        }
    }
    json!({})
}

fn write_openclaw_config(config: &Value) -> Result<(), String> {
    let path = openclaw_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create openclaw config dir: {}", e))?;
    }
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize openclaw config: {}", e))?;
    fs::write(&path, content)
        .map_err(|e| format!("Failed to write openclaw config: {}", e))
}

/// Save a provider API key to OpenClaw's auth-profiles.json
pub fn save_provider_key_to_openclaw(provider: &str, api_key: &str) -> Result<(), String> {
    let agent_id = "main";
    let mut store = read_auth_profiles(agent_id);
    let profile_id = format!("{}:default", provider);

    // Upsert profile
    store["profiles"][&profile_id] = json!({
        "type": "api_key",
        "provider": provider,
        "key": api_key,
    });

    // Update order
    if store.get("order").is_none() {
        store["order"] = json!({});
    }
    let order = store["order"][provider]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if !order.iter().any(|v| v.as_str() == Some(&profile_id)) {
        let mut new_order = order;
        new_order.push(json!(profile_id));
        store["order"][provider] = json!(new_order);
    }

    // Set as last good
    if store.get("lastGood").is_none() {
        store["lastGood"] = json!({});
    }
    store["lastGood"][provider] = json!(profile_id);

    write_auth_profiles(&store, agent_id)
}

/// Remove a provider API key from OpenClaw auth-profiles.json
pub fn remove_provider_key_from_openclaw(provider: &str) -> Result<(), String> {
    let agent_id = "main";
    let mut store = read_auth_profiles(agent_id);
    let profile_id = format!("{}:default", provider);

    // Remove profile
    if let Some(profiles) = store.get_mut("profiles").and_then(|p| p.as_object_mut()) {
        profiles.remove(&profile_id);
    }

    // Update order
    if let Some(order) = store.get_mut("order").and_then(|o| o.as_object_mut()) {
        if let Some(provider_order) = order.get_mut(provider) {
            if let Some(arr) = provider_order.as_array_mut() {
                arr.retain(|v| v.as_str() != Some(&profile_id));
                if arr.is_empty() {
                    order.remove(provider);
                }
            }
        }
    }

    // Remove lastGood
    if let Some(last_good) = store.get_mut("lastGood").and_then(|lg| lg.as_object_mut()) {
        if last_good.get(provider).and_then(|v| v.as_str()) == Some(&profile_id) {
            last_good.remove(provider);
        }
    }

    write_auth_profiles(&store, agent_id)
}

/// Set OpenClaw default model for a provider (using registry metadata)
pub fn set_openclaw_default_model(provider: &str, model_override: Option<&str>) -> Result<(), String> {
    let model = match model_override {
        Some(m) => m.to_string(),
        None => match providers::get_provider_default_model(provider) {
            Some(m) => m.to_string(),
            None => return Ok(()), // No default model for this provider
        },
    };

    let model_id = if model.starts_with(&format!("{}/", provider)) {
        model[provider.len() + 1..].to_string()
    } else {
        model.clone()
    };

    let mut config = read_openclaw_config();

    // Set agents.defaults.model.primary
    if config.get("agents").is_none() { config["agents"] = json!({}); }
    if config["agents"].get("defaults").is_none() { config["agents"]["defaults"] = json!({}); }
    config["agents"]["defaults"]["model"] = json!({ "primary": model });

    // Configure models.providers for providers with registry config
    if let Some(provider_cfg) = providers::get_provider_config(provider) {
        if config.get("models").is_none() { config["models"] = json!({}); }
        if config["models"].get("providers").is_none() { config["models"]["providers"] = json!({}); }

        let existing = config["models"]["providers"][provider].clone();
        let existing_obj = existing.as_object();

        // Merge existing models with registry models
        let mut merged_models: Vec<Value> = Vec::new();

        // Add registry models
        if let Some(ref registry_models) = provider_cfg.models {
            for m in registry_models {
                merged_models.push(json!({ "id": m.id, "name": m.name }));
            }
        }

        // Add existing models not already present
        if let Some(obj) = existing_obj {
            if let Some(existing_models) = obj.get("models").and_then(|m| m.as_array()) {
                for item in existing_models {
                    let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    if !id.is_empty() && !merged_models.iter().any(|m| m.get("id").and_then(|v| v.as_str()) == Some(id)) {
                        merged_models.push(item.clone());
                    }
                }
            }
        }

        // Ensure selected model is in the list
        if !model_id.is_empty() && !merged_models.iter().any(|m| m.get("id").and_then(|v| v.as_str()) == Some(&model_id)) {
            merged_models.push(json!({ "id": model_id, "name": model_id }));
        }

        let mut provider_entry = existing.as_object().cloned().unwrap_or_default();
        provider_entry.insert("baseUrl".to_string(), json!(provider_cfg.base_url));
        provider_entry.insert("api".to_string(), json!(provider_cfg.api));
        provider_entry.insert("apiKey".to_string(), json!(provider_cfg.api_key_env));
        provider_entry.insert("models".to_string(), json!(merged_models));

        config["models"]["providers"][provider] = json!(provider_entry);
    }

    // Ensure gateway mode
    if config.get("gateway").is_none() { config["gateway"] = json!({}); }
    if config["gateway"].get("mode").is_none() {
        config["gateway"]["mode"] = json!("local");
    }

    write_openclaw_config(&config)
}

/// Set OpenClaw default model with runtime config overrides (for custom/ollama providers)
pub fn set_openclaw_default_model_with_override(
    provider: &str,
    model_override: Option<&str>,
    base_url: Option<&str>,
    api: Option<&str>,
) -> Result<(), String> {
    let model = match model_override {
        Some(m) => m.to_string(),
        None => match providers::get_provider_default_model(provider) {
            Some(m) => m.to_string(),
            None => return Ok(()),
        },
    };

    let model_id = if model.starts_with(&format!("{}/", provider)) {
        model[provider.len() + 1..].to_string()
    } else {
        model.clone()
    };

    let mut config = read_openclaw_config();

    // Set agents.defaults.model.primary
    if config.get("agents").is_none() { config["agents"] = json!({}); }
    if config["agents"].get("defaults").is_none() { config["agents"]["defaults"] = json!({}); }
    config["agents"]["defaults"]["model"] = json!({ "primary": model });

    // Configure models.providers with runtime overrides
    if let (Some(base_url), Some(api)) = (base_url, api) {
        if config.get("models").is_none() { config["models"] = json!({}); }
        if config["models"].get("providers").is_none() { config["models"]["providers"] = json!({}); }

        let existing = config["models"]["providers"][provider].clone();
        let mut merged_models: Vec<Value> = Vec::new();

        if let Some(existing_models) = existing.as_object().and_then(|o| o.get("models")).and_then(|m| m.as_array()) {
            merged_models.extend(existing_models.iter().cloned());
        }
        if !model_id.is_empty() && !merged_models.iter().any(|m| m.get("id").and_then(|v| v.as_str()) == Some(&model_id)) {
            merged_models.push(json!({ "id": model_id, "name": model_id }));
        }

        let mut provider_entry = existing.as_object().cloned().unwrap_or_default();
        provider_entry.insert("baseUrl".to_string(), json!(base_url));
        provider_entry.insert("api".to_string(), json!(api));
        provider_entry.insert("models".to_string(), json!(merged_models));

        // Add apiKeyEnv if available from registry
        if let Some(env_var) = providers::get_provider_env_var(provider) {
            provider_entry.insert("apiKey".to_string(), json!(env_var));
        }

        config["models"]["providers"][provider] = json!(provider_entry);
    }

    // Ensure gateway mode
    if config.get("gateway").is_none() { config["gateway"] = json!({}); }
    if config["gateway"].get("mode").is_none() {
        config["gateway"]["mode"] = json!("local");
    }

    write_openclaw_config(&config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_profiles_path() {
        let path = auth_profiles_path("main");
        assert!(path.to_string_lossy().contains(".openclaw/agents/main/agent/auth-profiles.json"));
    }

    #[test]
    fn test_read_empty_auth_profiles() {
        let store = read_auth_profiles("nonexistent_agent_test");
        assert_eq!(store["version"], 1);
        assert!(store["profiles"].as_object().unwrap().is_empty());
    }
}
