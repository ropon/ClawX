// Tauri commands for knowledge:* channels
// 11 CRUD commands + 2 search/RAG commands (Week 4)
// + 4 simple commands + 4 pipeline commands (Week 7)

use crate::config;
use crate::database::vector_db;
use crate::embedding_api;
use crate::knowledge_pipeline;
use crate::rag_engine;
use crate::storage::knowledge::{self, KnowledgeStoreData};
use crate::storage::JsonStore;
use crate::web_scraper;
use crate::AppState;
use serde_json::json;
use tauri::{AppHandle, Manager, State};

fn store(state: &AppState) -> JsonStore<KnowledgeStoreData> {
    JsonStore::new(&state.data_dir, config::KNOWLEDGE_STORE_NAME)
}

#[tauri::command]
pub async fn knowledge_list(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let s = store(&state);
    let kbs = knowledge::get_all_knowledge_bases(&s).await?;
    Ok(json!({ "success": true, "knowledgeBases": kbs }))
}

#[tauri::command]
pub async fn knowledge_get(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing knowledge base id")?;
    let s = store(&state);
    let kb = knowledge::get_knowledge_base(&s, id).await?;
    Ok(json!({ "success": true, "knowledgeBase": kb }))
}

#[tauri::command]
pub async fn knowledge_create(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let config = _args.first()
        .cloned()
        .unwrap_or(serde_json::Value::Object(Default::default()));
    let s = store(&state);
    let kb = knowledge::create_knowledge_base(&s, config).await?;

    // Also create KB record in SQLite for vector search
    state.db_manager.with_knowledge_conn(|conn| {
        vector_db::create_kb_record(
            conn,
            &kb.id,
            &kb.name,
            &kb.embedding_model,
            kb.embedding_dimension,
        )
    })?;

    Ok(json!({ "success": true, "knowledgeBase": kb }))
}

#[tauri::command]
pub async fn knowledge_update(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing knowledge base id")?;
    let updates = _args.get(1)
        .cloned()
        .unwrap_or(serde_json::Value::Object(Default::default()));
    let s = store(&state);
    let kb = knowledge::update_knowledge_base(&s, id, updates).await?;
    Ok(json!({ "success": true, "knowledgeBase": kb }))
}

#[tauri::command]
pub async fn knowledge_delete(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing knowledge base id")?;

    // Clean up SQLite data first (chunks, documents, KB record)
    state.db_manager.with_knowledge_conn(|conn| {
        vector_db::drop_knowledge_base(conn, id)
    })?;

    // Then delete from JSON store
    let s = store(&state);
    let deleted = knowledge::delete_knowledge_base(&s, id).await?;
    Ok(json!({ "success": true, "deleted": deleted }))
}

#[tauri::command]
pub async fn knowledge_list_documents(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let kb_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing knowledge base id")?;
    let s = store(&state);
    let docs = knowledge::list_documents(&s, kb_id).await?;
    Ok(json!({ "success": true, "documents": docs }))
}

#[tauri::command]
pub async fn knowledge_get_document(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing document id")?;
    let s = store(&state);
    let doc = knowledge::get_document(&s, id).await?;
    Ok(json!({ "success": true, "document": doc }))
}

#[tauri::command]
pub async fn knowledge_create_document(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let config = _args.first()
        .cloned()
        .unwrap_or(serde_json::Value::Object(Default::default()));
    let s = store(&state);
    let doc = knowledge::create_document(&s, config).await?;
    Ok(json!({ "success": true, "document": doc }))
}

#[tauri::command]
pub async fn knowledge_update_document(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing document id")?;
    let updates = _args.get(1)
        .cloned()
        .unwrap_or(serde_json::Value::Object(Default::default()));
    let s = store(&state);
    let doc = knowledge::update_document(&s, id, updates).await?;
    Ok(json!({ "success": true, "document": doc }))
}

#[tauri::command]
pub async fn knowledge_delete_document(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing document id")?;
    let s = store(&state);
    let deleted = knowledge::delete_document(&s, id).await?;
    Ok(json!({ "success": true, "deleted": deleted }))
}

#[tauri::command]
pub async fn knowledge_refresh_stats(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let kb_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing knowledge base id")?;
    let s = store(&state);
    knowledge::refresh_kb_stats(&s, kb_id).await?;
    Ok(json!({ "success": true }))
}

// ── Search & RAG commands (Week 4) ──

