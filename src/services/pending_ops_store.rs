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

/// Load all pending/in-flight operations for an account that are ready to execute,
/// ordered by id (insertion order). Operations with a future `next_retry_at` are
/// skipped so that backoff delays do not block other operations (NFR-7).
pub fn load_pending_ops(
    conn: &Connection,
    account_id: &str,
) -> Result<Vec<PendingOperation>, DatabaseError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let mut stmt = conn.prepare(
        "SELECT id, account_id, kind, payload, state, retry_count, last_error, created_at, next_retry_at
         FROM pending_operations
         WHERE account_id = ?1 AND state IN ('pending', 'in_flight')
           AND (next_retry_at IS NULL OR next_retry_at <= ?2)
         ORDER BY id ASC",
    )?;

    let rows = stmt.query_map(rusqlite::params![account_id, now], |row| {
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
            next_retry_at: row.get(8)?,
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

/// Re-queue an operation back to pending with incremented retry count, error,
/// and a `next_retry_at` timestamp based on exponential backoff.
pub fn requeue_op(
    conn: &Connection,
    op_id: i64,
    error: &str,
    backoff_secs: u64,
) -> Result<i32, DatabaseError> {
    let next_retry_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
        + backoff_secs as i64;

    conn.execute(
        "UPDATE pending_operations
         SET state = ?1, retry_count = retry_count + 1, last_error = ?2, next_retry_at = ?3
         WHERE id = ?4",
        rusqlite::params![
            OperationState::Pending.as_str(),
            error,
            next_retry_at,
            op_id
        ],
    )?;

    let retry_count: i32 = conn.query_row(
        "SELECT retry_count FROM pending_operations WHERE id = ?1",
        rusqlite::params![op_id],
        |row| row.get(0),
    )?;
    Ok(retry_count)
}

/// Remove all pending/in-flight StoreFlags operations for a specific message.
///
/// Used during conflict resolution: when a new local flag change supersedes
/// a stale pending operation (e.g. flag after a pending unflag from a previous
/// session), the old operations are deleted so only the latest intent is queued.
///
/// Returns the number of operations removed.
pub fn remove_pending_store_flags_for_message(
    conn: &Connection,
    account_id: &str,
    message_id: i64,
) -> Result<usize, DatabaseError> {
    // StoreFlags payloads contain a "message_id" JSON field.
    // We match on kind = 'store_flags' and use json_extract to find the message_id.
    let count = conn.execute(
        "DELETE FROM pending_operations
         WHERE account_id = ?1
           AND kind = 'store_flags'
           AND state IN ('pending', 'in_flight')
           AND json_extract(payload, '$.message_id') = ?2",
        rusqlite::params![account_id, message_id],
    )?;
    Ok(count)
}

/// Remove all pending/in-flight MoveMessage operations for a specific message.
///
/// Used during conflict resolution: when a new move supersedes a stale pending
/// move (e.g. move to Archive then move to Trash before the first executes),
/// the old operations are deleted so only the latest intent is queued.
///
/// Returns the number of operations removed.
pub fn remove_pending_move_for_message(
    conn: &Connection,
    account_id: &str,
    message_id: i64,
) -> Result<usize, DatabaseError> {
    let count = conn.execute(
        "DELETE FROM pending_operations
         WHERE account_id = ?1
           AND kind = 'move_message'
           AND state IN ('pending', 'in_flight')
           AND json_extract(payload, '$.message_id') = ?2",
        rusqlite::params![account_id, message_id],
    )?;
    Ok(count)
}

/// Remove any pending copy operations for a specific message (supersession).
pub fn remove_pending_copy_for_message(
    conn: &Connection,
    account_id: &str,
    message_id: i64,
) -> Result<usize, DatabaseError> {
    let count = conn.execute(
        "DELETE FROM pending_operations
         WHERE account_id = ?1
           AND kind = 'copy_message'
           AND state IN ('pending', 'in_flight')
           AND json_extract(payload, '$.message_id') = ?2",
        rusqlite::params![account_id, message_id],
    )?;
    Ok(count)
}

/// List distinct account IDs that have pending or in-flight operations.
///
/// Used by the connectivity service to determine which accounts need
/// their operation queues replayed after network connectivity is restored.
pub fn list_accounts_with_pending_ops(conn: &Connection) -> Result<Vec<String>, DatabaseError> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT account_id FROM pending_operations WHERE state IN ('pending', 'in_flight')",
    )?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    let mut accounts = Vec::new();
    for row in rows {
        accounts.push(row?);
    }
    Ok(accounts)
}

