// Vector DB operations — chunks CRUD + cosine similarity search
// Pure Rust replacement for sqlite-vec virtual tables.
// Embeddings stored as BLOB (f32 little-endian bytes) in the chunks table.

use byteorder::{ByteOrder, LittleEndian};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChunkInsert {
    pub id: String,
    pub document_id: String,
    pub content: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub chunk_id: String,
    pub document_id: String,
    pub document_name: String,
    pub content: String,
    pub score: f32,
}

/// Serialize f32 vector to little-endian bytes
fn f32_vec_to_bytes(vec: &[f32]) -> Vec<u8> {
    let mut bytes = vec![0u8; vec.len() * 4];
    LittleEndian::write_f32_into(vec, &mut bytes);
    bytes
}

/// Deserialize little-endian bytes to f32 vector
fn bytes_to_f32_vec(bytes: &[u8]) -> Vec<f32> {
    let count = bytes.len() / 4;
    let mut vec = vec![0f32; count];
    LittleEndian::read_f32_into(bytes, &mut vec);
    vec
}

/// Cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;

    for i in 0..a.len() {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 {
        0.0
    } else {
        dot / denom
    }
}

/// Create a KB metadata record in the knowledge_bases table
pub fn create_kb_record(
    conn: &Connection,
    kb_id: &str,
    name: &str,
    model: &str,
    dim: u32,
) -> Result<(), String> {
    let now = chrono::Utc::now()
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    conn.execute(
        "INSERT OR IGNORE INTO knowledge_bases (id, name, embedding_model, embedding_dimension, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![kb_id, name, model, dim, now, now],
    )
    .map_err(|e| format!("Failed to create KB record: {}", e))?;
    Ok(())
}

/// Insert chunks with embeddings in a single transaction
pub fn insert_chunks(
    conn: &Connection,
    kb_id: &str,
    chunks: &[ChunkInsert],
) -> Result<(), String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Failed to begin transaction: {}", e))?;

    {
        let mut stmt = tx
            .prepare(
                "INSERT OR REPLACE INTO chunks (id, document_id, knowledge_base_id, content, metadata, embedding)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .map_err(|e| format!("Failed to prepare chunk insert: {}", e))?;

        for chunk in chunks {
            let metadata_str = serde_json::to_string(&chunk.metadata).unwrap_or_default();
            let embedding_bytes = f32_vec_to_bytes(&chunk.embedding);
            stmt.execute(params![
                chunk.id,
                chunk.document_id,
                kb_id,
                chunk.content,
                metadata_str,
                embedding_bytes,
            ])
            .map_err(|e| format!("Failed to insert chunk {}: {}", chunk.id, e))?;
        }
    }

    tx.commit()
        .map_err(|e| format!("Failed to commit chunks: {}", e))?;
    Ok(())
}

/// Search for similar chunks using cosine similarity (in-memory scan)
pub fn search_similar(
    conn: &Connection,
    kb_id: &str,
    query_embedding: &[f32],
    top_k: usize,
    threshold: f32,
) -> Result<Vec<SearchResult>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.document_id, c.content, c.embedding, d.file_name
             FROM chunks c
             JOIN documents d ON d.id = c.document_id
             WHERE c.knowledge_base_id = ?1 AND c.embedding IS NOT NULL",
        )
        .map_err(|e| format!("Failed to prepare search query: {}", e))?;

    let mut scored: Vec<SearchResult> = Vec::new();

    let rows = stmt
        .query_map([kb_id], |row| {
            let chunk_id: String = row.get(0)?;
            let document_id: String = row.get(1)?;
            let content: String = row.get(2)?;
            let embedding_bytes: Vec<u8> = row.get(3)?;
            let document_name: String = row.get(4)?;
            Ok((chunk_id, document_id, content, embedding_bytes, document_name))
        })
        .map_err(|e| format!("Failed to execute search: {}", e))?;

    for row_result in rows {
        let (chunk_id, document_id, content, embedding_bytes, document_name) =
            row_result.map_err(|e| format!("Row error: {}", e))?;

        let embedding = bytes_to_f32_vec(&embedding_bytes);
        let score = cosine_similarity(query_embedding, &embedding);

        if score >= threshold {
            scored.push(SearchResult {
                chunk_id,
                document_id,
                document_name,
                content,
                score,
            });
        }
    }

    // Sort by score descending, take top_k
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(top_k);

    Ok(scored)
}

/// Delete all chunks for a document within a knowledge base
pub fn delete_document_chunks(
    conn: &Connection,
    kb_id: &str,
    doc_id: &str,
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM chunks WHERE knowledge_base_id = ?1 AND document_id = ?2",
        params![kb_id, doc_id],
    )
    .map_err(|e| format!("Failed to delete document chunks: {}", e))?;
    Ok(())
}

