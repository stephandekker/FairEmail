//! Business logic for setting and removing message flags (local -> server).
//!
//! Generalises the mark-read pipeline to support any IMAP flag change:
//! set/remove Seen, Flagged, Answered, etc.

use rusqlite::Connection;

use crate::core::pending_operation::{OperationKind, StoreFlagsPayload};
use crate::services::database::DatabaseError;
use crate::services::folder_store;
use crate::services::message_store;
use crate::services::pending_ops_store;

/// Whether to add or remove a flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlagAction {
    /// Set (add) the flag on the message.
    Set,
    /// Remove the flag from the message.
    Remove,
}

/// Error type for flag-change operations.
#[derive(Debug, thiserror::Error)]
pub enum SetFlagsError {
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("message not found: {0}")]
    NotFound(i64),
    #[error("payload serialization error: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("folder is read-only: {0}")]
    ReadOnlyFolder(String),
}

/// Result of a successful flag-change operation.
#[derive(Debug)]
pub struct SetFlagsResult {
    /// The pending operation id that was enqueued.
    pub operation_id: i64,
    /// The new flags value on the message.
    pub new_flags: u32,
}

/// Change a flag on a message locally and enqueue a StoreFlags operation
/// for server synchronization.
///
/// 1. Checks whether the folder is read-only.
/// 2. Loads the message to get current flags and metadata.
/// 3. Applies the flag change locally and marks flags as pending sync.
/// 4. Persists a StoreFlags operation to the pending queue.
///
/// The caller is responsible for notifying the sync engine after this
/// returns (e.g. via `SyncEngineHandle::notify_account`).
pub fn set_message_flag(
    conn: &Connection,
    message_id: i64,
    folder_name: &str,
    flag: u32,
    action: FlagAction,
) -> Result<SetFlagsResult, SetFlagsError> {
    let msg = message_store::load_message(conn, message_id)?
        .ok_or(SetFlagsError::NotFound(message_id))?;

    // Check read-only before attempting any mutation.
    if folder_store::is_folder_read_only(conn, &msg.account_id, folder_name)? {
        return Err(SetFlagsError::ReadOnlyFolder(folder_name.to_string()));
    }

    let new_flags = match action {
        FlagAction::Set => msg.flags | flag,
        FlagAction::Remove => msg.flags & !flag,
    };

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

    Ok(SetFlagsResult {
        operation_id: op_id,
        new_flags,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::imap_check::ImapFolder;
    use crate::core::message::{NewMessage, FLAG_ANSWERED, FLAG_FLAGGED, FLAG_SEEN};
    use crate::core::pending_operation::StoreFlagsPayload;
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

    // --- Flagging ---

    #[test]
    fn set_flagged_creates_operation_and_updates_local() {
        let (_dir, conn) = setup_db();
        let fid = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let msg = make_message(50, 0);
        let mid = insert_message(&conn, &msg, fid).unwrap();

        let result = set_message_flag(&conn, mid, "INBOX", FLAG_FLAGGED, FlagAction::Set).unwrap();
        assert_eq!(result.new_flags, FLAG_FLAGGED);

        let loaded = load_message(&conn, mid).unwrap().unwrap();
        assert_eq!(loaded.flags, FLAG_FLAGGED);
        assert!(loaded.flags_pending_sync);

        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1);
        let payload: StoreFlagsPayload = serde_json::from_str(&ops[0].payload).unwrap();
        assert_eq!(payload.new_flags, FLAG_FLAGGED);
        assert_eq!(payload.uid, 50);
    }

    #[test]
    fn set_flagged_preserves_existing_flags() {
        let (_dir, conn) = setup_db();
        let fid = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let msg = make_message(51, FLAG_SEEN);
        let mid = insert_message(&conn, &msg, fid).unwrap();

        let result = set_message_flag(&conn, mid, "INBOX", FLAG_FLAGGED, FlagAction::Set).unwrap();
        assert_eq!(result.new_flags, FLAG_SEEN | FLAG_FLAGGED);
    }

    // --- Unflagging ---

    #[test]
    fn remove_flagged_creates_operation() {
        let (_dir, conn) = setup_db();
        let fid = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let msg = make_message(52, FLAG_FLAGGED | FLAG_SEEN);
        let mid = insert_message(&conn, &msg, fid).unwrap();

        let result =
            set_message_flag(&conn, mid, "INBOX", FLAG_FLAGGED, FlagAction::Remove).unwrap();
        assert_eq!(result.new_flags, FLAG_SEEN);

        let loaded = load_message(&conn, mid).unwrap().unwrap();
        assert_eq!(loaded.flags, FLAG_SEEN);
    }

    #[test]
    fn remove_flag_not_present_is_idempotent() {
        let (_dir, conn) = setup_db();
        let fid = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let msg = make_message(53, FLAG_SEEN);
        let mid = insert_message(&conn, &msg, fid).unwrap();

        let result =
            set_message_flag(&conn, mid, "INBOX", FLAG_FLAGGED, FlagAction::Remove).unwrap();
        assert_eq!(result.new_flags, FLAG_SEEN);

        // Operation still enqueued to ensure server state is consistent.
        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1);
    }

    // --- Answered ---

    #[test]
    fn set_answered_creates_operation() {
        let (_dir, conn) = setup_db();
        let fid = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let msg = make_message(54, FLAG_SEEN);
        let mid = insert_message(&conn, &msg, fid).unwrap();

        let result = set_message_flag(&conn, mid, "INBOX", FLAG_ANSWERED, FlagAction::Set).unwrap();
        assert_eq!(result.new_flags, FLAG_SEEN | FLAG_ANSWERED);

        let loaded = load_message(&conn, mid).unwrap().unwrap();
        assert_eq!(loaded.flags, FLAG_SEEN | FLAG_ANSWERED);
        assert!(loaded.flags_pending_sync);
    }

    // --- Mark unread (remove Seen) ---

    #[test]
    fn remove_seen_marks_unread() {
        let (_dir, conn) = setup_db();
        let fid = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let msg = make_message(55, FLAG_SEEN | FLAG_FLAGGED);
        let mid = insert_message(&conn, &msg, fid).unwrap();

        let result = set_message_flag(&conn, mid, "INBOX", FLAG_SEEN, FlagAction::Remove).unwrap();
        assert_eq!(result.new_flags, FLAG_FLAGGED);

        let loaded = load_message(&conn, mid).unwrap().unwrap();
        assert_eq!(loaded.flags, FLAG_FLAGGED);
        assert!(loaded.flags_pending_sync);
    }

    // --- Read-only folder ---

    #[test]
    fn read_only_folder_rejects_flag_change() {
        let (_dir, conn) = setup_db();
        let fid = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let msg = make_message(56, 0);
        let mid = insert_message(&conn, &msg, fid).unwrap();

        // Mark the folder as read-only.
        set_folder_read_only(&conn, "acct-1", "INBOX", true).unwrap();

        let result = set_message_flag(&conn, mid, "INBOX", FLAG_FLAGGED, FlagAction::Set);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SetFlagsError::ReadOnlyFolder(_)
        ));

        // No operation should have been enqueued.
        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert!(ops.is_empty());

        // Local flags should be unchanged.
        let loaded = load_message(&conn, mid).unwrap().unwrap();
        assert_eq!(loaded.flags, 0);
    }

    // --- Not found ---

    #[test]
    fn not_found_returns_error() {
        let (_dir, conn) = setup_db();

        let result = set_message_flag(&conn, 99999, "INBOX", FLAG_FLAGGED, FlagAction::Set);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SetFlagsError::NotFound(99999)
        ));
    }
}
