// Embedding API — Provider-based embedding via OpenAI-compatible endpoints
// Extended from existing embedding.rs; provides standalone functions for the knowledge pipeline.
// Defers to EmbeddingClient for actual API calls but adds discovery/detection utilities.

use crate::secure_storage;
use serde_json::{json, Value};
use std::path::Path;

/// Get the default embedding model name for a provider type
pub fn get_embedding_model_name(provider_type: &str) -> &str {
    match provider_type {
        "openai" | "azure" | "openrouter" => "text-embedding-3-small",
        "cohere" => "embed-english-v3.0",
        "voyage" => "voyage-2",
        _ => "text-embedding-3-small",
    }
}

/// Get the default dimension for a provider type
pub fn get_default_dimension(provider_type: &str) -> usize {
    match provider_type {
        "openai" | "azure" | "openrouter" => 1536,
        "cohere" => 1024,
        "voyage" => 1024,
        _ => 1536,
    }
}

/// Get available embedding options for UI display
pub fn get_embedding_options(data_dir: &Path) -> Result<Value, String> {
    let mut options = Vec::new();

    // Always include local option (placeholder — not yet functional in Tauri)
    options.push(json!({
        "label": "Local (all-MiniLM-L6-v2)",
        "value": "local:all-MiniLM-L6-v2",
        "dimension": 384,
        "available": false,
        "note": "Local ONNX embedding not available in Tauri mode",
    }));

    // Get default provider
    let default_provider_id = secure_storage::get_default_provider_id(data_dir)
        .ok()
        .flatten();

    if let Some(ref provider_id) = default_provider_id {
        if let Ok(Some(provider)) = secure_storage::get_provider(data_dir, provider_id) {
            let has_key = secure_storage::has_api_key(data_dir, provider_id).unwrap_or(false);
            if has_key {
                let model_name = get_embedding_model_name(&provider.provider_type);
                let dimension = get_default_dimension(&provider.provider_type);
                options.insert(
                    0,
                    json!({
                        "label": format!("{} ({})", provider.name, model_name),
                        "value": "provider:default",
                        "dimension": dimension,
                        "available": true,
                    }),
                );
            }
        }
    }

    // Add all providers with API keys as explicit options
    if let Ok(store) = secure_storage::get_all_providers_with_key_info(data_dir) {
        if let Some(providers_arr) = store.as_array() {
            for provider_val in providers_arr {
                let has_key = provider_val
                    .get("hasKey")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if !has_key {
                    continue;
                }

                let id = provider_val.get("id").and_then(|v| v.as_str()).unwrap_or("");
                // Skip default (already added above)
                if default_provider_id.as_deref() == Some(id) {
                    continue;
                }

                let name = provider_val
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Provider");
                let ptype = provider_val
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let model_name = get_embedding_model_name(ptype);
                let dimension = get_default_dimension(ptype);

                options.push(json!({
                    "label": format!("{} ({})", name, model_name),
                    "value": format!("provider:{}", id),
                    "dimension": dimension,
                    "available": true,
                }));
            }
        }
    }

    Ok(json!(options))
}

/// Detect the actual embedding dimension by making a test API call
pub async fn detect_dimension(data_dir: &Path, embedding_model: &str) -> Result<usize, String> {
    let client = crate::embedding::EmbeddingClient::from_model_string(data_dir, embedding_model)?;
    let test_embedding = client.embed_query("dimension detection test").await?;
    Ok(test_embedding.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_model_name() {
        assert_eq!(get_embedding_model_name("openai"), "text-embedding-3-small");
        assert_eq!(get_embedding_model_name("cohere"), "embed-english-v3.0");
        assert_eq!(get_embedding_model_name("unknown"), "text-embedding-3-small");
    }

    #[test]
    fn test_get_default_dimension() {
        assert_eq!(get_default_dimension("openai"), 1536);
        assert_eq!(get_default_dimension("cohere"), 1024);
    }

    #[test]
    fn test_get_embedding_options_empty() {
        let tmp = tempfile::TempDir::new().unwrap();
        let options = get_embedding_options(tmp.path()).unwrap();
        assert!(options.as_array().unwrap().len() >= 1); // at least local option
    }
}
