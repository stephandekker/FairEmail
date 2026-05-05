use std::cell::RefCell;
use std::path::PathBuf;

use rusqlite::Connection;
use serde::de::Error as _;
use uuid::Uuid;

use crate::core::{
    Account, AccountColor, AuthMethod, EncryptionMode, FetchSettings, KeepAliveSettings,
    NewAccountParams, Pop3Settings, Protocol, QuotaInfo, SecuritySettings, SmtpConfig,
    SwipeDefaults, SystemFolders,
};
use crate::services::account_store::StoreError;
use crate::services::database;

/// SQLite-backed account store. Drop-in replacement for the JSON-backed
/// `AccountStore` with the same public method signatures.
#[derive(Debug)]
pub struct SqliteAccountStore {
    conn: RefCell<Connection>,
}

impl SqliteAccountStore {
    /// Open the database at `db_path`, run migrations, and return a ready store.
    pub fn new(db_path: PathBuf) -> Result<Self, StoreError> {
        let conn = database::open_and_migrate(&db_path).map_err(|e| match e {
            database::DatabaseError::Sqlite(e) => StoreError::Database(e),
            database::DatabaseError::Io(e) => StoreError::Io(e),
        })?;
        Ok(Self {
            conn: RefCell::new(conn),
        })
    }

