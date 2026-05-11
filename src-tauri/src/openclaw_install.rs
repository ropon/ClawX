// First-run extraction of the bundled openclaw.tar.gz into ~/.openclaw/runtime/.
// Tauri ships a single tar.gz + version file as resources; the Rust side
// decompresses on demand so only 2 files (vs ~10k) hit macOS codesigning.

use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub fn runtime_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".openclaw")
        .join("runtime")
}

pub fn runtime_openclaw_dir() -> PathBuf {
    runtime_dir().join("openclaw")
}

fn marker_path() -> PathBuf {
    runtime_dir().join(".installed-version")
}

fn dev_build_dir() -> Option<PathBuf> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent()
        .map(|p| p.join("build").join("openclaw"))
}

fn bundled_version(app: &AppHandle) -> Result<String, String> {
    let p = app
        .path()
        .resource_dir()
        .map_err(|e| format!("resource_dir: {e}"))?
        .join("openclaw.version");
    fs::read_to_string(&p)
        .map(|s| s.trim().to_string())
        .map_err(|e| format!("read {}: {e}", p.display()))
}

fn installed_version() -> Option<String> {
    fs::read_to_string(marker_path())
        .ok()
        .map(|s| s.trim().to_string())
}

pub fn ensure_installed(app: &AppHandle) -> Result<PathBuf, String> {
    // Dev short-circuit: when the project's build/openclaw exists, use it directly.
    // This covers `tauri dev` (resource_dir would point at target/ and miss the archive).
    if let Some(dev) = dev_build_dir() {
        if dev.join("package.json").exists() {
            return Ok(dev);
        }
    }

    let bundled = bundled_version(app)?;
    if installed_version().as_deref() == Some(bundled.as_str())
        && runtime_openclaw_dir().join("package.json").exists()
    {
        return Ok(runtime_openclaw_dir());
    }

    let archive = app
        .path()
        .resource_dir()
        .map_err(|e| format!("resource_dir: {e}"))?
        .join("openclaw.tar.gz");

    if runtime_dir().exists() {
        fs::remove_dir_all(runtime_dir()).map_err(|e| format!("clean runtime: {e}"))?;
    }
    fs::create_dir_all(runtime_dir()).map_err(|e| format!("mkdir runtime: {e}"))?;

    let f = fs::File::open(&archive).map_err(|e| format!("open {}: {e}", archive.display()))?;
    let gz = flate2::read::GzDecoder::new(f);
    let mut ar = tar::Archive::new(gz);
    ar.unpack(runtime_dir()).map_err(|e| format!("unpack: {e}"))?;

    fs::write(marker_path(), &bundled).map_err(|e| format!("write marker: {e}"))?;
    Ok(runtime_openclaw_dir())
}
