//! Rebuild the SQLite index from the on-disk `.eml` content store.
//!
//! Walks `<content_root>/` for `.eml` files, verifies the SHA-256 hash matches
//! the filename, parses headers with `mail_parser`, and upserts rows into the
//! `messages` table (and the FTS5 mirror via triggers). Non-destructive: never
//! modifies or deletes `.eml` files.

use std::path::Path;

use rusqlite::Connection;

use crate::core::message::parse_raw_message;
use crate::services::database::DatabaseError;
use crate::services::folder_store::insert_folder;
use crate::services::fs_content_store::sha256_hex;
use crate::services::message_store::{find_folder_id, insert_message};

/// Result summary from a rebuild operation.
#[derive(Debug)]
pub struct RebuildResult {
    /// Number of `.eml` files found on disk.
    pub files_scanned: usize,
    /// Number of messages inserted into the database.
    pub messages_inserted: usize,
    /// Number of files skipped because their content_hash already existed.
    pub skipped_existing: usize,
    /// Number of files skipped due to hash mismatch or parse errors.
    pub skipped_errors: usize,
}

/// Errors from the rebuild pipeline.
#[derive(Debug, thiserror::Error)]
pub enum RebuildError {
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("no accounts found in database — cannot assign messages")]
    NoAccounts,
}

/// Rebuild the SQLite message index from the on-disk `.eml` files.
///
/// Requires that at least one account row exists in the database. Messages are
/// assigned to the first available account. Folder hints are read from the
/// `X-Folder` header; messages without one go into a synthetic "Recovered"
/// folder.
///
/// Idempotent: uses the `content_hash` column to skip already-indexed messages.
/// Non-destructive: only reads `.eml` files, never modifies or deletes them.
pub fn rebuild_index(
    conn: &Connection,
    content_root: &Path,
) -> Result<RebuildResult, RebuildError> {
    // Load the first available account to assign messages to.
    let account_id = load_first_account_id(conn)?;

    // Collect all .eml file paths.
    let eml_paths = collect_eml_files(content_root);

    let mut result = RebuildResult {
        files_scanned: eml_paths.len(),
        messages_inserted: 0,
        skipped_existing: 0,
        skipped_errors: 0,
    };

    if eml_paths.is_empty() {
        return Ok(result);
    }

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| RebuildError::Database(DatabaseError::Sqlite(e)))?;

    for path in &eml_paths {
        match process_eml_file(&tx, path, &account_id) {
            Ok(ProcessResult::Inserted) => result.messages_inserted += 1,
            Ok(ProcessResult::AlreadyExists) => result.skipped_existing += 1,
            Err(e) => {
                eprintln!("Warning: skipping {}: {e}", path.display());
                result.skipped_errors += 1;
            }
        }
    }

    tx.commit()
        .map_err(|e| RebuildError::Database(DatabaseError::Sqlite(e)))?;

    Ok(result)
}

enum ProcessResult {
    Inserted,
    AlreadyExists,
}

/// Process a single `.eml` file: verify hash, check for duplicates, parse, insert.
fn process_eml_file(
    conn: &Connection,
    path: &Path,
    account_id: &str,
) -> Result<ProcessResult, RebuildError> {
    let raw = std::fs::read(path)?;

    // Compute SHA-256 and verify it matches the filename.
    let computed_hash = sha256_hex(&raw);
    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    if computed_hash != file_stem {
        return Err(RebuildError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("hash mismatch: filename={file_stem}, computed={computed_hash}"),
        )));
    }

    // Idempotent: skip if a message with this content_hash already exists.
    let existing: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM messages WHERE content_hash = ?1",
            rusqlite::params![computed_hash],
            |row| row.get(0),
        )
        .map_err(|e| RebuildError::Database(DatabaseError::Sqlite(e)))?;
    if existing > 0 {
        return Ok(ProcessResult::AlreadyExists);
    }

    // Determine folder from X-Folder header, fallback to "Recovered".
    let folder_name = extract_folder_hint(&raw).unwrap_or_else(|| "Recovered".to_string());

    // Find or create the folder.
    let folder_id = match find_folder_id(conn, account_id, &folder_name)? {
        Some(id) => id,
        None => insert_folder(conn, account_id, &folder_name)?,
    };

    // Parse the message and insert.
    let new_msg = parse_raw_message(account_id, 0, None, 0, &computed_hash, &raw);
    insert_message(conn, &new_msg, folder_id)?;

    Ok(ProcessResult::Inserted)
}

