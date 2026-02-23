// Channel Config CRUD
// Manages channel configuration in ~/.openclaw/openclaw.json

use serde_json::{json, Map, Value};
use std::fs;
use std::path::PathBuf;

const OPENCLAW_DIR: &str = ".openclaw";
const CONFIG_FILE: &str = "openclaw.json";
const PLUGIN_CHANNELS: &[&str] = &["whatsapp"];

fn is_plugin_channel(channel_type: &str) -> bool {
    PLUGIN_CHANNELS.contains(&channel_type)
}

fn config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(OPENCLAW_DIR)
}

fn config_path() -> PathBuf {
    config_dir().join(CONFIG_FILE)
}

fn ensure_config_dir() -> Result<(), String> {
    let dir = config_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config dir: {}", e))?;
    }
    Ok(())
}

fn read_config() -> Result<Value, String> {
    ensure_config_dir()?;
    let path = config_path();
    if !path.exists() {
        return Ok(json!({}));
    }
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read config: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))
}

fn write_config(config: &Value) -> Result<(), String> {
    ensure_config_dir()?;
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(config_path(), content).map_err(|e| format!("Failed to write config: {}", e))
}

/// Save channel configuration
/// Ported from channel-config.ts:82-191
pub fn save_channel_config(channel_type: &str, config: Value) -> Result<(), String> {
    let mut current = read_config()?;

    // Plugin-based channels → plugins.entries.<type>
    if is_plugin_channel(channel_type) {
        let enabled = config
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Ensure plugins.entries path
        if current.get("plugins").is_none() {
            current["plugins"] = json!({});
        }
        if current["plugins"].get("entries").is_none() {
            current["plugins"]["entries"] = json!({});
        }

        // Merge with existing
        let existing = current["plugins"]["entries"]
            .get(channel_type)
            .cloned()
            .unwrap_or(json!({}));
        let mut merged = existing;
        if let Some(obj) = merged.as_object_mut() {
            obj.insert("enabled".to_string(), json!(enabled));
        }
        current["plugins"]["entries"][channel_type] = merged;

        write_config(&current)?;
        return Ok(());
    }

    // Regular channels → channels.<type>
    if current.get("channels").is_none() {
        current["channels"] = json!({});
    }

    let mut transformed = config.clone();

    // Discord: guildId/channelId → nested guilds structure
    if channel_type == "discord" {
        let guild_id = config
            .get("guildId")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string());
        let channel_id = config
            .get("channelId")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string());

        // Remove flat fields
        if let Some(obj) = transformed.as_object_mut() {
            obj.remove("guildId");
            obj.remove("channelId");
        }

        // Add standard Discord config
        transformed["groupPolicy"] = json!("allowlist");
        transformed["dm"] = json!({ "enabled": false });
        transformed["retry"] = json!({
            "attempts": 3,
            "minDelayMs": 500,
            "maxDelayMs": 30000,
            "jitter": 0.1,
        });

        // Build guilds structure
        if let Some(gid) = guild_id.filter(|s| !s.is_empty()) {
            let channel_key = channel_id
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "*".to_string());
            transformed["guilds"] = json!({
                gid: {
                    "users": ["*"],
                    "requireMention": true,
                    "channels": {
                        channel_key: { "allow": true, "requireMention": true }
                    }
                }
            });
        }
    }

    // Telegram: allowedUsers string → allowFrom array
    if channel_type == "telegram" {
        if let Some(allowed) = config.get("allowedUsers").and_then(|v| v.as_str()) {
            let users: Vec<Value> = allowed
                .split(',')
                .map(|u| u.trim())
                .filter(|u| !u.is_empty())
                .map(|u| json!(u))
                .collect();
            if !users.is_empty() {
                transformed["allowFrom"] = json!(users);
            }
            if let Some(obj) = transformed.as_object_mut() {
                obj.remove("allowedUsers");
            }
        }
    }

    // Set enabled default
    if transformed.get("enabled").is_none() {
        transformed["enabled"] = json!(true);
    }

    // Merge with existing channel config
    let existing = current["channels"]
        .get(channel_type)
        .cloned()
        .unwrap_or(json!({}));
    let merged = merge_objects(existing, transformed);
    current["channels"][channel_type] = merged;

    write_config(&current)
}

/// Get raw channel configuration
pub fn get_channel_config(channel_type: &str) -> Result<Option<Value>, String> {
    let config = read_config()?;
    Ok(config
        .get("channels")
        .and_then(|c| c.get(channel_type))
        .cloned())
}

