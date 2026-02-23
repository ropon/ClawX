// Knowledge storage — CRUD operations for KnowledgeBase and Document metadata

use crate::config;
use crate::storage::JsonStore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeBaseConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub embedding_model: String,
    pub embedding_dimension: u32,
    pub document_count: u32,
    pub total_chunks: u32,
    pub total_size: u64,
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watched_folder: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentConfig {
    pub id: String,
    pub knowledge_base_id: String,
    pub file_name: String,
    pub file_path: String,
    pub file_type: String,
    pub file_size: u64,
    pub chunk_count: u32,
    pub status: String, // "pending" | "processing" | "ready" | "error"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeStoreData {
    pub knowledge_bases: HashMap<String, KnowledgeBaseConfig>,
    pub documents: HashMap<String, DocumentConfig>,
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

// ── Knowledge Base CRUD ──

pub async fn get_all_knowledge_bases(
    store: &JsonStore<KnowledgeStoreData>,
) -> Result<Vec<KnowledgeBaseConfig>, String> {
    let data = store.load().await?;
    Ok(data.knowledge_bases.into_values().collect())
}

pub async fn get_knowledge_base(
    store: &JsonStore<KnowledgeStoreData>,
    id: &str,
) -> Result<Option<KnowledgeBaseConfig>, String> {
    let data = store.load().await?;
    Ok(data.knowledge_bases.get(id).cloned())
}

pub async fn create_knowledge_base(
    store: &JsonStore<KnowledgeStoreData>,
    config: serde_json::Value,
) -> Result<KnowledgeBaseConfig, String> {
    let mut data = store.load().await?;
    let now = now_iso();
    let id = format!("kb-{}", uuid::Uuid::new_v4());

    let kb = KnowledgeBaseConfig {
        id: id.clone(),
        name: str_field(&config, "name", "New Knowledge Base"),
        description: str_field(&config, "description", ""),
        embedding_model: str_field(&config, "embeddingModel", ""),
        embedding_dimension: config.get("embeddingDimension").and_then(|v| v.as_u64()).unwrap_or(384) as u32,
        document_count: 0,
        total_chunks: 0,
        total_size: 0,
        chunk_size: config.get("chunkSize").and_then(|v| v.as_u64()).unwrap_or(config::DEFAULT_CHUNK_SIZE as u64) as usize,
        chunk_overlap: config.get("chunkOverlap").and_then(|v| v.as_u64()).unwrap_or(config::DEFAULT_CHUNK_OVERLAP as u64) as usize,
        watched_folder: config.get("watchedFolder").and_then(|v| v.as_str().map(|s| s.to_string())),
        created_at: now.clone(),
        updated_at: now,
    };

    data.knowledge_bases.insert(id, kb.clone());
    store.save(&data).await?;
    Ok(kb)
}

pub async fn update_knowledge_base(
    store: &JsonStore<KnowledgeStoreData>,
    id: &str,
    updates: serde_json::Value,
) -> Result<Option<KnowledgeBaseConfig>, String> {
    let mut data = store.load().await?;
    let kb = match data.knowledge_bases.get(id) {
        Some(k) => k.clone(),
        None => return Ok(None),
    };

    let updated = KnowledgeBaseConfig {
        id: id.to_string(),
        name: str_or(&updates, "name", &kb.name),
        description: str_or(&updates, "description", &kb.description),
        embedding_model: str_or(&updates, "embeddingModel", &kb.embedding_model),
        embedding_dimension: updates.get("embeddingDimension").and_then(|v| v.as_u64()).unwrap_or(kb.embedding_dimension as u64) as u32,
        document_count: updates.get("documentCount").and_then(|v| v.as_u64()).unwrap_or(kb.document_count as u64) as u32,
        total_chunks: updates.get("totalChunks").and_then(|v| v.as_u64()).unwrap_or(kb.total_chunks as u64) as u32,
        total_size: updates.get("totalSize").and_then(|v| v.as_u64()).unwrap_or(kb.total_size),
        chunk_size: updates.get("chunkSize").and_then(|v| v.as_u64()).unwrap_or(kb.chunk_size as u64) as usize,
        chunk_overlap: updates.get("chunkOverlap").and_then(|v| v.as_u64()).unwrap_or(kb.chunk_overlap as u64) as usize,
        watched_folder: if updates.get("watchedFolder").is_some() {
            updates.get("watchedFolder").and_then(|v| v.as_str().map(|s| s.to_string()))
        } else {
            kb.watched_folder
        },
        created_at: kb.created_at,
        updated_at: now_iso(),
    };

    data.knowledge_bases.insert(id.to_string(), updated.clone());
    store.save(&data).await?;
    Ok(Some(updated))
}

pub async fn delete_knowledge_base(
    store: &JsonStore<KnowledgeStoreData>,
    id: &str,
) -> Result<bool, String> {
    let mut data = store.load().await?;

    if !data.knowledge_bases.contains_key(id) {
        return Ok(false);
    }

    // Cascade delete all documents for this KB
    data.documents.retain(|_, doc| doc.knowledge_base_id != id);
    data.knowledge_bases.remove(id);
    store.save(&data).await?;
    Ok(true)
}

// ── Document CRUD ──

pub async fn list_documents(
    store: &JsonStore<KnowledgeStoreData>,
    kb_id: &str,
) -> Result<Vec<DocumentConfig>, String> {
    let data = store.load().await?;
    Ok(data.documents.values()
        .filter(|d| d.knowledge_base_id == kb_id)
        .cloned()
        .collect())
}

pub async fn get_document(
    store: &JsonStore<KnowledgeStoreData>,
    id: &str,
) -> Result<Option<DocumentConfig>, String> {
    let data = store.load().await?;
    Ok(data.documents.get(id).cloned())
}

pub async fn create_document(
    store: &JsonStore<KnowledgeStoreData>,
    config: serde_json::Value,
) -> Result<DocumentConfig, String> {
    let mut data = store.load().await?;
    let now = now_iso();
    let id = format!("doc-{}", uuid::Uuid::new_v4());

    let doc = DocumentConfig {
        id: id.clone(),
        knowledge_base_id: str_field(&config, "knowledgeBaseId", ""),
        file_name: str_field(&config, "fileName", ""),
        file_path: str_field(&config, "filePath", ""),
        file_type: str_field(&config, "fileType", "txt"),
        file_size: config.get("fileSize").and_then(|v| v.as_u64()).unwrap_or(0),
        chunk_count: 0,
        status: "pending".to_string(),
        error: None,
        created_at: now.clone(),
        updated_at: now,
    };

    data.documents.insert(id, doc.clone());
    store.save(&data).await?;
    Ok(doc)
}

pub async fn update_document(
    store: &JsonStore<KnowledgeStoreData>,
    id: &str,
    updates: serde_json::Value,
) -> Result<Option<DocumentConfig>, String> {
    let mut data = store.load().await?;
    let doc = match data.documents.get(id) {
        Some(d) => d.clone(),
        None => return Ok(None),
    };

    let updated = DocumentConfig {
        id: id.to_string(),
        knowledge_base_id: doc.knowledge_base_id,
        file_name: str_or(&updates, "fileName", &doc.file_name),
        file_path: str_or(&updates, "filePath", &doc.file_path),
        file_type: str_or(&updates, "fileType", &doc.file_type),
        file_size: updates.get("fileSize").and_then(|v| v.as_u64()).unwrap_or(doc.file_size),
        chunk_count: updates.get("chunkCount").and_then(|v| v.as_u64()).unwrap_or(doc.chunk_count as u64) as u32,
        status: str_or(&updates, "status", &doc.status),
        error: if updates.get("error").is_some() {
            updates.get("error").and_then(|v| v.as_str().map(|s| s.to_string()))
        } else {
            doc.error
        },
        created_at: doc.created_at,
        updated_at: now_iso(),
    };

    data.documents.insert(id.to_string(), updated.clone());
    store.save(&data).await?;
    Ok(Some(updated))
}

pub async fn delete_document(
    store: &JsonStore<KnowledgeStoreData>,
    id: &str,
) -> Result<bool, String> {
    let mut data = store.load().await?;
    if data.documents.remove(id).is_none() {
        return Ok(false);
    }
    store.save(&data).await?;
    Ok(true)
}

pub async fn refresh_kb_stats(
    store: &JsonStore<KnowledgeStoreData>,
    kb_id: &str,
) -> Result<(), String> {
    let mut data = store.load().await?;
    let kb = match data.knowledge_bases.get_mut(kb_id) {
        Some(k) => k,
        None => return Ok(()),
    };

    let kb_docs: Vec<&DocumentConfig> = data.documents.values()
        .filter(|d| d.knowledge_base_id == kb_id)
        .collect();

    kb.document_count = kb_docs.len() as u32;
    kb.total_chunks = kb_docs.iter()
        .filter(|d| d.status == "ready")
        .map(|d| d.chunk_count)
        .sum();
    kb.total_size = kb_docs.iter().map(|d| d.file_size).sum();
    kb.updated_at = now_iso();

    store.save(&data).await?;
    Ok(())
}

// Helpers

fn str_field(val: &serde_json::Value, key: &str, default: &str) -> String {
    val.get(key).and_then(|v| v.as_str()).unwrap_or(default).to_string()
}

fn str_or(val: &serde_json::Value, key: &str, default: &str) -> String {
    val.get(key).and_then(|v| v.as_str()).unwrap_or(default).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn test_store() -> (TempDir, JsonStore<KnowledgeStoreData>) {
        let tmp = TempDir::new().unwrap();
        let store = JsonStore::new(tmp.path(), "test-knowledge");
        (tmp, store)
    }

    #[tokio::test]
    async fn test_kb_crud() {
        let (_tmp, store) = test_store().await;

        let config = serde_json::json!({
            "name": "Test KB",
            "description": "Test knowledge base",
            "embeddingModel": "local:test",
            "chunkSize": 200,
            "chunkOverlap": 50,
        });
        let kb = create_knowledge_base(&store, config).await.unwrap();
        assert!(kb.id.starts_with("kb-"));
        assert_eq!(kb.name, "Test KB");
        assert_eq!(kb.document_count, 0);

        // Update
        let updates = serde_json::json!({ "name": "Updated KB" });
        let updated = update_knowledge_base(&store, &kb.id, updates).await.unwrap().unwrap();
        assert_eq!(updated.name, "Updated KB");

        // List
        let all = get_all_knowledge_bases(&store).await.unwrap();
        assert_eq!(all.len(), 1);

        // Delete
        let deleted = delete_knowledge_base(&store, &kb.id).await.unwrap();
        assert!(deleted);

        let all = get_all_knowledge_bases(&store).await.unwrap();
        assert!(all.is_empty());
    }

    #[tokio::test]
    async fn test_document_crud() {
        let (_tmp, store) = test_store().await;

        let kb = create_knowledge_base(&store, serde_json::json!({ "name": "KB" })).await.unwrap();

        let doc_config = serde_json::json!({
            "knowledgeBaseId": kb.id,
            "fileName": "test.pdf",
            "filePath": "/tmp/test.pdf",
            "fileType": "pdf",
            "fileSize": 1024,
        });
        let doc = create_document(&store, doc_config).await.unwrap();
        assert!(doc.id.starts_with("doc-"));
        assert_eq!(doc.status, "pending");

        // List docs for KB
        let docs = list_documents(&store, &kb.id).await.unwrap();
        assert_eq!(docs.len(), 1);

        // Delete KB cascades
        delete_knowledge_base(&store, &kb.id).await.unwrap();
        let docs = list_documents(&store, &kb.id).await.unwrap();
        assert!(docs.is_empty());
    }

    #[tokio::test]
    async fn test_refresh_stats() {
        let (_tmp, store) = test_store().await;

        let kb = create_knowledge_base(&store, serde_json::json!({ "name": "Stats KB" })).await.unwrap();

        // Add two docs
        for i in 0..2 {
            let config = serde_json::json!({
                "knowledgeBaseId": kb.id,
                "fileName": format!("doc{}.txt", i),
                "filePath": format!("/tmp/doc{}.txt", i),
                "fileType": "txt",
                "fileSize": 512,
            });
            let doc = create_document(&store, config).await.unwrap();
            // Mark one as ready
            if i == 0 {
                update_document(&store, &doc.id, serde_json::json!({
                    "status": "ready",
                    "chunkCount": 10,
                })).await.unwrap();
            }
        }

        refresh_kb_stats(&store, &kb.id).await.unwrap();
        let updated_kb = get_knowledge_base(&store, &kb.id).await.unwrap().unwrap();
        assert_eq!(updated_kb.document_count, 2);
        assert_eq!(updated_kb.total_chunks, 10);
        assert_eq!(updated_kb.total_size, 1024);
    }
}
