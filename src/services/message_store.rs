//! Database persistence for the `messages` and `message_folders` tables.

use rusqlite::Connection;

use crate::core::message::{Message, NewMessage};
use crate::services::database::DatabaseError;

/// Insert a message and link it to a folder in a single operation.
/// Returns the new message row id.
pub fn insert_message(
    conn: &Connection,
    msg: &NewMessage,
    folder_id: i64,
) -> Result<i64, DatabaseError> {
    conn.execute(
        "INSERT INTO messages (
            account_id, uid, modseq, message_id, in_reply_to, references_header,
            from_addresses, to_addresses, cc_addresses, bcc_addresses,
            subject, date_received, date_sent, flags, size,
            content_hash, body_text, thread_id, server_thread_id
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
            ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19
        )",
        rusqlite::params![
            msg.account_id,
            msg.uid,
            msg.modseq.map(|v| v as i64),
            msg.message_id,
            msg.in_reply_to,
            msg.references_header,
            msg.from_addresses,
            msg.to_addresses,
            msg.cc_addresses,
            msg.bcc_addresses,
            msg.subject,
            msg.date_received,
            msg.date_sent,
            msg.flags,
            msg.size as i64,
            msg.content_hash,
            msg.body_text,
            msg.thread_id,
            msg.server_thread_id,
        ],
    )?;

    let message_id = conn.last_insert_rowid();

    conn.execute(
        "INSERT INTO message_folders (message_id, folder_id) VALUES (?1, ?2)",
        rusqlite::params![message_id, folder_id],
    )?;

    Ok(message_id)
}

/// Load a message by id.
pub fn load_message(conn: &Connection, id: i64) -> Result<Option<Message>, DatabaseError> {
    let mut stmt = conn.prepare(
        "SELECT id, account_id, uid, modseq, message_id, in_reply_to, references_header,
                from_addresses, to_addresses, cc_addresses, bcc_addresses,
                subject, date_received, date_sent, flags, size,
                content_hash, body_text, thread_id, server_thread_id,
                flags_pending_sync
         FROM messages WHERE id = ?1",
    )?;

    let result = stmt.query_row(rusqlite::params![id], |row| {
        Ok(Message {
            id: row.get(0)?,
            account_id: row.get(1)?,
            uid: row.get(2)?,
            modseq: row.get::<_, Option<i64>>(3)?.map(|v| v as u64),
            message_id: row.get(4)?,
            in_reply_to: row.get(5)?,
            references_header: row.get(6)?,
            from_addresses: row.get(7)?,
            to_addresses: row.get(8)?,
            cc_addresses: row.get(9)?,
            bcc_addresses: row.get(10)?,
            subject: row.get(11)?,
            date_received: row.get(12)?,
            date_sent: row.get(13)?,
            flags: row.get(14)?,
            size: row.get::<_, i64>(15)? as u64,
            content_hash: row.get(16)?,
            body_text: row.get(17)?,
            thread_id: row.get(18)?,
            server_thread_id: row.get(19)?,
            flags_pending_sync: row.get::<_, i32>(20)? != 0,
        })
    });

    match result {
        Ok(msg) => Ok(Some(msg)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DatabaseError::Sqlite(e)),
    }
}

/// Count how many message rows share the same `content_hash`.
pub fn count_by_content_hash(conn: &Connection, content_hash: &str) -> Result<i64, DatabaseError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM messages WHERE content_hash = ?1",
        rusqlite::params![content_hash],
        |row| row.get(0),
    )?;
    Ok(count)
}

