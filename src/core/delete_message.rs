//! Business logic for deleting a message (move to Trash).
//!
//! "Delete" always means "move to the account's configured Trash folder".
//! This reuses the move pipeline from `move_message`.

use rusqlite::Connection;

use crate::core::account::FolderRole;
use crate::core::move_message::{move_message, MoveMessageError};
use crate::services::database::DatabaseError;
use crate::services::folder_store;
use crate::services::message_store;

/// Error type for delete-message operations.
#[derive(Debug, thiserror::Error)]
pub enum DeleteMessageError {
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("message not found: {0}")]
    NotFound(i64),
    #[error("no trash folder configured for account: {0}")]
    NoTrashFolder(String),
    #[error("message is already in trash")]
    AlreadyInTrash,
    #[error("move failed: {0}")]
    Move(#[from] MoveMessageError),
}

/// Result of a successful delete-message operation.
#[derive(Debug)]
pub struct DeleteMessageResult {
    /// The pending operation id that was enqueued (for the underlying move).
    pub operation_id: i64,
    /// The name of the Trash folder the message was moved to.
    pub trash_folder: String,
}

/// Delete a message by moving it to the account's Trash folder.
///
/// 1. Loads the message to get the account_id.
/// 2. Looks up the Trash folder for the account.
/// 3. Checks the message is not already in Trash.
/// 4. Delegates to `move_message` to perform the actual move.
///
/// The caller is responsible for notifying the sync engine after this
/// returns (e.g. via `SyncEngineHandle::notify_account`).
pub fn delete_message(
    conn: &Connection,
    message_id: i64,
    source_folder: &str,
) -> Result<DeleteMessageResult, DeleteMessageError> {
    let msg = message_store::load_message(conn, message_id)?
        .ok_or(DeleteMessageError::NotFound(message_id))?;

    let trash_folder = folder_store::folder_name_by_role(conn, &msg.account_id, FolderRole::Trash)?
        .ok_or_else(|| DeleteMessageError::NoTrashFolder(msg.account_id.clone()))?;

    if source_folder == trash_folder {
        return Err(DeleteMessageError::AlreadyInTrash);
    }

    let result = move_message(conn, message_id, source_folder, &trash_folder)?;

    Ok(DeleteMessageResult {
        operation_id: result.operation_id,
        trash_folder,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::imap_check::ImapFolder;
    use crate::core::message::NewMessage;
    use crate::core::pending_operation::MoveMessagePayload;
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
    fn delete_moves_message_to_trash() {
        let (_dir, conn) = setup_db();
        let inbox_id = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let trash_id = find_folder_id(&conn, "acct-1", "Trash").unwrap().unwrap();
        let msg = make_message(10);
        let mid = insert_message(&conn, &msg, inbox_id).unwrap();

        let result = delete_message(&conn, mid, "INBOX").unwrap();
        assert!(result.operation_id > 0);
        assert_eq!(result.trash_folder, "Trash");

        // Message should be in Trash, not INBOX.
        let in_inbox: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM message_folders WHERE message_id = ?1 AND folder_id = ?2",
                rusqlite::params![mid, inbox_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(in_inbox, 0, "message should be removed from INBOX");

        let in_trash: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM message_folders WHERE message_id = ?1 AND folder_id = ?2",
                rusqlite::params![mid, trash_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(in_trash, 1, "message should be in Trash");
    }

    #[test]
    fn delete_enqueues_move_operation() {
        let (_dir, conn) = setup_db();
        let inbox_id = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let msg = make_message(11);
        let mid = insert_message(&conn, &msg, inbox_id).unwrap();

        delete_message(&conn, mid, "INBOX").unwrap();

        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1);
        let payload: MoveMessagePayload = serde_json::from_str(&ops[0].payload).unwrap();
        assert_eq!(payload.message_id, mid);
        assert_eq!(payload.source_folder, "INBOX");
        assert_eq!(payload.destination_folder, "Trash");
    }

    #[test]
    fn delete_preserves_message_data() {
        let (_dir, conn) = setup_db();
        let inbox_id = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let msg = make_message(12);
        let mid = insert_message(&conn, &msg, inbox_id).unwrap();

        delete_message(&conn, mid, "INBOX").unwrap();

        let loaded = load_message(&conn, mid).unwrap().unwrap();
        assert_eq!(loaded.uid, 12);
        assert_eq!(loaded.subject.as_deref(), Some("Test"));
    }

    #[test]
    fn delete_already_in_trash_returns_error() {
        let (_dir, conn) = setup_db();
        let trash_id = find_folder_id(&conn, "acct-1", "Trash").unwrap().unwrap();
        let msg = make_message(13);
        let mid = insert_message(&conn, &msg, trash_id).unwrap();

        let result = delete_message(&conn, mid, "Trash");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DeleteMessageError::AlreadyInTrash
        ));
    }

    #[test]
    fn delete_message_not_found_returns_error() {
        let (_dir, conn) = setup_db();

        let result = delete_message(&conn, 99999, "INBOX");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DeleteMessageError::NotFound(99999)
        ));
    }

    #[test]
    fn delete_no_trash_folder_returns_error() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_and_migrate(&db_path).unwrap();
        conn.execute(
            "INSERT INTO accounts (id, display_name, protocol, host, port, encryption, auth_method, username, credential)
             VALUES ('acct-2', 'Test', 'Imap', 'imap.example.com', 993, 'SslTls', 'Plain', 'user', '')",
            [],
        ).unwrap();
        // Only INBOX, no Trash folder.
        let folders = vec![ImapFolder {
            name: "INBOX".to_string(),
            attributes: "".to_string(),
            role: None,
        }];
        replace_folders(&conn, "acct-2", &folders).unwrap();

        let inbox_id = find_folder_id(&conn, "acct-2", "INBOX").unwrap().unwrap();
        let mut msg = make_message(14);
        msg.account_id = "acct-2".to_string();
        let mid = insert_message(&conn, &msg, inbox_id).unwrap();

        let result = delete_message(&conn, mid, "INBOX");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DeleteMessageError::NoTrashFolder(_)
        ));
    }
}
