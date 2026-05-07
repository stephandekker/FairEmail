//! Fetch routine that pulls messages from an IMAP folder, stores them in the
//! content store, and indexes them in SQLite.
//!
//! Supports both full-fetch (first sync) and incremental sync via CONDSTORE
//! or UID-set diff fallback.

use rusqlite::Connection;

use crate::core::content_store::ContentStore;
use crate::core::detect_new_messages::find_new_uids;
use crate::core::message::{flags_from_imap, parse_raw_message};
use crate::core::server_flag_detection::{
    detect_flag_change, make_flag_change_event, FlagChangeAction,
};
use crate::core::sync_event::SyncEvent;
use crate::services::imap_client::{fetch_folder_messages, ImapConnectParams};
use crate::services::message_store::{
    delete_messages_for_folder, find_folder_id, find_message_by_uid_in_folder_with_pending,
    insert_message, load_folder_sync_state, load_uids_for_folder, update_folder_sync_state,
    update_message_flags,
};
use crate::services::sync_state_store::load_sync_state;

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

/// Result of an incremental sync operation.
#[derive(Debug)]
pub struct IncrementalSyncResult {
    /// Number of new message bodies fetched.
    pub bodies_fetched: usize,
    /// Number of flag-only updates applied.
    pub flags_updated: usize,
    /// Whether a full re-fetch was triggered (UIDVALIDITY change).
    pub full_refetch: bool,
    /// Sync events to broadcast (flag changes from server).
    pub events: Vec<SyncEvent>,
    /// Content hashes of .eml files that should be deleted.
    pub orphaned_hashes: Vec<String>,
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

/// Incremental sync: uses CONDSTORE if available, falls back to UID-set diff.
///
/// On first sync (no cached uidvalidity), delegates to full fetch.
/// On UIDVALIDITY change, invalidates cached data and does a full re-fetch.
#[allow(dead_code)]
pub(crate) fn incremental_sync_folder(
    conn: &Connection,
    content_store: &dyn ContentStore,
    params: &ImapConnectParams,
    folder_name: &str,
) -> Result<IncrementalSyncResult, FetchError> {
    let folder_id = find_folder_id(conn, &params.account_id, folder_name)?
        .ok_or_else(|| FetchError::FolderNotFound(folder_name.to_string()))?;

    // Load cached sync state for this folder.
    let (cached_uidvalidity, cached_highestmodseq) = load_folder_sync_state(conn, folder_id)?;

    // If no cached uidvalidity, this is a first sync — do full fetch.
    let cached_uv = match cached_uidvalidity {
        Some(uv) => uv,
        None => {
            let full = fetch_and_store_folder(conn, content_store, params, folder_name)?;
            return Ok(IncrementalSyncResult {
                bodies_fetched: full.messages_fetched,
                flags_updated: 0,
                full_refetch: false,
                events: Vec::new(),
                orphaned_hashes: Vec::new(),
                uidvalidity: full.uidvalidity,
                highestmodseq: full.highestmodseq,
            });
        }
    };

    // Check if CONDSTORE is supported.
    let sync_state = load_sync_state(conn, &params.account_id)?;
    let condstore = sync_state
        .as_ref()
        .map(|s| s.condstore_supported || s.qresync_supported)
        .unwrap_or(false);

    if condstore {
        if let Some(modseq) = cached_highestmodseq {
            return sync_condstore(
                conn,
                content_store,
                params,
                folder_name,
                folder_id,
                cached_uv,
                modseq,
            );
        }
    }

    // Fallback: UID-set diff.
    sync_uid_diff(
        conn,
        content_store,
        params,
        folder_name,
        folder_id,
        cached_uv,
    )
}

/// CONDSTORE incremental sync path.
#[allow(dead_code)]
fn sync_condstore(
    conn: &Connection,
    content_store: &dyn ContentStore,
    params: &ImapConnectParams,
    folder_name: &str,
    folder_id: i64,
    cached_uidvalidity: u32,
    cached_modseq: u64,
) -> Result<IncrementalSyncResult, FetchError> {
    use crate::services::imap_client::fetch_changed_since;

    let result = fetch_changed_since(params, folder_name, cached_modseq)
        .map_err(|e| FetchError::Imap(format!("{e:?}")))?;

    // Check UIDVALIDITY.
    if result.select.uidvalidity != cached_uidvalidity {
        return handle_uidvalidity_change(conn, content_store, params, folder_name, folder_id);
    }

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| FetchError::Database(crate::services::database::DatabaseError::Sqlite(e)))?;

    let mut bodies_fetched = 0;
    let mut flags_updated = 0;
    let mut events = Vec::new();

    for changed in &result.changed {
        let existing = find_message_by_uid_in_folder_with_pending(
            &tx,
            &params.account_id,
            changed.uid,
            folder_id,
        )?;

        match (existing, &changed.body) {
            // Existing message — check flags with pending-sync guard.
            (Some((msg_id, old_flags, pending)), None | Some(_)) => {
                let new_flags = flags_from_imap(&changed.flags);
                match detect_flag_change(old_flags, new_flags, pending) {
                    FlagChangeAction::Apply { new_flags } => {
                        update_message_flags(&tx, msg_id, new_flags)?;
                        events.push(make_flag_change_event(
                            &params.account_id,
                            msg_id,
                            new_flags,
                        ));
                        flags_updated += 1;
                    }
                    FlagChangeAction::NoChange | FlagChangeAction::SkippedPendingSync => {}
                }
            }
            // New message (not in local DB).
            (None, Some(body)) => {
                let content_hash = content_store.put(body)?;
                let new_msg = parse_raw_message(
                    &params.account_id,
                    changed.uid,
                    changed.modseq,
                    flags_from_imap(&changed.flags),
                    &content_hash,
                    body,
                );
                insert_message(&tx, &new_msg, folder_id)?;
                bodies_fetched += 1;
            }
            // New message but no body (shouldn't happen with BODY.PEEK[] in FETCH).
            (None, None) => {
                // Skip — cannot store without body data.
            }
        }
    }

    // Update folder sync state with new highestmodseq.
    update_folder_sync_state(
        &tx,
        folder_id,
        result.select.uidvalidity,
        result.select.highestmodseq,
    )?;