#[tauri::command]
pub async fn knowledge_search(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let kb_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing knowledge base id")?;
    let query = _args.get(1)
        .and_then(|v| v.as_str())
        .ok_or("Missing query")?;

    // Get KB metadata to find embedding model
    let s = store(&state);
    let kb = knowledge::get_knowledge_base(&s, kb_id).await?
        .ok_or("Knowledge base not found")?;

    // Embed the query
    let embedder = crate::embedding::EmbeddingClient::from_model_string(
        &state.data_dir,
        &kb.embedding_model,
    )?;
    let query_embedding = embedder.embed_query(query).await?;

    // Search
    let results = state.db_manager.with_knowledge_conn(|conn| {
        vector_db::search_similar(conn, kb_id, &query_embedding, 5, 0.3)
    })?;

    Ok(json!({ "success": true, "results": results }))
}

#[tauri::command]
pub async fn knowledge_rag(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let kb_ids: Vec<String> = match _args.first() {
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        Some(serde_json::Value::String(s)) => vec![s.clone()],
        _ => return Err("Missing knowledge base ids".to_string()),
    };

    let query = _args.get(1)
        .and_then(|v| v.as_str())
        .ok_or("Missing query")?;

    let rag_config: Option<rag_engine::RAGConfig> = _args.get(2)
        .and_then(|v| serde_json::from_value(v.clone()).ok());

    let s = store(&state);
    let results = rag_engine::rag_retrieve(
        &state.db_manager,
        &state.data_dir,
        &s,
        &kb_ids,
        query,
        rag_config,
    )
    .await?;

    Ok(json!({ "success": true, "results": results }))
}

// ── Week 7: Simple commands ──

/// knowledge:removeDocument — Remove document and its chunks
/// _args: [documentId]
#[tauri::command]
pub async fn knowledge_remove_document(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let doc_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing document id")?;

    let s = store(&state);

    // Get document to find KB ID
    let doc = knowledge::get_document(&s, doc_id).await?
        .ok_or("Document not found")?;

    // Delete chunks from SQLite
    let kb_id = &doc.knowledge_base_id;
    state.db_manager.with_knowledge_conn(|conn| {
        vector_db::delete_document_chunks(conn, kb_id, doc_id)
    })?;

    // Delete document from JSON store
    knowledge::delete_document(&s, doc_id).await?;

    // Refresh stats
    knowledge::refresh_kb_stats(&s, kb_id).await?;

    Ok(json!({ "success": true }))
}

/// knowledge:getWatchStatus — Get folder watch status for a KB
/// _args: [kbId]
#[tauri::command]
pub async fn knowledge_get_watch_status(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let kb_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing knowledge base id")?;

    let s = store(&state);
    let kb = knowledge::get_knowledge_base(&s, kb_id).await?;

    let (watching, folder) = if let Some(ref kb) = kb {
        let is_watching = state
            .folder_watcher
            .lock()
            .map_err(|e| e.to_string())?
            .is_watching(kb_id);
        (is_watching, kb.watched_folder.clone())
    } else {
        (false, None)
    };

    Ok(json!({
        "success": true,
        "watching": watching,
        "watchedFolder": folder,
    }))
}

/// knowledge:getEmbeddingOptions — Get available embedding models
/// _args: []
#[tauri::command]
pub async fn knowledge_get_embedding_options(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let options = embedding_api::get_embedding_options(&state.data_dir)?;
    Ok(json!({ "success": true, "options": options }))
}

/// knowledge:detectDimension — Detect embedding dimension by test API call
/// _args: [embeddingModel]
#[tauri::command]
pub async fn knowledge_detect_dimension(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let embedding_model = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing embedding model")?;

    let dimension = embedding_api::detect_dimension(&state.data_dir, embedding_model).await?;
    Ok(json!({ "success": true, "dimension": dimension }))
}

// ── Week 7: Pipeline commands ──

/// Helper to spawn the knowledge pipeline in a background task
fn spawn_pipeline(
    app: AppHandle,
    kb_id: String,
    doc_id: String,
    file_path: String,
    file_type: String,
    embedding_model: String,
    chunk_size: usize,
    chunk_overlap: usize,
    embedding_dimension: u32,
) {
    tokio::spawn(async move {
        let state = app.state::<AppState>();
        if let Err(e) = knowledge_pipeline::process_document(
            &app,
            &state.data_dir,
            &state.db_manager,
            &kb_id,
            &doc_id,
            &file_path,
            &file_type,
            &embedding_model,
            chunk_size,
            chunk_overlap,
            embedding_dimension,
        )
        .await
        {
            eprintln!("[ClawX] Document processing failed: {}", e);
        }
    });
}

