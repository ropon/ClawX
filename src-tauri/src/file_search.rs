// File search engine — mdfind (macOS) / walkdir fallback

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const MAX_FILE_SIZE: usize = 51200; // 50KB

/// Binary extensions that should be rejected for content reading
const BINARY_EXTENSIONS: &[&str] = &[
    ".png", ".jpg", ".jpeg", ".gif", ".webp", ".bmp", ".ico", ".svg",
    ".mp4", ".webm", ".mov", ".avi", ".mkv",
    ".mp3", ".wav", ".ogg", ".flac", ".aac",
    ".pdf", ".doc", ".docx", ".xls", ".xlsx", ".ppt", ".pptx",
    ".zip", ".gz", ".tar", ".7z", ".rar", ".bz2",
    ".exe", ".dll", ".so", ".dylib", ".bin",
    ".woff", ".woff2", ".ttf", ".otf", ".eot",
    ".sqlite", ".db",
];

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
}

/// Check if a file path is safe (within home directory)
fn is_path_safe(file_path: &str) -> bool {
    let home = home_dir();
    match fs::canonicalize(file_path) {
        Ok(resolved) => resolved.starts_with(&home),
        Err(_) => {
            // If we can't canonicalize, check the normalized path
            let path = PathBuf::from(file_path);
            path.starts_with(&home)
        }
    }
}

/// Check if a file extension indicates binary content
fn is_binary_file(file_path: &str) -> bool {
    let ext = Path::new(file_path)
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
        .unwrap_or_default();
    BINARY_EXTENSIONS.contains(&ext.as_str())
}

/// Check for null bytes in the first 8KB (binary content detection)
fn has_binary_content(data: &[u8]) -> bool {
    let check_len = data.len().min(8192);
    data[..check_len].contains(&0)
}

/// Search files using system tools
pub fn search_files(query: &str, max_results: Option<usize>) -> Result<Vec<Value>, String> {
    let max = max_results.unwrap_or(20);

    if query.is_empty() || query.len() < 2 {
        return Ok(Vec::new());
    }

    let home = home_dir();
    let home_str = home.to_string_lossy().to_string();

    let output = if cfg!(target_os = "macos") {
        // macOS: use mdfind (Spotlight index)
        Command::new("mdfind")
            .args(["-name", query, "-onlyin", &home_str, "-limit", &max.to_string()])
            .output()
    } else {
        // Linux/other: use find
        Command::new("find")
            .args([&home_str, "-maxdepth", "5", "-iname", &format!("*{}*", query), "-type", "f"])
            .output()
    };

    let output = match output {
        Ok(o) => o,
        Err(_) => return Ok(Vec::new()),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_search_output(&stdout, max)
}

fn parse_search_output(stdout: &str, max_results: usize) -> Result<Vec<Value>, String> {
    let mut results = Vec::new();

    for line in stdout.lines() {
        if results.len() >= max_results {
            break;
        }

        let file_path = line.trim();
        if file_path.is_empty() || !is_path_safe(file_path) {
            continue;
        }

        let path = Path::new(file_path);
        let metadata = match fs::metadata(path) {
            Ok(m) if m.is_file() => m,
            _ => continue,
        };

        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let file_type = path
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
            .unwrap_or_default();

        let modified_at = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as f64)
            .unwrap_or(0.0);

        results.push(json!({
            "filePath": file_path,
            "fileName": file_name,
            "fileSize": metadata.len(),
            "modifiedAt": modified_at,
            "fileType": file_type,
        }));
    }

    Ok(results)
}

/// Read file content safely with size and binary checks
pub fn read_file_content(file_path: &str, max_size_bytes: Option<usize>) -> Result<Value, String> {
    let max_size = max_size_bytes.unwrap_or(MAX_FILE_SIZE);

    // Security: path must be within home directory
    if !is_path_safe(file_path) {
        return Ok(json!({
            "success": false,
            "error": "Path is outside home directory"
        }));
    }

    // Check for known binary extensions
    if is_binary_file(file_path) {
        return Ok(json!({
            "success": false,
            "error": "Binary files are not supported"
        }));
    }

    let path = Path::new(file_path);
    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) => {
            return Ok(json!({
                "success": false,
                "error": format!("Failed to read file: {}", e)
            }));
        }
    };

    if !metadata.is_file() {
        return Ok(json!({
            "success": false,
            "error": "Not a regular file"
        }));
    }

    let buffer = match fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            return Ok(json!({
                "success": false,
                "error": format!("Failed to read file: {}", e)
            }));
        }
    };

    // Check for binary content
    if has_binary_content(&buffer) {
        return Ok(json!({
            "success": false,
            "error": "File appears to contain binary content"
        }));
    }

    let truncated = buffer.len() > max_size;
    let content_bytes = &buffer[..buffer.len().min(max_size)];
    let mut content = String::from_utf8_lossy(content_bytes).to_string();

    if truncated {
        content.push_str("\n\n[... truncated]");
    }

    Ok(json!({
        "success": true,
        "content": content,
        "truncated": truncated,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_binary_file() {
        assert!(is_binary_file("/path/to/file.png"));
        assert!(is_binary_file("/path/to/file.PDF"));
        assert!(!is_binary_file("/path/to/file.txt"));
        assert!(!is_binary_file("/path/to/file.rs"));
    }

    #[test]
    fn test_has_binary_content() {
        assert!(has_binary_content(&[0x48, 0x65, 0x00, 0x6c]));
        assert!(!has_binary_content(b"Hello World"));
    }

    #[test]
    fn test_parse_search_output() {
        let results = parse_search_output("", 20).unwrap();
        assert!(results.is_empty());
    }
}