    /// Load all persisted accounts.
    pub fn load_all(&self) -> Result<Vec<Account>, StoreError> {
        let conn = self.conn.borrow();
        let mut stmt = conn.prepare(
            "SELECT id, display_name, protocol, host, port, encryption, auth_method,
                    username, credential, smtp_config, pop3_settings,
                    color_red, color_green, color_blue, avatar_path, category,
                    sync_enabled, on_demand, polling_interval_minutes,
                    unmetered_only, vpn_only, schedule_exempt, is_primary,
                    error_state, system_folders, swipe_defaults,
                    notifications_enabled, quota_used_bytes, quota_limit_bytes,
                    security_settings, fetch_settings, keep_alive_settings
             FROM accounts",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(AccountRow {
                id: row.get(0)?,
                display_name: row.get(1)?,
                protocol: row.get(2)?,
                host: row.get(3)?,
                port: row.get::<_, i64>(4)? as u16,
                encryption: row.get(5)?,
                auth_method: row.get(6)?,
                username: row.get(7)?,
                credential: row.get(8)?,
                smtp_config: row.get(9)?,
                pop3_settings: row.get(10)?,
                color_red: row.get(11)?,
                color_green: row.get(12)?,
                color_blue: row.get(13)?,
                avatar_path: row.get(14)?,
                category: row.get(15)?,
                sync_enabled: row.get(16)?,
                on_demand: row.get(17)?,
                polling_interval_minutes: row.get(18)?,
                unmetered_only: row.get(19)?,
                vpn_only: row.get(20)?,
                schedule_exempt: row.get(21)?,
                is_primary: row.get(22)?,
                error_state: row.get(23)?,
                system_folders: row.get(24)?,
                swipe_defaults: row.get(25)?,
                notifications_enabled: row.get(26)?,
                quota_used_bytes: row.get(27)?,
                quota_limit_bytes: row.get(28)?,
                security_settings: row.get(29)?,
                fetch_settings: row.get(30)?,
                keep_alive_settings: row.get(31)?,
            })
        })?;

        let mut accounts = Vec::new();
        for row_result in rows {
            let row = row_result?;
            accounts.push(row_to_account(row)?);
        }
        Ok(accounts)
    }

    /// Add a single account to the store.
    pub fn add(&self, account: Account) -> Result<(), StoreError> {
        let conn = self.conn.borrow();
        insert_account(&conn, &account)?;
        Ok(())
    }

    /// Replace an existing account (matched by id) with the given updated account.
    pub fn update(&self, updated: Account) -> Result<(), StoreError> {
        let conn = self.conn.borrow();
        let id_str = updated.id().to_string();

        // Check the account exists.
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM accounts WHERE id = ?1)",
            [&id_str],
            |row| row.get(0),
        )?;
        if !exists {
            return Err(StoreError::NotFound(updated.id()));
        }

        // Delete and re-insert (simpler than updating 30+ columns).
        conn.execute("DELETE FROM accounts WHERE id = ?1", [&id_str])?;
        insert_account(&conn, &updated)?;
        Ok(())
    }

    /// Remove an account by ID. Returns the removed account.
    pub fn delete(&self, id: Uuid) -> Result<Account, StoreError> {
        let conn = self.conn.borrow();
        let id_str = id.to_string();

        // Load the account first so we can return it.
        let account = self.load_by_id(&conn, id)?;

        conn.execute("DELETE FROM accounts WHERE id = ?1", [&id_str])?;
        Ok(account)
    }

    fn load_by_id(&self, conn: &Connection, id: Uuid) -> Result<Account, StoreError> {
        let id_str = id.to_string();
        let row = conn
            .query_row(
                "SELECT id, display_name, protocol, host, port, encryption, auth_method,
                    username, credential, smtp_config, pop3_settings,
                    color_red, color_green, color_blue, avatar_path, category,
                    sync_enabled, on_demand, polling_interval_minutes,
                    unmetered_only, vpn_only, schedule_exempt, is_primary,
                    error_state, system_folders, swipe_defaults,
                    notifications_enabled, quota_used_bytes, quota_limit_bytes,
                    security_settings, fetch_settings, keep_alive_settings
             FROM accounts WHERE id = ?1",
                [&id_str],
                |row| {
                    Ok(AccountRow {
                        id: row.get(0)?,
                        display_name: row.get(1)?,
                        protocol: row.get(2)?,
                        host: row.get(3)?,
                        port: row.get::<_, i64>(4)? as u16,
                        encryption: row.get(5)?,
                        auth_method: row.get(6)?,
                        username: row.get(7)?,
                        credential: row.get(8)?,
                        smtp_config: row.get(9)?,
                        pop3_settings: row.get(10)?,
                        color_red: row.get(11)?,
                        color_green: row.get(12)?,
                        color_blue: row.get(13)?,
                        avatar_path: row.get(14)?,
                        category: row.get(15)?,
                        sync_enabled: row.get(16)?,
                        on_demand: row.get(17)?,
                        polling_interval_minutes: row.get(18)?,
                        unmetered_only: row.get(19)?,
                        vpn_only: row.get(20)?,
                        schedule_exempt: row.get(21)?,
                        is_primary: row.get(22)?,
                        error_state: row.get(23)?,
                        system_folders: row.get(24)?,
                        swipe_defaults: row.get(25)?,
                        notifications_enabled: row.get(26)?,
                        quota_used_bytes: row.get(27)?,
                        quota_limit_bytes: row.get(28)?,
                        security_settings: row.get(29)?,
                        fetch_settings: row.get(30)?,
                        keep_alive_settings: row.get(31)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => StoreError::NotFound(id),
                other => StoreError::Database(other),
            })?;

        row_to_account(row)
    }

    /// Read all accounts that still have a non-empty credential column in the database.
    /// Used during the credential migration to move plaintext credentials to the keychain.
    /// Returns `(account_id, imap_credential, smtp_credential_opt)` tuples.
    pub fn read_plaintext_credentials(
        &self,
    ) -> Result<Vec<(Uuid, String, Option<String>)>, StoreError> {
        let conn = self.conn.borrow();
        let mut stmt = conn
            .prepare("SELECT id, credential, smtp_config FROM accounts WHERE credential != ''")?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })?;

        let mut results = Vec::new();
        for row_result in rows {
            let (id_str, credential, smtp_json) = row_result?;
            let id = Uuid::parse_str(&id_str)
                .map_err(|e| StoreError::Serialization(serde_json::Error::custom(e.to_string())))?;

            let smtp_credential = smtp_json.and_then(|json| {
                serde_json::from_str::<SmtpConfig>(&json)
                    .ok()
                    .map(|c| c.credential)
                    .filter(|s| !s.is_empty())
            });

            results.push((id, credential, smtp_credential));
        }
        Ok(results)
    }

    /// Clear the plaintext credential columns in the database after they have
    /// been migrated to the system keychain.
    pub fn clear_plaintext_credentials(&self) -> Result<(), StoreError> {
        let conn = self.conn.borrow();
        conn.execute(
            "UPDATE accounts SET credential = '' WHERE credential != ''",
            [],
        )?;
        conn.execute(
            "UPDATE accounts SET smtp_config = json_set(smtp_config, '$.credential', '')
             WHERE smtp_config IS NOT NULL AND json_extract(smtp_config, '$.credential') != ''",
            [],
        )?;
        Ok(())
    }

    /// Import accounts from JSON in a single transaction. Returns the number imported.
    /// Skips accounts whose ID already exists (idempotent).
    pub fn import_from_json(&self, accounts: &[Account]) -> Result<usize, StoreError> {
        let mut conn = self.conn.borrow_mut();
        let tx = conn.transaction()?;
        let mut imported = 0;

        for account in accounts {
            let id_str = account.id().to_string();
            let exists: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM accounts WHERE id = ?1)",
                [&id_str],
                |row| row.get(0),
            )?;
            if !exists {
                insert_account_tx(&tx, account)?;
                imported += 1;
            }
        }

        tx.commit()?;
        Ok(imported)
    }
}

