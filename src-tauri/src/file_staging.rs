// File staging — copy files to outbound directory + image preview generation
// Ported from ipc-handlers.ts registerFileHandlers()

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn outbound_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".openclaw")
        .join("media")
        .join("outbound")
}

/// Stage files from disk paths to outbound directory
pub fn stage_files(file_paths: Vec<String>) -> Result<Vec<Value>, String> {
    let out_dir = outbound_dir();
    fs::create_dir_all(&out_dir)
        .map_err(|e| format!("Failed to create outbound dir: {}", e))?;

    let mut results = Vec::new();
    for file_path in &file_paths {
        let path = Path::new(file_path);
        let id = uuid::Uuid::new_v4().to_string();
        let ext = path
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default();
        let staged_path = out_dir.join(format!("{}{}", id, ext));

        fs::copy(path, &staged_path)
            .map_err(|e| format!("Failed to copy file: {}", e))?;

        let metadata = fs::metadata(&staged_path)
            .map_err(|e| format!("Failed to stat staged file: {}", e))?;
        let mime_type = get_mime_type(&ext);
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let preview = if mime_type.starts_with("image/") {
            generate_image_preview(&staged_path, &mime_type)
        } else {
            None
        };

        results.push(json!({
            "id": id,
            "fileName": file_name,
            "mimeType": mime_type,
            "fileSize": metadata.len(),
            "stagedPath": staged_path.to_string_lossy(),
            "preview": preview,
        }));
    }

    Ok(results)
}

/// Stage a file from base64 buffer
pub fn stage_buffer(base64_data: &str, file_name: &str, mime_type: &str) -> Result<Value, String> {
    let out_dir = outbound_dir();
    fs::create_dir_all(&out_dir)
        .map_err(|e| format!("Failed to create outbound dir: {}", e))?;

    let id = uuid::Uuid::new_v4().to_string();
    let ext = Path::new(file_name)
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .or_else(|| mime_to_ext(mime_type).map(String::from))
        .unwrap_or_default();
    let staged_path = out_dir.join(format!("{}{}", id, ext));

    let buffer = BASE64
        .decode(base64_data)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;
    fs::write(&staged_path, &buffer)
        .map_err(|e| format!("Failed to write staged file: {}", e))?;

    let resolved_mime = if mime_type.is_empty() {
        get_mime_type(&ext)
    } else {
        mime_type.to_string()
    };

    let preview = if resolved_mime.starts_with("image/") {
        generate_image_preview(&staged_path, &resolved_mime)
    } else {
        None
    };

    Ok(json!({
        "id": id,
        "fileName": file_name,
        "mimeType": resolved_mime,
        "fileSize": buffer.len(),
        "stagedPath": staged_path.to_string_lossy(),
        "preview": preview,
    }))
}

/// Generate thumbnails for already staged files
pub fn get_thumbnails(paths: Vec<(String, String)>) -> Result<Value, String> {
    let mut results = serde_json::Map::new();

    for (file_path, mime_type) in &paths {
        let path = Path::new(file_path);
        if !path.exists() {
            results.insert(
                file_path.clone(),
                json!({ "preview": null, "fileSize": 0 }),
            );
            continue;
        }

        let file_size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        let preview = if mime_type.starts_with("image/") {
            generate_image_preview(path, mime_type)
        } else {
            None
        };

        results.insert(
            file_path.clone(),
            json!({ "preview": preview, "fileSize": file_size }),
        );
    }

    Ok(Value::Object(results))
}

/// Generate an image preview as a data URL (resized to max 512px dimension)
fn generate_image_preview(file_path: &Path, mime_type: &str) -> Option<String> {
    let img = image::open(file_path).ok()?;
    let (w, h) = (img.width(), img.height());
    let max_dim: u32 = 512;

    let resized = if w > max_dim || h > max_dim {
        if w >= h {
            img.resize(max_dim, max_dim, image::imageops::FilterType::Lanczos3)
        } else {
            img.resize(max_dim, max_dim, image::imageops::FilterType::Lanczos3)
        }
    } else {
        // Small image — use original bytes
        let buf = fs::read(file_path).ok()?;
        return Some(format!("data:{};base64,{}", mime_type, BASE64.encode(&buf)));
    };

    // Encode resized as PNG
    let mut png_buf = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_buf);
    resized
        .write_to(&mut cursor, image::ImageFormat::Png)
        .ok()?;

    Some(format!("data:image/png;base64,{}", BASE64.encode(&png_buf)))
}

fn get_mime_type(ext: &str) -> String {
    match ext.to_lowercase().as_str() {
        ".png" => "image/png",
        ".jpg" | ".jpeg" => "image/jpeg",
        ".gif" => "image/gif",
        ".webp" => "image/webp",
        ".svg" => "image/svg+xml",
        ".bmp" => "image/bmp",
        ".ico" => "image/x-icon",
        ".mp4" => "video/mp4",
        ".webm" => "video/webm",
        ".mov" => "video/quicktime",
        ".avi" => "video/x-msvideo",
        ".mkv" => "video/x-matroska",
        ".mp3" => "audio/mpeg",
        ".wav" => "audio/wav",
        ".ogg" => "audio/ogg",
        ".flac" => "audio/flac",
        ".pdf" => "application/pdf",
        ".zip" => "application/zip",
        ".gz" => "application/gzip",
        ".tar" => "application/x-tar",
        ".7z" => "application/x-7z-compressed",
        ".rar" => "application/vnd.rar",
        ".json" => "application/json",
        ".xml" => "application/xml",
        ".csv" => "text/csv",
        ".txt" => "text/plain",
        ".md" => "text/markdown",
        ".html" => "text/html",
        ".css" => "text/css",
        ".js" => "text/javascript",
        ".ts" => "text/typescript",
        ".py" => "text/x-python",
        ".doc" => "application/msword",
        ".docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        ".xls" => "application/vnd.ms-excel",
        ".xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        ".ppt" => "application/vnd.ms-powerpoint",
        ".pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        _ => "application/octet-stream",
    }
    .to_string()
}

fn mime_to_ext(mime_type: &str) -> Option<&'static str> {
    match mime_type {
        "image/png" => Some(".png"),
        "image/jpeg" => Some(".jpg"),
        "image/gif" => Some(".gif"),
        "image/webp" => Some(".webp"),
        "image/svg+xml" => Some(".svg"),
        "video/mp4" => Some(".mp4"),
        "audio/mpeg" => Some(".mp3"),
        "application/pdf" => Some(".pdf"),
        "application/json" => Some(".json"),
        "text/plain" => Some(".txt"),
        _ => None,
    }
}