/// knowledge:addDocument — Add and process a document
/// _args: [kbId, filePath, fileName, fileType, fileSize]
#[tauri::command]
pub async fn knowledge_add_document(
    app: AppHandle,
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let kb_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing knowledge base id")?;
    let file_path = _args.get(1)
        .and_then(|v| v.as_str())
        .ok_or("Missing file path")?;
    let file_name = _args.get(2)
        .and_then(|v| v.as_str())
        .ok_or("Missing file name")?;
    let file_type = _args.get(3)
        .and_then(|v| v.as_str())
        .ok_or("Missing file type")?;
    let file_size = _args.get(4)
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    // Create document record
    let s = store(&state);
    let doc = knowledge::create_document(&s, json!({
        "knowledgeBaseId": kb_id,
        "fileName": file_name,
        "filePath": file_path,
        "fileType": file_type,
        "fileSize": file_size,
    })).await?;

    // Get KB for pipeline config
    let kb = knowledge::get_knowledge_base(&s, kb_id).await?
        .ok_or("Knowledge base not found")?;

    // Spawn async pipeline processing (non-blocking)
    spawn_pipeline(
        app, kb_id.to_string(), doc.id.clone(), file_path.to_string(),
        file_type.to_string(), kb.embedding_model, kb.chunk_size,
        kb.chunk_overlap, kb.embedding_dimension,
    );

    Ok(json!({ "success": true, "document": doc }))
}

/// knowledge:addUrl — Scrape URL and process as document
/// _args: [kbId, url]
#[tauri::command]
pub async fn knowledge_add_url(
    app: AppHandle,
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let kb_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing knowledge base id")?;
    let url = _args.get(1)
        .and_then(|v| v.as_str())
        .ok_or("Missing URL")?;

    // Scrape the URL
    let scraped = web_scraper::scrape_url(url).await?;

    // Write to temp file
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("clawx-url-{}.txt", uuid::Uuid::new_v4()));
    std::fs::write(&temp_file, &scraped.text)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    let file_size = scraped.text.len() as u64;
    let file_path = temp_file.to_string_lossy().to_string();

    // Create document record
    let s = store(&state);
    let doc = knowledge::create_document(&s, json!({
        "knowledgeBaseId": kb_id,
        "fileName": scraped.title,
        "filePath": &file_path,
        "fileType": "txt",
        "fileSize": file_size,
    })).await?;

    // Get KB for pipeline config
    let kb = knowledge::get_knowledge_base(&s, kb_id).await?
        .ok_or("Knowledge base not found")?;

    // Spawn async pipeline processing
    spawn_pipeline(
        app, kb_id.to_string(), doc.id.clone(), file_path,
        "txt".to_string(), kb.embedding_model, kb.chunk_size,
        kb.chunk_overlap, kb.embedding_dimension,
    );

    Ok(json!({ "success": true, "document": doc }))
}

/// knowledge:reprocessDocument — Reprocess an existing document
/// _args: [documentId]
#[tauri::command]
pub async fn knowledge_reprocess_document(
    app: AppHandle,
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let doc_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing document id")?;

    let s = store(&state);

    // Get document
    let doc = knowledge::get_document(&s, doc_id).await?
        .ok_or("Document not found")?;

    // Get KB
    let kb = knowledge::get_knowledge_base(&s, &doc.knowledge_base_id).await?
        .ok_or("Knowledge base not found")?;

    // Delete old chunks
    let kb_id = doc.knowledge_base_id.as_str();
    state.db_manager.with_knowledge_conn(|conn| {
        vector_db::delete_document_chunks(conn, kb_id, doc_id)
    })?;

    // Reset document status
    knowledge::update_document(&s, doc_id, json!({
        "status": "pending",
        "chunkCount": 0,
    })).await?;

    // Spawn async reprocessing
    spawn_pipeline(
        app, doc.knowledge_base_id, doc_id.to_string(), doc.file_path,
        doc.file_type, kb.embedding_model, kb.chunk_size,
        kb.chunk_overlap, kb.embedding_dimension,
    );

    Ok(json!({ "success": true }))
}

/// knowledge:setWatchFolder — Set folder watching for a KB
/// _args: [kbId, folderPath (or null to stop)]
#[tauri::command]
pub async fn knowledge_set_watch_folder(
    _args: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let kb_id = _args.first()
        .and_then(|v| v.as_str())
        .ok_or("Missing knowledge base id")?;
    let folder_path = _args.get(1).and_then(|v| v.as_str());

    let s = store(&state);

    // Update KB watched folder
    let updates = if let Some(path) = folder_path {
        json!({ "watchedFolder": path })
    } else {
        json!({ "watchedFolder": null })
    };
    knowledge::update_knowledge_base(&s, kb_id, updates).await?;

    // Start or stop folder watcher
    let mut watcher = state.folder_watcher.lock().map_err(|e| e.to_string())?;

    if let Some(path) = folder_path {
        watcher.start_watching(kb_id, path, |_event| {
            // File events handled — in a full implementation, this would
            // trigger document addition/removal
        })?;
    } else {
        watcher.stop_watching(kb_id);
    }

    Ok(json!({ "success": true }))
}