/// Delete a message row. Returns the `content_hash` and whether this was
/// the last row referencing it (i.e., the `.eml` file should be deleted).
pub fn delete_message(conn: &Connection, id: i64) -> Result<Option<(String, bool)>, DatabaseError> {
    // Load the content hash before deleting.
    let hash: Option<String> = conn
        .query_row(
            "SELECT content_hash FROM messages WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .optional()?;

    let hash = match hash {
        Some(h) => h,
        None => return Ok(None),
    };

    // Delete the message row (cascade deletes message_folders).
    conn.execute("DELETE FROM messages WHERE id = ?1", rusqlite::params![id])?;

    // Check if any other rows still reference the same hash.
    let remaining = count_by_content_hash(conn, &hash)?;
    Ok(Some((hash, remaining == 0)))
}

/// Update the flags on a message row. Returns true if the row was found and updated.
pub fn update_message_flags(
    conn: &Connection,
    message_id: i64,
    new_flags: u32,
) -> Result<bool, DatabaseError> {
    let updated = conn.execute(
        "UPDATE messages SET flags = ?1 WHERE id = ?2",
        rusqlite::params![new_flags, message_id],
    )?;
    Ok(updated > 0)
}

/// Update the flags on a message and mark the change as pending server sync.
/// Returns true if the row was found and updated.
pub fn update_message_flags_pending(
    conn: &Connection,
    message_id: i64,
    new_flags: u32,
) -> Result<bool, DatabaseError> {
    let updated = conn.execute(
        "UPDATE messages SET flags = ?1, flags_pending_sync = 1 WHERE id = ?2",
        rusqlite::params![new_flags, message_id],
    )?;
    Ok(updated > 0)
}

/// Mark a message's flags as confirmed by the server (no longer pending sync).
/// Returns true if the row was found and updated.
pub fn mark_flags_confirmed(conn: &Connection, message_id: i64) -> Result<bool, DatabaseError> {
    let updated = conn.execute(
        "UPDATE messages SET flags_pending_sync = 0 WHERE id = ?1",
        rusqlite::params![message_id],
    )?;
    Ok(updated > 0)
}

/// Count messages for an account.
pub fn count_messages(conn: &Connection, account_id: &str) -> Result<i64, DatabaseError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM messages WHERE account_id = ?1",
        rusqlite::params![account_id],
        |row| row.get(0),
    )?;
    Ok(count)
}

/// Update the `uidvalidity` and `highestmodseq` on a folder row.
pub fn update_folder_sync_state(
    conn: &Connection,
    folder_id: i64,
    uidvalidity: u32,
    highestmodseq: Option<u64>,
) -> Result<(), DatabaseError> {
    conn.execute(
        "UPDATE folders SET uidvalidity = ?1, highestmodseq = ?2 WHERE id = ?3",
        rusqlite::params![uidvalidity, highestmodseq.map(|v| v as i64), folder_id],
    )?;
    Ok(())
}

/// Look up a folder's id by account_id and name.
pub fn find_folder_id(
    conn: &Connection,
    account_id: &str,
    folder_name: &str,
) -> Result<Option<i64>, DatabaseError> {
    let result = conn.query_row(
        "SELECT id FROM folders WHERE account_id = ?1 AND name = ?2",
        rusqlite::params![account_id, folder_name],
        |row| row.get(0),
    );
    match result {
        Ok(id) => Ok(Some(id)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DatabaseError::Sqlite(e)),
    }
}

/// Load the cached uidvalidity and highestmodseq for a folder.
pub fn load_folder_sync_state(
    conn: &Connection,
    folder_id: i64,
) -> Result<(Option<u32>, Option<u64>), DatabaseError> {
    let result = conn.query_row(
        "SELECT uidvalidity, highestmodseq FROM folders WHERE id = ?1",
        rusqlite::params![folder_id],
        |row| {
            let uv: Option<u32> = row.get(0)?;
            let hm: Option<i64> = row.get(1)?;
            Ok((uv, hm.map(|v| v as u64)))
        },
    );
    match result {
        Ok(pair) => Ok(pair),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok((None, None)),
        Err(e) => Err(DatabaseError::Sqlite(e)),
    }
}

