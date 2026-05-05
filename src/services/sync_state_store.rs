//! Persistence for the `sync_state` table.

use rusqlite::Connection;

use crate::core::sync_state::SyncState;
use crate::services::database::DatabaseError;

/// Upsert the sync state for an account.
pub fn upsert_sync_state(conn: &Connection, state: &SyncState) -> Result<(), DatabaseError> {
    conn.execute(
        "INSERT INTO sync_state (
            account_id, idle_supported, condstore_supported, qresync_supported,
            utf8_accept, max_message_size, auth_mechanisms, capabilities_raw, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        ON CONFLICT(account_id) DO UPDATE SET
            idle_supported = excluded.idle_supported,
            condstore_supported = excluded.condstore_supported,
            qresync_supported = excluded.qresync_supported,
            utf8_accept = excluded.utf8_accept,
            max_message_size = excluded.max_message_size,
            auth_mechanisms = excluded.auth_mechanisms,
            capabilities_raw = excluded.capabilities_raw,
            updated_at = excluded.updated_at",
        rusqlite::params![
            state.account_id,
            state.idle_supported,
            state.condstore_supported,
            state.qresync_supported,
            state.utf8_accept,
            state.max_message_size.map(|v| v as i64),
            state.auth_mechanisms,
            state.capabilities_raw,
            state.updated_at as i64,
        ],
    )?;
    Ok(())
}

/// Load the sync state for an account, if it exists.
pub fn load_sync_state(
    conn: &Connection,
    account_id: &str,
) -> Result<Option<SyncState>, DatabaseError> {
    let mut stmt = conn.prepare(
        "SELECT account_id, idle_supported, condstore_supported, qresync_supported,
                utf8_accept, max_message_size, auth_mechanisms, capabilities_raw, updated_at
         FROM sync_state WHERE account_id = ?1",
    )?;

    let result = stmt.query_row(rusqlite::params![account_id], |row| {
        Ok(SyncState {
            account_id: row.get(0)?,
            idle_supported: row.get(1)?,
            condstore_supported: row.get(2)?,
            qresync_supported: row.get(3)?,
            utf8_accept: row.get(4)?,
            max_message_size: row.get::<_, Option<i64>>(5)?.map(|v| v as u64),
            auth_mechanisms: row.get(6)?,
            capabilities_raw: row.get(7)?,
            updated_at: row.get::<_, i64>(8)? as u64,
        })
    });

    match result {
        Ok(state) => Ok(Some(state)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DatabaseError::Sqlite(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::database::open_and_migrate;
    use tempfile::TempDir;

    #[test]
    fn upsert_and_load_sync_state() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_and_migrate(&db_path).unwrap();

        // Insert an account first (foreign key)
        conn.execute(
            "INSERT INTO accounts (id, display_name, protocol, host, port, encryption, auth_method, username, credential)
             VALUES ('acct-1', 'Test', 'Imap', 'imap.example.com', 993, 'SslTls', 'Plain', 'user', '')",
            [],
        ).unwrap();

        let state = SyncState {
            account_id: "acct-1".to_string(),
            idle_supported: true,
            condstore_supported: true,
            qresync_supported: false,
            utf8_accept: true,
            max_message_size: Some(52428800),
            auth_mechanisms: "PLAIN LOGIN".to_string(),
            capabilities_raw: "IMAP4rev1 IDLE CONDSTORE UTF8=ACCEPT".to_string(),
            updated_at: 1700000000,
        };

        upsert_sync_state(&conn, &state).unwrap();
        let loaded = load_sync_state(&conn, "acct-1").unwrap().unwrap();
        assert!(loaded.idle_supported);
        assert!(loaded.condstore_supported);
        assert!(loaded.utf8_accept);
        assert_eq!(loaded.max_message_size, Some(52428800));
        assert_eq!(loaded.auth_mechanisms, "PLAIN LOGIN");
    }

    #[test]
    fn upsert_overwrites_existing() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_and_migrate(&db_path).unwrap();

        conn.execute(
            "INSERT INTO accounts (id, display_name, protocol, host, port, encryption, auth_method, username, credential)
             VALUES ('acct-1', 'Test', 'Imap', 'imap.example.com', 993, 'SslTls', 'Plain', 'user', '')",
            [],
        ).unwrap();

        let state1 = SyncState {
            account_id: "acct-1".to_string(),
            idle_supported: false,
            capabilities_raw: "IMAP4rev1".to_string(),
            updated_at: 1000,
            ..Default::default()
        };
        upsert_sync_state(&conn, &state1).unwrap();

        let state2 = SyncState {
            account_id: "acct-1".to_string(),
            idle_supported: true,
            capabilities_raw: "IMAP4rev1 IDLE".to_string(),
            updated_at: 2000,
            ..Default::default()
        };
        upsert_sync_state(&conn, &state2).unwrap();

        let loaded = load_sync_state(&conn, "acct-1").unwrap().unwrap();
        assert!(loaded.idle_supported);
        assert_eq!(loaded.updated_at, 2000);
    }

    #[test]
    fn load_nonexistent_returns_none() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_and_migrate(&db_path).unwrap();

        let loaded = load_sync_state(&conn, "nonexistent").unwrap();
        assert!(loaded.is_none());
    }
}
