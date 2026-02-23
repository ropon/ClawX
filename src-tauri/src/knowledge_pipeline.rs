// Knowledge pipeline — Orchestrates document ingestion: parse → chunk → embed → store

use crate::config;
use crate::database::vector_db::{self, ChunkInsert};
use crate::database::DbManager;
use crate::document_parser;
use crate::embedding;
use crate::storage::knowledge::{self, KnowledgeStoreData};
use crate::storage::JsonStore;
use crate::text_chunker;
use serde_json::json;
use std::path::Path;
use tauri::{AppHandle, Emitter};

const BATCH_SIZE: usize = 32;

/// Process a document through the full ingestion pipeline
pub async fn process_document(
    app: &AppHandle,
    data_dir: &Path,
    db_manager: &DbManager,
    kb_id: &str,
    document_id: &str,
    file_path: &str,
    file_type: &str,
    embedding_model: &str,
    chunk_size: usize,
    chunk_overlap: usize,
    embedding_dimension: u32,
) -> Result<(), String> {
    let store: JsonStore<KnowledgeStoreData> =
        JsonStore::new(data_dir, config::KNOWLEDGE_STORE_NAME);

    // Step 1: Update status to processing
    knowledge::update_document(
        &store,
        document_id,
        json!({ "status": "processing" }),
    )
    .await?;
    emit_progress(app, document_id, "parsing", 10);

    // Step 2: Parse document
    let parsed = document_parser::parse_document(file_path, file_type)?;
    if parsed.text.trim().is_empty() {
        knowledge::update_document(
            &store,
            document_id,
            json!({ "status": "error", "error": "No text content found in document" }),
        )
        .await?;
        return Err("No text content found in document".to_string());
    }

    // Step 3: Chunk text
    let chunks = text_chunker::chunk_text(
        &parsed.text,
        chunk_size,
        chunk_overlap,
    );
    if chunks.is_empty() {
        knowledge::update_document(
            &store,
            document_id,
            json!({ "status": "error", "error": "No chunks generated from document" }),
        )
        .await?;
        return Err("No chunks generated from document".to_string());
    }
    emit_progress(app, document_id, "chunking", 20);

    // Step 4: Get embedding client
    let embedder =
        embedding::EmbeddingClient::from_model_string(data_dir, embedding_model)?;

    // Ensure KB record exists in SQLite
    db_manager.with_knowledge_conn(|conn| {
        // Ignore error if record already exists
        let _ = vector_db::create_kb_record(
            conn,
            kb_id,
            "",
            embedding_model,
            embedding_dimension,
        );
        Ok(())
    })?;
    emit_progress(app, document_id, "embedding", 30);

    // Step 5: Embed in batches and collect chunks
    let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
    let total_chunks = texts.len();
    let mut all_chunk_inserts: Vec<ChunkInsert> = Vec::with_capacity(total_chunks);

    for batch_start in (0..total_chunks).step_by(BATCH_SIZE) {
        let batch_end = (batch_start + BATCH_SIZE).min(total_chunks);
        let batch: Vec<String> = texts[batch_start..batch_end].to_vec();

        let embeddings = embedder.embed(&batch).await?;

        for (i, embedding_vec) in embeddings.into_iter().enumerate() {
            let chunk_idx = batch_start + i;
            let chunk = &chunks[chunk_idx];

            // Estimate page number from chunk position
            let page = if let Some(page_count) = parsed.metadata.page_count {
                if page_count > 0 {
                    Some((chunk_idx * page_count / total_chunks) + 1)
                } else {
                    None
                }
            } else {
                None
            };

            all_chunk_inserts.push(ChunkInsert {
                id: format!("{}-chunk-{}", document_id, chunk_idx),
                document_id: document_id.to_string(),
                content: chunk.content.clone(),
                metadata: json!({
                    "lineStart": chunk.metadata.line_start,
                    "lineEnd": chunk.metadata.line_end,
                    "page": page,
                }),
                embedding: embedding_vec,
            });
        }

        // Report progress
        let progress = 30 + ((batch_end as f64 / total_chunks as f64) * 55.0) as u32;
        emit_progress(app, document_id, "embedding", progress);
    }

    // Step 6: Insert chunks into database
    emit_progress(app, document_id, "storing", 90);
    db_manager.with_knowledge_conn(|conn| {
        vector_db::insert_chunks(conn, kb_id, &all_chunk_inserts)
    })?;

    // Step 7: Update document status
    knowledge::update_document(
        &store,
        document_id,
        json!({
            "status": "ready",
            "chunkCount": total_chunks,
        }),
    )
    .await?;

    // Step 8: Refresh KB stats
    knowledge::refresh_kb_stats(&store, kb_id).await?;
    emit_progress(app, document_id, "done", 100);

    Ok(())
}

/// Emit progress event to frontend
fn emit_progress(app: &AppHandle, document_id: &str, stage: &str, percent: u32) {
    let _ = app.emit(
        "knowledge_documentProgress",
        json!({
            "documentId": document_id,
            "stage": stage,
            "percent": percent,
        }),
    );
}
