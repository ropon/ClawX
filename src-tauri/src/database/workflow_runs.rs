// Workflow Run CRUD operations on SQLite

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRun {
    pub id: String,
    pub workflow_id: String,
    pub status: String,
    pub trigger_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_input: Option<String>,
    pub started_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
    pub steps: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

fn row_to_run(row: &rusqlite::Row) -> Result<WorkflowRun, rusqlite::Error> {
    let steps_json: String = row.get("steps_json")?;
    let steps: serde_json::Value =
        serde_json::from_str(&steps_json).unwrap_or(serde_json::Value::Array(vec![]));

    Ok(WorkflowRun {
        id: row.get("id")?,
        workflow_id: row.get("workflow_id")?,
        status: row.get("status")?,
        trigger_type: row.get("trigger_type")?,
        trigger_input: row.get("trigger_input")?,
        started_at: row.get("started_at")?,
        completed_at: row.get("completed_at")?,
        steps,
        final_output: row.get("final_output")?,
        error: row.get("error")?,
    })
}

/// Get a single run by ID
pub fn get_run(conn: &Connection, id: &str) -> Result<Option<WorkflowRun>, String> {
    let mut stmt = conn
        .prepare("SELECT * FROM workflow_runs WHERE id = ?1")
        .map_err(|e| e.to_string())?;

    let run = stmt
        .query_row([id], row_to_run)
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            _ => Err(e.to_string()),
        })?;

    Ok(run)
}

/// Get runs for a specific workflow (most recent first)
pub fn get_runs_for_workflow(
    conn: &Connection,
    workflow_id: &str,
    limit: u32,
) -> Result<Vec<WorkflowRun>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT * FROM workflow_runs WHERE workflow_id = ?1 ORDER BY started_at DESC LIMIT ?2",
        )
        .map_err(|e| e.to_string())?;

    let runs = stmt
        .query_map(rusqlite::params![workflow_id, limit], row_to_run)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(runs)
}

/// Delete a single run
pub fn delete_run(conn: &Connection, id: &str) -> Result<bool, String> {
    let changes = conn
        .execute("DELETE FROM workflow_runs WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(changes > 0)
}

/// Clear all runs for a workflow
pub fn clear_runs_for_workflow(conn: &Connection, workflow_id: &str) -> Result<u32, String> {
    let changes = conn
        .execute(
            "DELETE FROM workflow_runs WHERE workflow_id = ?1",
            [workflow_id],
        )
        .map_err(|e| e.to_string())?;
    Ok(changes as u32)
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

    fn insert_test_run(conn: &Connection, id: &str, workflow_id: &str, started_at: i64) {
        conn.execute(
            "INSERT INTO workflow_runs (id, workflow_id, status, trigger_type, started_at, steps_json)
             VALUES (?1, ?2, 'completed', 'manual', ?3, '[]')",
            rusqlite::params![id, workflow_id, started_at],
        )
        .unwrap();
    }

    #[test]
    fn test_get_run() {
        let tmp = TempDir::new().unwrap();
        let mgr = test_mgr(&tmp);
        mgr.with_runs_conn(|conn| {
            insert_test_run(conn, "run-1", "wf-1", 1000);
            let run = get_run(conn, "run-1")?.unwrap();
            assert_eq!(run.id, "run-1");
            assert_eq!(run.workflow_id, "wf-1");
            assert_eq!(run.status, "completed");
            assert_eq!(run.started_at, 1000);

            let missing = get_run(conn, "nonexistent")?;
            assert!(missing.is_none());
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_get_runs_for_workflow() {
        let tmp = TempDir::new().unwrap();
        let mgr = test_mgr(&tmp);
        mgr.with_runs_conn(|conn| {
            insert_test_run(conn, "run-1", "wf-1", 1000);
            insert_test_run(conn, "run-2", "wf-1", 2000);
            insert_test_run(conn, "run-3", "wf-2", 3000);

            let runs = get_runs_for_workflow(conn, "wf-1", 50)?;
            assert_eq!(runs.len(), 2);
            assert_eq!(runs[0].id, "run-2"); // Most recent first
            assert_eq!(runs[1].id, "run-1");

            let limited = get_runs_for_workflow(conn, "wf-1", 1)?;
            assert_eq!(limited.len(), 1);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_delete_run() {
        let tmp = TempDir::new().unwrap();
        let mgr = test_mgr(&tmp);
        mgr.with_runs_conn(|conn| {
            insert_test_run(conn, "run-1", "wf-1", 1000);
            assert!(delete_run(conn, "run-1")?);
            assert!(!delete_run(conn, "run-1")?); // Already deleted
            assert!(get_run(conn, "run-1")?.is_none());
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_clear_runs() {
        let tmp = TempDir::new().unwrap();
        let mgr = test_mgr(&tmp);
        mgr.with_runs_conn(|conn| {
            insert_test_run(conn, "run-1", "wf-1", 1000);
            insert_test_run(conn, "run-2", "wf-1", 2000);
            insert_test_run(conn, "run-3", "wf-2", 3000);

            let cleared = clear_runs_for_workflow(conn, "wf-1")?;
            assert_eq!(cleared, 2);

            let runs = get_runs_for_workflow(conn, "wf-1", 50)?;
            assert!(runs.is_empty());

            // wf-2 runs unaffected
            let runs2 = get_runs_for_workflow(conn, "wf-2", 50)?;
            assert_eq!(runs2.len(), 1);
            Ok(())
        })
        .unwrap();
    }
}
