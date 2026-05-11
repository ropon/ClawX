// ClawX - Tauri v2 主入口
// Multi-Agent Desktop AI Assistant

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{
    menu::{MenuBuilder, MenuItem, PredefinedMenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Manager,
};
use serde::{Deserialize, Serialize};

// Week 3: Rust modules
mod config;
mod providers;
mod text_chunker;
mod storage;
mod commands;

// Week 4: Database + Search modules
mod database;
mod embedding;
mod rag_engine;
mod secure_storage;

// Week 5: Network modules
mod settings_store;
mod gateway;
mod channel_config;

// Week 6: Provider + Utility modules
mod openclaw_auth;
mod openclaw_paths;
mod openclaw_install;
mod provider_validate;
mod file_staging;
mod file_search;

// Week 7: Full migration modules
mod clawhub;
mod channel_validate;
mod document_parser;
mod web_scraper;
mod embedding_api;
mod knowledge_pipeline;
mod folder_watcher;
mod workflow_engine;
mod workflow_triggers;
mod gateway_lifecycle;

// Device authentication
mod device_identity;

// ========== 应用状态 ==========

pub struct AppState {
    pub tray_icon: Arc<Mutex<Option<TrayIcon>>>,
    pub data_dir: PathBuf,
    pub db_manager: database::DbManager,
    pub gateway_client: gateway::client::GatewayClient,
    // Week 7: New managed state
    pub folder_watcher: Arc<Mutex<folder_watcher::FolderWatcherManager>>,
    pub workflow_engine: Arc<tokio::sync::Mutex<workflow_engine::WorkflowEngine>>,
    pub workflow_triggers: Arc<Mutex<workflow_triggers::WorkflowTriggerManager>>,
    pub gateway_process: Arc<tokio::sync::Mutex<gateway_lifecycle::GatewayProcessManager>>,
}


// ========== 托盘模块 ==========

mod tray {
    use super::*;

    /// 创建系统托盘
    pub fn setup_tray(app: &AppHandle) -> Result<TrayIcon, Box<dyn std::error::Error>> {
        let show_item = MenuItem::with_id(app, "show", "Show ClawX", true, None::<&str>)?;
        let hide_item = MenuItem::with_id(app, "hide", "Hide ClawX", true, None::<&str>)?;
        let spotlight_item = MenuItem::with_id(app, "spotlight", "Toggle Spotlight", true, None::<&str>)?;
        let quit_item = PredefinedMenuItem::quit(app, Some("Quit ClawX"))?;

        let menu = MenuBuilder::new(app)
            .item(&show_item)
            .item(&hide_item)
            .separator()
            .item(&spotlight_item)
            .separator()
            .item(&quit_item)
            .build()?;

        let tray = TrayIconBuilder::with_id("main")
            .menu(&menu)
            .show_menu_on_left_click(true)
            .tooltip("ClawX - AI Assistant")
            .on_menu_event(move |app, event| {
                match event.id.as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.hide();
                        }
                    }
                    "spotlight" => {
                        let _ = window::toggle_spotlight_window(app);
                    }
                    _ => {}
                }
            })
            .build(app)?;

        Ok(tray)
    }
}

// ========== 窗口管理模块 ==========

mod window {
    use super::*;

    /// 切换 Spotlight 窗口显示/隐藏
    pub fn toggle_spotlight_window(app: &AppHandle) -> Result<(), String> {
        if let Some(w) = app.get_webview_window("spotlight") {
            let visible = w.is_visible().map_err(|e| e.to_string())?;
            if visible {
                w.hide().map_err(|e| e.to_string())?;
            } else {
                // 居中显示 Spotlight 窗口
                if let Ok(Some(monitor)) = w.current_monitor() {
                    let screen = monitor.size();
                    let size = w.inner_size().unwrap_or(tauri::PhysicalSize {
                        width: 680,
                        height: 480,
                    });
                    let x = (screen.width as i32 - size.width as i32) / 2;
                    let y = (screen.height as i32 / 4).max(100);
                    let _ = w.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                        x,
                        y,
                    }));
                }
                w.show().map_err(|e| e.to_string())?;
                w.set_focus().map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }
}

// ========== Tauri Commands: 窗口操作 ==========

