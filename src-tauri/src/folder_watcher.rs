// Folder watcher — Watch directories for file changes (knowledge base ingestion)

use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

const WATCHED_EXTENSIONS: &[&str] = &["pdf", "docx", "txt", "md", "csv"];

pub struct FolderWatcherManager {
    watchers: HashMap<String, WatcherEntry>,
}

struct WatcherEntry {
    _watcher: RecommendedWatcher,
    _thread: std::thread::JoinHandle<()>,
}

#[derive(Debug, Clone)]
pub struct FileEvent {
    pub path: String,
    pub file_name: String,
    pub file_type: String,
    pub file_size: u64,
    pub event_type: FileEventType,
}

#[derive(Debug, Clone)]
pub enum FileEventType {
    Added,
    Modified,
    Removed,
}

impl FolderWatcherManager {
    pub fn new() -> Self {
        Self {
            watchers: HashMap::new(),
        }
    }

    /// Start watching a folder for a knowledge base
    pub fn start_watching<F>(
        &mut self,
        kb_id: &str,
        folder_path: &str,
        callback: F,
    ) -> Result<(), String>
    where
        F: Fn(FileEvent) + Send + 'static,
    {
        // Stop existing watcher for this KB
        self.stop_watching(kb_id);

        let (tx, rx) = mpsc::channel();
        let _folder = folder_path.to_string();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            Config::default().with_poll_interval(Duration::from_millis(500)),
        )
        .map_err(|e| format!("Failed to create watcher: {}", e))?;

        watcher
            .watch(Path::new(folder_path), RecursiveMode::Recursive)
            .map_err(|e| format!("Failed to watch folder: {}", e))?;

        // Spawn thread to process events with debouncing
        let thread = std::thread::spawn(move || {
            let mut debounce_map: HashMap<String, std::time::Instant> = HashMap::new();
            let debounce_duration = Duration::from_millis(500);

            loop {
                match rx.recv_timeout(Duration::from_secs(1)) {
                    Ok(event) => {
                        for path in event.paths {
                            let path_str = path.to_string_lossy().to_string();

                            // Check extension
                            let ext = path
                                .extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("")
                                .to_lowercase();

                            if !WATCHED_EXTENSIONS.contains(&ext.as_str()) {
                                continue;
                            }

                            // Debounce
                            let now = std::time::Instant::now();
                            if let Some(last) = debounce_map.get(&path_str) {
                                if now.duration_since(*last) < debounce_duration {
                                    continue;
                                }
                            }
                            debounce_map.insert(path_str.clone(), now);

                            let file_name = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("")
                                .to_string();

                            let (event_type, file_size) = match event.kind {
                                EventKind::Create(_) => {
                                    let size = std::fs::metadata(&path)
                                        .map(|m| m.len())
                                        .unwrap_or(0);
                                    (FileEventType::Added, size)
                                }
                                EventKind::Modify(_) => {
                                    let size = std::fs::metadata(&path)
                                        .map(|m| m.len())
                                        .unwrap_or(0);
                                    (FileEventType::Modified, size)
                                }
                                EventKind::Remove(_) => (FileEventType::Removed, 0),
                                _ => continue,
                            };

                            callback(FileEvent {
                                path: path_str,
                                file_name,
                                file_type: ext.clone(),
                                file_size,
                                event_type,
                            });
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        });

        self.watchers.insert(
            kb_id.to_string(),
            WatcherEntry {
                _watcher: watcher,
                _thread: thread,
            },
        );

        Ok(())
    }

    /// Stop watching for a specific knowledge base
    pub fn stop_watching(&mut self, kb_id: &str) {
        self.watchers.remove(kb_id);
    }

    /// Get whether a KB is being watched
    pub fn is_watching(&self, kb_id: &str) -> bool {
        self.watchers.contains_key(kb_id)
    }

    /// Stop all watchers
    pub fn stop_all(&mut self) {
        self.watchers.clear();
    }
}

impl Default for FolderWatcherManager {
    fn default() -> Self {
        Self::new()
    }
}
