//! ContentStore trait for storing and retrieving raw `.eml` message content.

/// Errors from content store operations.
#[derive(Debug, thiserror::Error)]
pub enum ContentStoreError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("content not found for hash: {0}")]
    NotFound(String),
}

/// Trait for content-addressed storage of raw message bytes.
///
/// Implementations store each message as `<hash>.eml` where hash is the
/// SHA-256 of the raw bytes. `put` is idempotent: storing the same bytes
/// twice returns the same hash without creating a duplicate file.
pub trait ContentStore {
    /// Store raw message bytes and return the SHA-256 hex hash.
    fn put(&self, data: &[u8]) -> Result<String, ContentStoreError>;

    /// Retrieve raw message bytes by hash.
    fn get(&self, hash: &str) -> Result<Vec<u8>, ContentStoreError>;

    /// Delete the stored content for a hash.
    fn delete(&self, hash: &str) -> Result<(), ContentStoreError>;

    /// Check whether content exists for a hash.
    fn exists(&self, hash: &str) -> Result<bool, ContentStoreError>;
}
