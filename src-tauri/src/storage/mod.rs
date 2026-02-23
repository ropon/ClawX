// JSON file storage abstraction (single JSON file per store)

pub mod agents;
pub mod knowledge;
pub mod workflows;

use serde::{de::DeserializeOwned, Serialize};
use std::path::{Path, PathBuf};
use tokio::sync::Mutex;

/// Generic JSON file store with single-file-per-store behavior.
/// Each store maps to `{data_dir}/{name}.json`.
pub struct JsonStore<T: Serialize + DeserializeOwned + Default> {
    path: PathBuf,
    lock: Mutex<()>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned + Default> JsonStore<T> {
    pub fn new(data_dir: &Path, name: &str) -> Self {
        Self {
            path: data_dir.join(format!("{}.json", name)),
            lock: Mutex::new(()),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Load data from JSON file. Returns Default if file doesn't exist.
    pub async fn load(&self) -> Result<T, String> {
        let _guard = self.lock.lock().await;
        if !self.path.exists() {
            return Ok(T::default());
        }
        let content = tokio::fs::read_to_string(&self.path)
            .await
            .map_err(|e| format!("Failed to read {}: {}", self.path.display(), e))?;
        if content.trim().is_empty() {
            return Ok(T::default());
        }
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse {}: {}", self.path.display(), e))
    }

    /// Save data to JSON file atomically.
    pub async fn save(&self, data: &T) -> Result<(), String> {
        let _guard = self.lock.lock().await;
        let content = serde_json::to_string_pretty(data)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        // Write atomically via temp file
        let tmp_path = self.path.with_extension("json.tmp");
        tokio::fs::write(&tmp_path, &content)
            .await
            .map_err(|e| format!("Failed to write temp file: {}", e))?;
        tokio::fs::rename(&tmp_path, &self.path)
            .await
            .map_err(|e| format!("Failed to rename temp file: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::TempDir;

    #[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
    struct TestData {
        items: Vec<String>,
    }

    #[tokio::test]
    async fn test_load_nonexistent_returns_default() {
        let tmp = TempDir::new().unwrap();
        let store = JsonStore::<TestData>::new(tmp.path(), "test");
        let data = store.load().await.unwrap();
        assert_eq!(data, TestData::default());
    }

    #[tokio::test]
    async fn test_save_and_load() {
        let tmp = TempDir::new().unwrap();
        let store = JsonStore::<TestData>::new(tmp.path(), "test");

        let data = TestData {
            items: vec!["hello".to_string(), "world".to_string()],
        };
        store.save(&data).await.unwrap();

        let loaded = store.load().await.unwrap();
        assert_eq!(loaded, data);
    }
}