/// Raw row data from the database (avoids borrowing issues with closures).
struct AccountRow {
    id: String,
    display_name: String,
    protocol: String,
    host: String,
    port: u16,
    encryption: String,
    auth_method: String,
    username: String,
    credential: String,
    smtp_config: Option<String>,
    pop3_settings: Option<String>,
    color_red: Option<f64>,
    color_green: Option<f64>,
    color_blue: Option<f64>,
    avatar_path: Option<String>,
    category: Option<String>,
    sync_enabled: bool,
    on_demand: bool,
    polling_interval_minutes: Option<i64>,
    unmetered_only: bool,
    vpn_only: bool,
    schedule_exempt: bool,
    is_primary: bool,
    error_state: Option<String>,
    system_folders: Option<String>,
    swipe_defaults: Option<String>,
    notifications_enabled: bool,
    quota_used_bytes: Option<i64>,
    quota_limit_bytes: Option<i64>,
    security_settings: Option<String>,
    fetch_settings: Option<String>,
    keep_alive_settings: Option<String>,
}

fn parse_protocol(s: &str) -> Protocol {
    match s {
        "Pop3" => Protocol::Pop3,
        _ => Protocol::Imap,
    }
}

fn parse_encryption(s: &str) -> EncryptionMode {
    match s {
        "SslTls" => EncryptionMode::SslTls,
        "StartTls" => EncryptionMode::StartTls,
        _ => EncryptionMode::None,
    }
}

fn parse_auth_method(s: &str) -> AuthMethod {
    match s {
        "Login" => AuthMethod::Login,
        "OAuth2" => AuthMethod::OAuth2,
        _ => AuthMethod::Plain,
    }
}

fn row_to_account(row: AccountRow) -> Result<Account, StoreError> {
    let id = Uuid::parse_str(&row.id)
        .map_err(|e| StoreError::Serialization(serde_json::Error::custom(e.to_string())))?;

    let protocol = parse_protocol(&row.protocol);
    let encryption = parse_encryption(&row.encryption);
    let auth_method = parse_auth_method(&row.auth_method);

    let smtp: Option<SmtpConfig> = row
        .smtp_config
        .as_deref()
        .map(serde_json::from_str::<SmtpConfig>)
        .transpose()?;

    let pop3_settings: Option<Pop3Settings> = row
        .pop3_settings
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?;

    let color = match (row.color_red, row.color_green, row.color_blue) {
        (Some(r), Some(g), Some(b)) => Some(AccountColor::new(r as f32, g as f32, b as f32)),
        _ => None,
    };

    let system_folders: Option<SystemFolders> = row
        .system_folders
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?;

    let swipe_defaults: Option<SwipeDefaults> = row
        .swipe_defaults
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?;

    let quota = match (row.quota_used_bytes, row.quota_limit_bytes) {
        (Some(used), Some(limit)) => QuotaInfo::new(used as u64, limit as u64),
        _ => None,
    };

    let security_settings: Option<SecuritySettings> = row
        .security_settings
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?;

    let fetch_settings: Option<FetchSettings> = row
        .fetch_settings
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?;

    let keep_alive_settings: Option<KeepAliveSettings> = row
        .keep_alive_settings
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?;

    let params = NewAccountParams {
        display_name: row.display_name,
        protocol,
        host: row.host,
        port: row.port,
        encryption,
        auth_method,
        username: row.username,
        credential: row.credential, // Empty after migration; real credential lives in keychain.
        smtp,
        pop3_settings,
        color,
        avatar_path: row.avatar_path,
        category: row.category,
        sync_enabled: row.sync_enabled,
        on_demand: row.on_demand,
        polling_interval_minutes: row.polling_interval_minutes.map(|v| v as u32),
        unmetered_only: row.unmetered_only,
        vpn_only: row.vpn_only,
        schedule_exempt: row.schedule_exempt,
        system_folders,
        swipe_defaults,
        notifications_enabled: row.notifications_enabled,
        security_settings,
        fetch_settings,
        keep_alive_settings,
    };

    let mut account = Account::new_from_store(id, params)
        .map_err(|e| StoreError::Serialization(serde_json::Error::custom(e.to_string())))?;

    account.set_primary(row.is_primary);
    if let Some(err) = row.error_state {
        account.set_error_state(Some(err));
    }
    if let Some(q) = quota {
        account.set_quota(Some(q));
    }

    Ok(account)
}