/// Extract the `X-Folder` header value from raw email bytes.
fn extract_folder_hint(raw: &[u8]) -> Option<String> {
    let parsed = mail_parser::MessageParser::default().parse(raw)?;
    let header = parsed.header("X-Folder")?;
    match header {
        mail_parser::HeaderValue::Text(t) => {
            let trimmed = t.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        _ => None,
    }
}

/// Load the first account id from the database.
fn load_first_account_id(conn: &Connection) -> Result<String, RebuildError> {
    let result = conn.query_row("SELECT id FROM accounts LIMIT 1", [], |row| row.get(0));
    match result {
        Ok(id) => Ok(id),
        Err(rusqlite::Error::QueryReturnedNoRows) => Err(RebuildError::NoAccounts),
        Err(e) => Err(RebuildError::Database(DatabaseError::Sqlite(e))),
    }
}

/// Recursively collect all `.eml` file paths under the given root.
fn collect_eml_files(root: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if !root.exists() {
        return files;
    }
    collect_eml_recursive(root, &mut files);
    files
}

fn collect_eml_recursive(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_eml_recursive(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("eml") {
            out.push(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::content_store::ContentStore;
    use crate::services::database::open_and_migrate;
    use crate::services::fs_content_store::FsContentStore;
    use crate::services::message_store::count_messages;
    use tempfile::TempDir;

    /// Create a test database with one account.
    fn setup_db(dir: &Path) -> Connection {
        let db_path = dir.join("fairmail.db");
        let conn = open_and_migrate(&db_path).unwrap();
        conn.execute(
            "INSERT INTO accounts (id, display_name, protocol, host, port, encryption, auth_method, username, credential)
             VALUES ('acct-1', 'Test', 'Imap', 'imap.example.com', 993, 'SslTls', 'Plain', 'user', '')",
            [],
        ).unwrap();
        conn
    }

    fn make_raw_email(subject: &str, body: &str) -> Vec<u8> {
        format!(
            "From: test@example.com\r\n\
             To: recipient@example.com\r\n\
             Subject: {subject}\r\n\
             Message-ID: <{subject}@example.com>\r\n\
             Date: Mon, 1 Jan 2024 12:00:00 +0000\r\n\
             \r\n\
             {body}\r\n"
        )
        .into_bytes()
    }

    fn make_raw_email_with_folder(subject: &str, body: &str, folder: &str) -> Vec<u8> {
        format!(
            "From: test@example.com\r\n\
             To: recipient@example.com\r\n\
             Subject: {subject}\r\n\
             Message-ID: <{subject}@example.com>\r\n\
             X-Folder: {folder}\r\n\
             Date: Mon, 1 Jan 2024 12:00:00 +0000\r\n\
             \r\n\
             {body}\r\n"
        )
        .into_bytes()
    }

    /// Store raw email bytes via FsContentStore and return the hash.
    fn store_eml(content_store: &FsContentStore, data: &[u8]) -> String {
        content_store.put(data).unwrap()
    }

    #[test]
    fn rebuild_from_known_message_set() {
        let dir = TempDir::new().unwrap();
        let content_root = dir.path().join("messages");
        let content_store = FsContentStore::new(content_root.clone());

        // Store some .eml files.
        let email1 = make_raw_email("Rebuild Test One", "Body of first email");
        let email2 = make_raw_email("Rebuild Test Two", "Body of second email");
        let email3 = make_raw_email("Rebuild Test Three", "Body of third email");
        store_eml(&content_store, &email1);
        store_eml(&content_store, &email2);
        store_eml(&content_store, &email3);

        // Create a fresh database (simulating deletion + recreation).
        let conn = setup_db(dir.path());

        let result = rebuild_index(&conn, &content_root).unwrap();
        assert_eq!(result.files_scanned, 3);
        assert_eq!(result.messages_inserted, 3);
        assert_eq!(result.skipped_existing, 0);
        assert_eq!(result.skipped_errors, 0);

        // Verify row count.
        assert_eq!(count_messages(&conn, "acct-1").unwrap(), 3);

        // Verify a sampled subject.
        let subject: String = conn
            .query_row(
                "SELECT subject FROM messages WHERE subject = 'Rebuild Test One'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(subject, "Rebuild Test One");
    }

    #[test]
    fn rebuild_fts5_index_consistent() {
        let dir = TempDir::new().unwrap();
        let content_root = dir.path().join("messages");
        let content_store = FsContentStore::new(content_root.clone());

        let email = make_raw_email("Unicorn Sparkle Subject", "Magical rainbow body text");
        store_eml(&content_store, &email);

        let conn = setup_db(dir.path());
        rebuild_index(&conn, &content_root).unwrap();

        // FTS5 search for a known indexed term.
        let matched_subject: String = conn
            .query_row(
                "SELECT subject FROM messages WHERE id IN (SELECT rowid FROM messages_fts WHERE messages_fts MATCH 'Unicorn')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(matched_subject, "Unicorn Sparkle Subject");

        // Also verify body_text search.
        let matched_body: String = conn
            .query_row(
                "SELECT body_text FROM messages WHERE id IN (SELECT rowid FROM messages_fts WHERE messages_fts MATCH 'rainbow')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(matched_body.contains("rainbow"));
    }

    #[test]
    fn rebuild_does_not_modify_eml_files() {
        let dir = TempDir::new().unwrap();
        let content_root = dir.path().join("messages");
        let content_store = FsContentStore::new(content_root.clone());

        let email = make_raw_email("Preserve Test", "Do not touch this file");
        let hash = store_eml(&content_store, &email);

        // Record file metadata before rebuild.
        let eml_files_before = collect_eml_files(&content_root);
        let contents_before: Vec<Vec<u8>> = eml_files_before
            .iter()
            .map(|p| std::fs::read(p).unwrap())
            .collect();

        let conn = setup_db(dir.path());
        rebuild_index(&conn, &content_root).unwrap();

        // Verify files are unchanged.
        let eml_files_after = collect_eml_files(&content_root);
        assert_eq!(eml_files_before.len(), eml_files_after.len());
        for (path, original) in eml_files_before.iter().zip(contents_before.iter()) {
            let after = std::fs::read(path).unwrap();
            assert_eq!(original, &after, "file was modified: {}", path.display());
        }

        // Verify .eml still exists via content store.
        assert!(content_store.exists(&hash).unwrap());
    }

    #[test]
    fn rebuild_empty_content_store_is_noop() {
        let dir = TempDir::new().unwrap();
        let content_root = dir.path().join("messages");
        // Don't create any .eml files.

        let conn = setup_db(dir.path());
        let result = rebuild_index(&conn, &content_root).unwrap();

        assert_eq!(result.files_scanned, 0);
        assert_eq!(result.messages_inserted, 0);
        assert_eq!(result.skipped_existing, 0);
        assert_eq!(result.skipped_errors, 0);
        assert_eq!(count_messages(&conn, "acct-1").unwrap(), 0);
    }

    #[test]
    fn rebuild_is_idempotent() {
        let dir = TempDir::new().unwrap();
        let content_root = dir.path().join("messages");
        let content_store = FsContentStore::new(content_root.clone());

        let email1 = make_raw_email("Idempotent One", "Body one");
        let email2 = make_raw_email("Idempotent Two", "Body two");
        store_eml(&content_store, &email1);
        store_eml(&content_store, &email2);

        let conn = setup_db(dir.path());

        // First rebuild.
        let r1 = rebuild_index(&conn, &content_root).unwrap();
        assert_eq!(r1.messages_inserted, 2);

        // Second rebuild — should be all skips, no duplicates.
        let r2 = rebuild_index(&conn, &content_root).unwrap();
        assert_eq!(r2.messages_inserted, 0);
        assert_eq!(r2.skipped_existing, 2);
        assert_eq!(r2.skipped_errors, 0);

        // Still only 2 rows.
        assert_eq!(count_messages(&conn, "acct-1").unwrap(), 2);
    }

    #[test]
    fn rebuild_uses_x_folder_header() {
        let dir = TempDir::new().unwrap();
        let content_root = dir.path().join("messages");
        let content_store = FsContentStore::new(content_root.clone());

        let email = make_raw_email_with_folder("Folder Hint", "Body", "INBOX");
        store_eml(&content_store, &email);

        let conn = setup_db(dir.path());
        rebuild_index(&conn, &content_root).unwrap();

        // Verify message is in the INBOX folder.
        let folder_id = find_folder_id(&conn, "acct-1", "INBOX").unwrap();
        assert!(folder_id.is_some(), "INBOX folder should be created");

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM message_folders WHERE folder_id = ?1",
                rusqlite::params![folder_id.unwrap()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn rebuild_defaults_to_recovered_folder() {
        let dir = TempDir::new().unwrap();
        let content_root = dir.path().join("messages");
        let content_store = FsContentStore::new(content_root.clone());

        let email = make_raw_email("No Folder Hint", "Body");
        store_eml(&content_store, &email);

        let conn = setup_db(dir.path());
        rebuild_index(&conn, &content_root).unwrap();

        // Message should be in "Recovered" folder.
        let folder_id = find_folder_id(&conn, "acct-1", "Recovered").unwrap();
        assert!(folder_id.is_some(), "Recovered folder should be created");
    }
}