/// Load all operations for an account in all states (pending, in-flight, failed).
///
/// Used by the queue view UI (AC-16) to display the full operation queue
/// including failed operations with their error messages.
pub fn load_all_ops_for_account(
    conn: &Connection,
    account_id: &str,
) -> Result<Vec<PendingOperation>, DatabaseError> {
    let mut stmt = conn.prepare(
        "SELECT id, account_id, kind, payload, state, retry_count, last_error, created_at, next_retry_at
         FROM pending_operations
         WHERE account_id = ?1
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
            next_retry_at: row.get(8)?,
        })
    })?;

    let mut ops = Vec::new();
    for row in rows {
        ops.push(row?);
    }
    Ok(ops)
}

/// Load all operations across all accounts in all states.
///
/// Used by the queue view UI (AC-16) when showing the global operation queue.
pub fn load_all_ops(conn: &Connection) -> Result<Vec<PendingOperation>, DatabaseError> {
    let mut stmt = conn.prepare(
        "SELECT id, account_id, kind, payload, state, retry_count, last_error, created_at, next_retry_at
         FROM pending_operations
         ORDER BY id ASC",
    )?;

    let rows = stmt.query_map([], |row| {
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
            next_retry_at: row.get(8)?,
        })
    })?;

    let mut ops = Vec::new();
    for row in rows {
        ops.push(row?);
    }
    Ok(ops)
}

/// Retry a failed operation by resetting it to pending state (AC-18 manual retry).
///
/// Resets the state to `Pending`, clears the error, resets retry_count to 0,
/// and clears `next_retry_at` so it is immediately eligible for processing.
pub fn retry_failed_op(conn: &Connection, op_id: i64) -> Result<(), DatabaseError> {
    conn.execute(
        "UPDATE pending_operations
         SET state = ?1, retry_count = 0, last_error = NULL, next_retry_at = NULL
         WHERE id = ?2 AND state = ?3",
        rusqlite::params![
            OperationState::Pending.as_str(),
            op_id,
            OperationState::Failed.as_str()
        ],
    )?;
    Ok(())
}