    tx.commit()
        .map_err(|e| FetchError::Database(crate::services::database::DatabaseError::Sqlite(e)))?;

    // Emit NewMailReceived event if new messages were fetched.
    if bodies_fetched > 0 {
        events.push(SyncEvent::NewMailReceived {
            account_id: params.account_id.clone(),
            folder_name: folder_name.to_string(),
            bodies_fetched,
        });
    }

    Ok(IncrementalSyncResult {
        bodies_fetched,
        flags_updated,
        full_refetch: false,
        events,
        orphaned_hashes: Vec::new(),
        uidvalidity: result.select.uidvalidity,
        highestmodseq: result.select.highestmodseq,
    })
}

/// UID-set diff fallback sync path (no CONDSTORE).
#[allow(dead_code)]
fn sync_uid_diff(
    conn: &Connection,
    content_store: &dyn ContentStore,
    params: &ImapConnectParams,
    folder_name: &str,
    folder_id: i64,
    cached_uidvalidity: u32,
) -> Result<IncrementalSyncResult, FetchError> {
    use crate::services::imap_client::fetch_uid_diff;

    // Get local UIDs.
    let local_uids = load_uids_for_folder(conn, &params.account_id, folder_id)?;

    // First pass: get server UIDs + flags (UID FETCH 1:* (UID FLAGS)).
    // We pass an empty slice for new_uids first — we'll compute them from the diff.
    let diff_result =
        fetch_uid_diff(params, folder_name, &[]).map_err(|e| FetchError::Imap(format!("{e:?}")))?;

    // Check UIDVALIDITY.
    if diff_result.select.uidvalidity != cached_uidvalidity {
        return handle_uidvalidity_change(conn, content_store, params, folder_name, folder_id);
    }

    // Use core detection logic to find new UIDs.
    let new_uids = find_new_uids(&diff_result.server_uids, &local_uids);
    let local_uid_set: std::collections::HashSet<u32> = local_uids.into_iter().collect();

    // Detect flag changes for existing messages (UIDs in both local and server sets).
    let mut flags_updated = 0;
    let mut events = Vec::new();
    {
        let tx = conn.unchecked_transaction().map_err(|e| {
            FetchError::Database(crate::services::database::DatabaseError::Sqlite(e))
        })?;

        for entry in &diff_result.server_flags {
            if !local_uid_set.contains(&entry.uid) {
                continue; // New message — handled below.
            }
            if let Some((msg_id, old_flags, pending)) = find_message_by_uid_in_folder_with_pending(
                &tx,
                &params.account_id,
                entry.uid,
                folder_id,
            )? {
                let server_flags = flags_from_imap(&entry.flags);
                match detect_flag_change(old_flags, server_flags, pending) {
                    FlagChangeAction::Apply { new_flags } => {
                        update_message_flags(&tx, msg_id, new_flags)?;
                        events.push(make_flag_change_event(
                            &params.account_id,
                            msg_id,
                            new_flags,
                        ));
                        flags_updated += 1;
                    }
                    FlagChangeAction::NoChange | FlagChangeAction::SkippedPendingSync => {}
                }
            }
        }

        tx.commit().map_err(|e| {
            FetchError::Database(crate::services::database::DatabaseError::Sqlite(e))
        })?;
    }

    // If there are new messages, fetch them.
    let bodies_fetched = if !new_uids.is_empty() {
        let fetch_result = fetch_uid_diff(params, folder_name, &new_uids)
            .map_err(|e| FetchError::Imap(format!("{e:?}")))?;

        let tx = conn.unchecked_transaction().map_err(|e| {
            FetchError::Database(crate::services::database::DatabaseError::Sqlite(e))
        })?;

        let mut count = 0;
        for raw_msg in &fetch_result.new_messages {
            let content_hash = content_store.put(&raw_msg.data)?;
            let new_msg = parse_raw_message(
                &params.account_id,
                raw_msg.uid,
                None,
                flags_from_imap(&raw_msg.flags),
                &content_hash,
                &raw_msg.data,
            );
            insert_message(&tx, &new_msg, folder_id)?;
            count += 1;
        }

        update_folder_sync_state(
            &tx,
            folder_id,
            diff_result.select.uidvalidity,
            diff_result.select.highestmodseq,
        )?;

        tx.commit().map_err(|e| {
            FetchError::Database(crate::services::database::DatabaseError::Sqlite(e))
        })?;

        count
    } else {
        // No new messages — just update sync state.
        update_folder_sync_state(
            conn,
            folder_id,
            diff_result.select.uidvalidity,
            diff_result.select.highestmodseq,
        )?;
        0
    };

    // Emit NewMailReceived event if new messages were fetched.
    if bodies_fetched > 0 {
        events.push(SyncEvent::NewMailReceived {
            account_id: params.account_id.clone(),
            folder_name: folder_name.to_string(),
            bodies_fetched,
        });
    }

    Ok(IncrementalSyncResult {
        bodies_fetched,
        flags_updated,
        full_refetch: false,
        events,
        orphaned_hashes: Vec::new(),
        uidvalidity: diff_result.select.uidvalidity,
        highestmodseq: diff_result.select.highestmodseq,
    })
}

