// File staging commands (3)
// Handles file:stage/stageBuffer, media:getThumbnails

use crate::file_staging;
use serde_json::{json, Value};

#[tauri::command]
pub async fn file_stage(_args: Vec<Value>) -> Result<Value, String> {
    let file_paths: Vec<String> = _args.first()
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    if file_paths.is_empty() {
        return Ok(json!([]));
    }

    let results = file_staging::stage_files(file_paths)?;
    Ok(json!(results))
}

#[tauri::command]
pub async fn file_stage_buffer(_args: Vec<Value>) -> Result<Value, String> {
    let payload = _args.first().cloned().unwrap_or(Value::Null);
    let base64 = payload.get("base64")
        .and_then(|v| v.as_str())
        .ok_or("Missing base64 data")?;
    let file_name = payload.get("fileName")
        .and_then(|v| v.as_str())
        .ok_or("Missing fileName")?;
    let mime_type = payload.get("mimeType")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    file_staging::stage_buffer(base64, file_name, mime_type)
}

#[tauri::command]
pub async fn media_get_thumbnails(_args: Vec<Value>) -> Result<Value, String> {
    let paths_arg = _args.first()
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let paths: Vec<(String, String)> = paths_arg
        .iter()
        .filter_map(|item| {
            let file_path = item.get("filePath")?.as_str()?.to_string();
            let mime_type = item.get("mimeType")?.as_str()?.to_string();
            Some((file_path, mime_type))
        })
        .collect();

    file_staging::get_thumbnails(paths)
}
