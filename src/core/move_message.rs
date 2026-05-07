//! Business logic for moving a message between folders (local -> server).
//!
//! Updates the local folder assignment immediately and enqueues a MoveMessage
//! operation for server synchronization.

use rusqlite::Connection;

use crate::core::pending_operation::{MoveMessagePayload, OperationKind};
use crate::services::database::DatabaseError;
use crate::services::folder_store;
use crate::services::message_store;
use crate::services::pending_ops_store;

/// Error type for move-message operations.
#[derive(Debug, thiserror::Error)]
pub enum MoveMessageError {
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("message not found: {0}")]
    NotFound(i64),
    #[error("source folder not found: {0}")]
    SourceFolderNotFound(String),
    #[error("destination folder not found: {0}")]
    DestinationFolderNotFound(String),
    #[error("payload serialization error: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("source folder is read-only: {0}")]
    ReadOnlyFolder(String),
}

/// Result of a successful move-message operation.
#[derive(Debug)]
pub struct MoveMessageResult {
    /// The pending operation id that was enqueued.
    pub operation_id: i64,
}

/// Move a message from one folder to another locally and enqueue a MoveMessage
/// operation for server synchronization.
///
/// 1. Checks whether the source folder is read-only.
/// 2. Loads the message to get current metadata.
/// 3. Updates the local folder assignment (message_folders table).
/// 4. Supersedes any stale pending move operations for this message.
/// 5. Persists a MoveMessage operation to the pending queue.
///
/// The caller is responsible for notifying the sync engine after this
/// returns (e.g. via `SyncEngineHandle::notify_account`).
pub fn move_message(
    conn: &Connection,
    message_id: i64,
    source_folder: &str,
    destination_folder: &str,
) -> Result<MoveMessageResult, MoveMessageError> {
    let msg = message_store::load_message(conn, message_id)?
        .ok_or(MoveMessageError::NotFound(message_id))?;

    // Check read-only before attempting any mutation.
    if folder_store::is_folder_read_only(conn, &msg.account_id, source_folder)? {
        return Err(MoveMessageError::ReadOnlyFolder(source_folder.to_string()));
    }

    // Look up folder IDs.
    let src_folder_id = message_store::find_folder_id(conn, &msg.account_id, source_folder)?
        .ok_or_else(|| MoveMessageError::SourceFolderNotFound(source_folder.to_string()))?;
    let dst_folder_id = message_store::find_folder_id(conn, &msg.account_id, destination_folder)?
        .ok_or_else(|| {
        MoveMessageError::DestinationFolderNotFound(destination_folder.to_string())
    })?;

    // Update local folder assignment immediately.
    message_store::move_message_to_folder(conn, message_id, src_folder_id, dst_folder_id)?;

    // Supersede any stale pending move operations for this message.
    pending_ops_store::remove_pending_move_for_message(conn, &msg.account_id, message_id)?;

    // Enqueue the operation for server execution.
    let payload = MoveMessagePayload {
        message_id,
        uid: msg.uid,
        source_folder: source_folder.to_string(),
        destination_folder: destination_folder.to_string(),
    };
    let payload_json = serde_json::to_string(&payload)?;
    let op_id = pending_ops_store::insert_pending_op(
        conn,
        &msg.account_id,
        &OperationKind::MoveMessage,
        &payload_json,
    )?;

    Ok(MoveMessageResult {
        operation_id: op_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::imap_check::ImapFolder;
    use crate::core::message::NewMessage;
    use crate::core::pending_operation::MoveMessagePayload;
    use crate::services::database::open_and_migrate;
    use crate::services::folder_store::{replace_folders, set_folder_read_only};
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
                name: "Archive".to_string(),
                attributes: "".to_string(),
                role: None,
            },
            ImapFolder {
                name: "Trash".to_string(),
                attributes: "\\Trash".to_string(),
                role: Some(crate::core::account::FolderRole::Trash),
            },
            ImapFolder {
                name: "Junk".to_string(),
                attributes: "\\Junk".to_string(),
                role: Some(crate::core::account::FolderRole::Junk),
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
        }
    }

    #[test]
    fn move_creates_operation_and_updates_folder() {
        let (_dir, conn) = setup_db();
        let inbox_id = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let archive_id = find_folder_id(&conn, "acct-1", "Archive").unwrap().unwrap();
        let msg = make_message(50);
        let mid = insert_message(&conn, &msg, inbox_id).unwrap();

        let result = move_message(&conn, mid, "INBOX", "Archive").unwrap();
        assert!(result.operation_id > 0);

        // Message should now be in Archive, not INBOX.
        let in_inbox: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM message_folders WHERE message_id = ?1 AND folder_id = ?2",
                rusqlite::params![mid, inbox_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(in_inbox, 0, "message should be removed from INBOX");

        let in_archive: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM message_folders WHERE message_id = ?1 AND folder_id = ?2",
                rusqlite::params![mid, archive_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(in_archive, 1, "message should be in Archive");

        // Pending operation should exist.
        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1);
        let payload: MoveMessagePayload = serde_json::from_str(&ops[0].payload).unwrap();
        assert_eq!(payload.message_id, mid);
        assert_eq!(payload.uid, 50);
        assert_eq!(payload.source_folder, "INBOX");
        assert_eq!(payload.destination_folder, "Archive");
    }

    #[test]
    fn move_not_found_returns_error() {
        let (_dir, conn) = setup_db();
        let result = move_message(&conn, 99999, "INBOX", "Archive");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MoveMessageError::NotFound(99999)
        ));
    }

    #[test]
    fn move_from_read_only_folder_rejected() {
        let (_dir, conn) = setup_db();
        let inbox_id = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let msg = make_message(51);
        let mid = insert_message(&conn, &msg, inbox_id).unwrap();

        set_folder_read_only(&conn, "acct-1", "INBOX", true).unwrap();

        let result = move_message(&conn, mid, "INBOX", "Archive");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MoveMessageError::ReadOnlyFolder(_)
        ));

        // No operation should have been enqueued.
        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert!(ops.is_empty());
    }

    #[test]
    fn move_destination_not_found_returns_error() {
        let (_dir, conn) = setup_db();
        let inbox_id = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let msg = make_message(52);
        let mid = insert_message(&conn, &msg, inbox_id).unwrap();

        let result = move_message(&conn, mid, "INBOX", "NonExistent");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MoveMessageError::DestinationFolderNotFound(_)
        ));
    }

    #[test]
    fn move_supersedes_stale_pending_move() {
        let (_dir, conn) = setup_db();
        let inbox_id = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let msg = make_message(53);
        let mid = insert_message(&conn, &msg, inbox_id).unwrap();

        // First move: INBOX -> Archive
        move_message(&conn, mid, "INBOX", "Archive").unwrap();
        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1);

        // Second move: Archive -> Trash (supersedes the first)
        move_message(&conn, mid, "Archive", "Trash").unwrap();
        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1, "stale move removed, only new move remains");

        let payload: MoveMessagePayload = serde_json::from_str(&ops[0].payload).unwrap();
        assert_eq!(payload.destination_folder, "Trash");
    }

    #[test]
    fn move_does_not_affect_other_messages() {
        let (_dir, conn) = setup_db();
        let inbox_id = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let msg_a = make_message(54);
        let mid_a = insert_message(&conn, &msg_a, inbox_id).unwrap();
        let mut msg_b = make_message(55);
        msg_b.content_hash = "hash2".to_string();
        let mid_b = insert_message(&conn, &msg_b, inbox_id).unwrap();

        // Move both messages.
        move_message(&conn, mid_a, "INBOX", "Archive").unwrap();
        move_message(&conn, mid_b, "INBOX", "Trash").unwrap();
        assert_eq!(load_pending_ops(&conn, "acct-1").unwrap().len(), 2);

        // Re-move message A — should only supersede A's op, not B's.
        move_message(&conn, mid_a, "Archive", "Trash").unwrap();
        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 2, "B's op is untouched, A's old op replaced");
    }

    #[test]
    fn move_message_data_preserved() {
        let (_dir, conn) = setup_db();
        let inbox_id = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let msg = make_message(56);
        let mid = insert_message(&conn, &msg, inbox_id).unwrap();

        move_message(&conn, mid, "INBOX", "Archive").unwrap();

        // Message data should still be loadable.
        let loaded = load_message(&conn, mid).unwrap().unwrap();
        assert_eq!(loaded.uid, 56);
        assert_eq!(loaded.subject.as_deref(), Some("Test"));
    }
}
