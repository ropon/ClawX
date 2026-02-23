// Settings store reader
// Read-only access to clawx-settings.json for gatewayToken etc.

use serde_json::Value;
use std::path::Path;

use crate::config;

/// Read the gateway authentication token from clawx-settings.json
pub fn get_gateway_token(data_dir: &Path) -> Result<String, String> {
    let settings_path = data_dir.join("clawx-settings.json");
    if !settings_path.exists() {
        return Err("Settings file not found".to_string());
    }

    let content = std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings: {}", e))?;

    let json: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings: {}", e))?;

    json.get("gatewayToken")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "gatewayToken not found in settings".to_string())
}

/// Save the gateway token to clawx-settings.json (creates or updates the file)
pub fn save_gateway_token(data_dir: &Path, token: &str) -> Result<(), String> {
    let settings_path = data_dir.join("clawx-settings.json");

    // Read existing settings or start with empty object
    let mut json: serde_json::Map<String, Value> = if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path)
            .map_err(|e| format!("Failed to read settings: {}", e))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        // Ensure parent directory exists
        if let Some(parent) = settings_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create settings dir: {}", e))?;
        }
        serde_json::Map::new()
    };

    json.insert("gatewayToken".to_string(), Value::String(token.to_string()));

    let content = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    std::fs::write(&settings_path, content)
        .map_err(|e| format!("Failed to write settings: {}", e))?;

    Ok(())
}

/// Read shortcut configuration from clawx-settings.json
pub fn get_shortcut_config(data_dir: &Path) -> Result<Value, String> {
    let settings_path = data_dir.join("clawx-settings.json");
    let default = serde_json::json!({ "spotlight": "CmdOrCtrl+Shift+Space" });

    if !settings_path.exists() {
        return Ok(default);
    }

    let content = std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings: {}", e))?;

    let json: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings: {}", e))?;

    match json.get("shortcuts") {
        Some(shortcuts) if shortcuts.is_object() => Ok(shortcuts.clone()),
        _ => Ok(default),
    }
}

/// Save shortcut configuration to clawx-settings.json (merge into existing)
pub fn save_shortcut_config(data_dir: &Path, config: &Value) -> Result<(), String> {
    let settings_path = data_dir.join("clawx-settings.json");

    let mut json: serde_json::Map<String, Value> = if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path)
            .map_err(|e| format!("Failed to read settings: {}", e))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        if let Some(parent) = settings_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create settings dir: {}", e))?;
        }
        serde_json::Map::new()
    };

    json.insert("shortcuts".to_string(), config.clone());

    let content = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    std::fs::write(&settings_path, content)
        .map_err(|e| format!("Failed to write settings: {}", e))?;

    Ok(())
}

/// Get the default Gateway port
pub fn get_gateway_port() -> u16 {
    config::OPENCLAW_GATEWAY_PORT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_gateway_token_missing_file() {
        let result = get_gateway_token(Path::new("/nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn test_get_gateway_token_valid() {
        let dir = tempfile::tempdir().unwrap();
        let settings = serde_json::json!({ "gatewayToken": "abc123" });
        std::fs::write(dir.path().join("clawx-settings.json"), settings.to_string()).unwrap();

        let token = get_gateway_token(dir.path()).unwrap();
        assert_eq!(token, "abc123");
    }
}
