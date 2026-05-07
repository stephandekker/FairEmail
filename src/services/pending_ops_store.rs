//! Database persistence for the `pending_operations` table.

use rusqlite::Connection;

use crate::core::pending_operation::{OperationKind, OperationState, PendingOperation};
use crate::services::database::DatabaseError;

/// Insert a new pending operation. Returns the new row id.
pub fn insert_pending_op(
    conn: &Connection,
    account_id: &str,
    kind: &OperationKind,
    payload: &str,
) -> Result<i64, DatabaseError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    conn.execute(
        "INSERT INTO pending_operations (account_id, kind, payload, state, retry_count, created_at)
         VALUES (?1, ?2, ?3, ?4, 0, ?5)",
        rusqlite::params![
            account_id,
            kind.as_str(),
            payload,
            OperationState::Pending.as_str(),
            now,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Load all pending/in-flight operations for an account, ordered by id (insertion order).
pub fn load_pending_ops(
    conn: &Connection,
    account_id: &str,
) -> Result<Vec<PendingOperation>, DatabaseError> {
    let mut stmt = conn.prepare(
        "SELECT id, account_id, kind, payload, state, retry_count, last_error, created_at
         FROM pending_operations
         WHERE account_id = ?1 AND state IN ('pending', 'in_flight')
         ORDER BY id ASC",
    )?;

    let rows = stmt.query_map(rusqlite::params![account_id], |row| {
        let kind_str: String = row.get(2)?;
        let state_str: String = row.get(4)?;
        Ok(PendingOperation {
            id: row.get(0)?,
            account_id: row.get(1)?,
            kind: OperationKind::parse(&kind_str).unwrap_or(OperationKind::StoreFlags),
            payload: row.get(3)?,
            state: OperationState::parse(&state_str).unwrap_or(OperationState::Pending),
            retry_count: row.get(5)?,
            last_error: row.get(6)?,
            created_at: row.get(7)?,
        })
    })?;

    let mut ops = Vec::new();
    for row in rows {
        ops.push(row?);
    }
    Ok(ops)
}

/// Mark an operation as in-flight.
pub fn mark_in_flight(conn: &Connection, op_id: i64) -> Result<(), DatabaseError> {
    conn.execute(
        "UPDATE pending_operations SET state = ?1 WHERE id = ?2",
        rusqlite::params![OperationState::InFlight.as_str(), op_id],
    )?;
    Ok(())
}

/// Mark an operation as completed (delete it).
pub fn complete_op(conn: &Connection, op_id: i64) -> Result<(), DatabaseError> {
    conn.execute(
        "DELETE FROM pending_operations WHERE id = ?1",
        rusqlite::params![op_id],
    )?;
    Ok(())
}

/// Mark an operation as failed with an error message.
pub fn mark_failed(conn: &Connection, op_id: i64, error: &str) -> Result<(), DatabaseError> {
    conn.execute(
        "UPDATE pending_operations SET state = ?1, last_error = ?2 WHERE id = ?3",
        rusqlite::params![OperationState::Failed.as_str(), error, op_id],
    )?;
    Ok(())
}

/// Re-queue an operation back to pending with incremented retry count and error.
pub fn requeue_op(conn: &Connection, op_id: i64, error: &str) -> Result<i32, DatabaseError> {
    conn.execute(
        "UPDATE pending_operations
         SET state = ?1, retry_count = retry_count + 1, last_error = ?2
         WHERE id = ?3",
        rusqlite::params![OperationState::Pending.as_str(), error, op_id],
    )?;

    let retry_count: i32 = conn.query_row(
        "SELECT retry_count FROM pending_operations WHERE id = ?1",
        rusqlite::params![op_id],
        |row| row.get(0),
    )?;
    Ok(retry_count)
}

/// Count pending operations for an account.
pub fn count_pending_ops(conn: &Connection, account_id: &str) -> Result<i64, DatabaseError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM pending_operations WHERE account_id = ?1 AND state = 'pending'",
        rusqlite::params![account_id],
        |row| row.get(0),
    )?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::database::open_and_migrate;
    use tempfile::TempDir;

    fn setup_db() -> (TempDir, Connection) {
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
    fn insert_and_load_pending_op() {
        let (_dir, conn) = setup_db();
        let id = insert_pending_op(
            &conn,
            "acct-1",
            &OperationKind::StoreFlags,
            r#"{"test":true}"#,
        )
        .unwrap();
        assert!(id > 0);

        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].id, id);
        assert_eq!(ops[0].kind, OperationKind::StoreFlags);
        assert_eq!(ops[0].state, OperationState::Pending);
        assert_eq!(ops[0].retry_count, 0);
    }

    #[test]
    fn mark_in_flight_changes_state() {
        let (_dir, conn) = setup_db();
        let id = insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, "{}").unwrap();
        mark_in_flight(&conn, id).unwrap();

        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops[0].state, OperationState::InFlight);
    }

    #[test]
    fn complete_op_removes_row() {
        let (_dir, conn) = setup_db();
        let id = insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, "{}").unwrap();
        complete_op(&conn, id).unwrap();

        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert!(ops.is_empty());
    }

    #[test]
    fn mark_failed_sets_error() {
        let (_dir, conn) = setup_db();
        let id = insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, "{}").unwrap();
        mark_failed(&conn, id, "auth rejected").unwrap();

        // Failed ops should not appear in load_pending_ops (which only loads pending/in_flight)
        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert!(ops.is_empty());
    }

    #[test]
    fn requeue_increments_retry_count() {
        let (_dir, conn) = setup_db();
        let id = insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, "{}").unwrap();
        mark_in_flight(&conn, id).unwrap();

        let count = requeue_op(&conn, id, "network error").unwrap();
        assert_eq!(count, 1);

        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops[0].state, OperationState::Pending);
        assert_eq!(ops[0].retry_count, 1);
        assert_eq!(ops[0].last_error.as_deref(), Some("network error"));
    }

    #[test]
    fn operations_ordered_by_insertion() {
        let (_dir, conn) = setup_db();
        let id1 =
            insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, r#"{"n":1}"#).unwrap();
        let id2 =
            insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, r#"{"n":2}"#).unwrap();
        let id3 =
            insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, r#"{"n":3}"#).unwrap();

        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 3);
        assert_eq!(ops[0].id, id1);
        assert_eq!(ops[1].id, id2);
        assert_eq!(ops[2].id, id3);
    }

    #[test]
    fn count_pending_ops_only_counts_pending() {
        let (_dir, conn) = setup_db();
        insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, "{}").unwrap();
        let id2 = insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, "{}").unwrap();
        mark_failed(&conn, id2, "error").unwrap();

        assert_eq!(count_pending_ops(&conn, "acct-1").unwrap(), 1);
    }

    #[test]
    fn queue_holds_1000_entries_without_degradation() {
        let (_dir, conn) = setup_db();
        let mut ids = Vec::with_capacity(1000);
        for i in 0..1000 {
            let payload = format!(r#"{{"n":{}}}"#, i);
            let id =
                insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, &payload).unwrap();
            ids.push(id);
        }

        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1000);
        // Verify insertion order preserved
        for (i, op) in ops.iter().enumerate() {
            assert_eq!(op.id, ids[i]);
        }
        assert_eq!(count_pending_ops(&conn, "acct-1").unwrap(), 1000);
    }

    #[test]
    fn operations_survive_close_and_reopen() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");

        // Open DB, insert operations, then drop (close) the connection.
        {
            let conn = open_and_migrate(&db_path).unwrap();
            conn.execute(
                "INSERT INTO accounts (id, display_name, protocol, host, port, encryption, auth_method, username, credential)
                 VALUES ('acct-1', 'Test', 'Imap', 'imap.example.com', 993, 'SslTls', 'Plain', 'user', '')",
                [],
            ).unwrap();
            insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, r#"{"n":1}"#).unwrap();
            insert_pending_op(&conn, "acct-1", &OperationKind::MoveMessage, r#"{"n":2}"#).unwrap();
            insert_pending_op(&conn, "acct-1", &OperationKind::DeleteMessage, r#"{"n":3}"#)
                .unwrap();
            // Connection dropped here — simulates crash/restart.
        }

        // Reopen and verify operations survived.
        let conn2 = open_and_migrate(&db_path).unwrap();
        let ops = load_pending_ops(&conn2, "acct-1").unwrap();
        assert_eq!(ops.len(), 3);
        assert_eq!(ops[0].kind, OperationKind::StoreFlags);
        assert_eq!(ops[1].kind, OperationKind::MoveMessage);
        assert_eq!(ops[2].kind, OperationKind::DeleteMessage);
        // Verify ordering is preserved after reopen.
        assert!(ops[0].id < ops[1].id);
        assert!(ops[1].id < ops[2].id);
    }

    #[test]
    fn different_accounts_are_independent() {
        let (_dir, conn) = setup_db();
        conn.execute(
            "INSERT INTO accounts (id, display_name, protocol, host, port, encryption, auth_method, username, credential)
             VALUES ('acct-2', 'Test2', 'Imap', 'imap.example.com', 993, 'SslTls', 'Plain', 'user2', '')",
            [],
        ).unwrap();

        insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, r#"{"a":1}"#).unwrap();
        insert_pending_op(&conn, "acct-2", &OperationKind::MoveMessage, r#"{"b":2}"#).unwrap();

        let ops1 = load_pending_ops(&conn, "acct-1").unwrap();
        let ops2 = load_pending_ops(&conn, "acct-2").unwrap();
        assert_eq!(ops1.len(), 1);
        assert_eq!(ops2.len(), 1);
        assert_eq!(ops1[0].kind, OperationKind::StoreFlags);
        assert_eq!(ops2[0].kind, OperationKind::MoveMessage);
    }
}
