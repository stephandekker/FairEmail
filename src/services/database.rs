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
const CURRENT_VERSION: u32 = 1;

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

    // Set the schema version to current after all migrations.
    conn.pragma_update(None, "user_version", CURRENT_VERSION)?;

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
    fn respects_custom_data_dir() {
        let dir = TempDir::new().unwrap();
        let custom = dir.path().join("custom").join("subdir");
        let db_path = custom.join("fairmail.db");
        let _conn = open_and_migrate(&db_path).unwrap();
        assert!(db_path.exists());
    }
}
