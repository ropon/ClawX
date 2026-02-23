// FileSearch commands (2)
// Handles filesearch:search and filesearch:readContent

use crate::file_search;
use serde_json::{json, Value};

#[tauri::command]
pub async fn filesearch_search(_args: Vec<Value>) -> Result<Value, String> {
    let params = _args.first().cloned().unwrap_or(Value::Null);
    let query = params.get("query")
        .and_then(|v| v.as_str())
        .ok_or("Missing query")?;
    let max_results = params.get("maxResults")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize);

    match file_search::search_files(query, max_results) {
        Ok(results) => Ok(json!({ "success": true, "results": results })),
        Err(e) => Ok(json!({ "success": false, "error": e, "results": [] })),
    }
}

#[tauri::command]
pub async fn filesearch_read_content(_args: Vec<Value>) -> Result<Value, String> {
    let params = _args.first().cloned().unwrap_or(Value::Null);
    let file_path = params.get("filePath")
        .and_then(|v| v.as_str())
        .ok_or("Missing filePath")?;
    let max_size = params.get("maxSizeBytes")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize);

    file_search::read_file_content(file_path, max_size)
}
