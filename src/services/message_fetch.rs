//! Fetch routine that pulls messages from an IMAP folder, stores them in the
//! content store, and indexes them in SQLite.

use rusqlite::Connection;

use crate::core::content_store::ContentStore;
use crate::core::message::{flags_from_imap, parse_raw_message};
use crate::services::imap_client::{fetch_folder_messages, ImapConnectParams};
use crate::services::message_store::{find_folder_id, insert_message, update_folder_sync_state};

/// Errors from the message fetch pipeline.
#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("IMAP error: {0}")]
    Imap(String),
    #[error("database error: {0}")]
    Database(#[from] crate::services::database::DatabaseError),
    #[error("content store error: {0}")]
    ContentStore(#[from] crate::core::content_store::ContentStoreError),
    #[error("folder not found: {0}")]
    FolderNotFound(String),
}

/// Result of a folder fetch operation.
#[derive(Debug)]
pub struct FetchResult {
    pub messages_fetched: usize,
    pub uidvalidity: u32,
    pub highestmodseq: Option<u64>,
}

/// Fetch all messages from a folder, store them, and index in the database.
///
/// This is a first-pass (full) fetch — it does not do incremental sync.
pub(crate) fn fetch_and_store_folder(
    conn: &Connection,
    content_store: &dyn ContentStore,
    params: &ImapConnectParams,
    folder_name: &str,
) -> Result<FetchResult, FetchError> {
    // Look up the folder id in the database.
    let folder_id = find_folder_id(conn, &params.account_id, folder_name)?
        .ok_or_else(|| FetchError::FolderNotFound(folder_name.to_string()))?;

    // Fetch messages from the IMAP server.
    let fetch_result = fetch_folder_messages(params, folder_name)
        .map_err(|e| FetchError::Imap(format!("{e:?}")))?;

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| FetchError::Database(crate::services::database::DatabaseError::Sqlite(e)))?;

    let mut count = 0;
    for raw_msg in &fetch_result.messages {
        // Store raw bytes in content store (idempotent).
        let content_hash = content_store.put(&raw_msg.data)?;

        // Parse headers and derive body_text.
        let flags = flags_from_imap(&raw_msg.flags);
        let new_msg = parse_raw_message(
            &params.account_id,
            raw_msg.uid,
            None, // modseq not available in first-pass full fetch
            flags,
            &content_hash,
            &raw_msg.data,
        );

        // Insert into database.
        insert_message(&tx, &new_msg, folder_id)?;
        count += 1;
    }

    // Update folder sync state.
    update_folder_sync_state(
        &tx,
        folder_id,
        fetch_result.select.uidvalidity,
        fetch_result.select.highestmodseq,
    )?;

    tx.commit()
        .map_err(|e| FetchError::Database(crate::services::database::DatabaseError::Sqlite(e)))?;

    Ok(FetchResult {
        messages_fetched: count,
        uidvalidity: fetch_result.select.uidvalidity,
        highestmodseq: fetch_result.select.highestmodseq,
    })
}