#[tauri::command]
async fn window_minimize(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("main") {
        w.minimize().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn window_maximize(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("main") {
        let maximized = w.is_maximized().map_err(|e| e.to_string())?;
        if maximized {
            w.unmaximize().map_err(|e| e.to_string())?;
        } else {
            w.maximize().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
async fn window_close(app: AppHandle) -> Result<(), String> {
    // Hide instead of destroy so the tray "Show" entry can bring the window
    // back. Real exit is via the tray "Quit" menu item (PredefinedMenuItem::quit).
    if let Some(w) = app.get_webview_window("main") {
        w.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn window_is_maximized(app: AppHandle) -> Result<bool, String> {
    if let Some(w) = app.get_webview_window("main") {
        w.is_maximized().map_err(|e| e.to_string())
    } else {
        Ok(false)
    }
}

// ========== Tauri Commands: 快捷键 ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    pub spotlight: String,
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            spotlight: "CmdOrCtrl+Shift+Space".to_string(),
        }
    }
}

#[tauri::command]
async fn shortcut_get(state: tauri::State<'_, AppState>) -> Result<ShortcutConfig, String> {
    let config = settings_store::get_shortcut_config(&state.data_dir)?;
    let spotlight = config.get("spotlight")
        .and_then(|v| v.as_str())
        .unwrap_or("CmdOrCtrl+Shift+Space")
        .to_string();
    Ok(ShortcutConfig { spotlight })
}

#[tauri::command]
async fn shortcut_update(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    config: ShortcutConfig,
) -> Result<serde_json::Value, String> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    // 1. Unregister all existing shortcuts
    let _ = app.global_shortcut().unregister_all();

    // 2. Register the new shortcut
    let shortcut_str = config.spotlight.clone();
    let app_handle = app.clone();
    let shortcut = shortcut_str.parse::<tauri_plugin_global_shortcut::Shortcut>()
        .map_err(|e| format!("Invalid shortcut '{}': {}", shortcut_str, e))?;

    app.global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, event| {
            if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                let _ = window::toggle_spotlight_window(&app_handle);
            }
        })
        .map_err(|e| format!("Failed to register shortcut: {}", e))?;

    // 3. Persist to settings file
    let config_value = serde_json::json!({ "spotlight": config.spotlight });
    settings_store::save_shortcut_config(&state.data_dir, &config_value)?;

    println!("[ClawX] Shortcut updated and persisted: {:?}", config);
    Ok(serde_json::json!({ "success": true }))
}

// ========== Tauri Commands: 剪贴板 ==========

#[tauri::command]
async fn clipboard_read(app: AppHandle) -> Result<String, String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    let content = app.clipboard().read_text().map_err(|e| e.to_string())?;
    Ok(content)
}

#[tauri::command]
async fn clipboard_write(app: AppHandle, text: String) -> Result<(), String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    app.clipboard().write_text(text).map_err(|e| e.to_string())?;
    Ok(())
}

// ========== Tauri Commands: Spotlight ==========

#[tauri::command]
async fn spotlight_toggle(app: AppHandle) -> Result<(), String> {
    window::toggle_spotlight_window(&app)
}

