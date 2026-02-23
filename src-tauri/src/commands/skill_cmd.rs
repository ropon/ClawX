// Skill config commands (3)
// Handles skill:updateConfig/getConfig/getAllConfigs
// Reads/writes ~/.openclaw/openclaw.json skills.entries

use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

fn openclaw_config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".openclaw")
        .join("openclaw.json")
}

fn read_config() -> Value {
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

fn write_config(config: &Value) -> Result<(), String> {
    let path = openclaw_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config dir: {}", e))?;
    }
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(&path, content)
        .map_err(|e| format!("Failed to write config: {}", e))
}

#[tauri::command]
pub async fn skill_update_config(_args: Vec<Value>) -> Result<Value, String> {
    let params = _args.first().cloned().unwrap_or(Value::Null);
    let skill_key = params.get("skillKey")
        .and_then(|v| v.as_str())
        .ok_or("Missing skillKey")?
        .to_string();
    let api_key = params.get("apiKey").and_then(|v| v.as_str()).map(String::from);
    let env = params.get("env").cloned();

    let mut config = read_config();

    // Ensure skills.entries path
    if config.get("skills").is_none() { config["skills"] = json!({}); }
    if config["skills"].get("entries").is_none() { config["skills"]["entries"] = json!({}); }

    // Get or create entry
    let mut entry = config["skills"]["entries"][&skill_key].clone();
    if entry.is_null() { entry = json!({}); }

    // Update apiKey
    if let Some(key) = api_key {
        let trimmed = key.trim();
        if !trimmed.is_empty() {
            entry["apiKey"] = json!(trimmed);
        } else if let Some(obj) = entry.as_object_mut() {
            obj.remove("apiKey");
        }
    }

    // Update env
    if let Some(env_val) = env {
        if let Some(env_obj) = env_val.as_object() {
            let mut new_env = serde_json::Map::new();
            for (key, value) in env_obj {
                let trimmed_key = key.trim();
                if trimmed_key.is_empty() { continue; }
                if let Some(val_str) = value.as_str() {
                    let trimmed_val = val_str.trim();
                    if !trimmed_val.is_empty() {
                        new_env.insert(trimmed_key.to_string(), json!(trimmed_val));
                    }
                }
            }
            if new_env.is_empty() {
                if let Some(obj) = entry.as_object_mut() {
                    obj.remove("env");
                }
            } else {
                entry["env"] = Value::Object(new_env);
            }
        }
    }

    config["skills"]["entries"][&skill_key] = entry;
    write_config(&config)?;

    Ok(json!({ "success": true }))
}

#[tauri::command]
pub async fn skill_get_config(_args: Vec<Value>) -> Result<Value, String> {
    let skill_key = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing skill key")?;

    let config = read_config();
    let entry = config
        .get("skills")
        .and_then(|s| s.get("entries"))
        .and_then(|e| e.get(skill_key))
        .cloned();

    Ok(entry.unwrap_or(Value::Null))
}

#[tauri::command]
pub async fn skill_get_all_configs(_args: Vec<Value>) -> Result<Value, String> {
    let config = read_config();
    let entries = config
        .get("skills")
        .and_then(|s| s.get("entries"))
        .cloned()
        .unwrap_or(json!({}));
    Ok(entries)
}