/// Get channel config as form-friendly values (reverse transform)
/// Ported from channel-config.ts:210-261
pub fn get_channel_form_values(channel_type: &str) -> Result<Option<Map<String, Value>>, String> {
    let saved = match get_channel_config(channel_type)? {
        Some(v) => v,
        None => return Ok(None),
    };

    let mut values = Map::new();

    if channel_type == "discord" {
        // Extract token
        if let Some(token) = saved.get("token").and_then(|v| v.as_str()) {
            values.insert("token".to_string(), json!(token));
        }

        // Extract guildId and channelId from nested guilds structure
        if let Some(guilds) = saved.get("guilds").and_then(|v| v.as_object()) {
            if let Some((guild_id, guild_config)) = guilds.iter().next() {
                values.insert("guildId".to_string(), json!(guild_id));

                if let Some(channels) = guild_config.get("channels").and_then(|v| v.as_object()) {
                    let channel_ids: Vec<&String> =
                        channels.keys().filter(|id| id.as_str() != "*").collect();
                    if let Some(cid) = channel_ids.first() {
                        values.insert("channelId".to_string(), json!(*cid));
                    }
                }
            }
        }
    } else if channel_type == "telegram" {
        // Convert allowFrom array → allowedUsers string
        if let Some(allow_from) = saved.get("allowFrom").and_then(|v| v.as_array()) {
            let users: Vec<&str> = allow_from.iter().filter_map(|v| v.as_str()).collect();
            values.insert("allowedUsers".to_string(), json!(users.join(", ")));
        }
        // Extract other string values
        if let Some(obj) = saved.as_object() {
            for (key, val) in obj {
                if key != "enabled" && val.is_string() {
                    values.insert(key.clone(), val.clone());
                }
            }
        }
    } else {
        // Generic: extract all string values
        if let Some(obj) = saved.as_object() {
            for (key, val) in obj {
                if key != "enabled" && val.is_string() {
                    values.insert(key.clone(), val.clone());
                }
            }
        }
    }

    if values.is_empty() {
        Ok(None)
    } else {
        Ok(Some(values))
    }
}

/// Delete channel configuration
/// Ported from channel-config.ts:267-305
pub fn delete_channel_config(channel_type: &str) -> Result<(), String> {
    let mut current = read_config()?;

    if let Some(channels) = current.get_mut("channels").and_then(|c| c.as_object_mut()) {
        channels.remove(channel_type);
    } else if is_plugin_channel(channel_type) {
        if let Some(entries) = current
            .get_mut("plugins")
            .and_then(|p| p.get_mut("entries"))
            .and_then(|e| e.as_object_mut())
        {
            entries.remove(channel_type);
            // Cleanup empty objects
            if entries.is_empty() {
                if let Some(plugins) = current.get_mut("plugins").and_then(|p| p.as_object_mut()) {
                    plugins.remove("entries");
                    if plugins.is_empty() {
                        if let Some(root) = current.as_object_mut() {
                            root.remove("plugins");
                        }
                    }
                }
            }
        }
    }

    write_config(&current)?;

    // WhatsApp: delete credentials directory
    if channel_type == "whatsapp" {
        let wa_dir = config_dir().join("credentials").join("whatsapp");
        if wa_dir.exists() {
            let _ = fs::remove_dir_all(&wa_dir);
        }
    }

    Ok(())
}

/// List all configured (enabled) channels
/// Ported from channel-config.ts:310-341
pub fn list_configured_channels() -> Result<Vec<String>, String> {
    let config = read_config()?;
    let mut channels = Vec::new();

    if let Some(ch_obj) = config.get("channels").and_then(|c| c.as_object()) {
        for (channel_type, ch_config) in ch_obj {
            let enabled = ch_config
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(true); // default enabled
            if enabled {
                channels.push(channel_type.clone());
            }
        }
    }

    // Check WhatsApp credentials directory for sessions
    let wa_dir = config_dir().join("credentials").join("whatsapp");
    if wa_dir.exists() {
        if let Ok(entries) = fs::read_dir(&wa_dir) {
            let has_session = entries.filter_map(|e| e.ok()).any(|e| {
                e.file_type()
                    .map(|ft| ft.is_dir())
                    .unwrap_or(false)
            });
            if has_session && !channels.contains(&"whatsapp".to_string()) {
                channels.push("whatsapp".to_string());
            }
        }
    }

    Ok(channels)
}

/// Enable or disable a channel
/// Ported from channel-config.ts:346-377
pub fn set_channel_enabled(channel_type: &str, enabled: bool) -> Result<(), String> {
    let mut current = read_config()?;

    if is_plugin_channel(channel_type) {
        if current.get("plugins").is_none() {
            current["plugins"] = json!({});
        }
        if current["plugins"].get("entries").is_none() {
            current["plugins"]["entries"] = json!({});
        }
        if current["plugins"]["entries"].get(channel_type).is_none() {
            current["plugins"]["entries"][channel_type] = json!({});
        }
        current["plugins"]["entries"][channel_type]["enabled"] = json!(enabled);
    } else {
        if current.get("channels").is_none() {
            current["channels"] = json!({});
        }
        if current["channels"].get(channel_type).is_none() {
            current["channels"][channel_type] = json!({});
        }
        current["channels"][channel_type]["enabled"] = json!(enabled);
    }

    write_config(&current)
}

/// Merge two JSON objects (b overrides a)
fn merge_objects(a: Value, b: Value) -> Value {
    match (a, b) {
        (Value::Object(mut a_map), Value::Object(b_map)) => {
            for (key, val) in b_map {
                a_map.insert(key, val);
            }
            Value::Object(a_map)
        }
        (_, b) => b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_objects() {
        let a = json!({"a": 1, "b": 2});
        let b = json!({"b": 3, "c": 4});
        let merged = merge_objects(a, b);
        assert_eq!(merged, json!({"a": 1, "b": 3, "c": 4}));
    }

    #[test]
    fn test_is_plugin_channel() {
        assert!(is_plugin_channel("whatsapp"));
        assert!(!is_plugin_channel("discord"));
        assert!(!is_plugin_channel("telegram"));
    }
}
