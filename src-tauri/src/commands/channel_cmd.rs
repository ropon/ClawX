// Tauri commands for channel:* channels
// 6 CRUD commands + 2 validation commands (Week 7)

use crate::channel_config;
use crate::channel_validate;
use serde_json::{json, Value};
use std::collections::HashMap;

/// channel:saveConfig
/// _args: [channelType, config]
#[tauri::command]
pub async fn channel_save_config(_args: Vec<Value>) -> Result<Value, String> {
    let channel_type = _args
        .first()
        .and_then(|v| v.as_str())
        .ok_or("Missing channelType")?;
    let config = _args.get(1).cloned().ok_or("Missing config")?;

    channel_config::save_channel_config(channel_type, config)?;
    Ok(json!({ "success": true }))
}

/// channel:getConfig
/// _args: [channelType]
#[tauri::command]
pub async fn channel_get_config(_args: Vec<Value>) -> Result<Value, String> {
    let channel_type = _args
        .first()
        .and_then(|v| v.as_str())
        .ok_or("Missing channelType")?;

    let config = channel_config::get_channel_config(channel_type)?;
    Ok(json!({ "success": true, "config": config }))
}

/// channel:getFormValues
/// _args: [channelType]
#[tauri::command]
pub async fn channel_get_form_values(_args: Vec<Value>) -> Result<Value, String> {
    let channel_type = _args
        .first()
        .and_then(|v| v.as_str())
        .ok_or("Missing channelType")?;

    let values = channel_config::get_channel_form_values(channel_type)?;
    Ok(json!({ "success": true, "values": values }))
}

/// channel:deleteConfig
/// _args: [channelType]
#[tauri::command]
pub async fn channel_delete_config(_args: Vec<Value>) -> Result<Value, String> {
    let channel_type = _args
        .first()
        .and_then(|v| v.as_str())
        .ok_or("Missing channelType")?;

    channel_config::delete_channel_config(channel_type)?;
    Ok(json!({ "success": true }))
}

/// channel:listConfigured
/// _args: []
#[tauri::command]
pub async fn channel_list_configured(_args: Vec<Value>) -> Result<Value, String> {
    let channels = channel_config::list_configured_channels()?;
    Ok(json!({ "success": true, "channels": channels }))
}

/// channel:setEnabled
/// _args: [channelType, enabled]
#[tauri::command]
pub async fn channel_set_enabled(_args: Vec<Value>) -> Result<Value, String> {
    let channel_type = _args
        .first()
        .and_then(|v| v.as_str())
        .ok_or("Missing channelType")?;
    let enabled = _args
        .get(1)
        .and_then(|v| v.as_bool())
        .ok_or("Missing enabled")?;

    channel_config::set_channel_enabled(channel_type, enabled)?;
    Ok(json!({ "success": true }))
}

// ── Week 7: Validation commands ──

/// channel:validate — Run config validation
/// _args: [channelType]
#[tauri::command]
pub async fn channel_validate(_args: Vec<Value>) -> Result<Value, String> {
    let channel_type = _args
        .first()
        .and_then(|v| v.as_str())
        .ok_or("Missing channelType")?;

    channel_validate::validate_channel_config(channel_type).await
}

/// channel:validateCredentials — Verify API credentials
/// _args: [channelType, config]
#[tauri::command]
pub async fn channel_validate_credentials(_args: Vec<Value>) -> Result<Value, String> {
    let channel_type = _args
        .first()
        .and_then(|v| v.as_str())
        .ok_or("Missing channelType")?;

    let config_val = _args.get(1).cloned().unwrap_or(Value::Object(Default::default()));
    let config: HashMap<String, String> = if let Some(obj) = config_val.as_object() {
        obj.iter()
            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
            .collect()
    } else {
        HashMap::new()
    };

    channel_validate::validate_channel_credentials(channel_type, config).await
}