fn insert_account(conn: &Connection, account: &Account) -> Result<(), StoreError> {
    let id_str = account.id().to_string();
    let protocol = format!("{:?}", account.protocol());
    let encryption = format!("{:?}", account.encryption());
    let auth_method = format!("{:?}", account.auth_method());

    // Credentials are stored in the system keychain, not in the database.
    // We serialize SMTP config without the credential field value.
    let smtp_json = account
        .smtp()
        .map(|s| {
            let redacted = SmtpConfig {
                host: s.host.clone(),
                port: s.port,
                encryption: s.encryption,
                auth_method: s.auth_method,
                username: s.username.clone(),
                credential: String::new(),
            };
            serde_json::to_string(&redacted)
        })
        .transpose()?;

    let pop3_json = account
        .pop3_settings()
        .map(serde_json::to_string)
        .transpose()?;

    let (color_r, color_g, color_b) = match account.color() {
        Some(c) => (
            Some(c.red as f64),
            Some(c.green as f64),
            Some(c.blue as f64),
        ),
        None => (None, None, None),
    };

    let system_folders_json = account
        .system_folders()
        .map(serde_json::to_string)
        .transpose()?;

    let swipe_defaults_json = account
        .swipe_defaults()
        .map(serde_json::to_string)
        .transpose()?;

    let (quota_used, quota_limit) = match account.quota() {
        Some(q) => (Some(q.used_bytes as i64), Some(q.limit_bytes as i64)),
        None => (None, None),
    };

    let security_json = account
        .security_settings()
        .map(serde_json::to_string)
        .transpose()?;

    let fetch_json = account
        .fetch_settings()
        .map(serde_json::to_string)
        .transpose()?;

    let keep_alive_json = account
        .keep_alive_settings()
        .map(serde_json::to_string)
        .transpose()?;

    conn.execute(
        "INSERT INTO accounts (
            id, display_name, protocol, host, port, encryption, auth_method,
            username, credential, smtp_config, pop3_settings,
            color_red, color_green, color_blue, avatar_path, category,
            sync_enabled, on_demand, polling_interval_minutes,
            unmetered_only, vpn_only, schedule_exempt, is_primary,
            error_state, system_folders, swipe_defaults,
            notifications_enabled, quota_used_bytes, quota_limit_bytes,
            security_settings, fetch_settings, keep_alive_settings
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
            ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21,
            ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32
        )",
        rusqlite::params![
            id_str,
            account.display_name(),
            protocol,
            account.host(),
            account.port() as i64,
            encryption,
            auth_method,
            account.username(),
            "", // Credential stored in system keychain, not in database.
            smtp_json,
            pop3_json,
            color_r,
            color_g,
            color_b,
            account.avatar_path(),
            account.category(),
            account.sync_enabled(),
            account.on_demand(),
            account.polling_interval_minutes().map(|v| v as i64),
            account.unmetered_only(),
            account.vpn_only(),
            account.schedule_exempt(),
            account.is_primary(),
            account.error_state(),
            system_folders_json,
            swipe_defaults_json,
            account.notifications_enabled(),
            quota_used,
            quota_limit,
            security_json,
            fetch_json,
            keep_alive_json,
        ],
    )?;
    Ok(())
}

