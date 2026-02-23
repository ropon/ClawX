// Workflow triggers — Cron, file-change, clipboard, shortcut

use crate::storage::workflows::{TriggerType, WorkflowConfig, WorkflowTrigger};
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tauri::AppHandle;
use tokio::sync::mpsc;

/// Callback type for trigger events
pub type TriggerCallback = Arc<dyn Fn(String, String, String) + Send + Sync>;

pub struct WorkflowTriggerManager {
    /// Cron job handles: "workflowId:triggerId" → JoinHandle
    cron_handles: HashMap<String, tokio::task::JoinHandle<()>>,
    /// File watchers: "workflowId:triggerId" → RecommendedWatcher
    file_watchers: HashMap<String, RecommendedWatcher>,
    /// Clipboard triggers: "workflowId:triggerId" → pattern
    clipboard_triggers: HashMap<String, ClipboardTrigger>,
    /// Clipboard polling task
    clipboard_poll_handle: Option<tokio::task::JoinHandle<()>>,
    /// Shortcut map: accelerator → workflowId
    shortcut_map: HashMap<String, String>,
    /// Callback for trigger events
    on_trigger: Option<TriggerCallback>,
    /// Channel for trigger events from watcher threads
    trigger_tx: mpsc::UnboundedSender<TriggerEvent>,
    #[allow(dead_code)]
    trigger_rx: Option<mpsc::UnboundedReceiver<TriggerEvent>>,
}

#[allow(dead_code)]
struct ClipboardTrigger {
    workflow_id: String,
    pattern: regex::Regex,
}

#[allow(dead_code)]
struct TriggerEvent {
    workflow_id: String,
    trigger_type: String,
    input: String,
}

