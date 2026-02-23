// Tauri commands for clawhub:* channels
// 5 commands for ClawHub skill management

use crate::clawhub;
use serde_json::Value;

/// clawhub:search — Search for skills in ClawHub
/// _args: [query, limit?]
#[tauri::command]
pub async fn clawhub_search(_args: Vec<Value>) -> Result<Value, String> {
    let query = _args
        .first()
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let limit = _args.get(1).and_then(|v| v.as_u64()).map(|l| l as usize);

    clawhub::clawhub_search(query, limit).await
}

/// clawhub:install — Install a skill
/// _args: [slug, version?, force?]
#[tauri::command]
pub async fn clawhub_install(_args: Vec<Value>) -> Result<Value, String> {
    let slug = _args
        .first()
        .and_then(|v| v.as_str())
        .ok_or("Missing slug")?;
    let version = _args.get(1).and_then(|v| v.as_str());
    let force = _args
        .get(2)
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    clawhub::clawhub_install(slug, version, force).await
}

/// clawhub:uninstall — Uninstall a skill
/// _args: [slug]
#[tauri::command]
pub async fn clawhub_uninstall(_args: Vec<Value>) -> Result<Value, String> {
    let slug = _args
        .first()
        .and_then(|v| v.as_str())
        .ok_or("Missing slug")?;

    clawhub::clawhub_uninstall(slug)
}

/// clawhub:list — List installed skills
/// _args: []
#[tauri::command]
pub async fn clawhub_list(_args: Vec<Value>) -> Result<Value, String> {
    clawhub::clawhub_list().await
}

/// clawhub:openSkillReadme — Open skill README
/// _args: [slug]
#[tauri::command]
pub async fn clawhub_open_skill_readme(_args: Vec<Value>) -> Result<Value, String> {
    let slug = _args
        .first()
        .and_then(|v| v.as_str())
        .ok_or("Missing slug")?;

    clawhub::clawhub_open_skill_readme(slug)
}
