//! Business logic for permanently deleting (expunging) a message.
//!
//! "Permanent delete" sets the \Deleted flag locally and enqueues a
//! DeleteMessage pending operation for server execution. The message
//! is removed from the local database immediately.

use rusqlite::Connection;

use crate::core::pending_operation::{DeleteMessagePayload, OperationKind};
use crate::services::database::DatabaseError;
use crate::services::message_store;
use crate::services::pending_ops_store;

/// Error type for permanent-delete operations.
#[derive(Debug, thiserror::Error)]
pub enum PermanentDeleteError {
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("message not found: {0}")]
    NotFound(i64),
    #[error("payload serialization error: {0}")]
    Serialize(#[from] serde_json::Error),
}

/// Result of a successful permanent-delete operation.
#[derive(Debug)]
pub struct PermanentDeleteResult {
    /// The pending operation id that was enqueued.
    pub operation_id: i64,
    /// The content hash of the deleted message (for content-store cleanup).
    pub content_hash: Option<String>,
    /// Whether the content-store file should be deleted (no other references).
    pub should_delete_content: bool,
}

/// Permanently delete a message: remove it locally and enqueue a
/// DeleteMessage operation so the sync engine can STORE \Deleted + EXPUNGE
/// on the server.
///
/// The `folder_name` is the folder the message currently resides in on the
/// server (needed by the sync engine to SELECT the right mailbox).
///
/// The caller is responsible for:
/// - Notifying the sync engine (e.g. via `SyncEngineHandle::notify_account`).
/// - Deleting the `.eml` from the content store when `should_delete_content`
///   is true.
pub fn permanent_delete(
    conn: &Connection,
    message_id: i64,
    folder_name: &str,
) -> Result<PermanentDeleteResult, PermanentDeleteError> {
    let msg = message_store::load_message(conn, message_id)?
        .ok_or(PermanentDeleteError::NotFound(message_id))?;

    // Supersede any stale pending move or delete operations for this message.
    let _ = pending_ops_store::remove_pending_move_for_message(conn, &msg.account_id, message_id);

    // Enqueue the server-side operation.
    let payload = DeleteMessagePayload {
        message_id,
        uid: msg.uid,
        folder_name: folder_name.to_string(),
    };
    let payload_json = serde_json::to_string(&payload)?;
    let op_id = pending_ops_store::insert_pending_op(
        conn,
        &msg.account_id,
        &OperationKind::DeleteMessage,
        &payload_json,
    )?;

    // Remove the message from the local database.
    let delete_result = message_store::delete_message(conn, message_id)?;
    let (content_hash, should_delete_content) = match delete_result {
        Some((hash, last)) => (Some(hash), last),
        None => (None, false),
    };

    Ok(PermanentDeleteResult {
        operation_id: op_id,
        content_hash,
        should_delete_content,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::account::FolderRole;
    use crate::core::imap_check::ImapFolder;
    use crate::core::message::NewMessage;
    use crate::core::pending_operation::DeleteMessagePayload;
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
        let folders = vec![
            ImapFolder {
                name: "INBOX".to_string(),
                attributes: "".to_string(),
                role: None,
            },
            ImapFolder {
                name: "Trash".to_string(),
                attributes: "\\Trash".to_string(),
                role: Some(FolderRole::Trash),
            },
        ];
        replace_folders(&conn, "acct-1", &folders).unwrap();
        (dir, conn)
    }

    fn make_message(uid: u32) -> NewMessage {
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
            flags: 0,
            size: 100,
            content_hash: "hash1".to_string(),
            body_text: None,
            thread_id: None,
            server_thread_id: None,
            keywords: String::new(),
        }
    }

    #[test]
    fn permanent_delete_removes_message_locally() {
        let (_dir, conn) = setup_db();
        let trash_id = find_folder_id(&conn, "acct-1", "Trash").unwrap().unwrap();
        let msg = make_message(20);
        let mid = insert_message(&conn, &msg, trash_id).unwrap();

        let result = permanent_delete(&conn, mid, "Trash").unwrap();
        assert!(result.operation_id > 0);

        // Message should no longer exist.
        assert!(load_message(&conn, mid).unwrap().is_none());
    }

    #[test]
    fn permanent_delete_enqueues_delete_operation() {
        let (_dir, conn) = setup_db();
        let trash_id = find_folder_id(&conn, "acct-1", "Trash").unwrap().unwrap();
        let msg = make_message(21);
        let mid = insert_message(&conn, &msg, trash_id).unwrap();

        permanent_delete(&conn, mid, "Trash").unwrap();

        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].kind, OperationKind::DeleteMessage);

        let payload: DeleteMessagePayload = serde_json::from_str(&ops[0].payload).unwrap();
        assert_eq!(payload.message_id, mid);
        assert_eq!(payload.uid, 21);
        assert_eq!(payload.folder_name, "Trash");
    }

    #[test]
    fn permanent_delete_not_found_returns_error() {
        let (_dir, conn) = setup_db();
        let result = permanent_delete(&conn, 99999, "Trash");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PermanentDeleteError::NotFound(99999)
        ));
    }

    #[test]
    fn permanent_delete_returns_content_hash() {
        let (_dir, conn) = setup_db();
        let trash_id = find_folder_id(&conn, "acct-1", "Trash").unwrap().unwrap();
        let msg = make_message(22);
        let mid = insert_message(&conn, &msg, trash_id).unwrap();

        let result = permanent_delete(&conn, mid, "Trash").unwrap();
        assert_eq!(result.content_hash.as_deref(), Some("hash1"));
        assert!(result.should_delete_content);
    }

    #[test]
    fn permanent_delete_supersedes_pending_move() {
        let (_dir, conn) = setup_db();
        let inbox_id = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let msg = make_message(23);
        let mid = insert_message(&conn, &msg, inbox_id).unwrap();

        // First enqueue a move operation.
        crate::core::move_message::move_message(&conn, mid, "INBOX", "Trash").unwrap();
        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].kind, OperationKind::MoveMessage);

        // Now permanently delete — should supersede the move.
        permanent_delete(&conn, mid, "Trash").unwrap();
        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].kind, OperationKind::DeleteMessage);
    }

    #[test]
    fn permanent_delete_from_inbox_works() {
        let (_dir, conn) = setup_db();
        let inbox_id = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let msg = make_message(24);
        let mid = insert_message(&conn, &msg, inbox_id).unwrap();

        let result = permanent_delete(&conn, mid, "INBOX").unwrap();
        assert!(result.operation_id > 0);
        assert!(load_message(&conn, mid).unwrap().is_none());

        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        let payload: DeleteMessagePayload = serde_json::from_str(&ops[0].payload).unwrap();
        assert_eq!(payload.folder_name, "INBOX");
    }
}
