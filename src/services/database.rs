use std::path::Path;

use rusqlite::Connection;

/// Errors from the database layer.
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// The current schema version. Increment when adding new migrations.
const CURRENT_VERSION: u32 = 7;

/// Open (or create) the SQLite database at `db_path`, configure pragmas,
/// and run any pending migrations. Returns the open connection.
pub(crate) fn open_and_migrate(db_path: &Path) -> Result<Connection, DatabaseError> {
    // Ensure parent directory exists.
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(db_path)?;

    // Set pragmas: WAL journal mode, foreign keys on, synchronous=NORMAL.
    conn.pragma_update(None, "journal_mode", "wal")?;
    conn.pragma_update(None, "foreign_keys", "on")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;

    run_migrations(&conn)?;

    Ok(conn)
}

/// Run pending schema migrations based on the `user_version` pragma.
fn run_migrations(conn: &Connection) -> Result<(), DatabaseError> {
    let version: u32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

    if version < 1 {
        migrate_v1(conn)?;
    }
    if version < 2 {
        migrate_v2(conn)?;
    }
    if version < 3 {
        migrate_v3(conn)?;
    }
    if version < 4 {
        migrate_v4(conn)?;
    }
    if version < 5 {
        migrate_v5(conn)?;
    }
    if version < 6 {
        migrate_v6(conn)?;
    }
    if version < 7 {
        migrate_v7(conn)?;
    }

    // Set the schema version to current after all migrations.
    conn.pragma_update(None, "user_version", CURRENT_VERSION)?;

    Ok(())
}

/// Migration v7: Create the `identities` table (FR-4).
fn migrate_v7(conn: &Connection) -> Result<(), DatabaseError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS identities (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            account_id TEXT NOT NULL,
            email_address TEXT NOT NULL,
            display_name TEXT NOT NULL DEFAULT '',
            smtp_host TEXT NOT NULL DEFAULT '',
            smtp_port INTEGER NOT NULL DEFAULT 587,
            smtp_encryption TEXT NOT NULL DEFAULT 'StartTls',
            smtp_username TEXT NOT NULL DEFAULT '',
            smtp_realm TEXT NOT NULL DEFAULT '',
            use_ip_in_ehlo INTEGER NOT NULL DEFAULT 0,
            custom_ehlo TEXT,
            login_before_send INTEGER NOT NULL DEFAULT 0,
            max_message_size_cache INTEGER,
            FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_identities_account
            ON identities(account_id);",
    )?;
    Ok(())
}

/// Migration v6: Create the `connection_log` table (per-account, append-only).
fn migrate_v6(conn: &Connection) -> Result<(), DatabaseError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS connection_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            account_id TEXT NOT NULL,
            timestamp_secs INTEGER NOT NULL,
            event_type TEXT NOT NULL,
            message TEXT NOT NULL,
            FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_connection_log_account
            ON connection_log(account_id, timestamp_secs);",
    )?;
    Ok(())
}

/// Migration v5: Create the `folders` table (per-account folder enumeration).
fn migrate_v5(conn: &Connection) -> Result<(), DatabaseError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS folders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            account_id TEXT NOT NULL,
            name TEXT NOT NULL,
            attributes TEXT NOT NULL DEFAULT '',
            role TEXT,
            delimiter TEXT,
            FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
            UNIQUE(account_id, name)
        );",
    )?;
    Ok(())
}

/// Migration v4: Create the `sync_state` table (per-account capability cache).
fn migrate_v4(conn: &Connection) -> Result<(), DatabaseError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sync_state (
            account_id TEXT PRIMARY KEY NOT NULL,
            idle_supported INTEGER NOT NULL DEFAULT 0,
            condstore_supported INTEGER NOT NULL DEFAULT 0,
            qresync_supported INTEGER NOT NULL DEFAULT 0,
            utf8_accept INTEGER NOT NULL DEFAULT 0,
            max_message_size INTEGER,
            auth_mechanisms TEXT NOT NULL DEFAULT '',
            capabilities_raw TEXT NOT NULL DEFAULT '',
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
        );",
    )?;
    Ok(())
}

