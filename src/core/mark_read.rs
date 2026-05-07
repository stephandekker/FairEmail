//! Business logic for marking a message as read (local → server).

use rusqlite::Connection;

use crate::core::message::FLAG_SEEN;
use crate::core::pending_operation::{OperationKind, StoreFlagsPayload};
use crate::services::database::DatabaseError;
use crate::services::message_store;
use crate::services::pending_ops_store;

/// Error type for mark-read operations.
#[derive(Debug, thiserror::Error)]
pub enum MarkReadError {
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("message not found: {0}")]
    NotFound(i64),
    #[error("payload serialization error: {0}")]
    Serialize(#[from] serde_json::Error),
}

/// Result of marking a message as read.
#[derive(Debug)]
pub struct MarkReadResult {
    /// The pending operation id that was enqueued.
    pub operation_id: i64,
    /// The new flags value on the message.
    pub new_flags: u32,
}

/// Mark a message as read locally and enqueue a StoreFlags operation
/// for server synchronization.
///
/// 1. Loads the message to get current flags and metadata.
/// 2. Sets the SEEN flag locally and marks flags as pending sync.
/// 3. Persists a StoreFlags operation to the pending queue.
///
/// The caller is responsible for notifying the sync engine after this
/// returns (e.g. via `SyncEngineHandle::notify_account`).
pub fn mark_message_read(
    conn: &Connection,
    message_id: i64,
    folder_name: &str,
) -> Result<MarkReadResult, MarkReadError> {
    let msg = message_store::load_message(conn, message_id)?
        .ok_or(MarkReadError::NotFound(message_id))?;

    let new_flags = msg.flags | FLAG_SEEN;

    // Update flags locally and mark as pending sync.
    message_store::update_message_flags_pending(conn, message_id, new_flags)?;

    // Enqueue the operation for server execution.
    let payload = StoreFlagsPayload {
        message_id,
        uid: msg.uid,
        folder_name: folder_name.to_string(),
        new_flags,
    };
    let payload_json = serde_json::to_string(&payload)?;
    let op_id = pending_ops_store::insert_pending_op(
        conn,
        &msg.account_id,
        &OperationKind::StoreFlags,
        &payload_json,
    )?;

    Ok(MarkReadResult {
        operation_id: op_id,
        new_flags,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::imap_check::ImapFolder;
    use crate::core::message::{NewMessage, FLAG_FLAGGED, FLAG_SEEN};
    use crate::services::database::open_and_migrate;
    use crate::services::folder_store::replace_folders;
    use crate::services::message_store::{find_folder_id, insert_message, load_message};
    use crate::services::pending_ops_store::load_pending_ops;
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
        let folders = vec![ImapFolder {
            name: "INBOX".to_string(),
            attributes: "".to_string(),
            role: None,
        }];
        replace_folders(&conn, "acct-1", &folders).unwrap();
        (dir, conn)
    }

    fn make_message(uid: u32, flags: u32) -> NewMessage {
        NewMessage {
            account_id: "acct-1".to_string(),
            uid,
            modseq: None,
            message_id: None,
            in_reply_to: None,
            references_header: None,
            from_addresses: Some("test@example.com".to_string()),
            to_addresses: None,
            cc_addresses: None,
            bcc_addresses: None,
            subject: Some("Test".to_string()),
            date_received: Some(1700000000),
            date_sent: None,
            flags,
            size: 100,
            content_hash: "hash1".to_string(),
            body_text: None,
            thread_id: None,
            server_thread_id: None,
        }
    }

    #[test]
    fn mark_read_sets_seen_flag_and_enqueues_operation() {
        let (_dir, conn) = setup_db();
        let fid = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();

        // Insert an unread message.
        let msg = make_message(42, 0);
        let mid = insert_message(&conn, &msg, fid).unwrap();

        // Mark as read.
        let result = mark_message_read(&conn, mid, "INBOX").unwrap();
        assert_eq!(result.new_flags, FLAG_SEEN);

        // Verify local flags updated.
        let loaded = load_message(&conn, mid).unwrap().unwrap();
        assert_eq!(loaded.flags, FLAG_SEEN);
        assert!(loaded.flags_pending_sync);

        // Verify pending operation enqueued.
        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].kind, OperationKind::StoreFlags);
        let payload: StoreFlagsPayload = serde_json::from_str(&ops[0].payload).unwrap();
        assert_eq!(payload.message_id, mid);
        assert_eq!(payload.uid, 42);
        assert_eq!(payload.folder_name, "INBOX");
        assert_eq!(payload.new_flags, FLAG_SEEN);
    }

    #[test]
    fn mark_read_preserves_existing_flags() {
        let (_dir, conn) = setup_db();
        let fid = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();

        // Insert a flagged but unread message.
        let msg = make_message(43, FLAG_FLAGGED);
        let mid = insert_message(&conn, &msg, fid).unwrap();

        let result = mark_message_read(&conn, mid, "INBOX").unwrap();
        assert_eq!(result.new_flags, FLAG_SEEN | FLAG_FLAGGED);

        let loaded = load_message(&conn, mid).unwrap().unwrap();
        assert_eq!(loaded.flags, FLAG_SEEN | FLAG_FLAGGED);
    }

    #[test]
    fn mark_read_already_read_is_idempotent() {
        let (_dir, conn) = setup_db();
        let fid = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();

        // Insert an already-read message.
        let msg = make_message(44, FLAG_SEEN);
        let mid = insert_message(&conn, &msg, fid).unwrap();

        let result = mark_message_read(&conn, mid, "INBOX").unwrap();
        assert_eq!(result.new_flags, FLAG_SEEN);

        // Still enqueues the operation (server state might be stale).
        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1);
    }

    #[test]
    fn mark_read_not_found_returns_error() {
        let (_dir, conn) = setup_db();

        let result = mark_message_read(&conn, 99999, "INBOX");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MarkReadError::NotFound(99999)
        ));
    }

    #[test]
    fn flags_confirmed_clears_pending_sync() {
        let (_dir, conn) = setup_db();
        let fid = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();

        let msg = make_message(45, 0);
        let mid = insert_message(&conn, &msg, fid).unwrap();

        // Mark as read (pending sync).
        mark_message_read(&conn, mid, "INBOX").unwrap();
        let loaded = load_message(&conn, mid).unwrap().unwrap();
        assert!(loaded.flags_pending_sync);

        // Confirm flags from server.
        message_store::mark_flags_confirmed(&conn, mid).unwrap();
        let loaded = load_message(&conn, mid).unwrap().unwrap();
        assert!(!loaded.flags_pending_sync);
        assert_eq!(loaded.flags, FLAG_SEEN);
    }
}
