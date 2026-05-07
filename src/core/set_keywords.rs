//! Business logic for setting and removing custom IMAP keywords (local -> server).
//!
//! Extends the flag sync pipeline with keyword-specific handling and a
//! server capability check (FR-34): if the server does not support
//! user-defined flags, the keyword is stored locally only.

use rusqlite::Connection;

use crate::core::message::{keywords_add, keywords_remove};
use crate::core::pending_operation::{OperationKind, StoreKeywordsPayload};
use crate::core::set_flags::FlagAction;
use crate::services::database::DatabaseError;
use crate::services::folder_store;
use crate::services::message_store;
use crate::services::pending_ops_store;

/// Error type for keyword-change operations.
#[derive(Debug, thiserror::Error)]
pub enum SetKeywordsError {
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("message not found: {0}")]
    NotFound(i64),
    #[error("payload serialization error: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("folder is read-only: {0}")]
    ReadOnlyFolder(String),
}

/// Result of a successful keyword-change operation.
#[derive(Debug)]
pub struct SetKeywordsResult {
    /// The pending operation id (None if stored locally only).
    pub operation_id: Option<i64>,
    /// The new keywords value on the message.
    pub new_keywords: String,
}

/// Change a keyword on a message locally and, if the server supports user-defined
/// flags, enqueue a StoreKeywords operation for server synchronization.
///
/// `server_supports_keywords`: whether the server advertises `\*` in its
/// PERMANENTFLAGS (indicating user-defined flags are allowed). When false,
/// the keyword is stored locally only and no server operation is created (FR-34).
pub fn set_message_keyword(
    conn: &Connection,
    message_id: i64,
    folder_name: &str,
    keyword: &str,
    action: FlagAction,
    server_supports_keywords: bool,
) -> Result<SetKeywordsResult, SetKeywordsError> {
    let msg = message_store::load_message(conn, message_id)?
        .ok_or(SetKeywordsError::NotFound(message_id))?;

    // Check read-only before attempting any mutation.
    if folder_store::is_folder_read_only(conn, &msg.account_id, folder_name)? {
        return Err(SetKeywordsError::ReadOnlyFolder(folder_name.to_string()));
    }

    let new_keywords = match action {
        FlagAction::Set => keywords_add(&msg.keywords, keyword),
        FlagAction::Remove => keywords_remove(&msg.keywords, keyword),
    };

    if server_supports_keywords {
        // Mark as pending sync and enqueue server operation.
        message_store::update_message_keywords_pending(conn, message_id, &new_keywords)?;

        // Supersede stale pending StoreKeywords operations for this message.
        pending_ops_store::remove_pending_store_keywords_for_message(
            conn,
            &msg.account_id,
            message_id,
        )?;

        let payload = StoreKeywordsPayload {
            message_id,
            uid: msg.uid,
            folder_name: folder_name.to_string(),
            new_keywords: new_keywords.clone(),
        };
        let payload_json = serde_json::to_string(&payload)?;
        let op_id = pending_ops_store::insert_pending_op(
            conn,
            &msg.account_id,
            &OperationKind::StoreKeywords,
            &payload_json,
        )?;

        Ok(SetKeywordsResult {
            operation_id: Some(op_id),
            new_keywords,
        })
    } else {
        // Server does not support user-defined flags — store locally only (FR-34).
        message_store::update_message_keywords(conn, message_id, &new_keywords)?;

        Ok(SetKeywordsResult {
            operation_id: None,
            new_keywords,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::imap_check::ImapFolder;
    use crate::core::message::NewMessage;
    use crate::core::pending_operation::StoreKeywordsPayload;
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

    fn make_message(uid: u32, keywords: &str) -> NewMessage {
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
            keywords: keywords.to_string(),
        }
    }

    // --- Setting keywords with server support ---

    #[test]
    fn set_keyword_creates_operation_when_server_supports() {
        let (_dir, conn) = setup_db();
        let fid = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let msg = make_message(50, "");
        let mid = insert_message(&conn, &msg, fid).unwrap();

        let result =
            set_message_keyword(&conn, mid, "INBOX", "$Forwarded", FlagAction::Set, true).unwrap();
        assert_eq!(result.new_keywords, "$Forwarded");
        assert!(result.operation_id.is_some());

        let loaded = load_message(&conn, mid).unwrap().unwrap();
        assert_eq!(loaded.keywords, "$Forwarded");
        assert!(loaded.keywords_pending_sync);

        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1);
        let payload: StoreKeywordsPayload = serde_json::from_str(&ops[0].payload).unwrap();
        assert_eq!(payload.new_keywords, "$Forwarded");
        assert_eq!(payload.uid, 50);
    }

    #[test]
    fn remove_keyword_creates_operation_when_server_supports() {
        let (_dir, conn) = setup_db();
        let fid = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let msg = make_message(51, "$Forwarded,$Junk");
        let mid = insert_message(&conn, &msg, fid).unwrap();

        let result =
            set_message_keyword(&conn, mid, "INBOX", "$Junk", FlagAction::Remove, true).unwrap();
        assert_eq!(result.new_keywords, "$Forwarded");

        let loaded = load_message(&conn, mid).unwrap().unwrap();
        assert_eq!(loaded.keywords, "$Forwarded");
        assert!(loaded.keywords_pending_sync);
    }

    // --- Local-only when server does not support ---

    #[test]
    fn set_keyword_local_only_when_server_unsupported() {
        let (_dir, conn) = setup_db();
        let fid = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let msg = make_message(52, "");
        let mid = insert_message(&conn, &msg, fid).unwrap();

        let result =
            set_message_keyword(&conn, mid, "INBOX", "$Forwarded", FlagAction::Set, false).unwrap();
        assert_eq!(result.new_keywords, "$Forwarded");
        assert!(result.operation_id.is_none());

        let loaded = load_message(&conn, mid).unwrap().unwrap();
        assert_eq!(loaded.keywords, "$Forwarded");
        // Not pending sync because server doesn't support it.
        assert!(!loaded.keywords_pending_sync);

        // No operation enqueued.
        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert!(ops.is_empty());
    }

    // --- Read-only folder ---

    #[test]
    fn read_only_folder_rejects_keyword_change() {
        let (_dir, conn) = setup_db();
        let fid = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let msg = make_message(53, "");
        let mid = insert_message(&conn, &msg, fid).unwrap();
        set_folder_read_only(&conn, "acct-1", "INBOX", true).unwrap();

        let result = set_message_keyword(&conn, mid, "INBOX", "$Forwarded", FlagAction::Set, true);
        assert!(matches!(
            result.unwrap_err(),
            SetKeywordsError::ReadOnlyFolder(_)
        ));
    }

    // --- Supersession ---

    #[test]
    fn keyword_change_supersedes_stale_pending_op() {
        let (_dir, conn) = setup_db();
        let fid = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let msg = make_message(54, "$Junk");
        let mid = insert_message(&conn, &msg, fid).unwrap();

        // First: remove $Junk
        set_message_keyword(&conn, mid, "INBOX", "$Junk", FlagAction::Remove, true).unwrap();
        assert_eq!(load_pending_ops(&conn, "acct-1").unwrap().len(), 1);

        // Second: add $Junk back — supersedes the remove
        set_message_keyword(&conn, mid, "INBOX", "$Junk", FlagAction::Set, true).unwrap();
        let ops = load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1);

        let payload: StoreKeywordsPayload = serde_json::from_str(&ops[0].payload).unwrap();
        assert_eq!(payload.new_keywords, "$Junk");
    }

    // --- Not found ---

    #[test]
    fn not_found_returns_error() {
        let (_dir, conn) = setup_db();
        let result =
            set_message_keyword(&conn, 99999, "INBOX", "$Forwarded", FlagAction::Set, true);
        assert!(matches!(
            result.unwrap_err(),
            SetKeywordsError::NotFound(99999)
        ));
    }
}
