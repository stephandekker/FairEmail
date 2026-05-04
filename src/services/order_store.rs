use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use uuid::Uuid;

/// Errors from the order persistence layer.
#[derive(Debug, thiserror::Error)]
pub enum OrderStoreError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Persists the user-defined account order as a JSON array of UUID strings (FR-20, AC-8).
///
/// Writes are atomic: data is written to a temporary file in the same directory
/// and then renamed over the target.
#[derive(Debug, Clone)]
pub struct OrderStore {
    path: PathBuf,
}

impl OrderStore {
    /// Create a store backed by the given file path.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Load the persisted account order. Returns `None` if the file does not exist
    /// (indicating no custom order has been set yet).
    pub fn load(&self) -> Result<Option<Vec<Uuid>>, OrderStoreError> {
        if !self.path.exists() {
            return Ok(None);
        }
        let data = fs::read_to_string(&self.path)?;
        let order: Vec<Uuid> = serde_json::from_str(&data)?;
        Ok(Some(order))
    }

    /// Save the account order to disk atomically.
    pub fn save(&self, order: &[Uuid]) -> Result<(), OrderStoreError> {
        let json = serde_json::to_string_pretty(order)?;
        let dir = self.path.parent().unwrap_or_else(|| Path::new("."));
        let tmp_path = dir.join(".order.tmp");
        {
            let mut f = fs::File::create(&tmp_path)?;
            f.write_all(json.as_bytes())?;
            f.sync_all()?;
        }
        fs::rename(&tmp_path, &self.path)?;
        Ok(())
    }

    /// Clear the persisted order (reset to default).
    pub fn clear(&self) -> Result<(), OrderStoreError> {
        if self.path.exists() {
            fs::remove_file(&self.path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_returns_none_when_file_missing() {
        let dir = tempfile::tempdir().unwrap();
        let store = OrderStore::new(dir.path().join("order.json"));
        assert!(store.load().unwrap().is_none());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let store = OrderStore::new(dir.path().join("order.json"));
        let ids: Vec<Uuid> = (0..3).map(|_| Uuid::new_v4()).collect();
        store.save(&ids).unwrap();
        let loaded = store.load().unwrap().unwrap();
        assert_eq!(loaded, ids);
    }

    #[test]
    fn atomic_write_leaves_no_tmp_file() {
        let dir = tempfile::tempdir().unwrap();
        let store = OrderStore::new(dir.path().join("order.json"));
        store.save(&[Uuid::new_v4()]).unwrap();
        assert!(!dir.path().join(".order.tmp").exists());
    }

    #[test]
    fn clear_removes_file() {
        let dir = tempfile::tempdir().unwrap();
        let store = OrderStore::new(dir.path().join("order.json"));
        store.save(&[Uuid::new_v4()]).unwrap();
        assert!(dir.path().join("order.json").exists());
        store.clear().unwrap();
        assert!(!dir.path().join("order.json").exists());
    }

    #[test]
    fn clear_when_no_file_is_ok() {
        let dir = tempfile::tempdir().unwrap();
        let store = OrderStore::new(dir.path().join("order.json"));
        store.clear().unwrap(); // Should not error.
    }
}
