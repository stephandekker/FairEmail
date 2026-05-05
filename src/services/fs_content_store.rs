//! Filesystem-based `ContentStore` implementation.
//!
//! Stores `.eml` files at `<root>/<aa>/<bb>/<sha256>.eml`, where `aa` and `bb`
//! are the first two pairs of hex digits of the SHA-256 hash. Writes are atomic
//! (temp file + rename).

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use sha2::{Digest, Sha256};

use crate::core::content_store::{ContentStore, ContentStoreError};

/// Content store backed by the filesystem.
pub struct FsContentStore {
    root: PathBuf,
}

impl FsContentStore {
    /// Create a new `FsContentStore` rooted at the given directory.
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Compute the filesystem path for a given hash.
    fn path_for_hash(&self, hash: &str) -> PathBuf {
        let aa = &hash[0..2];
        let bb = &hash[2..4];
        self.root.join(aa).join(bb).join(format!("{hash}.eml"))
    }
}

/// Compute the SHA-256 hex digest of raw bytes.
pub fn sha256_hex(data: &[u8]) -> String {
    let hash = Sha256::digest(data);
    hex::encode(hash)
}

impl ContentStore for FsContentStore {
    fn put(&self, data: &[u8]) -> Result<String, ContentStoreError> {
        let hash = sha256_hex(data);
        let path = self.path_for_hash(&hash);

        // Idempotent: if the file already exists, skip.
        if path.exists() {
            return Ok(hash);
        }

        // Ensure parent directory exists.
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Atomic write: temp file in same directory, then rename.
        let tmp_path = path.with_extension("eml.tmp");
        let mut file = fs::File::create(&tmp_path)?;
        file.write_all(data)?;
        file.sync_all()?;
        fs::rename(&tmp_path, &path)?;

        Ok(hash)
    }

    fn get(&self, hash: &str) -> Result<Vec<u8>, ContentStoreError> {
        let path = self.path_for_hash(hash);
        fs::read(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ContentStoreError::NotFound(hash.to_string())
            } else {
                ContentStoreError::Io(e)
            }
        })
    }

    fn delete(&self, hash: &str) -> Result<(), ContentStoreError> {
        let path = self.path_for_hash(hash);
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(ContentStoreError::Io(e)),
        }
    }

    fn exists(&self, hash: &str) -> Result<bool, ContentStoreError> {
        Ok(self.path_for_hash(hash).exists())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn put_and_get_roundtrip() {
        let dir = TempDir::new().unwrap();
        let store = FsContentStore::new(dir.path().to_path_buf());
        let data = b"From: test@example.com\r\nSubject: Hello\r\n\r\nBody\r\n";
        let hash = store.put(data).unwrap();
        assert!(!hash.is_empty());
        let retrieved = store.get(&hash).unwrap();
        assert_eq!(retrieved, data);
    }

    #[test]
    fn put_is_idempotent() {
        let dir = TempDir::new().unwrap();
        let store = FsContentStore::new(dir.path().to_path_buf());
        let data = b"Same content";
        let hash1 = store.put(data).unwrap();
        let hash2 = store.put(data).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn path_uses_sharding() {
        let store = FsContentStore::new(PathBuf::from("/root"));
        let path = store.path_for_hash("abcdef1234567890");
        assert_eq!(
            path,
            std::path::Path::new("/root/ab/cd/abcdef1234567890.eml")
        );
    }

    #[test]
    fn get_not_found() {
        let dir = TempDir::new().unwrap();
        let store = FsContentStore::new(dir.path().to_path_buf());
        let err = store.get("nonexistent").unwrap_err();
        assert!(matches!(err, ContentStoreError::NotFound(_)));
    }

    #[test]
    fn exists_returns_false_then_true() {
        let dir = TempDir::new().unwrap();
        let store = FsContentStore::new(dir.path().to_path_buf());
        let data = b"test data";
        let hash = sha256_hex(data);
        assert!(!store.exists(&hash).unwrap());
        store.put(data).unwrap();
        assert!(store.exists(&hash).unwrap());
    }

    #[test]
    fn delete_removes_file() {
        let dir = TempDir::new().unwrap();
        let store = FsContentStore::new(dir.path().to_path_buf());
        let data = b"delete me";
        let hash = store.put(data).unwrap();
        assert!(store.exists(&hash).unwrap());
        store.delete(&hash).unwrap();
        assert!(!store.exists(&hash).unwrap());
    }

    #[test]
    fn delete_nonexistent_is_ok() {
        let dir = TempDir::new().unwrap();
        let store = FsContentStore::new(dir.path().to_path_buf());
        assert!(store.delete("nonexistent").is_ok());
    }
}
