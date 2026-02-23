// Embedding API client — calls OpenAI-compatible /embeddings endpoint

use crate::providers;
use crate::secure_storage;
use serde::{Deserialize, Serialize};
use std::path::Path;

const BATCH_SIZE: usize = 32;
const DEFAULT_EMBEDDING_MODEL: &str = "text-embedding-3-small";
const DEFAULT_DIMENSION: u32 = 1536;

#[derive(Debug)]
pub struct EmbeddingClient {
    base_url: String,
    api_key: String,
    model: String,
    #[allow(dead_code)]
    pub dimension: u32,
}

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
    index: usize,
}

impl EmbeddingClient {
    /// Create an embedding client from a model string like "provider:default" or "provider:<id>"
    pub fn from_model_string(data_dir: &Path, embedding_model: &str) -> Result<Self, String> {
        if embedding_model.starts_with("local:") {
            return Err(
                "Local ONNX embedding not supported in Tauri mode yet. Use provider-based embeddings."
                    .to_string(),
            );
        }

        if !embedding_model.starts_with("provider:") {
            return Err(format!("Unknown embedding model format: {}", embedding_model));
        }

        let suffix = &embedding_model["provider:".len()..];

        let provider_id = if suffix == "default" {
            secure_storage::get_default_provider_id(data_dir)?
                .ok_or("No default provider configured for embeddings")?
        } else {
            suffix.to_string()
        };

        let provider = secure_storage::get_provider(data_dir, &provider_id)?
            .ok_or(format!("Provider not found: {}", provider_id))?;

        let api_key = secure_storage::get_api_key(data_dir, &provider_id)?
            .ok_or(format!("No API key for provider: {}", provider.name))?;

        let base_url = Self::resolve_base_url(&provider)?;

        Ok(Self {
            base_url,
            api_key,
            model: DEFAULT_EMBEDDING_MODEL.to_string(),
            dimension: DEFAULT_DIMENSION,
        })
    }

    fn resolve_base_url(provider: &secure_storage::ProviderEntry) -> Result<String, String> {
        if let Some(ref url) = provider.base_url {
            let trimmed = url.trim_end_matches('/');
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
        // Fallback to provider registry
        if let Some(config) = providers::get_provider_config(&provider.provider_type) {
            return Ok(config.base_url.trim_end_matches('/').to_string());
        }
        Err(format!(
            "No base URL for provider type: {}",
            provider.provider_type
        ))
    }

    /// Embed multiple texts with batching (batch size 32)
    pub async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, String> {
        let client = reqwest::Client::new();
        let mut all_embeddings: Vec<Vec<f32>> = Vec::with_capacity(texts.len());

        for batch_start in (0..texts.len()).step_by(BATCH_SIZE) {
            let batch_end = (batch_start + BATCH_SIZE).min(texts.len());
            let batch: Vec<String> = texts[batch_start..batch_end].to_vec();

            let request = EmbeddingRequest {
                model: self.model.clone(),
                input: batch,
            };

            let resp = client
                .post(format!("{}/embeddings", self.base_url))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", self.api_key))
                .json(&request)
                .send()
                .await
                .map_err(|e| format!("Embedding API request failed: {}", e))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let err_text = resp.text().await.unwrap_or_else(|_| "Unknown error".into());
                return Err(format!("Embedding API error ({}): {}", status, err_text));
            }

            let data: EmbeddingResponse = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse embedding response: {}", e))?;

            // Sort by index to maintain order
            let mut sorted = data.data;
            sorted.sort_by_key(|d| d.index);
            all_embeddings.extend(sorted.into_iter().map(|d| d.embedding));
        }

        Ok(all_embeddings)
    }

    /// Embed a single query string
    pub async fn embed_query(&self, query: &str) -> Result<Vec<f32>, String> {
        let results = self.embed(&[query.to_string()]).await?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| "Empty embedding response".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_model_rejected() {
        let tmp = tempfile::TempDir::new().unwrap();
        let result = EmbeddingClient::from_model_string(tmp.path(), "local:all-MiniLM-L6-v2");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Local ONNX embedding not supported"));
    }

    #[test]
    fn test_unknown_format_rejected() {
        let tmp = tempfile::TempDir::new().unwrap();
        let result = EmbeddingClient::from_model_string(tmp.path(), "unknown:model");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown embedding model format"));
    }

    #[test]
    fn test_missing_provider_rejected() {
        let tmp = tempfile::TempDir::new().unwrap();
        let result = EmbeddingClient::from_model_string(tmp.path(), "provider:default");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_model_string_with_provider() {
        let tmp = tempfile::TempDir::new().unwrap();
        // Write a test providers store
        let store = serde_json::json!({
            "providers": {
                "p1": {
                    "id": "p1",
                    "name": "TestAI",
                    "type": "openai",
                    "baseUrl": "https://api.test.com/v1",
                    "enabled": true
                }
            },
            "apiKeys": { "p1": "sk-test" },
            "defaultProvider": "p1"
        });
        std::fs::write(
            tmp.path().join("clawx-providers.json"),
            serde_json::to_string(&store).unwrap(),
        )
        .unwrap();

        let client = EmbeddingClient::from_model_string(tmp.path(), "provider:default").unwrap();
        assert_eq!(client.base_url, "https://api.test.com/v1");
        assert_eq!(client.model, "text-embedding-3-small");
        assert_eq!(client.dimension, 1536);

        let client2 = EmbeddingClient::from_model_string(tmp.path(), "provider:p1").unwrap();
        assert_eq!(client2.base_url, "https://api.test.com/v1");
    }
}
