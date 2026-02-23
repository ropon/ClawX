// UV commands (2)
// Handles uv:check and uv:install-all

use serde_json::{json, Value};
use std::process::Command;

/// Check if uv is installed (in PATH or bundled)
fn check_uv_installed() -> bool {
    // Check system PATH
    let in_path = Command::new("which")
        .arg("uv")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if in_path {
        return true;
    }

    // Check common bundled locations (Tauri resources)
    // In Tauri mode, uv should already be in PATH or user-installed
    false
}

/// Get the uv binary path
fn get_uv_bin() -> String {
    // Try system PATH first
    if Command::new("which")
        .arg("uv")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return "uv".to_string();
    }

    // Fallback
    "uv".to_string()
}

#[tauri::command]
pub async fn uv_check(_args: Vec<Value>) -> Result<Value, String> {
    Ok(json!(check_uv_installed()))
}

#[tauri::command]
pub async fn uv_install_all(_args: Vec<Value>) -> Result<Value, String> {
    // Step 1: Check if uv is installed
    if !check_uv_installed() {
        // Try to install uv via curl on macOS/Linux
        if cfg!(unix) {
            let install_result = Command::new("sh")
                .args(["-c", "curl -LsSf https://astral.sh/uv/install.sh | sh"])
                .output();

            match install_result {
                Ok(output) if output.status.success() => {
                    println!("[ClawX] uv installed successfully");
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Ok(json!({
                        "success": false,
                        "error": format!("uv installation failed: {}", stderr),
                    }));
                }
                Err(e) => {
                    return Ok(json!({
                        "success": false,
                        "error": format!("Failed to run uv installer: {}", e),
                    }));
                }
            }
        } else {
            return Ok(json!({
                "success": false,
                "error": "uv not found and automatic installation not supported on this platform",
            }));
        }
    }

    // Step 2: Install Python 3.12 via uv
    let uv_bin = get_uv_bin();
    let python_result = Command::new(&uv_bin)
        .args(["python", "install", "3.12"])
        .output();

    match python_result {
        Ok(output) if output.status.success() => {
            Ok(json!({ "success": true }))
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Ok(json!({
                "success": false,
                "error": format!("Python installation failed: {}", stderr),
            }))
        }
        Err(e) => {
            Ok(json!({
                "success": false,
                "error": format!("Failed to run uv: {}", e),
            }))
        }
    }
}
