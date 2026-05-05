//! Persistence for the `connection_log` table.

use rusqlite::Connection;

use crate::core::connection_log::{ConnectionLogEventType, ConnectionLogRecord};
use crate::services::database::DatabaseError;

/// Append a batch of log records for an account.
pub fn append_connection_logs(
    conn: &Connection,
    records: &[ConnectionLogRecord],
) -> Result<(), DatabaseError> {
    let mut stmt = conn.prepare(
        "INSERT INTO connection_log (account_id, timestamp_secs, event_type, message)
         VALUES (?1, ?2, ?3, ?4)",
    )?;

    for record in records {
        stmt.execute(rusqlite::params![
            record.account_id,
            record.timestamp_secs as i64,
            record.event_type.as_str(),
            record.message,
        ])?;
    }

    Ok(())
}

/// Load recent connection log entries for an account (newest first).
pub fn load_connection_logs(
    conn: &Connection,
    account_id: &str,
    limit: u32,
) -> Result<Vec<ConnectionLogRecord>, DatabaseError> {
    let mut stmt = conn.prepare(
        "SELECT id, account_id, timestamp_secs, event_type, message
         FROM connection_log
         WHERE account_id = ?1
         ORDER BY timestamp_secs DESC, id DESC
         LIMIT ?2",
    )?;

    let rows = stmt.query_map(rusqlite::params![account_id, limit], |row| {
        let id: i64 = row.get(0)?;
        let account_id: String = row.get(1)?;
        let timestamp_secs: i64 = row.get(2)?;
        let event_type_str: String = row.get(3)?;
        let message: String = row.get(4)?;

        let event_type =
            ConnectionLogEventType::parse(&event_type_str).unwrap_or(ConnectionLogEventType::Error);

        Ok(ConnectionLogRecord {
            id: Some(id),
            account_id,
            timestamp_secs: timestamp_secs as u64,
            event_type,
            message,
        })
    })?;

    let mut records = Vec::new();
    for row in rows {
        records.push(row?);
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::database::open_and_migrate;
    use tempfile::TempDir;

    fn setup_db() -> (tempfile::TempDir, Connection) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_and_migrate(&db_path).unwrap();
        conn.execute(
            "INSERT INTO accounts (id, display_name, protocol, host, port, encryption, auth_method, username, credential)
             VALUES ('acct-1', 'Test', 'Imap', 'imap.example.com', 993, 'SslTls', 'Plain', 'user', '')",
            [],
        ).unwrap();
        (dir, conn)
    }

    #[test]
    fn append_and_load_logs() {
        let (_dir, conn) = setup_db();

        let records = vec![
            ConnectionLogRecord {
                id: None,
                account_id: "acct-1".to_string(),
                timestamp_secs: 1000,
                event_type: ConnectionLogEventType::ConnectAttempt,
                message: "Connecting to imap.example.com:993".to_string(),
            },
            ConnectionLogRecord {
                id: None,
                account_id: "acct-1".to_string(),
                timestamp_secs: 1001,
                event_type: ConnectionLogEventType::TlsHandshake,
                message: "TLS handshake successful".to_string(),
            },
            ConnectionLogRecord {
                id: None,
                account_id: "acct-1".to_string(),
                timestamp_secs: 1002,
                event_type: ConnectionLogEventType::LoginResult,
                message: "Login successful".to_string(),
            },
        ];

        append_connection_logs(&conn, &records).unwrap();
        let loaded = load_connection_logs(&conn, "acct-1", 100).unwrap();
        assert_eq!(loaded.len(), 3);
        // Newest first
        assert_eq!(loaded[0].timestamp_secs, 1002);
        assert_eq!(loaded[0].event_type, ConnectionLogEventType::LoginResult);
        assert_eq!(loaded[2].event_type, ConnectionLogEventType::ConnectAttempt);
    }

    #[test]
    fn load_respects_limit() {
        let (_dir, conn) = setup_db();

        let records: Vec<ConnectionLogRecord> = (0..10)
            .map(|i| ConnectionLogRecord {
                id: None,
                account_id: "acct-1".to_string(),
                timestamp_secs: 1000 + i,
                event_type: ConnectionLogEventType::ConnectAttempt,
                message: format!("Entry {i}"),
            })
            .collect();

        append_connection_logs(&conn, &records).unwrap();
        let loaded = load_connection_logs(&conn, "acct-1", 3).unwrap();
        assert_eq!(loaded.len(), 3);
    }

    #[test]
    fn load_empty_returns_empty() {
        let (_dir, conn) = setup_db();
        let loaded = load_connection_logs(&conn, "acct-1", 100).unwrap();
        assert!(loaded.is_empty());
    }
}