/// Migration v3: Schema marker for credential-to-keychain migration.
/// The actual data migration (moving plaintext credentials into the system
/// keychain and clearing the database column) is handled by
/// `migrate_credentials_to_keychain` in main.rs, which runs AFTER this
/// schema migration. This migration is intentionally a no-op on the schema
/// because the column already accepts empty strings.
fn migrate_v3(_conn: &Connection) -> Result<(), DatabaseError> {
    // No schema changes needed — the credential column already accepts TEXT.
    // Plaintext credential clearing is handled by the application-level
    // migration after credentials have been written to the keychain.
    Ok(())
}

/// Migration v2: Create the `settings` and `account_order` tables.
fn migrate_v2(conn: &Connection) -> Result<(), DatabaseError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS account_order (
            position INTEGER PRIMARY KEY NOT NULL,
            account_id TEXT NOT NULL
        );",
    )?;
    Ok(())
}

/// Migration v1: Create the `accounts` table with FR-4 columns.
fn migrate_v1(conn: &Connection) -> Result<(), DatabaseError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS accounts (
            id TEXT PRIMARY KEY NOT NULL,
            display_name TEXT NOT NULL,
            protocol TEXT NOT NULL,
            host TEXT NOT NULL,
            port INTEGER NOT NULL,
            encryption TEXT NOT NULL,
            auth_method TEXT NOT NULL,
            username TEXT NOT NULL,
            credential TEXT NOT NULL,
            smtp_config TEXT,
            pop3_settings TEXT,
            color_red REAL,
            color_green REAL,
            color_blue REAL,
            avatar_path TEXT,
            category TEXT,
            sync_enabled INTEGER NOT NULL DEFAULT 1,
            on_demand INTEGER NOT NULL DEFAULT 0,
            polling_interval_minutes INTEGER,
            unmetered_only INTEGER NOT NULL DEFAULT 0,
            vpn_only INTEGER NOT NULL DEFAULT 0,
            schedule_exempt INTEGER NOT NULL DEFAULT 0,
            is_primary INTEGER NOT NULL DEFAULT 0,
            error_state TEXT,
            system_folders TEXT,
            swipe_defaults TEXT,
            notifications_enabled INTEGER NOT NULL DEFAULT 1,
            quota_used_bytes INTEGER,
            quota_limit_bytes INTEGER,
            security_settings TEXT,
            fetch_settings TEXT,
            keep_alive_settings TEXT
        );",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn creates_database_with_wal_and_foreign_keys() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("fairmail.db");
        let conn = open_and_migrate(&db_path).unwrap();

        let journal_mode: String = conn
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .unwrap();
        assert_eq!(journal_mode.to_lowercase(), "wal");

        let fk: i32 = conn
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .unwrap();
        assert_eq!(fk, 1);

        let sync: i32 = conn
            .pragma_query_value(None, "synchronous", |row| row.get(0))
            .unwrap();
        // NORMAL = 1
        assert_eq!(sync, 1);
    }

    #[test]
    fn creates_accounts_table() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("fairmail.db");
        let conn = open_and_migrate(&db_path).unwrap();

        // Verify accounts table exists by querying it.
        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn migration_is_idempotent() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("fairmail.db");

        // Open twice — second open should be a no-op.
        let conn1 = open_and_migrate(&db_path).unwrap();
        drop(conn1);
        let conn2 = open_and_migrate(&db_path).unwrap();

        let version: u32 = conn2
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, CURRENT_VERSION);
    }

    #[test]
    fn creates_settings_and_account_order_tables() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("fairmail.db");
        let conn = open_and_migrate(&db_path).unwrap();

        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM settings", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);

        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM account_order", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn respects_custom_data_dir() {
        let dir = TempDir::new().unwrap();
        let custom = dir.path().join("custom").join("subdir");
        let db_path = custom.join("fairmail.db");
        let _conn = open_and_migrate(&db_path).unwrap();
        assert!(db_path.exists());
    }
}
