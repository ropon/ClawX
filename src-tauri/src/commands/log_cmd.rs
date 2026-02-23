// Log commands (5)
// Handles log:readFile/getFilePath/getDir/listFiles/getRecent

use crate::AppState;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

fn logs_dir(data_dir: &std::path::Path) -> PathBuf {
    data_dir.join("logs")
}

fn current_log_file(data_dir: &std::path::Path) -> Option<PathBuf> {
    let dir = logs_dir(data_dir);
    if !dir.exists() {
        return None;
    }
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let path = dir.join(format!("clawx-{}.log", today));
    if path.exists() { Some(path) } else { None }
}

#[tauri::command]
pub async fn log_read_file(state: tauri::State<'_, AppState>, _args: Vec<Value>) -> Result<Value, String> {
    let tail_lines = _args.first()
        .and_then(|v| v.as_u64())
        .unwrap_or(200) as usize;

    let log_file = match current_log_file(&state.data_dir) {
        Some(p) => p,
        None => return Ok(json!("(No log file found)")),
    };

    let content = fs::read_to_string(&log_file)
        .map_err(|e| format!("Failed to read log file: {}", e))?;

    let lines: Vec<&str> = content.lines().collect();
    if lines.len() <= tail_lines {
        Ok(json!(content))
    } else {
        Ok(json!(lines[lines.len() - tail_lines..].join("\n")))
    }
}

#[tauri::command]
pub async fn log_get_file_path(state: tauri::State<'_, AppState>, _args: Vec<Value>) -> Result<Value, String> {
    Ok(json!(current_log_file(&state.data_dir).map(|p| p.to_string_lossy().to_string())))
}

#[tauri::command]
pub async fn log_get_dir(state: tauri::State<'_, AppState>, _args: Vec<Value>) -> Result<Value, String> {
    Ok(json!(logs_dir(&state.data_dir).to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn log_list_files(state: tauri::State<'_, AppState>, _args: Vec<Value>) -> Result<Value, String> {
    let dir = logs_dir(&state.data_dir);
    if !dir.exists() {
        return Ok(json!([]));
    }

    let mut files = Vec::new();
    let entries = fs::read_dir(&dir)
        .map_err(|e| format!("Failed to read log dir: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("log") {
            continue;
        }
        let metadata = match fs::metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let modified = metadata.modified()
            .ok()
            .and_then(|t| {
                let dt: chrono::DateTime<chrono::Utc> = t.into();
                Some(dt.to_rfc3339())
            })
            .unwrap_or_default();

        files.push(json!({
            "name": name,
            "path": path.to_string_lossy(),
            "size": metadata.len(),
            "modified": modified,
        }));
    }

    // Sort by modified descending
    files.sort_by(|a, b| {
        let am = a.get("modified").and_then(|v| v.as_str()).unwrap_or("");
        let bm = b.get("modified").and_then(|v| v.as_str()).unwrap_or("");
        bm.cmp(am)
    });

    Ok(json!(files))
}

/// Get recent log entries — reads the last N lines from today's log file.
/// Returns recent log entries.
#[tauri::command]
pub async fn log_get_recent(state: tauri::State<'_, AppState>, _args: Vec<Value>) -> Result<Value, String> {
    let count = _args.first()
        .and_then(|v| v.as_u64())
        .unwrap_or(100) as usize;

    let log_file = match current_log_file(&state.data_dir) {
        Some(p) => p,
        None => return Ok(json!([])),
    };

    let content = fs::read_to_string(&log_file)
        .map_err(|e| format!("Failed to read log file: {}", e))?;

    let lines: Vec<&str> = content.lines().collect();
    let start = if lines.len() > count { lines.len() - count } else { 0 };
    let recent: Vec<Value> = lines[start..].iter().map(|line| {
        // Try to parse as JSON log entry, fallback to raw string
        serde_json::from_str::<Value>(line).unwrap_or_else(|_| json!({
            "message": line,
            "level": "info",
            "timestamp": ""
        }))
    }).collect();

    Ok(json!(recent))
}
