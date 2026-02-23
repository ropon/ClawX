// RAG Engine — Retrieval-Augmented Generation
// Queries multiple knowledge bases and returns ranked context chunks.

use crate::database::vector_db::{self, SearchResult};
use crate::database::DbManager;
use crate::embedding::EmbeddingClient;
use crate::storage::knowledge::KnowledgeStoreData;
use crate::storage::JsonStore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RAGConfig {
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    #[serde(default = "default_score_threshold")]
    pub score_threshold: f32,
    #[serde(default = "default_max_context_tokens")]
    pub max_context_tokens: usize,
}

fn default_top_k() -> usize {
    5
}
fn default_score_threshold() -> f32 {
    0.3
}
fn default_max_context_tokens() -> usize {
    4000
}

impl Default for RAGConfig {
    fn default() -> Self {
        Self {
            top_k: default_top_k(),
            score_threshold: default_score_threshold(),
            max_context_tokens: default_max_context_tokens(),
        }
    }
}

/// Retrieve relevant context from multiple knowledge bases
pub async fn rag_retrieve(
    db_manager: &DbManager,
    data_dir: &Path,
    kb_store: &JsonStore<KnowledgeStoreData>,
    kb_ids: &[String],
    query: &str,
    config: Option<RAGConfig>,
) -> Result<Vec<SearchResult>, String> {
    let cfg = config.unwrap_or_default();
    let mut all_results: Vec<SearchResult> = Vec::new();

    // Group KBs by embedding model to reuse query embeddings
    let mut model_groups: HashMap<String, Vec<String>> = HashMap::new();
    let store_data = kb_store.load().await?;

    for kb_id in kb_ids {
        if let Some(kb) = store_data.knowledge_bases.get(kb_id) {
            model_groups
                .entry(kb.embedding_model.clone())
                .or_default()
                .push(kb_id.clone());
        }
    }

    // Search each model group
    for (model, group_kb_ids) in &model_groups {
        let embedder = match EmbeddingClient::from_model_string(data_dir, model) {
            Ok(e) => e,
            Err(err) => {
                eprintln!("[RAG] Failed to get embedder for model {}: {}", model, err);
                continue;
            }
        };

        let query_embedding = match embedder.embed_query(query).await {
            Ok(e) => e,
            Err(err) => {
                eprintln!("[RAG] Failed to embed query for model {}: {}", model, err);
                continue;
            }
        };

        for kb_id in group_kb_ids {
            let kb_id_owned = kb_id.clone();
            let query_emb = query_embedding.clone();
            let top_k = cfg.top_k;
            let threshold = cfg.score_threshold;

            match db_manager.with_knowledge_conn(|conn| {
                vector_db::search_similar(conn, &kb_id_owned, &query_emb, top_k, threshold)
            }) {
                Ok(results) => all_results.extend(results),
                Err(err) => {
                    eprintln!("[RAG] Search failed for KB {}: {}", kb_id, err);
                }
            }
        }
    }

    // Sort by score descending, take top_k
    all_results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    all_results.truncate(cfg.top_k);

    // Trim to max context tokens (1 token ≈ 4 chars)
    let max_chars = cfg.max_context_tokens * 4;
    let mut total_chars = 0usize;
    let trimmed: Vec<SearchResult> = all_results
        .into_iter()
        .take_while(|r| {
            if total_chars + r.content.len() > max_chars {
                return false;
            }
            total_chars += r.content.len();
            true
        })
        .collect();

    Ok(trimmed)
}