/// Perform incremental sync using only in-process data (for testing).
/// This variant takes pre-fetched IMAP results instead of connecting to a server.
#[cfg(test)]
pub(crate) fn incremental_sync_condstore_with_data(
    conn: &Connection,
    content_store: &dyn ContentStore,
    account_id: &str,
    folder_name: &str,
    select_uidvalidity: u32,
    select_highestmodseq: Option<u64>,
    changed_messages: Vec<crate::services::imap_client::ChangedMessage>,
) -> Result<IncrementalSyncResult, FetchError> {
    let folder_id = find_folder_id(conn, account_id, folder_name)?
        .ok_or_else(|| FetchError::FolderNotFound(folder_name.to_string()))?;

    let (cached_uidvalidity, _cached_highestmodseq) = load_folder_sync_state(conn, folder_id)?;

    // Check UIDVALIDITY.
    if let Some(cached_uv) = cached_uidvalidity {
        if select_uidvalidity != cached_uv {
            // UIDVALIDITY changed — invalidate.
            let tx = conn.unchecked_transaction().map_err(|e| {
                FetchError::Database(crate::services::database::DatabaseError::Sqlite(e))
            })?;
            let orphaned_hashes = delete_messages_for_folder(&tx, account_id, folder_id)?;
            update_folder_sync_state(&tx, folder_id, 0, None)?;
            tx.commit().map_err(|e| {
                FetchError::Database(crate::services::database::DatabaseError::Sqlite(e))
            })?;
            for hash in &orphaned_hashes {
                let _ = content_store.delete(hash);
            }
            return Ok(IncrementalSyncResult {
                bodies_fetched: 0,
                flags_updated: 0,
                full_refetch: true,
                events: Vec::new(),
                orphaned_hashes,
                uidvalidity: select_uidvalidity,
                highestmodseq: select_highestmodseq,
            });
        }
    }

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| FetchError::Database(crate::services::database::DatabaseError::Sqlite(e)))?;

    let mut bodies_fetched = 0;
    let mut flags_updated = 0;
    let mut events = Vec::new();

    for changed in &changed_messages {
        let existing =
            find_message_by_uid_in_folder_with_pending(&tx, account_id, changed.uid, folder_id)?;

        match (existing, &changed.body) {
            (Some((msg_id, old_flags, pending)), _) => {
                let new_flags = flags_from_imap(&changed.flags);
                match detect_flag_change(old_flags, new_flags, pending) {
                    FlagChangeAction::Apply { new_flags } => {
                        update_message_flags(&tx, msg_id, new_flags)?;
                        events.push(make_flag_change_event(account_id, msg_id, new_flags));
                        flags_updated += 1;
                    }
                    FlagChangeAction::NoChange | FlagChangeAction::SkippedPendingSync => {}
                }
            }
            (None, Some(body)) => {
                let content_hash = content_store.put(body)?;
                let new_msg = parse_raw_message(
                    account_id,
                    changed.uid,
                    changed.modseq,
                    flags_from_imap(&changed.flags),
                    &content_hash,
                    body,
                );
                insert_message(&tx, &new_msg, folder_id)?;
                bodies_fetched += 1;
            }
            (None, None) => {}
        }
    }

    update_folder_sync_state(&tx, folder_id, select_uidvalidity, select_highestmodseq)?;

    tx.commit()
        .map_err(|e| FetchError::Database(crate::services::database::DatabaseError::Sqlite(e)))?;

    // Emit NewMailReceived event if new messages were fetched.
    if bodies_fetched > 0 {
        events.push(SyncEvent::NewMailReceived {
            account_id: account_id.to_string(),
            folder_name: folder_name.to_string(),
            bodies_fetched,
        });
    }

    Ok(IncrementalSyncResult {
        bodies_fetched,
        flags_updated,
        full_refetch: false,
        events,
        orphaned_hashes: Vec::new(),
        uidvalidity: select_uidvalidity,
        highestmodseq: select_highestmodseq,
    })
}

/// Perform UID-set-diff sync using only in-process data (for testing).
#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn incremental_sync_uid_diff_with_data(
    conn: &Connection,
    content_store: &dyn ContentStore,
    account_id: &str,
    folder_name: &str,
    select_uidvalidity: u32,
    select_highestmodseq: Option<u64>,
    _server_uids: &[u32],
    server_flags: &[crate::services::imap_client::UidFlagEntry],
    new_messages: Vec<crate::services::imap_client::RawFetchedMessage>,
) -> Result<IncrementalSyncResult, FetchError> {
    use std::collections::HashSet;

    let folder_id = find_folder_id(conn, account_id, folder_name)?
        .ok_or_else(|| FetchError::FolderNotFound(folder_name.to_string()))?;

    let (cached_uidvalidity, _) = load_folder_sync_state(conn, folder_id)?;

    // Check UIDVALIDITY.
    if let Some(cached_uv) = cached_uidvalidity {
        if select_uidvalidity != cached_uv {
            let tx = conn.unchecked_transaction().map_err(|e| {
                FetchError::Database(crate::services::database::DatabaseError::Sqlite(e))
            })?;
            let orphaned_hashes = delete_messages_for_folder(&tx, account_id, folder_id)?;
            update_folder_sync_state(&tx, folder_id, 0, None)?;
            tx.commit().map_err(|e| {
                FetchError::Database(crate::services::database::DatabaseError::Sqlite(e))
            })?;
            for hash in &orphaned_hashes {
                let _ = content_store.delete(hash);
            }
            return Ok(IncrementalSyncResult {
                bodies_fetched: 0,
                flags_updated: 0,
                full_refetch: true,
                events: Vec::new(),
                orphaned_hashes,
                uidvalidity: select_uidvalidity,
                highestmodseq: select_highestmodseq,
            });
        }
    }

    let local_uids: HashSet<u32> = load_uids_for_folder(conn, account_id, folder_id)?
        .into_iter()
        .collect();

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| FetchError::Database(crate::services::database::DatabaseError::Sqlite(e)))?;

    // Detect flag changes for existing messages.
    let mut flags_updated = 0;
    let mut events = Vec::new();
    for entry in server_flags {
        if !local_uids.contains(&entry.uid) {
            continue;
        }
        if let Some((msg_id, old_flags, pending)) =
            find_message_by_uid_in_folder_with_pending(&tx, account_id, entry.uid, folder_id)?
        {
            let sf = flags_from_imap(&entry.flags);
            match detect_flag_change(old_flags, sf, pending) {
                FlagChangeAction::Apply { new_flags } => {
                    update_message_flags(&tx, msg_id, new_flags)?;
                    events.push(make_flag_change_event(account_id, msg_id, new_flags));
                    flags_updated += 1;
                }
                FlagChangeAction::NoChange | FlagChangeAction::SkippedPendingSync => {}
            }
        }
    }

    // Insert new messages.
    let mut bodies_fetched = 0;
    for raw_msg in &new_messages {
        if local_uids.contains(&raw_msg.uid) {
            continue;
        }
        let content_hash = content_store.put(&raw_msg.data)?;
        let new_msg = parse_raw_message(
            account_id,
            raw_msg.uid,
            None,
            flags_from_imap(&raw_msg.flags),
            &content_hash,
            &raw_msg.data,
        );
        insert_message(&tx, &new_msg, folder_id)?;
        bodies_fetched += 1;
    }

    update_folder_sync_state(&tx, folder_id, select_uidvalidity, select_highestmodseq)?;

    tx.commit()
        .map_err(|e| FetchError::Database(crate::services::database::DatabaseError::Sqlite(e)))?;

    // Emit NewMailReceived event if new messages were fetched.
    if bodies_fetched > 0 {
        events.push(SyncEvent::NewMailReceived {
            account_id: account_id.to_string(),
            folder_name: folder_name.to_string(),
            bodies_fetched,
        });
    }

    Ok(IncrementalSyncResult {
        bodies_fetched,
        flags_updated,
        full_refetch: false,
        events,
        orphaned_hashes: Vec::new(),
        uidvalidity: select_uidvalidity,
        highestmodseq: select_highestmodseq,
    })
}

