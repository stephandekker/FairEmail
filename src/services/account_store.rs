use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::core::Account;

/// Errors from the account persistence layer.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Persists accounts as a JSON array in a single file.
///
/// Writes are atomic: data is written to a temporary file in the same directory
/// and then renamed over the target (NFR-3).
#[derive(Debug, Clone)]
pub struct AccountStore {
    path: PathBuf,
}

impl AccountStore {
    /// Create a store backed by the given file path.
    /// The parent directory must already exist.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Load all persisted accounts. Returns an empty vec if the file does not
    /// exist yet.
    pub fn load_all(&self) -> Result<Vec<Account>, StoreError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let data = fs::read_to_string(&self.path)?;
        let accounts: Vec<Account> = serde_json::from_str(&data)?;
        Ok(accounts)
    }

    /// Add a single account. The full list is rewritten atomically (NFR-3).
    /// No application-imposed limit on the number of accounts (FR-1).
    pub fn add(&self, account: Account) -> Result<(), StoreError> {
        let mut accounts = self.load_all()?;
        accounts.push(account);
        self.write_all(&accounts)
    }

    /// Atomically write the full account list to disk.
    fn write_all(&self, accounts: &[Account]) -> Result<(), StoreError> {
        let json = serde_json::to_string_pretty(accounts)?;

        let dir = self.path.parent().unwrap_or_else(|| Path::new("."));

        // Write to a temporary file in the same directory, then rename.
        // rename() on the same filesystem is atomic on POSIX.
        let tmp_path = dir.join(".accounts.tmp");
        {
            let mut f = fs::File::create(&tmp_path)?;
            f.write_all(json.as_bytes())?;
            f.sync_all()?;
        }
        fs::rename(&tmp_path, &self.path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{AuthMethod, EncryptionMode, NewAccountParams, Protocol};

    fn make_account(name: &str) -> Account {
        Account::new(NewAccountParams {
            display_name: name.into(),
            protocol: Protocol::Imap,
            host: "imap.example.com".into(),
            port: 993,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "secret".into(),
        })
        .unwrap()
    }

    #[test]
    fn empty_store_returns_empty_vec() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::new(dir.path().join("accounts.json"));
        assert!(store.load_all().unwrap().is_empty());
    }

    #[test]
    fn add_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::new(dir.path().join("accounts.json"));
        let acct = make_account("Test");
        let id = acct.id();
        store.add(acct).unwrap();

        let loaded = store.load_all().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id(), id);
        assert_eq!(loaded[0].display_name(), "Test");
    }

    #[test]
    fn multiple_accounts_no_limit() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::new(dir.path().join("accounts.json"));
        for i in 0..100 {
            store.add(make_account(&format!("Acct {i}"))).unwrap();
        }
        assert_eq!(store.load_all().unwrap().len(), 100);
    }

    #[test]
    fn atomic_write_leaves_no_tmp_file() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::new(dir.path().join("accounts.json"));
        store.add(make_account("A")).unwrap();
        assert!(!dir.path().join(".accounts.tmp").exists());
    }
}
