// OpenClaw path resolution + status checking

use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

fn openclaw_config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".openclaw")
}

/// Get OpenClaw config directory (~/.openclaw)
pub fn get_openclaw_config_dir() -> String {
    openclaw_config_dir().to_string_lossy().to_string()
}

/// Get OpenClaw skills directory (~/.openclaw/skills)
pub fn get_openclaw_skills_dir() -> String {
    openclaw_config_dir()
        .join("skills")
        .to_string_lossy()
        .to_string()
}

/// Get OpenClaw package directory
/// Searches multiple locations in priority order
pub fn get_openclaw_dir() -> String {
    // 1. Dev mode: <project_root>/build/openclaw (set at compile time)
    let manifest_dir = env!("CARGO_MANIFEST_DIR"); // src-tauri/
    let project_root = PathBuf::from(manifest_dir).parent().map(|p| p.to_path_buf());
    if let Some(ref root) = project_root {
        let build_dir = root.join("build").join("openclaw");
        if build_dir.join("package.json").exists() {
            return build_dir.to_string_lossy().to_string();
        }
    }

    // 2. Packaged mode: adjacent to executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            // macOS: ClawX.app/Contents/MacOS/ → Resources/openclaw
            let resources = exe_dir.join("../Resources/openclaw");
            if resources.join("package.json").exists() {
                return resources.to_string_lossy().to_string();
            }
            // Flat layout: next to binary
            let adjacent = exe_dir.join("openclaw");
            if adjacent.join("package.json").exists() {
                return adjacent.to_string_lossy().to_string();
            }
        }
    }

    // 3. Global install: ~/.openclaw/node_modules/openclaw
    let global = openclaw_config_dir()
        .join("node_modules")
        .join("openclaw");
    if global.join("package.json").exists() {
        return global.to_string_lossy().to_string();
    }

    // 4. npm global locations
    if let Some(home) = dirs::home_dir() {
        let npm_global = home.join(".npm-global").join("lib").join("node_modules").join("openclaw");
        if npm_global.join("package.json").exists() {
            return npm_global.to_string_lossy().to_string();
        }
    }

    // Fallback: return the build path for a helpful error message
    if let Some(ref root) = project_root {
        return root.join("build").join("openclaw").to_string_lossy().to_string();
    }
    global.to_string_lossy().to_string()
}

/// Get OpenClaw entry script path
pub fn get_openclaw_entry_path() -> String {
    let dir = get_openclaw_dir();
    PathBuf::from(&dir)
        .join("openclaw.mjs")
        .to_string_lossy()
        .to_string()
}

/// Check if OpenClaw package exists
pub fn is_openclaw_present() -> bool {
    let dir = get_openclaw_dir();
    let pkg_json = PathBuf::from(&dir).join("package.json");
    PathBuf::from(&dir).exists() && pkg_json.exists()
}

/// Get OpenClaw status for environment check
pub fn get_openclaw_status() -> Value {
    let dir = get_openclaw_dir();
    let dir_path = PathBuf::from(&dir);
    let pkg_exists = is_openclaw_present();
    let entry_path = get_openclaw_entry_path();

    // Check if built (has dist folder)
    let is_built = dir_path.join("dist").exists();

    // Read version from package.json
    let version = dir_path.join("package.json")
        .exists()
        .then(|| {
            fs::read_to_string(dir_path.join("package.json"))
                .ok()
                .and_then(|content| serde_json::from_str::<Value>(&content).ok())
                .and_then(|pkg| pkg.get("version").and_then(|v| v.as_str()).map(String::from))
        })
        .flatten();

    json!({
        "packageExists": pkg_exists,
        "isBuilt": is_built,
        "entryPath": entry_path,
        "dir": dir,
        "version": version,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_dir() {
        let dir = get_openclaw_config_dir();
        assert!(dir.contains(".openclaw"));
    }

    #[test]
    fn test_skills_dir() {
        let dir = get_openclaw_skills_dir();
        assert!(dir.contains(".openclaw/skills"));
    }

    #[test]
    fn test_status_structure() {
        let status = get_openclaw_status();
        assert!(status.get("packageExists").is_some());
        assert!(status.get("entryPath").is_some());
        assert!(status.get("dir").is_some());
    }
}