/// Load all UIDs for messages in a specific folder.
pub fn load_uids_for_folder(
    conn: &Connection,
    account_id: &str,
    folder_id: i64,
) -> Result<Vec<u32>, DatabaseError> {
    let mut stmt = conn.prepare(
        "SELECT m.uid FROM messages m
         JOIN message_folders mf ON mf.message_id = m.id
         WHERE m.account_id = ?1 AND mf.folder_id = ?2
         ORDER BY m.uid",
    )?;
    let uids = stmt
        .query_map(rusqlite::params![account_id, folder_id], |row| row.get(0))?
        .collect::<Result<Vec<u32>, _>>()?;
    Ok(uids)
}

/// Delete all messages for a folder, returning content hashes that should be
/// deleted from the content store (those with no remaining references).
pub fn delete_messages_for_folder(
    conn: &Connection,
    account_id: &str,
    folder_id: i64,
) -> Result<Vec<String>, DatabaseError> {
    // Find message IDs and content hashes for this folder.
    let mut stmt = conn.prepare(
        "SELECT m.id, m.content_hash FROM messages m
         JOIN message_folders mf ON mf.message_id = m.id
         WHERE m.account_id = ?1 AND mf.folder_id = ?2",
    )?;
    let rows: Vec<(i64, String)> = stmt
        .query_map(rusqlite::params![account_id, folder_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut orphaned_hashes = Vec::new();
    for (msg_id, hash) in &rows {
        // Delete message_folders link first, then message row.
        conn.execute(
            "DELETE FROM message_folders WHERE message_id = ?1 AND folder_id = ?2",
            rusqlite::params![msg_id, folder_id],
        )?;
        conn.execute(
            "DELETE FROM messages WHERE id = ?1",
            rusqlite::params![msg_id],
        )?;
        // Check if any other rows still reference the same hash.
        let remaining = count_by_content_hash(conn, hash)?;
        if remaining == 0 {
            orphaned_hashes.push(hash.clone());
        }
    }

    Ok(orphaned_hashes)
}

/// Find a message by UID and folder, returning (message_id, current_flags).
pub fn find_message_by_uid_in_folder(
    conn: &Connection,
    account_id: &str,
    uid: u32,
    folder_id: i64,
) -> Result<Option<(i64, u32)>, DatabaseError> {
    let result = conn.query_row(
        "SELECT m.id, m.flags FROM messages m
         JOIN message_folders mf ON mf.message_id = m.id
         WHERE m.account_id = ?1 AND m.uid = ?2 AND mf.folder_id = ?3",
        rusqlite::params![account_id, uid, folder_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    );
    match result {
        Ok(pair) => Ok(Some(pair)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DatabaseError::Sqlite(e)),
    }
}

/// Helper to make `query_row` return `Option` on no rows.
trait OptionalRow {
    fn optional(self) -> Result<Option<String>, rusqlite::Error>;
}

impl OptionalRow for Result<String, rusqlite::Error> {
    fn optional(self) -> Result<Option<String>, rusqlite::Error> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::imap_check::ImapFolder;
    use crate::core::message::NewMessage;
    use crate::services::database::open_and_migrate;
    use crate::services::folder_store::replace_folders;
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

    fn folder_id(conn: &Connection) -> i64 {
        find_folder_id(conn, "acct-1", "INBOX").unwrap().unwrap()
    }

    fn make_message(hash: &str) -> NewMessage {
        NewMessage {
            account_id: "acct-1".to_string(),
            uid: 1,
            modseq: Some(100),
            message_id: Some("<test@example.com>".to_string()),
            in_reply_to: None,
            references_header: None,
            from_addresses: Some("test@example.com".to_string()),
            to_addresses: Some("other@example.com".to_string()),
            cc_addresses: None,
            bcc_addresses: None,
            subject: Some("Test Subject".to_string()),
            date_received: Some(1700000000),
            date_sent: Some(1700000000),
            flags: 0,
            size: 1024,
            content_hash: hash.to_string(),
            body_text: Some("Hello".to_string()),
            thread_id: None,
            server_thread_id: None,
        }
    }

    #[test]
    fn insert_and_load_message() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let msg = make_message("abc123");
        let id = insert_message(&conn, &msg, fid).unwrap();
        let loaded = load_message(&conn, id).unwrap().unwrap();
        assert_eq!(loaded.uid, 1);
        assert_eq!(loaded.content_hash, "abc123");
        assert_eq!(loaded.subject.as_deref(), Some("Test Subject"));
    }

    #[test]
    fn message_folders_link_created() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let msg = make_message("abc123");
        let mid = insert_message(&conn, &msg, fid).unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM message_folders WHERE message_id = ?1 AND folder_id = ?2",
                rusqlite::params![mid, fid],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn delete_message_last_reference_signals_file_delete() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let msg = make_message("unique_hash");
        let id = insert_message(&conn, &msg, fid).unwrap();
        let result = delete_message(&conn, id).unwrap().unwrap();
        assert_eq!(result.0, "unique_hash");
        assert!(result.1, "should signal file deletion when last reference");
    }

    #[test]
    fn delete_message_shared_hash_keeps_file() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let msg1 = make_message("shared_hash");
        let mut msg2 = make_message("shared_hash");
        msg2.uid = 2;
        let id1 = insert_message(&conn, &msg1, fid).unwrap();
        let _id2 = insert_message(&conn, &msg2, fid).unwrap();

        let result = delete_message(&conn, id1).unwrap().unwrap();
        assert_eq!(result.0, "shared_hash");
        assert!(
            !result.1,
            "should NOT signal file deletion when other references exist"
        );
    }

    #[test]
    fn count_messages_for_account() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        assert_eq!(count_messages(&conn, "acct-1").unwrap(), 0);
        let msg = make_message("hash1");
        insert_message(&conn, &msg, fid).unwrap();
        assert_eq!(count_messages(&conn, "acct-1").unwrap(), 1);
    }

    #[test]
    fn update_folder_sync_state_sets_values() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        update_folder_sync_state(&conn, fid, 12345, Some(67890)).unwrap();

        let (uv, hm): (Option<u32>, Option<i64>) = conn
            .query_row(
                "SELECT uidvalidity, highestmodseq FROM folders WHERE id = ?1",
                rusqlite::params![fid],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(uv, Some(12345));
        assert_eq!(hm, Some(67890));
    }

    #[test]
    fn find_folder_id_returns_none_for_missing() {
        let (_dir, conn) = setup_db();
        assert!(find_folder_id(&conn, "acct-1", "NonExistent")
            .unwrap()
            .is_none());
    }

    #[test]
    fn fts5_insert_populates_index() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let msg = make_message("fts_hash");
        let id = insert_message(&conn, &msg, fid).unwrap();

        let (subject, body): (Option<String>, Option<String>) = conn
            .query_row(
                "SELECT subject, body_text FROM messages_fts WHERE rowid = ?1",
                rusqlite::params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(subject.as_deref(), Some("Test Subject"));
        assert_eq!(body.as_deref(), Some("Hello"));
    }

    #[test]
    fn fts5_update_reflects_changes() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let msg = make_message("fts_upd");
        let id = insert_message(&conn, &msg, fid).unwrap();

        conn.execute(
            "UPDATE messages SET subject = ?1, body_text = ?2 WHERE id = ?3",
            rusqlite::params!["Updated Subject", "Updated body", id],
        )
        .unwrap();

        let (subject, body): (String, String) = conn
            .query_row(
                "SELECT subject, body_text FROM messages_fts WHERE rowid = ?1",
                rusqlite::params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(subject, "Updated Subject");
        assert_eq!(body, "Updated body");
    }

    #[test]
    fn fts5_delete_removes_from_index() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let msg = make_message("fts_del");
        let id = insert_message(&conn, &msg, fid).unwrap();

        conn.execute("DELETE FROM messages WHERE id = ?1", rusqlite::params![id])
            .unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM messages_fts WHERE rowid = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn fts5_match_query_returns_expected_rows() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);

        let mut msg1 = make_message("fts_m1");
        msg1.subject = Some("Rust programming language".to_string());
        msg1.body_text = Some("Rust is a systems programming language".to_string());
        let id1 = insert_message(&conn, &msg1, fid).unwrap();

        let mut msg2 = make_message("fts_m2");
        msg2.uid = 2;
        msg2.subject = Some("Python tutorial".to_string());
        msg2.body_text = Some("Python is great for scripting".to_string());
        let _id2 = insert_message(&conn, &msg2, fid).unwrap();

        let matched_id: i64 = conn
            .query_row(
                "SELECT rowid FROM messages_fts WHERE messages_fts MATCH 'Rust'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(matched_id, id1);
    }

    #[test]
    fn load_folder_sync_state_returns_cached_values() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);

        // Initially no sync state.
        let (uv, hm) = load_folder_sync_state(&conn, fid).unwrap();
        assert!(uv.is_none());
        assert!(hm.is_none());

        // Set sync state.
        update_folder_sync_state(&conn, fid, 42, Some(100)).unwrap();
        let (uv, hm) = load_folder_sync_state(&conn, fid).unwrap();
        assert_eq!(uv, Some(42));
        assert_eq!(hm, Some(100));
    }

    #[test]
    fn load_uids_for_folder_returns_sorted() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);

        let mut msg1 = make_message("h1");
        msg1.uid = 5;
        insert_message(&conn, &msg1, fid).unwrap();

        let mut msg2 = make_message("h2");
        msg2.uid = 2;
        insert_message(&conn, &msg2, fid).unwrap();

        let mut msg3 = make_message("h3");
        msg3.uid = 10;
        insert_message(&conn, &msg3, fid).unwrap();

        let uids = load_uids_for_folder(&conn, "acct-1", fid).unwrap();
        assert_eq!(uids, vec![2, 5, 10]);
    }

    #[test]
    fn delete_messages_for_folder_returns_orphaned_hashes() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);

        // Insert two messages with unique hashes.
        let msg1 = make_message("unique_h1");
        insert_message(&conn, &msg1, fid).unwrap();

        let mut msg2 = make_message("unique_h2");
        msg2.uid = 2;
        insert_message(&conn, &msg2, fid).unwrap();

        let orphaned = delete_messages_for_folder(&conn, "acct-1", fid).unwrap();
        assert_eq!(orphaned.len(), 2);
        assert!(orphaned.contains(&"unique_h1".to_string()));
        assert!(orphaned.contains(&"unique_h2".to_string()));

        // No messages should remain.
        assert_eq!(count_messages(&conn, "acct-1").unwrap(), 0);
    }

    #[test]
    fn delete_messages_for_folder_keeps_shared_hashes() {
        let (_dir, conn) = setup_db();

        // Create both folders at once (replace_folders replaces all).
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
        ];
        replace_folders(&conn, "acct-1", &folders).unwrap();
        let fid = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let fid2 = find_folder_id(&conn, "acct-1", "Archive").unwrap().unwrap();

        // Insert same hash in both folders.
        let msg1 = make_message("shared_hash");
        insert_message(&conn, &msg1, fid).unwrap();

        let mut msg2 = make_message("shared_hash");
        msg2.uid = 2;
        insert_message(&conn, &msg2, fid2).unwrap();

        // Delete messages from first folder only.
        let orphaned = delete_messages_for_folder(&conn, "acct-1", fid).unwrap();
        // Hash still referenced by msg in Archive — should NOT be orphaned.
        assert!(orphaned.is_empty());
    }

    #[test]
    fn find_message_by_uid_in_folder_found() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);

        let mut msg = make_message("h1");
        msg.uid = 42;
        msg.flags = 1; // SEEN
        let _id = insert_message(&conn, &msg, fid).unwrap();

        let found = find_message_by_uid_in_folder(&conn, "acct-1", 42, fid).unwrap();
        assert!(found.is_some());
        let (mid, flags) = found.unwrap();
        assert!(mid > 0);
        assert_eq!(flags, 1);
    }

    #[test]
    fn find_message_by_uid_in_folder_not_found() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);

        let found = find_message_by_uid_in_folder(&conn, "acct-1", 999, fid).unwrap();
        assert!(found.is_none());
    }
}
