// ClawHub CLI interaction — search, install, uninstall, list, open readme

use crate::openclaw_paths;
use serde_json::{json, Value};
use std::path::PathBuf;

/// Strip ANSI escape sequences from CLI output
fn strip_ansi(s: &str) -> String {
    use std::sync::OnceLock;
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        regex::Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]|\x1b\].*?\x07")
            .expect("static ANSI regex must compile")
    });
    re.replace_all(s, "").to_string()
}

/// Get the clawhub CLI path and runner args
fn get_cli_command() -> (String, Vec<String>) {
    let config_dir = openclaw_paths::get_openclaw_config_dir();
    let cli_binary = PathBuf::from(&config_dir)
        .join("node_modules")
        .join(".bin")
        .join("clawhub");

    if cli_binary.exists() {
        return (cli_binary.to_string_lossy().to_string(), vec![]);
    }

    // Fallback: use node with the entry script
    let entry = PathBuf::from(&config_dir)
        .join("node_modules")
        .join("clawhub")
        .join("dist")
        .join("cli.js");

    ("node".to_string(), vec![entry.to_string_lossy().to_string()])
}

/// Run a clawhub CLI command and return stdout
async fn run_command(args: &[&str]) -> Result<String, String> {
    let (program, mut prefix_args) = get_cli_command();
    let config_dir = openclaw_paths::get_openclaw_config_dir();

    for a in args {
        prefix_args.push(a.to_string());
    }

    let output = tokio::process::Command::new(&program)
        .args(&prefix_args)
        .env("CI", "true")
        .env("FORCE_COLOR", "0")
        .env("CLAWHUB_WORKDIR", &config_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to run clawhub CLI: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "clawhub CLI exited with {}: {}{}",
            output.status,
            strip_ansi(&stderr),
            if !stdout.is_empty() {
                format!("\n{}", strip_ansi(&stdout))
            } else {
                String::new()
            }
        ));
    }

    Ok(strip_ansi(&String::from_utf8_lossy(&output.stdout)))
}

/// clawhub:search — Search for skills
pub async fn clawhub_search(query: &str, limit: Option<usize>) -> Result<Value, String> {
    let mut args = vec![];

    let is_empty = query.trim().is_empty();
    if is_empty {
        args.push("explore");
    } else {
        args.push("search");
        args.push(query);
    }

    let limit_str;
    if let Some(l) = limit {
        args.push("--limit");
        limit_str = l.to_string();
        args.push(&limit_str);
    }

    let stdout = run_command(&args).await?;
    let mut results = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Parse: slug  version  description (or with timestamp for explore)
        let parts: Vec<&str> = trimmed.splitn(3, char::is_whitespace).collect();
        if parts.len() >= 3 {
            let slug = parts[0].trim();
            let version = parts[1].trim().trim_start_matches('v');
            let description = parts[2].trim();

            if !slug.is_empty() && !version.is_empty() {
                results.push(json!({
                    "slug": slug,
                    "version": version,
                    "description": description,
                }));
            }
        }
    }

    Ok(json!({ "success": true, "results": results }))
}

/// clawhub:install — Install a skill
pub async fn clawhub_install(
    slug: &str,
    version: Option<&str>,
    force: bool,
) -> Result<Value, String> {
    let mut args = vec!["install", slug];

    let version_flag;
    if let Some(v) = version {
        args.push("--version");
        version_flag = v.to_string();
        args.push(&version_flag);
    }

    if force {
        args.push("--force");
    }

    run_command(&args).await?;
    Ok(json!({ "success": true }))
}

/// clawhub:uninstall — Uninstall a skill (direct fs operation)
pub fn clawhub_uninstall(slug: &str) -> Result<Value, String> {
    let config_dir = openclaw_paths::get_openclaw_config_dir();
    let config_path = PathBuf::from(&config_dir);

    // Remove skill directory
    let skill_dir = config_path.join("skills").join(slug);
    if skill_dir.exists() {
        std::fs::remove_dir_all(&skill_dir)
            .map_err(|e| format!("Failed to remove skill directory: {}", e))?;
    }

    // Update lock.json
    let lock_path = config_path.join(".clawhub").join("lock.json");
    if lock_path.exists() {
        let content = std::fs::read_to_string(&lock_path)
            .map_err(|e| format!("Failed to read lock.json: {}", e))?;
        if let Ok(mut lock) = serde_json::from_str::<Value>(&content) {
            if let Some(obj) = lock.as_object_mut() {
                obj.remove(slug);
                let updated = serde_json::to_string_pretty(&lock)
                    .map_err(|e| format!("Failed to serialize lock.json: {}", e))?;
                std::fs::write(&lock_path, updated)
                    .map_err(|e| format!("Failed to write lock.json: {}", e))?;
            }
        }
    }

    Ok(json!({ "success": true }))
}

/// clawhub:list — List installed skills
pub async fn clawhub_list() -> Result<Value, String> {
    let stdout = run_command(&["list"]).await?;
    let mut skills = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed.splitn(2, char::is_whitespace).collect();
        if parts.len() >= 2 {
            let slug = parts[0].trim();
            let version = parts[1].trim().trim_start_matches('v');
            if !slug.is_empty() {
                skills.push(json!({
                    "slug": slug,
                    "version": version,
                }));
            }
        }
    }

    Ok(json!({ "success": true, "skills": skills }))
}

/// clawhub:openSkillReadme — Open a skill's README file
pub fn clawhub_open_skill_readme(slug: &str) -> Result<Value, String> {
    let config_dir = openclaw_paths::get_openclaw_config_dir();
    let skill_dir = PathBuf::from(&config_dir).join("skills").join(slug);

    if !skill_dir.exists() {
        return Err(format!("Skill directory not found: {}", slug));
    }

    // Look for readme variants
    let readme_names = ["SKILL.md", "README.md", "skill.md", "readme.md"];
    let readme_path = readme_names
        .iter()
        .map(|name| skill_dir.join(name))
        .find(|p| p.exists());

    let path_to_open = match readme_path {
        Some(p) => p,
        None => skill_dir.clone(), // Fallback: open the directory
    };

    // Use `open` on macOS
    std::process::Command::new("open")
        .arg(path_to_open.to_string_lossy().to_string())
        .spawn()
        .map_err(|e| format!("Failed to open readme: {}", e))?;

    Ok(json!({ "success": true }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi() {
        assert_eq!(strip_ansi("\x1b[32mhello\x1b[0m"), "hello");
        assert_eq!(strip_ansi("no escape"), "no escape");
        assert_eq!(strip_ansi("\x1b[1;31mbold red\x1b[0m"), "bold red");
    }
}
