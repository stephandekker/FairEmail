//! Business logic for copying a message to another folder (local -> server).
//!
//! Adds the message to the destination folder locally and enqueues a CopyMessage
//! operation for server synchronization. The original message remains in the
//! source folder.

use rusqlite::Connection;

use crate::core::pending_operation::{CopyMessagePayload, OperationKind};
use crate::services::database::DatabaseError;
use crate::services::message_store;
use crate::services::pending_ops_store;

/// Error type for copy-message operations.
#[derive(Debug, thiserror::Error)]
pub enum CopyMessageError {
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("message not found: {0}")]
    NotFound(i64),
    #[error("destination folder not found: {0}")]
    DestinationFolderNotFound(String),
    #[error("payload serialization error: {0}")]
    Serialize(#[from] serde_json::Error),
}

/// Result of a successful copy-message operation.
#[derive(Debug)]
pub struct CopyMessageResult {
    /// The pending operation id that was enqueued.
    pub operation_id: i64,
}

/// Copy a message to another folder locally and enqueue a CopyMessage
/// operation for server synchronization.
///
/// 1. Loads the message to get current metadata.
/// 2. Validates the destination folder exists.
/// 3. Adds the destination folder link locally (message_folders table).
/// 4. Persists a CopyMessage operation to the pending queue.
///
/// The original message remains in the source folder (unchanged).
/// The caller is responsible for notifying the sync engine after this
/// returns (e.g. via `SyncEngineHandle::notify_account`).
pub fn copy_message(
    conn: &Connection,
    message_id: i64,
    source_folder: &str,
    destination_folder: &str,
) -> Result<CopyMessageResult, CopyMessageError> {
    let msg = message_store::load_message(conn, message_id)?
        .ok_or(CopyMessageError::NotFound(message_id))?;

    // Look up destination folder ID.
    let dst_folder_id = message_store::find_folder_id(conn, &msg.account_id, destination_folder)?
        .ok_or_else(|| {
        CopyMessageError::DestinationFolderNotFound(destination_folder.to_string())
    })?;

    // Add the message to the destination folder locally.
    message_store::copy_message_to_folder(conn, message_id, dst_folder_id)?;

    // Enqueue the operation for server execution.
    let payload = CopyMessagePayload {
        message_id,
        uid: msg.uid,
        source_folder: source_folder.to_string(),
        destination_folder: destination_folder.to_string(),
    };
    let payload_json = serde_json::to_string(&payload)?;
    let op_id = pending_ops_store::insert_pending_op(
        conn,
        &msg.account_id,
        &OperationKind::CopyMessage,
        &payload_json,
    )?;

    Ok(CopyMessageResult {
        operation_id: op_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::imap_check::ImapFolder;
    use crate::core::message::NewMessage;
    use crate::core::pending_operation::CopyMessagePayload;
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
                name: "Archive".to_string(),
                attributes: "".to_string(),
                role: None,
            },
            ImapFolder {
                name: "Trash".to_string(),
                attributes: "\\Trash".to_string(),
                role: Some(crate::core::account::FolderRole::Trash),
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
    fn copy_creates_operation_and_adds_to_destination() {
        let (_dir, conn) = setup_db();
        let inbox_id = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let archive_id = find_folder_id(&conn, "acct-1", "Archive").unwrap().unwrap();
        let msg = make_message(50);
        let mid = insert_message(&conn, &msg, inbox_id).unwrap();

        let result = copy_message(&conn, mid, "INBOX", "Archive").unwrap();
        assert!(result.operation_id > 0);

        // Message should be in BOTH INBOX and Archive.
        let in_inbox: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM message_folders WHERE message_id = ?1 AND folder_id = ?2",
                rusqlite::params![mid, inbox_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(in_inbox, 1, "message should remain in INBOX");

        let in_archive: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM message_folders WHERE message_id = ?1 AND folder_id = ?2",
                rusqlite::params![mid, archive_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(in_archive, 1, "message should also be in Archive");

        // Pending operation should exist.
        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1);
        let payload: CopyMessagePayload = serde_json::from_str(&ops[0].payload).unwrap();
        assert_eq!(payload.message_id, mid);
        assert_eq!(payload.uid, 50);
        assert_eq!(payload.source_folder, "INBOX");
        assert_eq!(payload.destination_folder, "Archive");
    }

    #[test]
    fn copy_not_found_returns_error() {
        let (_dir, conn) = setup_db();
        let result = copy_message(&conn, 99999, "INBOX", "Archive");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CopyMessageError::NotFound(99999)
        ));
    }

    #[test]
    fn copy_destination_not_found_returns_error() {
        let (_dir, conn) = setup_db();
        let inbox_id = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let msg = make_message(52);
        let mid = insert_message(&conn, &msg, inbox_id).unwrap();

        let result = copy_message(&conn, mid, "INBOX", "NonExistent");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CopyMessageError::DestinationFolderNotFound(_)
        ));
    }

    #[test]
    fn copy_preserves_original_message_data() {
        let (_dir, conn) = setup_db();
        let inbox_id = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let msg = make_message(56);
        let mid = insert_message(&conn, &msg, inbox_id).unwrap();

        copy_message(&conn, mid, "INBOX", "Archive").unwrap();

        // Message data should still be loadable and unchanged.
        let loaded = load_message(&conn, mid).unwrap().unwrap();
        assert_eq!(loaded.uid, 56);
        assert_eq!(loaded.subject.as_deref(), Some("Test"));
    }

    #[test]
    fn copy_does_not_affect_other_messages() {
        let (_dir, conn) = setup_db();
        let inbox_id = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let msg_a = make_message(54);
        let mid_a = insert_message(&conn, &msg_a, inbox_id).unwrap();
        let mut msg_b = make_message(55);
        msg_b.content_hash = "hash2".to_string();
        let mid_b = insert_message(&conn, &msg_b, inbox_id).unwrap();

        // Copy message A to Archive.
        copy_message(&conn, mid_a, "INBOX", "Archive").unwrap();

        // Message B should still only be in INBOX.
        let archive_id = find_folder_id(&conn, "acct-1", "Archive").unwrap().unwrap();
        let b_in_archive: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM message_folders WHERE message_id = ?1 AND folder_id = ?2",
                rusqlite::params![mid_b, archive_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(b_in_archive, 0, "message B should not be in Archive");

        let b_in_inbox: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM message_folders WHERE message_id = ?1 AND folder_id = ?2",
                rusqlite::params![mid_b, inbox_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(b_in_inbox, 1, "message B should remain in INBOX");
    }
}
