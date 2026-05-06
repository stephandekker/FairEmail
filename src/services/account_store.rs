use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::core::Account;

/// Errors from the account persistence layer.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("account not found: {0}")]
    NotFound(Uuid),
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
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

    /// Replace an existing account (matched by id) with the given updated account.
    /// The full list is rewritten atomically (NFR-3).
    pub fn update(&self, updated: Account) -> Result<(), StoreError> {
        let mut accounts = self.load_all()?;
        let pos = accounts
            .iter()
            .position(|a| a.id() == updated.id())
            .ok_or(StoreError::NotFound(updated.id()))?;
        accounts[pos] = updated;
        self.write_all(&accounts)
    }

    /// Remove an account by ID. The full list is rewritten atomically (NFR-3).
    /// Returns the removed account, or `NotFound` if no account with that ID exists.
    pub fn delete(&self, id: Uuid) -> Result<Account, StoreError> {
        let mut accounts = self.load_all()?;
        let pos = accounts
            .iter()
            .position(|a| a.id() == id)
            .ok_or(StoreError::NotFound(id))?;
        let removed = accounts.remove(pos);
        self.write_all(&accounts)?;
        Ok(removed)
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
            smtp: None,
            pop3_settings: None,
            color: None,
            avatar_path: None,
            category: None,
            sync_enabled: true,
            on_demand: false,
            polling_interval_minutes: None,
            unmetered_only: false,
            vpn_only: false,
            schedule_exempt: false,
            system_folders: None,
            swipe_defaults: None,
            notifications_enabled: true,
            security_settings: None,
            fetch_settings: None,
            keep_alive_settings: None,
            oauth_tenant: None,
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

    #[test]
    fn update_persists_changed_account() {
        use crate::core::UpdateAccountParams;

        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::new(dir.path().join("accounts.json"));
        let acct = make_account("Original");
        let id = acct.id();
        store.add(acct).unwrap();

        // Load, update, and persist.
        let mut loaded = store.load_all().unwrap();
        assert_eq!(loaded.len(), 1);
        loaded[0]
            .update(UpdateAccountParams {
                display_name: "Updated".into(),
                protocol: Protocol::Pop3,
                host: "pop.example.com".into(),
                port: 995,
                encryption: EncryptionMode::StartTls,
                auth_method: AuthMethod::Login,
                username: "updated@example.com".into(),
                credential: "new-secret".into(),
                smtp: None,
                pop3_settings: None,
                color: None,
                avatar_path: None,
                category: None,
                sync_enabled: true,
                on_demand: false,
                polling_interval_minutes: None,
                unmetered_only: false,
                vpn_only: false,
                schedule_exempt: false,
                system_folders: None,
                swipe_defaults: None,
                notifications_enabled: true,
                security_settings: None,
                fetch_settings: None,
                keep_alive_settings: None,
                oauth_tenant: None,
            })
            .unwrap();
        store.update(loaded[0].clone()).unwrap();

        // Reload and verify.
        let reloaded = store.load_all().unwrap();
        assert_eq!(reloaded.len(), 1);
        assert_eq!(reloaded[0].id(), id);
        assert_eq!(reloaded[0].display_name(), "Updated");
        assert_eq!(reloaded[0].host(), "pop.example.com");
    }

    #[test]
    fn update_returns_not_found_for_missing_id() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::new(dir.path().join("accounts.json"));
        store.add(make_account("Existing")).unwrap();

        // Try to update an account that was never stored.
        let other = make_account("Other");
        let result = store.update(other);
        assert!(matches!(result, Err(StoreError::NotFound(_))));
    }

    #[test]
    fn update_is_atomic() {
        use crate::core::UpdateAccountParams;

        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::new(dir.path().join("accounts.json"));
        let acct = make_account("A");
        store.add(acct).unwrap();

        let mut loaded = store.load_all().unwrap();
        loaded[0]
            .update(UpdateAccountParams {
                display_name: "B".into(),
                protocol: Protocol::Imap,
                host: "imap.example.com".into(),
                port: 993,
                encryption: EncryptionMode::SslTls,
                auth_method: AuthMethod::Plain,
                username: "user@example.com".into(),
                credential: "secret".into(),
                smtp: None,
                pop3_settings: None,
                color: None,
                avatar_path: None,
                category: None,
                sync_enabled: true,
                on_demand: false,
                polling_interval_minutes: None,
                unmetered_only: false,
                vpn_only: false,
                schedule_exempt: false,
                system_folders: None,
                swipe_defaults: None,
                notifications_enabled: true,
                security_settings: None,
                fetch_settings: None,
                keep_alive_settings: None,
                oauth_tenant: None,
            })
            .unwrap();
        store.update(loaded[0].clone()).unwrap();

        // No temp file left behind.
        assert!(!dir.path().join(".accounts.tmp").exists());
    }

    #[test]
    fn delete_removes_account() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::new(dir.path().join("accounts.json"));
        let acct1 = make_account("A");
        let acct2 = make_account("B");
        let id1 = acct1.id();
        let id2 = acct2.id();
        store.add(acct1).unwrap();
        store.add(acct2).unwrap();

        let removed = store.delete(id1).unwrap();
        assert_eq!(removed.id(), id1);

        let remaining = store.load_all().unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id(), id2);
    }

    #[test]
    fn delete_returns_not_found_for_missing_id() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::new(dir.path().join("accounts.json"));
        store.add(make_account("A")).unwrap();
        let result = store.delete(Uuid::new_v4());
        assert!(matches!(result, Err(StoreError::NotFound(_))));
    }

    #[test]
    fn delete_is_atomic() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::new(dir.path().join("accounts.json"));
        let acct = make_account("A");
        let id = acct.id();
        store.add(acct).unwrap();
        store.delete(id).unwrap();
        assert!(!dir.path().join(".accounts.tmp").exists());
    }
}
