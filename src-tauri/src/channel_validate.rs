// Channel validation — config check + credential verification

use crate::openclaw_paths;
use serde_json::{json, Value};
use std::collections::HashMap;

/// channel:validate — Run openclaw doctor to validate channel config
pub async fn validate_channel_config(channel_type: &str) -> Result<Value, String> {
    let entry_path = openclaw_paths::get_openclaw_entry_path();

    let output = tokio::process::Command::new("node")
        .args(&[&entry_path, "doctor", "--json"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to run openclaw doctor: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Try to parse JSON output
    if let Ok(result) = serde_json::from_str::<Value>(&stdout) {
        // Filter for the specific channel type if present
        if let Some(channels) = result.get("channels").and_then(|c| c.as_array()) {
            let channel_status = channels
                .iter()
                .find(|c| c.get("type").and_then(|t| t.as_str()) == Some(channel_type));

            if let Some(status) = channel_status {
                return Ok(json!({
                    "success": true,
                    "valid": status.get("configured").and_then(|v| v.as_bool()).unwrap_or(false),
                    "status": status,
                }));
            }
        }
        return Ok(json!({
            "success": true,
            "valid": false,
            "status": { "type": channel_type, "configured": false },
        }));
    }

    // Fallback if output is not JSON
    Ok(json!({
        "success": true,
        "valid": output.status.success(),
        "output": stdout.to_string(),
    }))
}

/// channel:validateCredentials — Verify API credentials by making test API calls
pub async fn validate_channel_credentials(
    channel_type: &str,
    config: HashMap<String, String>,
) -> Result<Value, String> {
    let client = reqwest::Client::new();

    match channel_type {
        "discord" => {
            let token = config
                .get("token")
                .or_else(|| config.get("botToken"))
                .ok_or("Missing Discord bot token")?;

            let resp = client
                .get("https://discord.com/api/v10/users/@me")
                .header("Authorization", format!("Bot {}", token))
                .send()
                .await
                .map_err(|e| format!("Discord API request failed: {}", e))?;

            if resp.status().is_success() {
                let body: Value = resp.json().await.map_err(|e| e.to_string())?;
                Ok(json!({
                    "success": true,
                    "valid": true,
                    "botName": body.get("username").and_then(|u| u.as_str()),
                    "botId": body.get("id").and_then(|i| i.as_str()),
                }))
            } else {
                let status = resp.status().as_u16();
                let err_text = resp.text().await.unwrap_or_default();
                Ok(json!({
                    "success": true,
                    "valid": false,
                    "error": format!("Discord API error ({}): {}", status, err_text),
                }))
            }
        }

        "telegram" => {
            let token = config
                .get("token")
                .or_else(|| config.get("botToken"))
                .ok_or("Missing Telegram bot token")?;

            let resp = client
                .get(format!("https://api.telegram.org/bot{}/getMe", token))
                .send()
                .await
                .map_err(|e| format!("Telegram API request failed: {}", e))?;

            if resp.status().is_success() {
                let body: Value = resp.json().await.map_err(|e| e.to_string())?;
                let ok = body.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
                if ok {
                    let result = body.get("result");
                    Ok(json!({
                        "success": true,
                        "valid": true,
                        "botName": result.and_then(|r| r.get("first_name")).and_then(|n| n.as_str()),
                        "botUsername": result.and_then(|r| r.get("username")).and_then(|u| u.as_str()),
                    }))
                } else {
                    Ok(json!({
                        "success": true,
                        "valid": false,
                        "error": body.get("description").and_then(|d| d.as_str()).unwrap_or("Unknown error"),
                    }))
                }
            } else {
                let status = resp.status().as_u16();
                let err_text = resp.text().await.unwrap_or_default();
                Ok(json!({
                    "success": true,
                    "valid": false,
                    "error": format!("Telegram API error ({}): {}", status, err_text),
                }))
            }
        }

        _ => {
            // No online validation available for other channel types
            Ok(json!({
                "success": true,
                "valid": true,
                "warnings": ["No online validation available for this channel type"],
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unknown_channel_validation() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(validate_channel_credentials(
            "unknown",
            HashMap::new(),
        ));
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val.get("valid").and_then(|v| v.as_bool()), Some(true));
    }
}
