// SQLite database connection management
// Manages two databases: knowledge (vectors/chunks) and workflow runs
// Uses WAL mode for concurrent read/write

pub mod vector_db;
pub mod workflow_runs;

use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct DbManager {
    knowledge_conn: Mutex<Option<Connection>>,
    runs_conn: Mutex<Option<Connection>>,
    knowledge_dir: PathBuf,
    workflows_dir: PathBuf,
}

impl DbManager {
    pub fn new() -> Self {
        let openclaw_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".openclaw");

        Self {
            knowledge_conn: Mutex::new(None),
            runs_conn: Mutex::new(None),
            knowledge_dir: openclaw_dir.join("knowledge"),
            workflows_dir: openclaw_dir.join("workflows"),
        }
    }

    /// Get or lazily initialize the knowledge database connection
    pub fn with_knowledge_conn<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&Connection) -> Result<R, String>,
    {
        let mut guard = self.knowledge_conn.lock().map_err(|e| e.to_string())?;
        if guard.is_none() {
            std::fs::create_dir_all(&self.knowledge_dir)
                .map_err(|e| format!("Failed to create knowledge dir: {}", e))?;
            let db_path = self.knowledge_dir.join("knowledge.db");
            let conn = Connection::open(&db_path)
                .map_err(|e| format!("Failed to open knowledge DB: {}", e))?;
            Self::init_knowledge_db(&conn)?;
            *guard = Some(conn);
        }
        let conn = guard.as_ref().expect("connection must exist after initialization");
        f(conn)
    }

    /// Get or lazily initialize the workflow runs database connection
    pub fn with_runs_conn<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&Connection) -> Result<R, String>,
    {
        let mut guard = self.runs_conn.lock().map_err(|e| e.to_string())?;
        if guard.is_none() {
            std::fs::create_dir_all(&self.workflows_dir)
                .map_err(|e| format!("Failed to create workflows dir: {}", e))?;
            let db_path = self.workflows_dir.join("runs.db");
            let conn = Connection::open(&db_path)
                .map_err(|e| format!("Failed to open runs DB: {}", e))?;
            Self::init_runs_db(&conn)?;
            *guard = Some(conn);
        }
        let conn = guard.as_ref().expect("connection must exist after initialization");
        f(conn)
    }

    fn init_knowledge_db(conn: &Connection) -> Result<(), String> {
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000;")
            .map_err(|e| format!("Failed to set knowledge DB pragmas: {}", e))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS knowledge_bases (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT DEFAULT '',
                embedding_model TEXT NOT NULL,
                embedding_dimension INTEGER NOT NULL,
                document_count INTEGER DEFAULT 0,
                total_chunks INTEGER DEFAULT 0,
                total_size INTEGER DEFAULT 0,
                chunk_size INTEGER DEFAULT 500,
                chunk_overlap INTEGER DEFAULT 100,
                watched_folder TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                knowledge_base_id TEXT NOT NULL,
                file_name TEXT NOT NULL,
                file_path TEXT NOT NULL,
                file_type TEXT NOT NULL,
                file_size INTEGER DEFAULT 0,
                chunk_count INTEGER DEFAULT 0,
                status TEXT DEFAULT 'pending',
                error TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (knowledge_base_id) REFERENCES knowledge_bases(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS chunks (
                id TEXT PRIMARY KEY,
                document_id TEXT NOT NULL,
                knowledge_base_id TEXT NOT NULL,
                content TEXT NOT NULL,
                metadata TEXT DEFAULT '{}',
                embedding BLOB,
                FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
                FOREIGN KEY (knowledge_base_id) REFERENCES knowledge_bases(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_chunks_kb ON chunks(knowledge_base_id);
            CREATE INDEX IF NOT EXISTS idx_chunks_doc ON chunks(document_id);",
        )
        .map_err(|e| format!("Failed to init knowledge schema: {}", e))?;

        // Add embedding column if it doesn't exist (migration from Week 3 schema)
        // SQLite doesn't support IF NOT EXISTS for ALTER TABLE, so we check first
        let has_embedding: bool = conn
            .prepare("SELECT COUNT(*) FROM pragma_table_info('chunks') WHERE name = 'embedding'")
            .and_then(|mut stmt| stmt.query_row([], |row| row.get::<_, i64>(0)))
            .map(|count| count > 0)
            .unwrap_or(false);

        if !has_embedding {
            conn.execute_batch("ALTER TABLE chunks ADD COLUMN embedding BLOB")
                .map_err(|e| format!("Failed to add embedding column: {}", e))?;
        }

        Ok(())
    }

    fn init_runs_db(conn: &Connection) -> Result<(), String> {
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000;")
            .map_err(|e| format!("Failed to set runs DB pragmas: {}", e))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS workflow_runs (
                id TEXT PRIMARY KEY,
                workflow_id TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                trigger_type TEXT NOT NULL DEFAULT 'manual',
                trigger_input TEXT,
                started_at INTEGER NOT NULL,
                completed_at INTEGER,
                steps_json TEXT DEFAULT '[]',
                final_output TEXT,
                error TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_runs_workflow_id ON workflow_runs(workflow_id);
            CREATE INDEX IF NOT EXISTS idx_runs_started_at ON workflow_runs(started_at DESC);",
        )
        .map_err(|e| format!("Failed to init runs schema: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_db_manager(tmp: &TempDir) -> DbManager {
        DbManager {
            knowledge_conn: Mutex::new(None),
            runs_conn: Mutex::new(None),
            knowledge_dir: tmp.path().join("knowledge"),
            workflows_dir: tmp.path().join("workflows"),
        }
    }

    #[test]
    fn test_knowledge_db_init() {
        let tmp = TempDir::new().unwrap();
        let mgr = test_db_manager(&tmp);
        let result = mgr.with_knowledge_conn(|conn| {
            let count: i64 = conn
                .prepare("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='chunks'")
                .unwrap()
                .query_row([], |row| row.get(0))
                .unwrap();
            Ok(count)
        });
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_runs_db_init() {
        let tmp = TempDir::new().unwrap();
        let mgr = test_db_manager(&tmp);
        let result = mgr.with_runs_conn(|conn| {
            let count: i64 = conn
                .prepare("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='workflow_runs'")
                .unwrap()
                .query_row([], |row| row.get(0))
                .unwrap();
            Ok(count)
        });
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_embedding_column_exists() {
        let tmp = TempDir::new().unwrap();
        let mgr = test_db_manager(&tmp);
        let result = mgr.with_knowledge_conn(|conn| {
            let has: bool = conn
                .prepare("SELECT COUNT(*) FROM pragma_table_info('chunks') WHERE name = 'embedding'")
                .unwrap()
                .query_row([], |row| row.get::<_, i64>(0))
                .map(|c| c > 0)
                .unwrap();
            Ok(has)
        });
        assert!(result.unwrap());
    }
}