/// Handle UIDVALIDITY change: invalidate folder, delete stale rows, re-fetch.
#[allow(dead_code)]
fn handle_uidvalidity_change(
    conn: &Connection,
    content_store: &dyn ContentStore,
    params: &ImapConnectParams,
    folder_name: &str,
    folder_id: i64,
) -> Result<IncrementalSyncResult, FetchError> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| FetchError::Database(crate::services::database::DatabaseError::Sqlite(e)))?;

    // Delete stale messages for this folder; collect orphaned hashes.
    let orphaned_hashes = delete_messages_for_folder(&tx, &params.account_id, folder_id)?;

    // Reset folder sync state.
    update_folder_sync_state(&tx, folder_id, 0, None)?;

    tx.commit()
        .map_err(|e| FetchError::Database(crate::services::database::DatabaseError::Sqlite(e)))?;

    // Delete orphaned .eml files from content store.
    for hash in &orphaned_hashes {
        let _ = content_store.delete(hash);
    }

    // Re-fetch from scratch.
    let full = fetch_and_store_folder(conn, content_store, params, folder_name)?;

    Ok(IncrementalSyncResult {
        bodies_fetched: full.messages_fetched,
        flags_updated: 0,
        full_refetch: true,
        events: Vec::new(),
        orphaned_hashes,
        uidvalidity: full.uidvalidity,
        highestmodseq: full.highestmodseq,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::imap_check::ImapFolder;
    use crate::core::message::{NewMessage, FLAG_FLAGGED, FLAG_SEEN};
    use crate::services::database::open_and_migrate;
    use crate::services::folder_store::replace_folders;
    use crate::services::imap_client::{ChangedMessage, RawFetchedMessage};
    use crate::services::memory_content_store::MemoryContentStore;
    use crate::services::message_store::{count_messages, find_message_by_uid_in_folder};
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

    fn make_raw_email(subject: &str) -> Vec<u8> {
        format!(
            "From: test@example.com\r\n\
             Subject: {subject}\r\n\
             Message-ID: <{subject}@example.com>\r\n\
             Date: Mon, 1 Jan 2024 12:00:00 +0000\r\n\
             \r\n\
             Body of {subject}\r\n"
        )
        .into_bytes()
    }

    fn seed_initial_messages(conn: &Connection, folder_id: i64) {
        // Simulate a first sync with 3 messages, uidvalidity=100, highestmodseq=50.
        let msg1 = NewMessage {
            account_id: "acct-1".to_string(),
            uid: 1,
            modseq: Some(10),
            message_id: Some("<msg1@example.com>".to_string()),
            in_reply_to: None,
            references_header: None,
            from_addresses: Some("a@example.com".to_string()),
            to_addresses: Some("b@example.com".to_string()),
            cc_addresses: None,
            bcc_addresses: None,
            subject: Some("Message 1".to_string()),
            date_received: Some(1700000000),
            date_sent: Some(1700000000),
            flags: 0,
            size: 100,
            content_hash: "hash1".to_string(),
            body_text: Some("body1".to_string()),
            thread_id: None,
            server_thread_id: None,
        };
        insert_message(conn, &msg1, folder_id).unwrap();

        let mut msg2 = msg1.clone();
        msg2.uid = 2;
        msg2.modseq = Some(20);
        msg2.message_id = Some("<msg2@example.com>".to_string());
        msg2.subject = Some("Message 2".to_string());
        msg2.content_hash = "hash2".to_string();
        msg2.body_text = Some("body2".to_string());
        insert_message(conn, &msg2, folder_id).unwrap();

        let mut msg3 = msg1.clone();
        msg3.uid = 3;
        msg3.modseq = Some(30);
        msg3.message_id = Some("<msg3@example.com>".to_string());
        msg3.subject = Some("Message 3".to_string());
        msg3.content_hash = "hash3".to_string();
        msg3.body_text = Some("body3".to_string());
        insert_message(conn, &msg3, folder_id).unwrap();

        update_folder_sync_state(conn, folder_id, 100, Some(50)).unwrap();
    }

    // --- CONDSTORE tests ---

    #[test]
    fn condstore_unchanged_folder_fetches_zero_bodies() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let store = MemoryContentStore::new();

        seed_initial_messages(&conn, fid);

        // Second sync with no changes — CHANGEDSINCE returns nothing.
        let result = incremental_sync_condstore_with_data(
            &conn,
            &store,
            "acct-1",
            "INBOX",
            100,        // same uidvalidity
            Some(50),   // same highestmodseq
            Vec::new(), // no changed messages
        )
        .unwrap();

        assert_eq!(result.bodies_fetched, 0);
        assert_eq!(result.flags_updated, 0);
        assert!(!result.full_refetch);
    }

    #[test]
    fn condstore_one_new_message_fetches_one_body() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let store = MemoryContentStore::new();

        seed_initial_messages(&conn, fid);

        // Server has one new message (uid=4).
        let new_body = make_raw_email("NewMessage");
        let changed = vec![ChangedMessage {
            uid: 4,
            flags: String::new(),
            modseq: Some(60),
            body: Some(new_body),
        }];

        let result = incremental_sync_condstore_with_data(
            &conn,
            &store,
            "acct-1",
            "INBOX",
            100,
            Some(60),
            changed,
        )
        .unwrap();

        assert_eq!(result.bodies_fetched, 1);
        assert_eq!(result.flags_updated, 0);
        assert!(!result.full_refetch);
    }

    #[test]
    fn condstore_flag_change_updates_flags_emits_event() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let store = MemoryContentStore::new();

        seed_initial_messages(&conn, fid);

        // Server reports uid=2 flags changed to \Seen.
        let changed = vec![ChangedMessage {
            uid: 2,
            flags: "\\Seen".to_string(),
            modseq: Some(55),
            body: None,
        }];

        let result = incremental_sync_condstore_with_data(
            &conn,
            &store,
            "acct-1",
            "INBOX",
            100,
            Some(55),
            changed,
        )
        .unwrap();

        assert_eq!(result.bodies_fetched, 0, "no body fetches for flag change");
        assert_eq!(result.flags_updated, 1);
        assert_eq!(result.events.len(), 1);

        // Verify the flag was persisted.
        let (msg_id, new_flags) = find_message_by_uid_in_folder(&conn, "acct-1", 2, fid)
            .unwrap()
            .unwrap();
        assert_eq!(new_flags, FLAG_SEEN);

        // Verify the event.
        match &result.events[0] {
            SyncEvent::ServerFlagChange {
                account_id,
                message_id,
                new_flags,
            } => {
                assert_eq!(account_id, "acct-1");
                assert_eq!(*message_id, msg_id);
                assert_eq!(*new_flags, FLAG_SEEN);
            }
            _ => panic!("expected ServerFlagChange event"),
        }
    }

    #[test]
    fn condstore_uidvalidity_change_invalidates_and_refetches() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let store = MemoryContentStore::new();

        // Seed content store with files for the initial messages.
        store.put(b"content1").unwrap(); // hash1 equivalent
        store.put(b"content2").unwrap();
        store.put(b"content3").unwrap();

        seed_initial_messages(&conn, fid);
        assert_eq!(
            crate::services::message_store::count_messages(&conn, "acct-1").unwrap(),
            3
        );

        // UIDVALIDITY changed (200 != 100).
        let result = incremental_sync_condstore_with_data(
            &conn,
            &store,
            "acct-1",
            "INBOX",
            200, // different uidvalidity!
            Some(10),
            Vec::new(),
        )
        .unwrap();

        assert!(result.full_refetch);
        // Stale messages should be deleted.
        assert_eq!(
            crate::services::message_store::count_messages(&conn, "acct-1").unwrap(),
            0
        );
        // Orphaned hashes should be returned.
        assert_eq!(result.orphaned_hashes.len(), 3);
    }

    // --- UID-set diff tests ---

    #[test]
    fn uid_diff_unchanged_folder_fetches_zero_bodies() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let store = MemoryContentStore::new();

        seed_initial_messages(&conn, fid);

        // Server has same UIDs as local, no flag changes.
        let result = incremental_sync_uid_diff_with_data(
            &conn,
            &store,
            "acct-1",
            "INBOX",
            100,
            None,
            &[1, 2, 3],
            &[], // no flag data — flags unchanged
            Vec::new(),
        )
        .unwrap();

        assert_eq!(result.bodies_fetched, 0);
        assert!(!result.full_refetch);
    }

    #[test]
    fn uid_diff_one_new_message_fetches_one_body() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let store = MemoryContentStore::new();

        seed_initial_messages(&conn, fid);

        // Server has uid=4 which is new.
        let new_body = make_raw_email("NewUidDiffMessage");
        let new_messages = vec![RawFetchedMessage {
            uid: 4,
            flags: String::new(),
            data: new_body,
        }];

        let result = incremental_sync_uid_diff_with_data(
            &conn,
            &store,
            "acct-1",
            "INBOX",
            100,
            None,
            &[1, 2, 3, 4],
            &[], // no flag changes for existing messages
            new_messages,
        )
        .unwrap();

        assert_eq!(result.bodies_fetched, 1);
        assert!(!result.full_refetch);
    }

    #[test]
    fn uid_diff_uidvalidity_change_invalidates() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let store = MemoryContentStore::new();

        seed_initial_messages(&conn, fid);

        // UIDVALIDITY changed.
        let result = incremental_sync_uid_diff_with_data(
            &conn,
            &store,
            "acct-1",
            "INBOX",
            999, // different!
            None,
            &[1, 2, 3],
            &[],
            Vec::new(),
        )
        .unwrap();

        assert!(result.full_refetch);
        assert_eq!(
            crate::services::message_store::count_messages(&conn, "acct-1").unwrap(),
            0
        );
    }

    // --- Server flag change detection tests ---

    #[test]
    fn condstore_skips_flag_update_when_pending_sync() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let store = MemoryContentStore::new();

        seed_initial_messages(&conn, fid);

        // Mark message uid=2 as having pending local flag changes.
        let (msg_id, _) = find_message_by_uid_in_folder(&conn, "acct-1", 2, fid)
            .unwrap()
            .unwrap();
        crate::services::message_store::update_message_flags_pending(&conn, msg_id, FLAG_FLAGGED)
            .unwrap();

        // Server reports uid=2 flags changed to \Seen (different from local \Flagged).
        let changed = vec![ChangedMessage {
            uid: 2,
            flags: "\\Seen".to_string(),
            modseq: Some(55),
            body: None,
        }];

        let result = incremental_sync_condstore_with_data(
            &conn,
            &store,
            "acct-1",
            "INBOX",
            100,
            Some(55),
            changed,
        )
        .unwrap();

        // Should NOT update flags because pending_sync is set.
        assert_eq!(result.flags_updated, 0);
        assert!(result.events.is_empty());

        // Verify local flags are still \Flagged (not overwritten with \Seen).
        let (_, flags) = find_message_by_uid_in_folder(&conn, "acct-1", 2, fid)
            .unwrap()
            .unwrap();
        assert_eq!(
            flags, FLAG_FLAGGED,
            "local flags should not be overwritten when pending_sync"
        );
    }

    #[test]
    fn uid_diff_detects_flag_change_on_existing_message() {
        use crate::services::imap_client::UidFlagEntry;

        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let store = MemoryContentStore::new();

        seed_initial_messages(&conn, fid);

        // Server reports uid=1 now has \Seen flag.
        let server_flags = vec![
            UidFlagEntry {
                uid: 1,
                flags: "\\Seen".to_string(),
            },
            UidFlagEntry {
                uid: 2,
                flags: String::new(),
            },
            UidFlagEntry {
                uid: 3,
                flags: String::new(),
            },
        ];

        let result = incremental_sync_uid_diff_with_data(
            &conn,
            &store,
            "acct-1",
            "INBOX",
            100,
            None,
            &[1, 2, 3],
            &server_flags,
            Vec::new(),
        )
        .unwrap();

        assert_eq!(result.bodies_fetched, 0);
        assert_eq!(result.flags_updated, 1);
        assert_eq!(result.events.len(), 1);

        // Verify the flag was persisted.
        let (msg_id, new_flags) = find_message_by_uid_in_folder(&conn, "acct-1", 1, fid)
            .unwrap()
            .unwrap();
        assert_eq!(new_flags, FLAG_SEEN);

        // Verify the event.
        match &result.events[0] {
            SyncEvent::ServerFlagChange {
                account_id,
                message_id,
                new_flags,
            } => {
                assert_eq!(account_id, "acct-1");
                assert_eq!(*message_id, msg_id);
                assert_eq!(*new_flags, FLAG_SEEN);
            }
            _ => panic!("expected ServerFlagChange event"),
        }
    }

    #[test]
    fn uid_diff_skips_flag_update_when_pending_sync() {
        use crate::services::imap_client::UidFlagEntry;

        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let store = MemoryContentStore::new();

        seed_initial_messages(&conn, fid);

        // Mark message uid=1 as having pending local flag changes.
        let (msg_id, _) = find_message_by_uid_in_folder(&conn, "acct-1", 1, fid)
            .unwrap()
            .unwrap();
        crate::services::message_store::update_message_flags_pending(&conn, msg_id, FLAG_FLAGGED)
            .unwrap();

        // Server reports uid=1 has \Seen.
        let server_flags = vec![UidFlagEntry {
            uid: 1,
            flags: "\\Seen".to_string(),
        }];

        let result = incremental_sync_uid_diff_with_data(
            &conn,
            &store,
            "acct-1",
            "INBOX",
            100,
            None,
            &[1, 2, 3],
            &server_flags,
            Vec::new(),
        )
        .unwrap();

        assert_eq!(result.flags_updated, 0);
        assert!(result.events.is_empty());

        // Verify local flags unchanged.
        let (_, flags) = find_message_by_uid_in_folder(&conn, "acct-1", 1, fid)
            .unwrap()
            .unwrap();
        assert_eq!(flags, FLAG_FLAGGED);
    }

    #[test]
    fn uid_diff_detects_multiple_flag_types() {
        use crate::core::message::{FLAG_ANSWERED, FLAG_DELETED, FLAG_DRAFT};
        use crate::services::imap_client::UidFlagEntry;

        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let store = MemoryContentStore::new();

        seed_initial_messages(&conn, fid);

        // Server reports all five standard flags on uid=1.
        let all_flags = "\\Seen \\Flagged \\Answered \\Deleted \\Draft";
        let server_flags = vec![UidFlagEntry {
            uid: 1,
            flags: all_flags.to_string(),
        }];

        let result = incremental_sync_uid_diff_with_data(
            &conn,
            &store,
            "acct-1",
            "INBOX",
            100,
            None,
            &[1, 2, 3],
            &server_flags,
            Vec::new(),
        )
        .unwrap();

        assert_eq!(result.flags_updated, 1);

        let (_, new_flags) = find_message_by_uid_in_folder(&conn, "acct-1", 1, fid)
            .unwrap()
            .unwrap();
        assert_eq!(
            new_flags,
            FLAG_SEEN | FLAG_FLAGGED | FLAG_ANSWERED | FLAG_DELETED | FLAG_DRAFT
        );
    }

    #[test]
    fn condstore_detects_all_five_standard_flags() {
        use crate::core::message::{FLAG_ANSWERED, FLAG_DELETED, FLAG_DRAFT};

        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let store = MemoryContentStore::new();

        seed_initial_messages(&conn, fid);

        // Server reports uid=3 with all five flags.
        let changed = vec![ChangedMessage {
            uid: 3,
            flags: "\\Seen \\Flagged \\Answered \\Deleted \\Draft".to_string(),
            modseq: Some(60),
            body: None,
        }];

        let result = incremental_sync_condstore_with_data(
            &conn,
            &store,
            "acct-1",
            "INBOX",
            100,
            Some(60),
            changed,
        )
        .unwrap();

        assert_eq!(result.flags_updated, 1);

        let (_, new_flags) = find_message_by_uid_in_folder(&conn, "acct-1", 3, fid)
            .unwrap()
            .unwrap();
        assert_eq!(
            new_flags,
            FLAG_SEEN | FLAG_FLAGGED | FLAG_ANSWERED | FLAG_DELETED | FLAG_DRAFT
        );
    }

    // --- Conflict resolution integration tests (story 5) ---

    #[test]
    fn conflict_server_change_skipped_pending_local_flag_then_local_wins() {
        // AC1+AC3 integration: local flag change creates pending op, server
        // sends a different flag change during sync — local intent is preserved.
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let store = MemoryContentStore::new();

        seed_initial_messages(&conn, fid);

        // User flags message uid=1 locally.
        let (msg_id, _) = find_message_by_uid_in_folder(&conn, "acct-1", 1, fid)
            .unwrap()
            .unwrap();
        crate::services::message_store::update_message_flags_pending(&conn, msg_id, FLAG_FLAGGED)
            .unwrap();

        // Server simultaneously reports uid=1 as \Seen (different change).
        let changed = vec![ChangedMessage {
            uid: 1,
            flags: "\\Seen".to_string(),
            modseq: Some(55),
            body: None,
        }];

        let result = incremental_sync_condstore_with_data(
            &conn,
            &store,
            "acct-1",
            "INBOX",
            100,
            Some(55),
            changed,
        )
        .unwrap();

        // Server change must be skipped — local intent wins.
        assert_eq!(result.flags_updated, 0);
        assert!(result.events.is_empty());

        // Local flags remain as the user set them.
        let (_, flags) = find_message_by_uid_in_folder(&conn, "acct-1", 1, fid)
            .unwrap()
            .unwrap();
        assert_eq!(flags, FLAG_FLAGGED);
    }

    #[test]
    fn conflict_server_accepted_when_no_pending_op() {
        // AC2: When no pending local op exists, server state is accepted.
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let store = MemoryContentStore::new();

        seed_initial_messages(&conn, fid);

        // No pending local changes on uid=2.
        let changed = vec![ChangedMessage {
            uid: 2,
            flags: "\\Seen \\Flagged".to_string(),
            modseq: Some(55),
            body: None,
        }];

        let result = incremental_sync_condstore_with_data(
            &conn,
            &store,
            "acct-1",
            "INBOX",
            100,
            Some(55),
            changed,
        )
        .unwrap();

        assert_eq!(result.flags_updated, 1);
        assert_eq!(result.events.len(), 1);

        let (_, flags) = find_message_by_uid_in_folder(&conn, "acct-1", 2, fid)
            .unwrap()
            .unwrap();
        assert_eq!(flags, FLAG_SEEN | FLAG_FLAGGED);
    }

    #[test]
    fn conflict_deleted_flag_from_server_skipped_when_local_pending() {
        // AC4: Server sets \Deleted while local has a pending flag change.
        // Conflict resolution prefers keeping the message (no data loss).
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let store = MemoryContentStore::new();

        seed_initial_messages(&conn, fid);

        // User has a pending local flag change on uid=3.
        let (msg_id, _) = find_message_by_uid_in_folder(&conn, "acct-1", 3, fid)
            .unwrap()
            .unwrap();
        crate::services::message_store::update_message_flags_pending(
            &conn,
            msg_id,
            crate::core::message::FLAG_SEEN,
        )
        .unwrap();

        // Server reports uid=3 as \Deleted.
        let changed = vec![ChangedMessage {
            uid: 3,
            flags: "\\Deleted".to_string(),
            modseq: Some(60),
            body: None,
        }];

        let result = incremental_sync_condstore_with_data(
            &conn,
            &store,
            "acct-1",
            "INBOX",
            100,
            Some(60),
            changed,
        )
        .unwrap();

        // Server's \Deleted must be skipped — message is preserved.
        assert_eq!(result.flags_updated, 0);
        assert!(result.events.is_empty());

        let (_, flags) = find_message_by_uid_in_folder(&conn, "acct-1", 3, fid)
            .unwrap()
            .unwrap();
        assert_eq!(
            flags,
            crate::core::message::FLAG_SEEN,
            "message kept with local flags, not deleted"
        );
    }

    #[test]
    fn uid_diff_flag_change_with_new_message_simultaneously() {
        use crate::services::imap_client::UidFlagEntry;

        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let store = MemoryContentStore::new();

        seed_initial_messages(&conn, fid);

        // Server reports uid=2 flagged, and a new uid=4.
        let server_flags = vec![
            UidFlagEntry {
                uid: 1,
                flags: String::new(),
            },
            UidFlagEntry {
                uid: 2,
                flags: "\\Flagged".to_string(),
            },
            UidFlagEntry {
                uid: 3,
                flags: String::new(),
            },
            UidFlagEntry {
                uid: 4,
                flags: String::new(),
            },
        ];
        let new_body = make_raw_email("NewMsg4");
        let new_messages = vec![RawFetchedMessage {
            uid: 4,
            flags: String::new(),
            data: new_body,
        }];

        let result = incremental_sync_uid_diff_with_data(
            &conn,
            &store,
            "acct-1",
            "INBOX",
            100,
            None,
            &[1, 2, 3, 4],
            &server_flags,
            new_messages,
        )
        .unwrap();

        assert_eq!(result.bodies_fetched, 1);
        assert_eq!(result.flags_updated, 1);
        // 1 flag-change event + 1 NewMailReceived event.
        assert_eq!(result.events.len(), 2);
    }

    // --- New message detection tests (story 10) ---

    #[test]
    fn condstore_new_message_emits_new_mail_received_event() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let store = MemoryContentStore::new();

        seed_initial_messages(&conn, fid);

        let new_body = make_raw_email("Detected");
        let changed = vec![ChangedMessage {
            uid: 4,
            flags: "\\Seen".to_string(),
            modseq: Some(60),
            body: Some(new_body),
        }];

        let result = incremental_sync_condstore_with_data(
            &conn,
            &store,
            "acct-1",
            "INBOX",
            100,
            Some(60),
            changed,
        )
        .unwrap();

        assert_eq!(result.bodies_fetched, 1);
        // Should contain a NewMailReceived event.
        let new_mail_events: Vec<_> = result
            .events
            .iter()
            .filter(|e| matches!(e, SyncEvent::NewMailReceived { .. }))
            .collect();
        assert_eq!(new_mail_events.len(), 1);
        match &new_mail_events[0] {
            SyncEvent::NewMailReceived {
                account_id,
                folder_name,
                bodies_fetched,
            } => {
                assert_eq!(account_id, "acct-1");
                assert_eq!(folder_name, "INBOX");
                assert_eq!(*bodies_fetched, 1);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn uid_diff_new_message_emits_new_mail_received_event() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let store = MemoryContentStore::new();

        seed_initial_messages(&conn, fid);

        let new_body = make_raw_email("UidDiffDetected");
        let new_messages = vec![RawFetchedMessage {
            uid: 5,
            flags: String::new(),
            data: new_body,
        }];

        let result = incremental_sync_uid_diff_with_data(
            &conn,
            &store,
            "acct-1",
            "INBOX",
            100,
            None,
            &[1, 2, 3, 5],
            &[],
            new_messages,
        )
        .unwrap();

        assert_eq!(result.bodies_fetched, 1);
        let new_mail_events: Vec<_> = result
            .events
            .iter()
            .filter(|e| matches!(e, SyncEvent::NewMailReceived { .. }))
            .collect();
        assert_eq!(new_mail_events.len(), 1);
        match &new_mail_events[0] {
            SyncEvent::NewMailReceived {
                account_id,
                folder_name,
                bodies_fetched,
            } => {
                assert_eq!(account_id, "acct-1");
                assert_eq!(folder_name, "INBOX");
                assert_eq!(*bodies_fetched, 1);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn no_new_mail_event_when_no_new_messages() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let store = MemoryContentStore::new();

        seed_initial_messages(&conn, fid);

        // CONDSTORE: no changed messages.
        let result = incremental_sync_condstore_with_data(
            &conn,
            &store,
            "acct-1",
            "INBOX",
            100,
            Some(50),
            Vec::new(),
        )
        .unwrap();

        assert_eq!(result.bodies_fetched, 0);
        let new_mail_events: Vec<_> = result
            .events
            .iter()
            .filter(|e| matches!(e, SyncEvent::NewMailReceived { .. }))
            .collect();
        assert!(new_mail_events.is_empty());
    }

    #[test]
    fn no_duplicates_on_repeated_sync_condstore() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let store = MemoryContentStore::new();

        seed_initial_messages(&conn, fid);

        // First sync: new message uid=4.
        let new_body = make_raw_email("NoDup");
        let changed = vec![ChangedMessage {
            uid: 4,
            flags: String::new(),
            modseq: Some(60),
            body: Some(new_body.clone()),
        }];

        let result1 = incremental_sync_condstore_with_data(
            &conn,
            &store,
            "acct-1",
            "INBOX",
            100,
            Some(60),
            changed,
        )
        .unwrap();
        assert_eq!(result1.bodies_fetched, 1);
        assert_eq!(count_messages(&conn, "acct-1").unwrap(), 4);

        // Second sync: same message uid=4 reported again (e.g. flag change).
        let changed2 = vec![ChangedMessage {
            uid: 4,
            flags: String::new(),
            modseq: Some(60),
            body: Some(new_body),
        }];

        let result2 = incremental_sync_condstore_with_data(
            &conn,
            &store,
            "acct-1",
            "INBOX",
            100,
            Some(60),
            changed2,
        )
        .unwrap();
        // Should NOT create a duplicate — uid=4 already exists.
        assert_eq!(result2.bodies_fetched, 0);
        assert_eq!(count_messages(&conn, "acct-1").unwrap(), 4);
    }

    #[test]
    fn no_duplicates_on_repeated_sync_uid_diff() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let store = MemoryContentStore::new();

        seed_initial_messages(&conn, fid);

        // First sync: new message uid=4.
        let new_body = make_raw_email("NoDupUid");
        let new_messages = vec![RawFetchedMessage {
            uid: 4,
            flags: String::new(),
            data: new_body.clone(),
        }];

        let result1 = incremental_sync_uid_diff_with_data(
            &conn,
            &store,
            "acct-1",
            "INBOX",
            100,
            None,
            &[1, 2, 3, 4],
            &[],
            new_messages,
        )
        .unwrap();
        assert_eq!(result1.bodies_fetched, 1);
        assert_eq!(count_messages(&conn, "acct-1").unwrap(), 4);

        // Second sync: uid=4 still on server.
        let new_messages2 = vec![RawFetchedMessage {
            uid: 4,
            flags: String::new(),
            data: new_body,
        }];

        let result2 = incremental_sync_uid_diff_with_data(
            &conn,
            &store,
            "acct-1",
            "INBOX",
            100,
            None,
            &[1, 2, 3, 4],
            &[],
            new_messages2,
        )
        .unwrap();
        // uid=4 already in local store — should not be inserted again.
        assert_eq!(result2.bodies_fetched, 0);
        assert_eq!(count_messages(&conn, "acct-1").unwrap(), 4);
    }

    #[test]
    fn new_message_has_correct_envelope_and_flags() {
        let (_dir, conn) = setup_db();
        let fid = folder_id(&conn);
        let store = MemoryContentStore::new();

        seed_initial_messages(&conn, fid);

        let new_body = make_raw_email("EnvelopeCheck");
        let changed = vec![ChangedMessage {
            uid: 10,
            flags: "\\Seen \\Flagged".to_string(),
            modseq: Some(70),
            body: Some(new_body),
        }];

        incremental_sync_condstore_with_data(
            &conn,
            &store,
            "acct-1",
            "INBOX",
            100,
            Some(70),
            changed,
        )
        .unwrap();

        // Verify the message was inserted with correct envelope and flags.
        let (msg_id, flags) = find_message_by_uid_in_folder(&conn, "acct-1", 10, fid)
            .unwrap()
            .unwrap();
        assert_eq!(flags, FLAG_SEEN | FLAG_FLAGGED);

        let msg = crate::services::message_store::load_message(&conn, msg_id)
            .unwrap()
            .unwrap();
        assert_eq!(msg.subject.as_deref(), Some("EnvelopeCheck"));
        assert_eq!(msg.uid, 10);
        assert_eq!(msg.account_id, "acct-1");
    }

    #[test]
    fn new_message_appears_in_correct_folder() {
        let (_dir, conn) = setup_db();
        let store = MemoryContentStore::new();

        // Create a second folder.
        let folders = vec![
            ImapFolder {
                name: "INBOX".to_string(),
                attributes: "".to_string(),
                role: None,
            },
            ImapFolder {
                name: "Sent".to_string(),
                attributes: "".to_string(),
                role: None,
            },
        ];
        replace_folders(&conn, "acct-1", &folders).unwrap();
        let sent_fid = find_folder_id(&conn, "acct-1", "Sent").unwrap().unwrap();
        update_folder_sync_state(&conn, sent_fid, 200, Some(10)).unwrap();

        // New message in Sent folder.
        let new_body = make_raw_email("SentMsg");
        let changed = vec![ChangedMessage {
            uid: 1,
            flags: "\\Seen".to_string(),
            modseq: Some(20),
            body: Some(new_body),
        }];

        let result = incremental_sync_condstore_with_data(
            &conn,
            &store,
            "acct-1",
            "Sent",
            200,
            Some(20),
            changed,
        )
        .unwrap();

        assert_eq!(result.bodies_fetched, 1);

        // Verify message is in Sent folder, not INBOX.
        let found_in_sent = find_message_by_uid_in_folder(&conn, "acct-1", 1, sent_fid).unwrap();
        assert!(found_in_sent.is_some());

        let inbox_fid = find_folder_id(&conn, "acct-1", "INBOX").unwrap().unwrap();
        let found_in_inbox = find_message_by_uid_in_folder(&conn, "acct-1", 1, inbox_fid).unwrap();
        assert!(found_in_inbox.is_none());

        // NewMailReceived event should reference "Sent".
        match &result.events.last().unwrap() {
            SyncEvent::NewMailReceived { folder_name, .. } => {
                assert_eq!(folder_name, "Sent");
            }
            _ => panic!("expected NewMailReceived event"),
        }
    }
}
