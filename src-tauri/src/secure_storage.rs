// Secure storage — CRUD access to clawx-providers.json
// The file stores provider configurations, API keys, and default provider.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderEntry {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: String,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProvidersStore {
    #[serde(default)]
    pub providers: HashMap<String, ProviderEntry>,
    #[serde(default)]
    pub api_keys: HashMap<String, String>,
    #[serde(default)]
    pub default_provider: Option<String>,
}

fn store_path(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join("clawx-providers.json")
}

fn load_store(data_dir: &Path) -> Result<ProvidersStore, String> {
    let path = store_path(data_dir);
    if !path.exists() {
        return Ok(ProvidersStore::default());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read providers store: {}", e))?;
    if content.trim().is_empty() {
        return Ok(ProvidersStore::default());
    }
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse providers store: {}", e))
}

/// Atomic write: write to temp file then rename
fn save_store(data_dir: &Path, store: &ProvidersStore) -> Result<(), String> {
    let path = store_path(data_dir);
    let content = serde_json::to_string_pretty(store)
        .map_err(|e| format!("Failed to serialize providers store: {}", e))?;
    let tmp_path = path.with_extension("json.tmp");
    std::fs::write(&tmp_path, &content)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;
    std::fs::rename(&tmp_path, &path)
        .map_err(|e| format!("Failed to rename temp file: {}", e))?;
    Ok(())
}

/// Get a provider config by ID
pub fn get_provider(data_dir: &Path, provider_id: &str) -> Result<Option<ProviderEntry>, String> {
    let store = load_store(data_dir)?;
    Ok(store.providers.get(provider_id).cloned())
}

/// Get the default provider ID
pub fn get_default_provider_id(data_dir: &Path) -> Result<Option<String>, String> {
    let store = load_store(data_dir)?;
    Ok(store.default_provider)
}

/// Get the API key for a provider
pub fn get_api_key(data_dir: &Path, provider_id: &str) -> Result<Option<String>, String> {
    let store = load_store(data_dir)?;
    Ok(store.api_keys.get(provider_id).cloned())
}

/// Save a provider configuration
pub fn save_provider(data_dir: &Path, config: ProviderEntry) -> Result<(), String> {
    let mut store = load_store(data_dir)?;
    store.providers.insert(config.id.clone(), config);
    save_store(data_dir, &store)
}

/// Delete a provider configuration and its API key
pub fn delete_provider(data_dir: &Path, provider_id: &str) -> Result<Option<ProviderEntry>, String> {
    let mut store = load_store(data_dir)?;
    let removed = store.providers.remove(provider_id);
    store.api_keys.remove(provider_id);
    if store.default_provider.as_deref() == Some(provider_id) {
        store.default_provider = None;
    }
    save_store(data_dir, &store)?;
    Ok(removed)
}

/// Store an API key
pub fn store_api_key(data_dir: &Path, provider_id: &str, api_key: &str) -> Result<(), String> {
    let mut store = load_store(data_dir)?;
    store.api_keys.insert(provider_id.to_string(), api_key.to_string());
    save_store(data_dir, &store)
}

/// Delete an API key
pub fn delete_api_key_entry(data_dir: &Path, provider_id: &str) -> Result<(), String> {
    let mut store = load_store(data_dir)?;
    store.api_keys.remove(provider_id);
    save_store(data_dir, &store)
}

/// Check if a provider has an API key
pub fn has_api_key(data_dir: &Path, provider_id: &str) -> Result<bool, String> {
    let store = load_store(data_dir)?;
    Ok(store.api_keys.contains_key(provider_id))
}

/// Set the default provider ID
pub fn set_default_provider_id(data_dir: &Path, provider_id: &str) -> Result<(), String> {
    let mut store = load_store(data_dir)?;
    store.default_provider = Some(provider_id.to_string());
    save_store(data_dir, &store)
}

/// Get all providers with hasKey and keyMasked info (for UI display)
pub fn get_all_providers_with_key_info(data_dir: &Path) -> Result<Value, String> {
    let store = load_store(data_dir)?;
    let mut results = Vec::new();

    for (_, provider) in &store.providers {
        let api_key = store.api_keys.get(&provider.id);
        let has_key = api_key.is_some();
        let key_masked = api_key.map(|k| mask_key(k));

        let mut entry = serde_json::to_value(provider)
            .map_err(|e| format!("Failed to serialize provider: {}", e))?;
        if let Some(obj) = entry.as_object_mut() {
            obj.insert("hasKey".to_string(), json!(has_key));
            obj.insert("keyMasked".to_string(), json!(key_masked));
        }
        results.push(entry);
    }

    Ok(json!(results))
}

fn mask_key(key: &str) -> String {
    if key.len() > 12 {
        format!("{}{}{}",
            &key[..4],
            "*".repeat(key.len() - 8),
            &key[key.len()-4..])
    } else {
        "*".repeat(key.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_test_store(dir: &Path) {
        let store = serde_json::json!({
            "providers": {
                "p1": {
                    "id": "p1",
                    "name": "Test OpenAI",
                    "type": "openai",
                    "baseUrl": "https://api.openai.com/v1",
                    "enabled": true
                }
            },
            "apiKeys": {
                "p1": "sk-test-key-123"
            },
            "defaultProvider": "p1"
        });
        std::fs::write(
            dir.join("clawx-providers.json"),
            serde_json::to_string_pretty(&store).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn test_load_missing_file() {
        let tmp = TempDir::new().unwrap();
        let store = load_store(tmp.path()).unwrap();
        assert!(store.providers.is_empty());
        assert!(store.default_provider.is_none());
    }

    #[test]
    fn test_get_provider() {
        let tmp = TempDir::new().unwrap();
        write_test_store(tmp.path());
        let provider = get_provider(tmp.path(), "p1").unwrap().unwrap();
        assert_eq!(provider.name, "Test OpenAI");
        assert_eq!(provider.provider_type, "openai");
        assert!(get_provider(tmp.path(), "nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_get_api_key() {
        let tmp = TempDir::new().unwrap();
        write_test_store(tmp.path());
        let key = get_api_key(tmp.path(), "p1").unwrap().unwrap();
        assert_eq!(key, "sk-test-key-123");
    }

    #[test]
    fn test_get_default_provider() {
        let tmp = TempDir::new().unwrap();
        write_test_store(tmp.path());
        let id = get_default_provider_id(tmp.path()).unwrap().unwrap();
        assert_eq!(id, "p1");
    }

    #[test]
    fn test_save_provider() {
        let tmp = TempDir::new().unwrap();
        let entry = ProviderEntry {
            id: "p2".to_string(),
            name: "Test Anthropic".to_string(),
            provider_type: "anthropic".to_string(),
            base_url: None,
            model: None,
            enabled: true,
            created_at: None,
            updated_at: None,
        };
        save_provider(tmp.path(), entry).unwrap();
        let p = get_provider(tmp.path(), "p2").unwrap().unwrap();
        assert_eq!(p.name, "Test Anthropic");
    }

    #[test]
    fn test_store_and_delete_api_key() {
        let tmp = TempDir::new().unwrap();
        store_api_key(tmp.path(), "p1", "sk-abc").unwrap();
        assert!(has_api_key(tmp.path(), "p1").unwrap());
        assert_eq!(get_api_key(tmp.path(), "p1").unwrap().unwrap(), "sk-abc");
        delete_api_key_entry(tmp.path(), "p1").unwrap();
        assert!(!has_api_key(tmp.path(), "p1").unwrap());
    }

    #[test]
    fn test_delete_provider() {
        let tmp = TempDir::new().unwrap();
        write_test_store(tmp.path());
        let removed = delete_provider(tmp.path(), "p1").unwrap();
        assert!(removed.is_some());
        assert!(get_provider(tmp.path(), "p1").unwrap().is_none());
        assert!(get_default_provider_id(tmp.path()).unwrap().is_none());
    }

    #[test]
    fn test_mask_key() {
        // "sk-test-key-123456" is 18 chars, so mask is first 4 + (18-8)=10 stars + last 4
        assert_eq!(mask_key("sk-test-key-123456"), "sk-t**********3456");
        assert_eq!(mask_key("short"), "*****");
    }
}
