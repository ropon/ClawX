// Tauri commands for chat:* channels
// 1 command: chat:sendWithMedia
// Ported from ipc-handlers.ts:501-582

use crate::AppState;
use serde_json::{json, Value};
use std::fs;
use tauri::State;

const VISION_MIME_TYPES: &[&str] = &["image/png", "image/jpeg", "image/bmp", "image/webp"];

/// chat:sendWithMedia
/// _args: [params] where params = { sessionKey, message, deliver?, idempotencyKey, media? }
#[tauri::command]
pub async fn chat_send_with_media(
    _args: Vec<Value>,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let params = _args.first().ok_or("Missing params")?;

    let session_key = params
        .get("sessionKey")
        .and_then(|v| v.as_str())
        .ok_or("Missing sessionKey")?;
    let mut message = params
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let deliver = params
        .get("deliver")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let idempotency_key = params
        .get("idempotencyKey")
        .and_then(|v| v.as_str())
        .ok_or("Missing idempotencyKey")?;

    let media = params
        .get("media")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut image_attachments: Vec<Value> = Vec::new();
    let mut file_references: Vec<String> = Vec::new();

    for m in &media {
        let file_path = m.get("filePath").and_then(|v| v.as_str()).unwrap_or("");
        let mime_type = m.get("mimeType").and_then(|v| v.as_str()).unwrap_or("");
        let file_name = m.get("fileName").and_then(|v| v.as_str()).unwrap_or("");

        // Always add file path reference
        file_references.push(format!(
            "[media attached: {} ({}) | {}]",
            file_path, mime_type, file_path
        ));

        // Vision types: read and encode as base64
        if VISION_MIME_TYPES.contains(&mime_type) {
            match fs::read(file_path) {
                Ok(bytes) => {
                    use base64::Engine;
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                    image_attachments.push(json!({
                        "content": b64,
                        "mimeType": mime_type,
                        "fileName": file_name,
                    }));
                }
                Err(e) => {
                    eprintln!("[chat:sendWithMedia] Failed to read file {}: {}", file_path, e);
                }
            }
        }
    }

    // Append file references to message
    if !file_references.is_empty() {
        let refs = file_references.join("\n");
        if message.is_empty() {
            message = refs;
        } else {
            message = format!("{}\n\n{}", message, refs);
        }
    }

    let mut rpc_params = json!({
        "sessionKey": session_key,
        "message": message,
        "deliver": deliver,
        "idempotencyKey": idempotency_key,
    });

    if !image_attachments.is_empty() {
        rpc_params["attachments"] = json!(image_attachments);
    }

    // Longer timeout when images are present
    let timeout_ms = if !image_attachments.is_empty() {
        120000
    } else {
        30000
    };

    match state
        .gateway_client
        .rpc("chat.send", Some(rpc_params), timeout_ms)
        .await
    {
        Ok(result) => Ok(json!({ "success": true, "result": result })),
        Err(e) => Ok(json!({ "success": false, "error": e })),
    }
}