/// Drop all data for a knowledge base (chunks, documents, KB record)
pub fn drop_knowledge_base(conn: &Connection, kb_id: &str) -> Result<(), String> {
    conn.execute(
        "DELETE FROM chunks WHERE knowledge_base_id = ?1",
        [kb_id],
    )
    .map_err(|e| format!("Failed to delete KB chunks: {}", e))?;

    conn.execute(
        "DELETE FROM documents WHERE knowledge_base_id = ?1",
        [kb_id],
    )
    .map_err(|e| format!("Failed to delete KB documents: {}", e))?;

    conn.execute("DELETE FROM knowledge_bases WHERE id = ?1", [kb_id])
        .map_err(|e| format!("Failed to delete KB record: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::DbManager;
    use std::sync::Mutex;
    use tempfile::TempDir;

    fn test_mgr(tmp: &TempDir) -> DbManager {
        DbManager {
            knowledge_conn: Mutex::new(None),
            runs_conn: Mutex::new(None),
            knowledge_dir: tmp.path().join("knowledge"),
            workflows_dir: tmp.path().join("workflows"),
        }
    }

    #[test]
    fn test_f32_serialization_roundtrip() {
        let original = vec![0.1f32, 0.2, -0.5, 1.0, 0.0];
        let bytes = f32_vec_to_bytes(&original);
        let recovered = bytes_to_f32_vec(&bytes);
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &a) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!(cosine_similarity(&a, &b).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];
        assert!((cosine_similarity(&a, &b) + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_insert_and_search() {
        let tmp = TempDir::new().unwrap();
        let mgr = test_mgr(&tmp);
        mgr.with_knowledge_conn(|conn| {
            // Create KB record
            create_kb_record(conn, "kb-1", "Test KB", "provider:default", 3)?;

            // Insert a document record for the JOIN
            conn.execute(
                "INSERT INTO documents (id, knowledge_base_id, file_name, file_path, file_type, created_at, updated_at)
                 VALUES ('doc-1', 'kb-1', 'test.txt', '/tmp/test.txt', 'txt', '2024-01-01', '2024-01-01')",
                [],
            ).map_err(|e| e.to_string())?;

            // Insert chunks
            let chunks = vec![
                ChunkInsert {
                    id: "c1".into(),
                    document_id: "doc-1".into(),
                    content: "Rust is a systems programming language".into(),
                    metadata: serde_json::json!({}),
                    embedding: vec![0.8, 0.1, 0.1],
                },
                ChunkInsert {
                    id: "c2".into(),
                    document_id: "doc-1".into(),
                    content: "Python is great for scripting".into(),
                    metadata: serde_json::json!({}),
                    embedding: vec![0.1, 0.8, 0.1],
                },
            ];
            insert_chunks(conn, "kb-1", &chunks)?;

            // Search with a query similar to chunk 1
            let query = vec![0.9, 0.1, 0.0];
            let results = search_similar(conn, "kb-1", &query, 5, 0.1)?;
            assert_eq!(results.len(), 2);
            assert_eq!(results[0].chunk_id, "c1"); // Most similar
            assert!(results[0].score > results[1].score);
            assert_eq!(results[0].document_name, "test.txt");

            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_delete_document_chunks() {
        let tmp = TempDir::new().unwrap();
        let mgr = test_mgr(&tmp);
        mgr.with_knowledge_conn(|conn| {
            create_kb_record(conn, "kb-1", "Test KB", "provider:default", 3)?;

            conn.execute(
                "INSERT INTO documents (id, knowledge_base_id, file_name, file_path, file_type, created_at, updated_at)
                 VALUES ('doc-1', 'kb-1', 'test.txt', '/tmp/test.txt', 'txt', '2024-01-01', '2024-01-01')",
                [],
            ).map_err(|e| e.to_string())?;

            let chunks = vec![ChunkInsert {
                id: "c1".into(),
                document_id: "doc-1".into(),
                content: "test content".into(),
                metadata: serde_json::json!({}),
                embedding: vec![0.5, 0.5, 0.0],
            }];
            insert_chunks(conn, "kb-1", &chunks)?;

            delete_document_chunks(conn, "kb-1", "doc-1")?;

            let count: i64 = conn
                .prepare("SELECT COUNT(*) FROM chunks WHERE knowledge_base_id = 'kb-1'")
                .unwrap()
                .query_row([], |row| row.get(0))
                .unwrap();
            assert_eq!(count, 0);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_drop_knowledge_base() {
        let tmp = TempDir::new().unwrap();
        let mgr = test_mgr(&tmp);
        mgr.with_knowledge_conn(|conn| {
            create_kb_record(conn, "kb-1", "Test KB", "provider:default", 3)?;
            drop_knowledge_base(conn, "kb-1")?;

            let count: i64 = conn
                .prepare("SELECT COUNT(*) FROM knowledge_bases WHERE id = 'kb-1'")
                .unwrap()
                .query_row([], |row| row.get(0))
                .unwrap();
            assert_eq!(count, 0);
            Ok(())
        })
        .unwrap();
    }
}
