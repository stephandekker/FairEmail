//! In-memory mock `ContentStore` for tests (no disk access).

use std::cell::RefCell;
use std::collections::HashMap;

use crate::core::content_store::{ContentStore, ContentStoreError};
use crate::services::fs_content_store::sha256_hex;

/// In-memory content store for unit tests.
pub struct MemoryContentStore {
    data: RefCell<HashMap<String, Vec<u8>>>,
}

impl Default for MemoryContentStore {
    fn default() -> Self {
        Self {
            data: RefCell::new(HashMap::new()),
        }
    }
}

impl MemoryContentStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ContentStore for MemoryContentStore {
    fn put(&self, data: &[u8]) -> Result<String, ContentStoreError> {
        let hash = sha256_hex(data);
        self.data.borrow_mut().insert(hash.clone(), data.to_vec());
        Ok(hash)
    }

    fn get(&self, hash: &str) -> Result<Vec<u8>, ContentStoreError> {
        self.data
            .borrow()
            .get(hash)
            .cloned()
            .ok_or_else(|| ContentStoreError::NotFound(hash.to_string()))
    }

    fn delete(&self, hash: &str) -> Result<(), ContentStoreError> {
        self.data.borrow_mut().remove(hash);
        Ok(())
    }

    fn exists(&self, hash: &str) -> Result<bool, ContentStoreError> {
        Ok(self.data.borrow().contains_key(hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_store_roundtrip() {
        let store = MemoryContentStore::new();
        let data = b"test email content";
        let hash = store.put(data).unwrap();
        assert_eq!(store.get(&hash).unwrap(), data);
    }

    #[test]
    fn memory_store_exists() {
        let store = MemoryContentStore::new();
        let hash = sha256_hex(b"test");
        assert!(!store.exists(&hash).unwrap());
        store.put(b"test").unwrap();
        assert!(store.exists(&hash).unwrap());
    }

    #[test]
    fn memory_store_delete() {
        let store = MemoryContentStore::new();
        let hash = store.put(b"test").unwrap();
        store.delete(&hash).unwrap();
        assert!(!store.exists(&hash).unwrap());
    }

    #[test]
    fn memory_store_not_found() {
        let store = MemoryContentStore::new();
        assert!(matches!(
            store.get("no-such-hash").unwrap_err(),
            ContentStoreError::NotFound(_)
        ));
    }
}