/// Dismiss (cancel) a failed operation by removing it from the queue (AC-18).
pub fn dismiss_op(conn: &Connection, op_id: i64) -> Result<(), DatabaseError> {
    conn.execute(
        "DELETE FROM pending_operations WHERE id = ?1 AND state = ?2",
        rusqlite::params![op_id, OperationState::Failed.as_str()],
    )?;
    Ok(())
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

        let count = requeue_op(&conn, id, "network error", 0).unwrap();
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
    fn remove_pending_store_flags_for_message_deletes_matching() {
        let (_dir, conn) = setup_db();
        let payload_a = r#"{"message_id":42,"uid":100,"folder_name":"INBOX","new_flags":1}"#;
        let payload_b = r#"{"message_id":99,"uid":200,"folder_name":"INBOX","new_flags":2}"#;

        insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, payload_a).unwrap();
        insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, payload_b).unwrap();
        assert_eq!(load_pending_ops(&conn, "acct-1").unwrap().len(), 2);

        // Remove ops for message_id=42.
        let removed = remove_pending_store_flags_for_message(&conn, "acct-1", 42).unwrap();
        assert_eq!(removed, 1);

        // Only message_id=99 op remains.
        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1);
        assert!(ops[0].payload.contains("99"));
    }

    #[test]
    fn remove_pending_store_flags_ignores_non_store_flags_ops() {
        let (_dir, conn) = setup_db();
        let store_payload = r#"{"message_id":42,"uid":100,"folder_name":"INBOX","new_flags":1}"#;
        let move_payload =
            r#"{"message_id":42,"uid":100,"source_folder":"INBOX","destination_folder":"Archive"}"#;

        insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, store_payload).unwrap();
        insert_pending_op(&conn, "acct-1", &OperationKind::MoveMessage, move_payload).unwrap();

        let removed = remove_pending_store_flags_for_message(&conn, "acct-1", 42).unwrap();
        assert_eq!(removed, 1, "only StoreFlags op removed");
        assert_eq!(load_pending_ops(&conn, "acct-1").unwrap().len(), 1);
    }

    #[test]
    fn load_all_ops_for_account_includes_failed() {
        let (_dir, conn) = setup_db();
        let id1 = insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, "{}").unwrap();
        let id2 = insert_pending_op(&conn, "acct-1", &OperationKind::MoveMessage, "{}").unwrap();
        mark_failed(&conn, id2, "auth error").unwrap();

        let ops = load_all_ops_for_account(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 2);
        assert_eq!(ops[0].id, id1);
        assert_eq!(ops[0].state, OperationState::Pending);
        assert_eq!(ops[1].id, id2);
        assert_eq!(ops[1].state, OperationState::Failed);
        assert_eq!(ops[1].last_error.as_deref(), Some("auth error"));
    }

    #[test]
    fn load_all_ops_returns_all_accounts() {
        let (_dir, conn) = setup_db();
        conn.execute(
            "INSERT INTO accounts (id, display_name, protocol, host, port, encryption, auth_method, username, credential)
             VALUES ('acct-2', 'Test2', 'Imap', 'imap.example.com', 993, 'SslTls', 'Plain', 'user2', '')",
            [],
        ).unwrap();

        insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, "{}").unwrap();
        insert_pending_op(&conn, "acct-2", &OperationKind::MoveMessage, "{}").unwrap();

        let ops = load_all_ops(&conn).unwrap();
        assert_eq!(ops.len(), 2);
        assert_eq!(ops[0].account_id, "acct-1");
        assert_eq!(ops[1].account_id, "acct-2");
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

    #[test]
    fn requeue_with_backoff_sets_next_retry_at() {
        let (_dir, conn) = setup_db();
        let id = insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, "{}").unwrap();
        mark_in_flight(&conn, id).unwrap();

        // Requeue with 30 seconds backoff.
        let count = requeue_op(&conn, id, "timeout", 30).unwrap();
        assert_eq!(count, 1);

        // The op should NOT appear in load_pending_ops because next_retry_at is in the future.
        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert!(
            ops.is_empty(),
            "op with future next_retry_at should be skipped"
        );

        // But it should still appear in load_all_ops (for UI display).
        let all_ops = load_all_ops(&conn).unwrap();
        assert_eq!(all_ops.len(), 1);
        assert!(all_ops[0].next_retry_at.is_some());
    }

    #[test]
    fn retry_failed_op_resets_to_pending() {
        let (_dir, conn) = setup_db();
        let id = insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, "{}").unwrap();
        mark_failed(&conn, id, "auth error").unwrap();

        // Verify it's failed.
        let ops = load_all_ops(&conn).unwrap();
        assert_eq!(ops[0].state, OperationState::Failed);

        // Retry it.
        retry_failed_op(&conn, id).unwrap();

        // Should now be pending with reset state.
        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].state, OperationState::Pending);
        assert_eq!(ops[0].retry_count, 0);
        assert!(ops[0].last_error.is_none());
        assert!(ops[0].next_retry_at.is_none());
    }

    #[test]
    fn retry_failed_op_only_affects_failed_state() {
        let (_dir, conn) = setup_db();
        let id = insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, "{}").unwrap();
        // Op is in Pending state — retry_failed_op should not change it.
        retry_failed_op(&conn, id).unwrap();

        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops[0].state, OperationState::Pending);
        assert_eq!(ops[0].retry_count, 0); // unchanged
    }

    #[test]
    fn dismiss_op_removes_failed_operation() {
        let (_dir, conn) = setup_db();
        let id = insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, "{}").unwrap();
        mark_failed(&conn, id, "permanent error").unwrap();

        dismiss_op(&conn, id).unwrap();

        let ops = load_all_ops(&conn).unwrap();
        assert!(ops.is_empty());
    }

    #[test]
    fn dismiss_op_only_affects_failed_state() {
        let (_dir, conn) = setup_db();
        let id = insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, "{}").unwrap();
        // Op is Pending — dismiss should not remove it.
        dismiss_op(&conn, id).unwrap();

        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1);
    }

    #[test]
    fn failed_ops_do_not_block_pending_ops() {
        let (_dir, conn) = setup_db();
        let id1 = insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, "{}").unwrap();
        let _id2 = insert_pending_op(&conn, "acct-1", &OperationKind::MoveMessage, "{}").unwrap();

        // Fail the first operation.
        mark_failed(&conn, id1, "permanent error").unwrap();

        // load_pending_ops should only return the second (pending) operation.
        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].kind, OperationKind::MoveMessage);
    }
}
