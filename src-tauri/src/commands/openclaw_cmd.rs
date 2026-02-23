// OpenClaw commands (7)
// Handles openclaw:status/isReady/getDir/getConfigDir/getSkillsDir/getCliCommand/installCliMac

use crate::openclaw_paths;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

#[tauri::command]
pub async fn openclaw_status(_args: Vec<Value>) -> Result<Value, String> {
    Ok(openclaw_paths::get_openclaw_status())
}

#[tauri::command]
pub async fn openclaw_is_ready(_args: Vec<Value>) -> Result<Value, String> {
    Ok(json!(openclaw_paths::is_openclaw_present()))
}

#[tauri::command]
pub async fn openclaw_get_dir(_args: Vec<Value>) -> Result<Value, String> {
    Ok(json!(openclaw_paths::get_openclaw_dir()))
}

#[tauri::command]
pub async fn openclaw_get_config_dir(_args: Vec<Value>) -> Result<Value, String> {
    Ok(json!(openclaw_paths::get_openclaw_config_dir()))
}

#[tauri::command]
pub async fn openclaw_get_skills_dir(_args: Vec<Value>) -> Result<Value, String> {
    let dir = openclaw_paths::get_openclaw_skills_dir();
    // Ensure directory exists
    let path = PathBuf::from(&dir);
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    Ok(json!(dir))
}

#[tauri::command]
pub async fn openclaw_get_cli_command(_args: Vec<Value>) -> Result<Value, String> {
    let status = openclaw_paths::get_openclaw_status();
    let pkg_exists = status.get("packageExists").and_then(|v| v.as_bool()).unwrap_or(false);
    let entry_path = status.get("entryPath").and_then(|v| v.as_str()).unwrap_or("");

    if !pkg_exists {
        let dir = status.get("dir").and_then(|v| v.as_str()).unwrap_or("unknown");
        return Ok(json!({
            "success": false,
            "error": format!("OpenClaw package not found at: {}", dir),
        }));
    }

    if !PathBuf::from(entry_path).exists() {
        return Ok(json!({
            "success": false,
            "error": format!("OpenClaw entry script not found at: {}", entry_path),
        }));
    }

    // Build CLI command
    let home = dirs::home_dir().unwrap_or_default();

    // Check if ~/.local/bin/openclaw exists (installed wrapper)
    let local_bin = home.join(".local").join("bin").join("openclaw");
    if local_bin.exists() {
        return Ok(json!({
            "success": true,
            "command": format!("\"{}\"", local_bin.to_string_lossy()),
        }));
    }

    // Fallback: use node + entry path
    Ok(json!({
        "success": true,
        "command": format!("node \"{}\"", entry_path),
    }))
}

#[tauri::command]
pub async fn openclaw_install_cli_mac(_args: Vec<Value>) -> Result<Value, String> {
    if !cfg!(target_os = "macos") {
        return Ok(json!({
            "success": false,
            "error": "Install is only supported on macOS.",
        }));
    }

    let entry_path = openclaw_paths::get_openclaw_entry_path();
    if !PathBuf::from(&entry_path).exists() {
        return Ok(json!({
            "success": false,
            "error": format!("OpenClaw entry not found at: {}", entry_path),
        }));
    }

    let home = dirs::home_dir().ok_or("Cannot determine home directory")?;
    let target_dir = home.join(".local").join("bin");
    let target = target_dir.join("openclaw");

    // Write shell wrapper script
    let script = format!(
        "#!/bin/sh\nnode \"{}\" \"$@\"\n",
        entry_path.replace('\\', "\\\\").replace('"', "\\\""),
    );

    fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;
    fs::write(&target, &script)
        .map_err(|e| format!("Failed to write script: {}", e))?;

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&target)
            .map_err(|e| format!("Failed to read permissions: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&target, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }

    Ok(json!({
        "success": true,
        "path": target.to_string_lossy(),
    }))
}