#[tauri::command]
async fn spotlight_hide(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("spotlight") {
        w.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ========== Tauri Commands: 应用信息 ==========

#[tauri::command]
async fn app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

#[tauri::command]
async fn app_platform() -> Result<String, String> {
    let platform = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    };
    Ok(platform.to_string())
}

// ========== 主应用入口 ==========

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .on_window_event(|window, event| {
            // Intercept main-window close (macOS red dot, Windows X) — hide
            // to tray instead of tearing down so the tray "Show" entry works
            // and the app stays alive in the background. Tray "Quit" remains
            // the real exit path.
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .invoke_handler(tauri::generate_handler![
            // Week 1: 窗口操作 (4)
            window_minimize,
            window_maximize,
            window_close,
            window_is_maximized,
            // Week 1: 快捷键 (2)
            shortcut_get,
            shortcut_update,
            // Week 1: 剪贴板 (2)
            clipboard_read,
            clipboard_write,
            // Week 1: Spotlight (2)
            spotlight_toggle,
            spotlight_hide,
            // Week 1: 应用信息 (2)
            app_version,
            app_platform,
            // Week 3: Agents (9)
            commands::agents_cmd::agent_list,
            commands::agents_cmd::agent_get,
            commands::agents_cmd::agent_create,
            commands::agents_cmd::agent_update,
            commands::agents_cmd::agent_delete,
            commands::agents_cmd::agent_get_active,
            commands::agents_cmd::agent_set_active,
            commands::agents_cmd::agent_export,
            commands::agents_cmd::agent_import,
            // Week 3: Knowledge (11)
            commands::knowledge_cmd::knowledge_list,
            commands::knowledge_cmd::knowledge_get,
            commands::knowledge_cmd::knowledge_create,
            commands::knowledge_cmd::knowledge_update,
            commands::knowledge_cmd::knowledge_delete,
            commands::knowledge_cmd::knowledge_list_documents,
            commands::knowledge_cmd::knowledge_get_document,
            commands::knowledge_cmd::knowledge_create_document,
            commands::knowledge_cmd::knowledge_update_document,
            commands::knowledge_cmd::knowledge_delete_document,
            commands::knowledge_cmd::knowledge_refresh_stats,
            // Week 3: Workflows (8)
            commands::workflows_cmd::workflow_list,
            commands::workflows_cmd::workflow_get,
            commands::workflows_cmd::workflow_create,
            commands::workflows_cmd::workflow_update,
            commands::workflows_cmd::workflow_delete,
            commands::workflows_cmd::workflow_duplicate,
            commands::workflows_cmd::workflow_export,
            commands::workflows_cmd::workflow_import,
            // Week 4: Workflow Runs (4)
            commands::workflows_cmd::workflow_get_runs,
            commands::workflows_cmd::workflow_get_run,
            commands::workflows_cmd::workflow_delete_run,
            commands::workflows_cmd::workflow_clear_runs,
            // Week 4: Knowledge Search (2)
            commands::knowledge_cmd::knowledge_search,
            commands::knowledge_cmd::knowledge_rag,
            // Week 5: Gateway (5)
            commands::gateway_cmd::gateway_status,
            commands::gateway_cmd::gateway_is_connected,
            commands::gateway_cmd::gateway_rpc,
            commands::gateway_cmd::gateway_get_control_ui_url,
            commands::gateway_cmd::gateway_health,
            // Week 5: Cron (6)
            commands::cron_cmd::cron_list,
            commands::cron_cmd::cron_create,
            commands::cron_cmd::cron_update,
            commands::cron_cmd::cron_delete,
            commands::cron_cmd::cron_toggle,
            commands::cron_cmd::cron_trigger,
            // Week 5: Channel Config (6)
            commands::channel_cmd::channel_save_config,
            commands::channel_cmd::channel_get_config,
            commands::channel_cmd::channel_get_form_values,
            commands::channel_cmd::channel_delete_config,
            commands::channel_cmd::channel_list_configured,
            commands::channel_cmd::channel_set_enabled,
            // Week 5: Chat (1)
            commands::chat_cmd::chat_send_with_media,
            // Week 6: Provider (12)
            commands::provider_cmd::provider_list,
            commands::provider_cmd::provider_get,
            commands::provider_cmd::provider_save,
            commands::provider_cmd::provider_delete,
            commands::provider_cmd::provider_set_api_key,
            commands::provider_cmd::provider_update_with_key,
            commands::provider_cmd::provider_delete_api_key,
            commands::provider_cmd::provider_has_api_key,
            commands::provider_cmd::provider_get_api_key,
            commands::provider_cmd::provider_set_default,
            commands::provider_cmd::provider_get_default,
            commands::provider_cmd::provider_validate_key,
            // Week 6: Skill (3)
            commands::skill_cmd::skill_update_config,
            commands::skill_cmd::skill_get_config,
            commands::skill_cmd::skill_get_all_configs,
            // Week 6: OpenClaw (7)
            commands::openclaw_cmd::openclaw_status,
            commands::openclaw_cmd::openclaw_is_ready,
            commands::openclaw_cmd::openclaw_get_dir,
            commands::openclaw_cmd::openclaw_get_config_dir,
            commands::openclaw_cmd::openclaw_get_skills_dir,
            commands::openclaw_cmd::openclaw_get_cli_command,
            commands::openclaw_cmd::openclaw_install_cli_mac,
            commands::openclaw_cmd::openclaw_ensure_installed,
            // Week 6: Log (5)
            commands::log_cmd::log_read_file,
            commands::log_cmd::log_get_file_path,
            commands::log_cmd::log_get_dir,
            commands::log_cmd::log_list_files,
            commands::log_cmd::log_get_recent,
            // Week 6: File/Media (3)
            commands::file_cmd::file_stage,
            commands::file_cmd::file_stage_buffer,
            commands::file_cmd::media_get_thumbnails,
            // Week 6: FileSearch (2)
            commands::filesearch_cmd::filesearch_search,
            commands::filesearch_cmd::filesearch_read_content,
            // Week 6: UV (2)
            commands::uv_cmd::uv_check,
            commands::uv_cmd::uv_install_all,
            // Week 7: ClawHub (5)
            commands::clawhub_cmd::clawhub_search,
            commands::clawhub_cmd::clawhub_install,
            commands::clawhub_cmd::clawhub_uninstall,
            commands::clawhub_cmd::clawhub_list,
            commands::clawhub_cmd::clawhub_open_skill_readme,
            // Week 7: Channel Validation (2)
            commands::channel_cmd::channel_validate,
            commands::channel_cmd::channel_validate_credentials,
            // Week 7: Knowledge Simple (4)
            commands::knowledge_cmd::knowledge_remove_document,
            commands::knowledge_cmd::knowledge_get_watch_status,
            commands::knowledge_cmd::knowledge_get_embedding_options,
            commands::knowledge_cmd::knowledge_detect_dimension,
            // Week 7: Knowledge Pipeline (4)
            commands::knowledge_cmd::knowledge_add_document,
            commands::knowledge_cmd::knowledge_add_url,
            commands::knowledge_cmd::knowledge_reprocess_document,
            commands::knowledge_cmd::knowledge_set_watch_folder,
            // Week 7: Workflow Runtime (6)
            commands::workflows_cmd::workflow_execute,
            commands::workflows_cmd::workflow_cancel,
            commands::workflows_cmd::workflow_register_triggers,
            commands::workflows_cmd::workflow_unregister_triggers,
            commands::workflows_cmd::workflow_get_templates,
            commands::workflows_cmd::workflow_import_template,
            // Week 7: Gateway Lifecycle (3)
            commands::gateway_cmd::gateway_start,
            commands::gateway_cmd::gateway_stop,
            commands::gateway_cmd::gateway_restart,
            // Screenshot (1)
            commands::screenshot_cmd::screenshot_capture,
        ])
        .setup(|app| {
            println!("[ClawX] Application starting...");

            // 初始化数据目录
            let app_data_dir = app.path().app_data_dir().unwrap_or_default();
            std::fs::create_dir_all(&app_data_dir).ok();
            println!("[ClawX] Data directory: {}", app_data_dir.display());

            // 初始化数据库管理器
            let db_manager = database::DbManager::new();

            // 初始化 Gateway WebSocket 客户端 (Actor pattern)
            let gateway_client = gateway::client::GatewayClient::new(app_data_dir.clone());

            // Set AppHandle on gateway client for push event forwarding
            let app_handle = app.handle().clone();
            let gateway_client_ref = &gateway_client;
            tauri::async_runtime::block_on(async {
                gateway_client_ref.set_app_handle(app_handle).await;
            });

            // 初始化应用状态
            let app_state = AppState {
                tray_icon: Arc::new(Mutex::new(None)),
                data_dir: app_data_dir.clone(),
                db_manager,
                gateway_client,
                folder_watcher: Arc::new(Mutex::new(folder_watcher::FolderWatcherManager::new())),
                workflow_engine: Arc::new(tokio::sync::Mutex::new(workflow_engine::WorkflowEngine::new())),
                workflow_triggers: Arc::new(Mutex::new(workflow_triggers::WorkflowTriggerManager::new())),
                gateway_process: Arc::new(tokio::sync::Mutex::new(gateway_lifecycle::GatewayProcessManager::new())),
            };

            // 初始化系统托盘
            match tray::setup_tray(&app.handle()) {
                Ok(tray_icon) => {
                    if let Ok(mut tray_state) = app_state.tray_icon.lock() {
                        *tray_state = Some(tray_icon);
                    }
                    println!("[ClawX] Tray icon initialized");
                }
                Err(e) => {
                    eprintln!("[ClawX] Failed to setup tray: {}", e);
                }
            }

            // 将应用状态注入 Tauri
            app.manage(app_state);

            // Spotlight 窗口默认隐藏 (tauri.conf.json 已设置 visible: false)
            if let Some(spotlight) = app.get_webview_window("spotlight") {
                let _ = spotlight.hide();
                println!("[ClawX] Spotlight window ready (hidden)");
            }

            // Register persisted global shortcut for Spotlight
            {
                use tauri_plugin_global_shortcut::GlobalShortcutExt;
                let shortcut_config = settings_store::get_shortcut_config(&app_data_dir)
                    .unwrap_or_else(|_| serde_json::json!({ "spotlight": "CmdOrCtrl+Shift+Space" }));
                let shortcut_str = shortcut_config.get("spotlight")
                    .and_then(|v| v.as_str())
                    .unwrap_or("CmdOrCtrl+Shift+Space");
                match shortcut_str.parse::<tauri_plugin_global_shortcut::Shortcut>() {
                    Ok(shortcut) => {
                        let app_handle = app.handle().clone();
                        if let Err(e) = app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
                            if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                                let _ = window::toggle_spotlight_window(&app_handle);
                            }
                        }) {
                            eprintln!("[ClawX] Failed to register spotlight shortcut: {}", e);
                        } else {
                            println!("[ClawX] Spotlight shortcut registered: {}", shortcut_str);
                        }
                    }
                    Err(e) => {
                        eprintln!("[ClawX] Invalid spotlight shortcut '{}': {}", shortcut_str, e);
                    }
                }
            }

            println!("[ClawX] Setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running ClawX");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_config_default() {
        let config = ShortcutConfig::default();
        assert_eq!(config.spotlight, "CmdOrCtrl+Shift+Space");
    }

    #[test]
    fn test_app_version() {
        let version = env!("CARGO_PKG_VERSION");
        assert!(!version.is_empty());
    }
}
