// Screenshot capture command — platform-specific screen capture
// macOS: screencapture -i (interactive selection)
// Linux: gnome-screenshot -a / slurp+grim fallback
// Windows: PowerShell screen capture

use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Get the staging directory for screenshots
fn get_screenshot_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?
        .join("screenshots");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create screenshot dir: {}", e))?;
    Ok(dir)
}

/// Generate a unique screenshot filename
fn screenshot_filename() -> String {
    let now = chrono::Utc::now();
    format!("screenshot-{}.png", now.format("%Y%m%d-%H%M%S-%3f"))
}

/// Read file and create base64 data URL preview
fn create_preview(path: &std::path::Path) -> Result<(String, u64), String> {
    let data = std::fs::read(path)
        .map_err(|e| format!("Failed to read screenshot: {}", e))?;
    let file_size = data.len() as u64;
    let b64 = STANDARD.encode(&data);
    let preview = format!("data:image/png;base64,{}", b64);
    Ok((preview, file_size))
}

/// Capture screenshot on macOS using screencapture CLI
#[cfg(target_os = "macos")]
async fn platform_capture(output_path: &std::path::Path) -> Result<bool, String> {
    let output_str = output_path
        .to_str()
        .ok_or("Invalid output path")?;

    // screencapture -i: interactive selection mode
    // screencapture -x: no sound
    let status = tokio::process::Command::new("screencapture")
        .args(["-i", "-x", output_str])
        .status()
        .await
        .map_err(|e| format!("Failed to run screencapture: {}", e))?;

    if !status.success() {
        // User cancelled the selection (exit code != 0)
        return Ok(false);
    }

    // Check if file was actually created (user may cancel with Escape)
    Ok(output_path.exists() && std::fs::metadata(output_path).map(|m| m.len() > 0).unwrap_or(false))
}

/// Capture screenshot on Linux using gnome-screenshot or grim+slurp
#[cfg(target_os = "linux")]
async fn platform_capture(output_path: &std::path::Path) -> Result<bool, String> {
    let output_str = output_path
        .to_str()
        .ok_or("Invalid output path")?;

    // Try gnome-screenshot first
    let gnome_result = tokio::process::Command::new("gnome-screenshot")
        .args(["-a", "-f", output_str])
        .status()
        .await;

    if let Ok(status) = gnome_result {
        if status.success() && output_path.exists() {
            return Ok(true);
        }
    }

    // Fallback: try slurp + grim (Wayland)
    let slurp_output = tokio::process::Command::new("slurp")
        .output()
        .await;

    if let Ok(slurp) = slurp_output {
        if slurp.status.success() {
            let geometry = String::from_utf8_lossy(&slurp.stdout).trim().to_string();
            if !geometry.is_empty() {
                let grim_status = tokio::process::Command::new("grim")
                    .args(["-g", &geometry, output_str])
                    .status()
                    .await
                    .map_err(|e| format!("Failed to run grim: {}", e))?;

                if grim_status.success() && output_path.exists() {
                    return Ok(true);
                }
            }
        }
    }

    // Fallback: try scrot
    let scrot_result = tokio::process::Command::new("scrot")
        .args(["-s", output_str])
        .status()
        .await;

    if let Ok(status) = scrot_result {
        if status.success() && output_path.exists() {
            return Ok(true);
        }
    }

    Err("No screenshot tool available. Install gnome-screenshot, grim+slurp, or scrot.".to_string())
}

/// Capture screenshot on Windows using PowerShell
#[cfg(target_os = "windows")]
async fn platform_capture(output_path: &std::path::Path) -> Result<bool, String> {
    let output_str = output_path
        .to_str()
        .ok_or("Invalid output path")?;

    // Use PowerShell to capture screen via .NET APIs
    let ps_script = format!(
        r#"
        Add-Type -AssemblyName System.Windows.Forms
        Add-Type -AssemblyName System.Drawing
        $screen = [System.Windows.Forms.Screen]::PrimaryScreen
        $bitmap = New-Object System.Drawing.Bitmap($screen.Bounds.Width, $screen.Bounds.Height)
        $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
        $graphics.CopyFromScreen($screen.Bounds.Location, [System.Drawing.Point]::Empty, $screen.Bounds.Size)
        $bitmap.Save('{}', [System.Drawing.Imaging.ImageFormat]::Png)
        $graphics.Dispose()
        $bitmap.Dispose()
        "#,
        output_str.replace('\\', "\\\\").replace('\'', "\\'")
    );

    let status = tokio::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_script])
        .status()
        .await
        .map_err(|e| format!("Failed to run PowerShell screenshot: {}", e))?;

    Ok(status.success() && output_path.exists())
}

#[tauri::command]
pub async fn screenshot_capture(app: AppHandle) -> Result<Value, String> {
    let screenshot_dir = get_screenshot_dir(&app)?;
    let filename = screenshot_filename();
    let output_path = screenshot_dir.join(&filename);

    let captured = platform_capture(&output_path).await?;

    if !captured {
        return Ok(json!({ "cancelled": true }));
    }

    let (preview, file_size) = create_preview(&output_path)?;

    Ok(json!({
        "stagedPath": output_path.to_string_lossy(),
        "preview": preview,
        "fileName": filename,
        "mimeType": "image/png",
        "fileSize": file_size,
    }))
}