impl WorkflowTriggerManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            cron_handles: HashMap::new(),
            file_watchers: HashMap::new(),
            clipboard_triggers: HashMap::new(),
            clipboard_poll_handle: None,
            shortcut_map: HashMap::new(),
            on_trigger: None,
            trigger_tx: tx,
            trigger_rx: Some(rx),
        }
    }

    /// Set the trigger callback
    pub fn set_callback(&mut self, callback: TriggerCallback) {
        self.on_trigger = Some(callback);
    }

    /// Register all triggers for a workflow
    pub fn register_triggers(
        &mut self,
        app: &AppHandle,
        workflow: &WorkflowConfig,
    ) -> Result<(), String> {
        // Unregister existing triggers for this workflow
        self.unregister_triggers(&workflow.id);

        for trigger in &workflow.triggers {
            if !trigger.enabled {
                continue;
            }

            match trigger.trigger_type {
                TriggerType::Cron => {
                    self.register_cron_trigger(&workflow.id, trigger)?;
                }
                TriggerType::FileChange => {
                    self.register_file_trigger(&workflow.id, trigger)?;
                }
                TriggerType::Clipboard => {
                    self.register_clipboard_trigger(&workflow.id, trigger)?;
                }
                TriggerType::Shortcut => {
                    self.register_shortcut_trigger(app, &workflow.id, trigger)?;
                }
                TriggerType::Manual => {
                    // Manual triggers don't need registration
                }
            }
        }

        Ok(())
    }

    /// Unregister all triggers for a workflow
    pub fn unregister_triggers(&mut self, workflow_id: &str) {
        // Cancel cron jobs
        let cron_keys: Vec<String> = self
            .cron_handles
            .keys()
            .filter(|k| k.starts_with(&format!("{}:", workflow_id)))
            .cloned()
            .collect();
        for key in cron_keys {
            if let Some(handle) = self.cron_handles.remove(&key) {
                handle.abort();
            }
        }

        // Stop file watchers
        let watcher_keys: Vec<String> = self
            .file_watchers
            .keys()
            .filter(|k| k.starts_with(&format!("{}:", workflow_id)))
            .cloned()
            .collect();
        for key in watcher_keys {
            self.file_watchers.remove(&key);
        }

        // Remove clipboard triggers
        let clip_keys: Vec<String> = self
            .clipboard_triggers
            .keys()
            .filter(|k| k.starts_with(&format!("{}:", workflow_id)))
            .cloned()
            .collect();
        for key in clip_keys {
            self.clipboard_triggers.remove(&key);
        }

        // Remove shortcuts
        let shortcut_keys: Vec<String> = self
            .shortcut_map
            .iter()
            .filter(|(_, wf_id)| *wf_id == workflow_id)
            .map(|(accel, _)| accel.clone())
            .collect();
        for key in shortcut_keys {
            self.shortcut_map.remove(&key);
        }
    }

    /// Destroy all triggers
    pub fn destroy_all(&mut self) {
        for (_, handle) in self.cron_handles.drain() {
            handle.abort();
        }
        self.file_watchers.clear();
        self.clipboard_triggers.clear();
        if let Some(handle) = self.clipboard_poll_handle.take() {
            handle.abort();
        }
        self.shortcut_map.clear();
    }

    // ── Private: Cron ──

    fn register_cron_trigger(
        &mut self,
        workflow_id: &str,
        trigger: &WorkflowTrigger,
    ) -> Result<(), String> {
        let cron_expr = trigger
            .cron_expr
            .as_ref()
            .ok_or("Cron trigger missing expression")?;

        // Validate cron expression
        let schedule = cron_expr
            .parse::<cron::Schedule>()
            .map_err(|e| format!("Invalid cron expression '{}': {}", cron_expr, e))?;

        let key = format!("{}:{}", workflow_id, trigger.id);
        let tx = self.trigger_tx.clone();
        let wf_id = workflow_id.to_string();
        let expr = cron_expr.clone();

        let handle = tokio::spawn(async move {
            loop {
                let next = schedule.upcoming(chrono::Utc).next();
                if let Some(next_time) = next {
                    let duration = (next_time - chrono::Utc::now())
                        .to_std()
                        .unwrap_or(Duration::from_secs(60));

                    tokio::time::sleep(duration).await;

                    let _ = tx.send(TriggerEvent {
                        workflow_id: wf_id.clone(),
                        trigger_type: "cron".to_string(),
                        input: format!("Cron trigger: {}", expr),
                    });
                } else {
                    // No more upcoming times, wait and retry
                    tokio::time::sleep(Duration::from_secs(60)).await;
                }
            }
        });

        self.cron_handles.insert(key, handle);
        Ok(())
    }

    // ── Private: File Change ──

    fn register_file_trigger(
        &mut self,
        workflow_id: &str,
        trigger: &WorkflowTrigger,
    ) -> Result<(), String> {
        let watch_path = trigger
            .watch_path
            .as_ref()
            .ok_or("File trigger missing watch path")?;

        let key = format!("{}:{}", workflow_id, trigger.id);
        let tx = self.trigger_tx.clone();
        let wf_id = workflow_id.to_string();
        let file_pattern = trigger.file_pattern.clone();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    match event.kind {
                        EventKind::Create(_) | EventKind::Modify(_) => {
                            for path in &event.paths {
                                let path_str = path.to_string_lossy().to_string();

                                // Check file pattern if specified
                                if let Some(ref pattern) = file_pattern {
                                    let file_name = path
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("");
                                    if let Ok(re) = regex::Regex::new(pattern) {
                                        if !re.is_match(file_name) {
                                            continue;
                                        }
                                    }
                                }

                                let _ = tx.send(TriggerEvent {
                                    workflow_id: wf_id.clone(),
                                    trigger_type: "file-change".to_string(),
                                    input: format!("File changed: {}", path_str),
                                });
                            }
                        }
                        _ => {}
                    }
                }
            },
            Config::default(),
        )
        .map_err(|e| format!("Failed to create file watcher: {}", e))?;

        watcher
            .watch(Path::new(watch_path), RecursiveMode::Recursive)
            .map_err(|e| format!("Failed to watch path: {}", e))?;

        self.file_watchers.insert(key, watcher);
        Ok(())
    }

    // ── Private: Clipboard ──

    fn register_clipboard_trigger(
        &mut self,
        workflow_id: &str,
        trigger: &WorkflowTrigger,
    ) -> Result<(), String> {
        let pattern_str = trigger
            .clipboard_pattern
            .as_ref()
            .ok_or("Clipboard trigger missing pattern")?;

        let pattern = regex::Regex::new(pattern_str)
            .map_err(|e| format!("Invalid clipboard pattern: {}", e))?;

        let key = format!("{}:{}", workflow_id, trigger.id);
        self.clipboard_triggers.insert(
            key,
            ClipboardTrigger {
                workflow_id: workflow_id.to_string(),
                pattern,
            },
        );

        Ok(())
    }

    // ── Private: Shortcut ──

    fn register_shortcut_trigger(
        &mut self,
        _app: &AppHandle,
        workflow_id: &str,
        trigger: &WorkflowTrigger,
    ) -> Result<(), String> {
        let accelerator = trigger
            .shortcut_accelerator
            .as_ref()
            .ok_or("Shortcut trigger missing accelerator")?;

        // Store the mapping; actual shortcut registration is handled by Tauri plugin
        self.shortcut_map
            .insert(accelerator.clone(), workflow_id.to_string());

        Ok(())
    }
}

impl Default for WorkflowTriggerManager {
    fn default() -> Self {
        Self::new()
    }
}
