// Provider commands (12)
// Handles provider:list/get/save/delete/setApiKey/updateWithKey/deleteApiKey/hasApiKey/getApiKey/setDefault/getDefault/validateKey

use crate::openclaw_auth;
use crate::provider_validate;
use crate::providers;
use crate::secure_storage;
use crate::secure_storage::ProviderEntry;
use crate::AppState;
use serde_json::{json, Value};

#[tauri::command]
pub async fn provider_list(state: tauri::State<'_, AppState>, _args: Vec<Value>) -> Result<Value, String> {
    secure_storage::get_all_providers_with_key_info(&state.data_dir)
}

#[tauri::command]
pub async fn provider_get(state: tauri::State<'_, AppState>, _args: Vec<Value>) -> Result<Value, String> {
    let provider_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing provider ID")?;
    let provider = secure_storage::get_provider(&state.data_dir, provider_id)?;
    Ok(serde_json::to_value(provider).unwrap_or(Value::Null))
}

#[tauri::command]
pub async fn provider_save(state: tauri::State<'_, AppState>, _args: Vec<Value>) -> Result<Value, String> {
    let config: ProviderEntry = serde_json::from_value(_args.first().cloned().unwrap_or(Value::Null))
        .map_err(|e| format!("Invalid provider config: {}", e))?;
    let api_key = _args.get(1).and_then(|v| v.as_str()).map(String::from);

    let provider_type = config.provider_type.clone();
    secure_storage::save_provider(&state.data_dir, config)?;

    if let Some(key) = api_key {
        let provider_id = _args.first()
            .and_then(|v| v.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        secure_storage::store_api_key(&state.data_dir, provider_id, &key)?;
        let _ = openclaw_auth::save_provider_key_to_openclaw(&provider_type, &key);
    }

    Ok(json!({ "success": true }))
}

#[tauri::command]
pub async fn provider_delete(state: tauri::State<'_, AppState>, _args: Vec<Value>) -> Result<Value, String> {
    let provider_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing provider ID")?;

    let existing = secure_storage::get_provider(&state.data_dir, provider_id)?;
    secure_storage::delete_provider(&state.data_dir, provider_id)?;

    if let Some(provider) = existing {
        let _ = openclaw_auth::remove_provider_key_from_openclaw(&provider.provider_type);
    }

    Ok(json!({ "success": true }))
}

#[tauri::command]
pub async fn provider_set_api_key(state: tauri::State<'_, AppState>, _args: Vec<Value>) -> Result<Value, String> {
    let provider_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing provider ID")?;
    let api_key = _args.get(1)
        .and_then(|v| v.as_str())
        .ok_or("Missing API key")?;

    secure_storage::store_api_key(&state.data_dir, provider_id, api_key)?;

    let provider = secure_storage::get_provider(&state.data_dir, provider_id)?;
    let provider_type = provider.map(|p| p.provider_type).unwrap_or_else(|| provider_id.to_string());
    let _ = openclaw_auth::save_provider_key_to_openclaw(&provider_type, api_key);

    Ok(json!({ "success": true }))
}

#[tauri::command]
pub async fn provider_update_with_key(state: tauri::State<'_, AppState>, _args: Vec<Value>) -> Result<Value, String> {
    let provider_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing provider ID")?;
    let updates = _args.get(1).cloned().unwrap_or(json!({}));
    let api_key = _args.get(2).and_then(|v| v.as_str()).map(String::from);

    let existing = match secure_storage::get_provider(&state.data_dir, provider_id)? {
        Some(p) => p,
        None => return Ok(json!({ "success": false, "error": "Provider not found" })),
    };

    let previous_key = secure_storage::get_api_key(&state.data_dir, provider_id)?;
    let previous_type = existing.provider_type.clone();

    // Build updated config by merging updates into existing
    let mut config_value = serde_json::to_value(&existing)
        .map_err(|e| format!("Failed to serialize existing config: {}", e))?;
    if let (Some(base), Some(patch)) = (config_value.as_object_mut(), updates.as_object()) {
        for (k, v) in patch {
            base.insert(k.clone(), v.clone());
        }
        base.insert("updatedAt".to_string(), json!(chrono::Utc::now().to_rfc3339()));
    }

    let next_config: ProviderEntry = serde_json::from_value(config_value.clone())
        .map_err(|e| format!("Failed to parse updated config: {}", e))?;

    match (|| -> Result<(), String> {
        secure_storage::save_provider(&state.data_dir, next_config.clone())?;

        if let Some(key) = &api_key {
            let trimmed = key.trim();
            if !trimmed.is_empty() {
                secure_storage::store_api_key(&state.data_dir, provider_id, trimmed)?;
                let _ = openclaw_auth::save_provider_key_to_openclaw(&next_config.provider_type, trimmed);
            } else {
                secure_storage::delete_api_key_entry(&state.data_dir, provider_id)?;
                let _ = openclaw_auth::remove_provider_key_from_openclaw(&next_config.provider_type);
            }
        }
        Ok(())
    })() {
        Ok(()) => Ok(json!({ "success": true })),
        Err(e) => {
            // Best-effort rollback
            let _ = secure_storage::save_provider(&state.data_dir, existing);
            if let Some(prev_key) = previous_key {
                let _ = secure_storage::store_api_key(&state.data_dir, provider_id, &prev_key);
                let _ = openclaw_auth::save_provider_key_to_openclaw(&previous_type, &prev_key);
            } else {
                let _ = secure_storage::delete_api_key_entry(&state.data_dir, provider_id);
                let _ = openclaw_auth::remove_provider_key_from_openclaw(&previous_type);
            }
            Ok(json!({ "success": false, "error": e }))
        }
    }
}

#[tauri::command]
pub async fn provider_delete_api_key(state: tauri::State<'_, AppState>, _args: Vec<Value>) -> Result<Value, String> {
    let provider_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing provider ID")?;

    secure_storage::delete_api_key_entry(&state.data_dir, provider_id)?;

    let provider = secure_storage::get_provider(&state.data_dir, provider_id)?;
    let provider_type = provider.map(|p| p.provider_type).unwrap_or_else(|| provider_id.to_string());
    let _ = openclaw_auth::remove_provider_key_from_openclaw(&provider_type);

    Ok(json!({ "success": true }))
}

#[tauri::command]
pub async fn provider_has_api_key(state: tauri::State<'_, AppState>, _args: Vec<Value>) -> Result<Value, String> {
    let provider_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing provider ID")?;
    let has = secure_storage::has_api_key(&state.data_dir, provider_id)?;
    Ok(json!(has))
}

#[tauri::command]
pub async fn provider_get_api_key(state: tauri::State<'_, AppState>, _args: Vec<Value>) -> Result<Value, String> {
    let provider_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing provider ID")?;
    let key = secure_storage::get_api_key(&state.data_dir, provider_id)?;
    Ok(json!(key))
}

#[tauri::command]
pub async fn provider_set_default(state: tauri::State<'_, AppState>, _args: Vec<Value>) -> Result<Value, String> {
    let provider_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing provider ID")?;

    secure_storage::set_default_provider_id(&state.data_dir, provider_id)?;

    // Update OpenClaw config to use this provider's default model
    if let Some(provider) = secure_storage::get_provider(&state.data_dir, provider_id)? {
        let model_override = provider.model
            .as_ref()
            .map(|m| format!("{}/{}", provider.provider_type, m));

        if provider.provider_type == "custom" || provider.provider_type == "ollama" {
            let _ = openclaw_auth::set_openclaw_default_model_with_override(
                &provider.provider_type,
                model_override.as_deref(),
                provider.base_url.as_deref(),
                Some("openai-completions"),
            );
        } else {
            let _ = openclaw_auth::set_openclaw_default_model(
                &provider.provider_type,
                model_override.as_deref(),
            );
        }

        // Keep auth-profiles in sync
        if let Some(key) = secure_storage::get_api_key(&state.data_dir, provider_id)? {
            let _ = openclaw_auth::save_provider_key_to_openclaw(&provider.provider_type, &key);
        }
    }

    Ok(json!({ "success": true }))
}

#[tauri::command]
pub async fn provider_get_default(state: tauri::State<'_, AppState>, _args: Vec<Value>) -> Result<Value, String> {
    let default_id = secure_storage::get_default_provider_id(&state.data_dir)?;
    Ok(json!(default_id))
}

#[tauri::command]
pub async fn provider_validate_key(state: tauri::State<'_, AppState>, _args: Vec<Value>) -> Result<Value, String> {
    let provider_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing provider ID")?;
    let api_key = _args.get(1)
        .and_then(|v| v.as_str())
        .ok_or("Missing API key")?;
    let options = _args.get(2).cloned().unwrap_or(Value::Null);
    let caller_base_url = options.get("baseUrl").and_then(|v| v.as_str()).map(String::from);

    // Resolve provider type and base URL
    let provider = secure_storage::get_provider(&state.data_dir, provider_id)?;
    let provider_type = provider.as_ref()
        .map(|p| p.provider_type.clone())
        .unwrap_or_else(|| provider_id.to_string());

    let registry_base_url = providers::get_provider_config(&provider_type)
        .map(|c| c.base_url.to_string());

    // Prefer caller-supplied baseUrl > stored config > registry
    let resolved_base_url = caller_base_url
        .or_else(|| provider.as_ref().and_then(|p| p.base_url.clone()))
        .or(registry_base_url);

    provider_validate::validate_api_key(&provider_type, api_key, resolved_base_url.as_deref()).await
}