fn insert_account_tx(tx: &rusqlite::Transaction, account: &Account) -> Result<(), StoreError> {
    let id_str = account.id().to_string();
    let protocol = format!("{:?}", account.protocol());
    let encryption = format!("{:?}", account.encryption());
    let auth_method = format!("{:?}", account.auth_method());

    // Credentials are stored in the system keychain, not in the database.
    let smtp_json = account
        .smtp()
        .map(|s| {
            let redacted = SmtpConfig {
                host: s.host.clone(),
                port: s.port,
                encryption: s.encryption,
                auth_method: s.auth_method,
                username: s.username.clone(),
                credential: String::new(),
            };
            serde_json::to_string(&redacted)
        })
        .transpose()?;

    let pop3_json = account
        .pop3_settings()
        .map(serde_json::to_string)
        .transpose()?;

    let (color_r, color_g, color_b) = match account.color() {
        Some(c) => (
            Some(c.red as f64),
            Some(c.green as f64),
            Some(c.blue as f64),
        ),
        None => (None, None, None),
    };

    let system_folders_json = account
        .system_folders()
        .map(serde_json::to_string)
        .transpose()?;

    let swipe_defaults_json = account
        .swipe_defaults()
        .map(serde_json::to_string)
        .transpose()?;

    let (quota_used, quota_limit) = match account.quota() {
        Some(q) => (Some(q.used_bytes as i64), Some(q.limit_bytes as i64)),
        None => (None, None),
    };

    let security_json = account
        .security_settings()
        .map(serde_json::to_string)
        .transpose()?;

    let fetch_json = account
        .fetch_settings()
        .map(serde_json::to_string)
        .transpose()?;

    let keep_alive_json = account
        .keep_alive_settings()
        .map(serde_json::to_string)
        .transpose()?;

    tx.execute(
        "INSERT INTO accounts (
            id, display_name, protocol, host, port, encryption, auth_method,
            username, credential, smtp_config, pop3_settings,
            color_red, color_green, color_blue, avatar_path, category,
            sync_enabled, on_demand, polling_interval_minutes,
            unmetered_only, vpn_only, schedule_exempt, is_primary,
            error_state, system_folders, swipe_defaults,
            notifications_enabled, quota_used_bytes, quota_limit_bytes,
            security_settings, fetch_settings, keep_alive_settings
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
            ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21,
            ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32
        )",
        rusqlite::params![
            id_str,
            account.display_name(),
            protocol,
            account.host(),
            account.port() as i64,
            encryption,
            auth_method,
            account.username(),
            "", // Credential stored in system keychain, not in database.
            smtp_json,
            pop3_json,
            color_r,
            color_g,
            color_b,
            account.avatar_path(),
            account.category(),
            account.sync_enabled(),
            account.on_demand(),
            account.polling_interval_minutes().map(|v| v as i64),
            account.unmetered_only(),
            account.vpn_only(),
            account.schedule_exempt(),
            account.is_primary(),
            account.error_state(),
            system_folders_json,
            swipe_defaults_json,
            account.notifications_enabled(),
            quota_used,
            quota_limit,
            security_json,
            fetch_json,
            keep_alive_json,
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{AuthMethod, EncryptionMode, NewAccountParams, Protocol};
    use tempfile::TempDir;

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
        })
        .unwrap()
    }

    fn make_store() -> (TempDir, SqliteAccountStore) {
        let dir = TempDir::new().unwrap();
        let store = SqliteAccountStore::new(dir.path().join("fairmail.db")).unwrap();
        (dir, store)
    }

    #[test]
    fn empty_store_returns_empty_vec() {
        let (_dir, store) = make_store();
        assert!(store.load_all().unwrap().is_empty());
    }

    #[test]
    fn add_and_load_roundtrip() {
        let (_dir, store) = make_store();
        let acct = make_account("Test");
        let id = acct.id();
        store.add(acct).unwrap();

        let loaded = store.load_all().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id(), id);
        assert_eq!(loaded[0].display_name(), "Test");
    }

    #[test]
    fn update_persists() {
        use crate::core::UpdateAccountParams;

        let (_dir, store) = make_store();
        let acct = make_account("Original");
        let id = acct.id();
        store.add(acct).unwrap();

        let mut loaded = store.load_all().unwrap();
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
            })
            .unwrap();
        store.update(loaded[0].clone()).unwrap();

        let reloaded = store.load_all().unwrap();
        assert_eq!(reloaded.len(), 1);
        assert_eq!(reloaded[0].id(), id);
        assert_eq!(reloaded[0].display_name(), "Updated");
        assert_eq!(reloaded[0].host(), "pop.example.com");
    }

    #[test]
    fn update_not_found() {
        let (_dir, store) = make_store();
        store.add(make_account("Existing")).unwrap();
        let other = make_account("Other");
        let result = store.update(other);
        assert!(matches!(result, Err(StoreError::NotFound(_))));
    }

    #[test]
    fn delete_removes_account() {
        let (_dir, store) = make_store();
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
    fn delete_not_found() {
        let (_dir, store) = make_store();
        store.add(make_account("A")).unwrap();
        let result = store.delete(Uuid::new_v4());
        assert!(matches!(result, Err(StoreError::NotFound(_))));
    }

    #[test]
    fn import_from_json_is_idempotent() {
        let (_dir, store) = make_store();
        let acct = make_account("Imported");
        let id = acct.id();

        let imported = store.import_from_json(std::slice::from_ref(&acct)).unwrap();
        assert_eq!(imported, 1);

        // Second import of same account is a no-op.
        let imported2 = store.import_from_json(std::slice::from_ref(&acct)).unwrap();
        assert_eq!(imported2, 0);

        let all = store.load_all().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id(), id);
    }

    #[test]
    fn roundtrip_with_all_optional_fields() {
        let (_dir, store) = make_store();
        let acct = Account::new(NewAccountParams {
            display_name: "Full".into(),
            protocol: Protocol::Pop3,
            host: "pop.example.com".into(),
            port: 995,
            encryption: EncryptionMode::StartTls,
            auth_method: AuthMethod::OAuth2,
            username: "user@example.com".into(),
            credential: "oauth-token".into(),
            smtp: Some(SmtpConfig {
                host: "smtp.example.com".into(),
                port: 587,
                encryption: EncryptionMode::StartTls,
                auth_method: AuthMethod::Plain,
                username: "user@example.com".into(),
                credential: "smtp-pass".into(),
            }),
            pop3_settings: Some(Pop3Settings::default()),
            color: Some(AccountColor::new(0.5, 0.7, 0.9)),
            avatar_path: Some("/tmp/avatar.png".into()),
            category: Some("Work".into()),
            sync_enabled: false,
            on_demand: true,
            polling_interval_minutes: Some(15),
            unmetered_only: true,
            vpn_only: true,
            schedule_exempt: true,
            system_folders: Some(SystemFolders {
                drafts: Some("Drafts".into()),
                sent: Some("Sent".into()),
                archive: None,
                trash: Some("Trash".into()),
                junk: None,
            }),
            swipe_defaults: Some(SwipeDefaults::default()),
            notifications_enabled: false,
            security_settings: Some(SecuritySettings {
                dnssec: true,
                dane: true,
                insecure: false,
                certificate_fingerprint: Some("abc123".into()),
                client_certificate: None,
                auth_realm: Some("example.com".into()),
            }),
            fetch_settings: Some(FetchSettings {
                partial_fetch: true,
                raw_fetch: false,
                ignore_size_limits: true,
                date_header_preference: crate::core::DateHeaderPreference::DateHeader,
                utf8_support: true,
            }),
            keep_alive_settings: Some(KeepAliveSettings {
                use_noop_instead_of_idle: true,
            }),
        })
        .unwrap();

        let id = acct.id();
        store.add(acct).unwrap();

        let loaded = store.load_all().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id(), id);
        assert_eq!(loaded[0].display_name(), "Full");
        assert_eq!(loaded[0].protocol(), Protocol::Pop3);
        assert!(loaded[0].smtp().is_some());
        assert!(loaded[0].pop3_settings().is_some());
        assert!(loaded[0].color().is_some());
        assert_eq!(loaded[0].avatar_path(), Some("/tmp/avatar.png"));
        assert_eq!(loaded[0].category(), Some("Work"));
        assert!(!loaded[0].sync_enabled());
        assert!(loaded[0].on_demand());
        assert_eq!(loaded[0].polling_interval_minutes(), Some(15));
        assert!(loaded[0].unmetered_only());
        assert!(loaded[0].vpn_only());
        assert!(loaded[0].schedule_exempt());
        assert!(loaded[0].system_folders().is_some());
        assert!(loaded[0].swipe_defaults().is_some());
        assert!(!loaded[0].notifications_enabled());
        assert!(loaded[0].security_settings().is_some());
        assert!(loaded[0].fetch_settings().is_some());
        assert!(loaded[0].keep_alive_settings().is_some());
    }

    #[test]
    fn credential_never_stored_in_database() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("fairmail.db");
        let store = SqliteAccountStore::new(db_path.clone()).unwrap();

        let acct = Account::new(NewAccountParams {
            display_name: "Test".into(),
            protocol: Protocol::Imap,
            host: "imap.example.com".into(),
            port: 993,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "super-secret-password".into(),
            smtp: Some(SmtpConfig {
                host: "smtp.example.com".into(),
                port: 587,
                encryption: EncryptionMode::StartTls,
                auth_method: AuthMethod::Plain,
                username: "user@example.com".into(),
                credential: "smtp-secret-password".into(),
            }),
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
        })
        .unwrap();

        store.add(acct).unwrap();

        // Read the raw database content and verify no credential text is present.
        let raw_db = std::fs::read_to_string(&db_path).unwrap_or_default();
        let raw_bytes = std::fs::read(&db_path).unwrap();
        let raw_str = String::from_utf8_lossy(&raw_bytes);
        assert!(
            !raw_str.contains("super-secret-password"),
            "IMAP credential found in raw database file"
        );
        assert!(
            !raw_str.contains("smtp-secret-password"),
            "SMTP credential found in raw database file"
        );
        // Also check via SQL query.
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        let cred: String = conn
            .query_row("SELECT credential FROM accounts LIMIT 1", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(cred, "", "credential column should be empty");

        let smtp_json: String = conn
            .query_row("SELECT smtp_config FROM accounts LIMIT 1", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert!(
            !smtp_json.contains("smtp-secret-password"),
            "SMTP credential found in smtp_config JSON"
        );
        // Verify the smtp credential field in JSON is empty.
        let smtp: SmtpConfig = serde_json::from_str(&smtp_json).unwrap();
        assert_eq!(smtp.credential, "", "smtp credential should be empty in DB");
        drop(raw_db);
    }

    #[test]
    fn read_plaintext_credentials_for_migration() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("fairmail.db");
        let store = SqliteAccountStore::new(db_path.clone()).unwrap();

        // Manually insert an account with a plaintext credential via raw SQL
        // to simulate pre-migration state.
        {
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            conn.execute(
                "INSERT INTO accounts (id, display_name, protocol, host, port, encryption,
                 auth_method, username, credential, smtp_config)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                rusqlite::params![
                    "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
                    "Legacy",
                    "Imap",
                    "imap.example.com",
                    993i64,
                    "SslTls",
                    "Plain",
                    "user@example.com",
                    "legacy-password",
                    r#"{"host":"smtp.example.com","port":587,"encryption":"StartTls","auth_method":"Plain","username":"user@example.com","credential":"smtp-legacy-pass"}"#,
                ],
            )
            .unwrap();
        }

        let creds = store.read_plaintext_credentials().unwrap();
        assert_eq!(creds.len(), 1);
        let (id, imap_cred, smtp_cred) = &creds[0];
        assert_eq!(id.to_string(), "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee");
        assert_eq!(imap_cred, "legacy-password");
        assert_eq!(smtp_cred.as_deref(), Some("smtp-legacy-pass"));

        // Clear and verify.
        store.clear_plaintext_credentials().unwrap();
        let creds_after = store.read_plaintext_credentials().unwrap();
        assert!(creds_after.is_empty());
    }
}
