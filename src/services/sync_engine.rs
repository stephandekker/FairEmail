//! Background sync engine that owns network I/O for accounts.
//!
//! Runs on a dedicated `tokio` multi-threaded runtime hosted on a worker thread.
//! Spawns one task per active account. Processes `pending_operations` in insertion
//! order and emits typed events on a broadcast channel.
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::broadcast;

/// Type alias for the account-params lookup function.
type AccountParamsFn = dyn Fn(&str) -> Option<ImapConnectParams> + Send + Sync;

use crate::core::content_store::ContentStore;
use crate::core::message::FLAG_SEEN;
use crate::core::pending_operation::{OperationKind, OperationState, StoreFlagsPayload};
use crate::core::sync_event::SyncEvent;
use crate::services::database::{open_and_migrate, DatabaseError};
use crate::services::idle_service::{self, IdleWaiter, RealIdleWaiter};
use crate::services::imap_client::ImapConnectParams;
use crate::services::pending_ops_store;

/// Errors that can occur during sync operations.
#[derive(Debug, thiserror::Error)]
pub(crate) enum SyncError {
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("IMAP error: {0}")]
    Imap(String),
    #[error("credential error: {0}")]
    Credential(String),
    #[error("payload parse error: {0}")]
    PayloadParse(String),
}

/// Whether a sync error is transient (retryable) or permanent.
pub(crate) fn is_transient_error(error: &SyncError) -> bool {
    match error {
        SyncError::Database(_) => false,
        SyncError::Imap(msg) => {
            let lower = msg.to_lowercase();
            // Permanent IMAP errors
            if lower.contains("authentication")
                || lower.contains("auth")
                || lower.contains("login")
                || lower.contains("no such mailbox")
                || lower.contains("quota")
                || lower.contains("permission")
            {
                return false;
            }
            // Everything else is transient (network, timeout, etc.)
            true
        }
        SyncError::Credential(_) => false,
        SyncError::PayloadParse(_) => false,
    }
}

/// Backoff durations for transient failure retries (capped at 1 hour).
const BACKOFF_SECS: &[u64] = &[5, 30, 120, 600, 3600];

/// Get the backoff duration for a given retry count.
pub(crate) fn backoff_duration(retry_count: i32) -> std::time::Duration {
    let idx = (retry_count as usize).min(BACKOFF_SECS.len() - 1);
    std::time::Duration::from_secs(BACKOFF_SECS[idx])
}

/// Trait abstracting IMAP flag-store operations for testability.
pub(crate) trait ImapFlagStore: Send + Sync {
    /// Set flags on a message by UID in the given folder.
    /// Returns Ok(()) on success, or an error string.
    fn store_flags(
        &self,
        params: &ImapConnectParams,
        folder_name: &str,
        uid: u32,
        flags: u32,
    ) -> Result<(), String>;
}

/// Real IMAP flag store that connects to the server.
pub(crate) struct RealImapFlagStore;

impl ImapFlagStore for RealImapFlagStore {
    fn store_flags(
        &self,
        params: &ImapConnectParams,
        folder_name: &str,
        uid: u32,
        flags: u32,
    ) -> Result<(), String> {
        store_flags_on_server(params, folder_name, uid, flags)
    }
}

/// Mock IMAP flag store for testing.
pub(crate) struct MockImapFlagStore {
    pub should_fail: Option<String>,
}

impl ImapFlagStore for MockImapFlagStore {
    fn store_flags(
        &self,
        _params: &ImapConnectParams,
        _folder_name: &str,
        _uid: u32,
        _flags: u32,
    ) -> Result<(), String> {
        match &self.should_fail {
            Some(err) => Err(err.clone()),
            None => Ok(()),
        }
    }
}

/// Perform the actual IMAP STORE command on the server.
fn store_flags_on_server(
    params: &ImapConnectParams,
    folder_name: &str,
    uid: u32,
    flags: u32,
) -> Result<(), String> {
    use crate::core::account::EncryptionMode;
    use native_tls::TlsConnector;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    let addr_str = format!("{}:{}", params.host, params.port);
    let addr: std::net::SocketAddr = addr_str
        .parse()
        .or_else(|_| {
            use std::net::ToSocketAddrs;
            addr_str
                .to_socket_addrs()
                .map_err(|e| e.to_string())?
                .next()
                .ok_or_else(|| "DNS resolution failed".to_string())
        })
        .map_err(|e| format!("DNS resolution failed: {e}"))?;

    let tcp = TcpStream::connect_timeout(&addr, Duration::from_secs(30))
        .map_err(|e| format!("connection failed: {e}"))?;
    tcp.set_read_timeout(Some(Duration::from_secs(30))).ok();
    tcp.set_write_timeout(Some(Duration::from_secs(30))).ok();

    // Helper closures for reading/writing on either plain or TLS stream
    macro_rules! run_session {
        ($reader:expr, $writer:expr) => {{
            // Read greeting
            let mut line = String::new();
            $reader
                .read_line(&mut line)
                .map_err(|e| format!("read greeting: {e}"))?;
            if !line.to_uppercase().starts_with("* OK")
                && !line.to_uppercase().starts_with("* PREAUTH")
            {
                return Err(format!("unexpected greeting: {}", line.trim()));
            }

            // Login
            let username = imap_quote(&params.username);
            let password = imap_quote(&params.password);
            let cmd = format!("A001 LOGIN {username} {password}\r\n");
            $writer
                .write_all(cmd.as_bytes())
                .map_err(|e| format!("write login: {e}"))?;
            $writer.flush().map_err(|e| format!("flush login: {e}"))?;

            let mut resp = String::new();
            loop {
                let mut l = String::new();
                $reader
                    .read_line(&mut l)
                    .map_err(|e| format!("read login response: {e}"))?;
                resp.push_str(&l);
                if l.starts_with("A001") {
                    break;
                }
            }
            if !resp.contains("A001 OK") {
                return Err("authentication failed".to_string());
            }

            // Select folder
            let quoted_folder = imap_quote(folder_name);
            let cmd = format!("A002 SELECT {quoted_folder}\r\n");
            $writer
                .write_all(cmd.as_bytes())
                .map_err(|e| format!("write select: {e}"))?;
            $writer.flush().map_err(|e| format!("flush select: {e}"))?;

            loop {
                let mut l = String::new();
                $reader
                    .read_line(&mut l)
                    .map_err(|e| format!("read select: {e}"))?;
                if l.starts_with("A002") {
                    if !l.contains("OK") {
                        return Err(format!("SELECT failed: {}", l.trim()));
                    }
                    break;
                }
            }

            // Build flags string
            let mut flag_parts = Vec::new();
            if flags & FLAG_SEEN != 0 {
                flag_parts.push("\\Seen");
            }
            if flags & crate::core::message::FLAG_ANSWERED != 0 {
                flag_parts.push("\\Answered");
            }
            if flags & crate::core::message::FLAG_FLAGGED != 0 {
                flag_parts.push("\\Flagged");
            }
            if flags & crate::core::message::FLAG_DELETED != 0 {
                flag_parts.push("\\Deleted");
            }
            if flags & crate::core::message::FLAG_DRAFT != 0 {
                flag_parts.push("\\Draft");
            }
            let flags_str = flag_parts.join(" ");

            // UID STORE
            let cmd = format!("A003 UID STORE {uid} FLAGS ({flags_str})\r\n");
            $writer
                .write_all(cmd.as_bytes())
                .map_err(|e| format!("write STORE: {e}"))?;
            $writer.flush().map_err(|e| format!("flush STORE: {e}"))?;

            loop {
                let mut l = String::new();
                $reader
                    .read_line(&mut l)
                    .map_err(|e| format!("read STORE response: {e}"))?;
                if l.starts_with("A003") {
                    if !l.contains("OK") {
                        return Err(format!("STORE failed: {}", l.trim()));
                    }
                    break;
                }
            }

            // Logout
            let cmd = "A099 LOGOUT\r\n";
            let _ = $writer.write_all(cmd.as_bytes());
            let _ = $writer.flush();

            Ok(())
        }};
    }

    match params.encryption {
        EncryptionMode::SslTls => {
            let mut builder = TlsConnector::builder();
            if params.insecure || params.accepted_fingerprint.is_some() {
                builder.danger_accept_invalid_certs(true);
                builder.danger_accept_invalid_hostnames(true);
            }
            let connector = builder
                .build()
                .map_err(|e| format!("TLS build error: {e}"))?;
            let tls = connector
                .connect(&params.host, tcp)
                .map_err(|e| format!("TLS handshake failed: {e}"))?;
            let mut reader = BufReader::new(tls);
            // We need to split reading and writing. For TLS, we can get_mut on the BufReader.
            // However, BufReader borrows the stream. We use a different approach:
            // read from BufReader, write via get_mut.
            run_session!(reader, reader.get_mut())
        }
        EncryptionMode::None => {
            let tcp_clone = tcp.try_clone().map_err(|e| format!("clone tcp: {e}"))?;
            let mut reader = BufReader::new(tcp);
            let mut writer = tcp_clone;
            run_session!(reader, writer)
        }
        EncryptionMode::StartTls => {
            // Read greeting on plain, then upgrade
            let tcp_clone = tcp.try_clone().map_err(|e| format!("clone tcp: {e}"))?;
            let mut reader = BufReader::new(tcp);
            let mut writer = tcp_clone;

            let mut line = String::new();
            reader
                .read_line(&mut line)
                .map_err(|e| format!("read greeting: {e}"))?;
            if !line.to_uppercase().starts_with("* OK")
                && !line.to_uppercase().starts_with("* PREAUTH")
            {
                return Err(format!("unexpected greeting: {}", line.trim()));
            }

            // Send STARTTLS
            writer
                .write_all(b"A000 STARTTLS\r\n")
                .map_err(|e| format!("write STARTTLS: {e}"))?;
            writer.flush().map_err(|e| format!("flush STARTTLS: {e}"))?;

            let mut resp = String::new();
            loop {
                let mut l = String::new();
                reader
                    .read_line(&mut l)
                    .map_err(|e| format!("read STARTTLS resp: {e}"))?;
                resp.push_str(&l);
                if l.starts_with("A000") {
                    break;
                }
            }
            if !resp.to_uppercase().contains("OK") {
                return Err("STARTTLS rejected".to_string());
            }

            // Upgrade to TLS
            let tcp_inner = reader.into_inner();
            let mut builder = TlsConnector::builder();
            if params.insecure || params.accepted_fingerprint.is_some() {
                builder.danger_accept_invalid_certs(true);
                builder.danger_accept_invalid_hostnames(true);
            }
            let connector = builder.build().map_err(|e| format!("TLS build: {e}"))?;
            let tls = connector
                .connect(&params.host, tcp_inner)
                .map_err(|e| format!("STARTTLS upgrade: {e}"))?;
            let mut reader = BufReader::new(tls);
            run_session!(reader, reader.get_mut())
        }
    }
}

fn imap_quote(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// Handle to the running sync engine. Dropping it signals shutdown.
pub(crate) struct SyncEngineHandle {
    _shutdown_tx: tokio::sync::watch::Sender<bool>,
    _runtime_thread: std::thread::JoinHandle<()>,
    notify_tx: tokio::sync::mpsc::UnboundedSender<String>,
    idle_tx: tokio::sync::mpsc::UnboundedSender<IdleCommand>,
}

/// Commands for the IDLE subsystem.
enum IdleCommand {
    /// Start IDLE monitoring for an account (after initial sync).
    StartIdle { account_id: String },
}

impl SyncEngineHandle {
    /// Notify the engine that an account has new pending operations.
    pub fn notify_account(&self, account_id: &str) {
        let _ = self.notify_tx.send(account_id.to_string());
    }

    /// Start IDLE monitoring for an account after its initial sync completes.
    pub fn start_idle(&self, account_id: &str) {
        let _ = self.idle_tx.send(IdleCommand::StartIdle {
            account_id: account_id.to_string(),
        });
    }
}

/// Start the sync engine on a dedicated worker thread.
/// Returns a handle to interact with the engine and a broadcast receiver for events.
pub(crate) fn start_sync_engine(
    db_path: PathBuf,
    event_sender: broadcast::Sender<SyncEvent>,
    flag_store: Arc<dyn ImapFlagStore>,
    account_params_fn: Arc<AccountParamsFn>,
) -> SyncEngineHandle {
    start_sync_engine_with_idle(
        db_path,
        event_sender,
        flag_store,
        account_params_fn,
        Arc::new(RealIdleWaiter),
        None,
    )
}

/// Start the sync engine with explicit idle waiter and content store (for testing).
pub(crate) fn start_sync_engine_with_idle(
    db_path: PathBuf,
    event_sender: broadcast::Sender<SyncEvent>,
    flag_store: Arc<dyn ImapFlagStore>,
    account_params_fn: Arc<AccountParamsFn>,
    idle_waiter: Arc<dyn IdleWaiter>,
    content_store: Option<Arc<dyn ContentStore + Send + Sync>>,
) -> SyncEngineHandle {
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let (notify_tx, notify_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let (idle_tx, idle_rx) = tokio::sync::mpsc::unbounded_channel::<IdleCommand>();

    let thread = std::thread::Builder::new()
        .name("sync-engine".to_string())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("failed to build tokio runtime for sync engine");

            rt.block_on(engine_loop(
                db_path,
                event_sender,
                flag_store,
                account_params_fn,
                idle_waiter,
                content_store,
                shutdown_rx,
                notify_rx,
                idle_rx,
            ));
        })
        .expect("failed to spawn sync engine thread");

    SyncEngineHandle {
        _shutdown_tx: shutdown_tx,
        _runtime_thread: thread,
        notify_tx,
        idle_tx,
    }
}

#[allow(clippy::too_many_arguments)]
async fn engine_loop(
    db_path: PathBuf,
    event_sender: broadcast::Sender<SyncEvent>,
    flag_store: Arc<dyn ImapFlagStore>,
    account_params_fn: Arc<AccountParamsFn>,
    idle_waiter: Arc<dyn IdleWaiter>,
    content_store: Option<Arc<dyn ContentStore + Send + Sync>>,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
    mut notify_rx: tokio::sync::mpsc::UnboundedReceiver<String>,
    mut idle_rx: tokio::sync::mpsc::UnboundedReceiver<IdleCommand>,
) {
    loop {
        tokio::select! {
            _ = shutdown_rx.changed() => {
                break;
            }
            Some(account_id) = notify_rx.recv() => {
                let db_path = db_path.clone();
                let event_sender = event_sender.clone();
                let flag_store = flag_store.clone();
                let account_params_fn = account_params_fn.clone();

                tokio::spawn(async move {
                    process_account_ops(
                        &db_path,
                        &account_id,
                        &event_sender,
                        flag_store,
                        account_params_fn.as_ref(),
                    ).await;
                });
            }
            Some(cmd) = idle_rx.recv() => {
                match cmd {
                    IdleCommand::StartIdle { account_id } => {
                        // Only start IDLE if we have a content store.
                        if let Some(ref cs) = content_store {
                            let idle_shutdown_rx = shutdown_rx.clone();
                            tokio::spawn(idle_service::run_idle_loop(
                                account_id,
                                db_path.clone(),
                                account_params_fn.clone(),
                                event_sender.clone(),
                                cs.clone(),
                                idle_waiter.clone(),
                                idle_shutdown_rx,
                            ));
                        }
                    }
                }
            }
        }
    }
}

async fn process_account_ops(
    db_path: &std::path::Path,
    account_id: &str,
    event_sender: &broadcast::Sender<SyncEvent>,
    flag_store: Arc<dyn ImapFlagStore>,
    account_params_fn: &(dyn Fn(&str) -> Option<ImapConnectParams> + Send + Sync),
) {
    let conn = match open_and_migrate(db_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("sync engine: failed to open db: {e}");
            return;
        }
    };

    let ops = match pending_ops_store::load_pending_ops(&conn, account_id) {
        Ok(ops) => ops,
        Err(e) => {
            eprintln!("sync engine: failed to load pending ops: {e}");
            return;
        }
    };

    if ops.is_empty() {
        return;
    }

    let params = match account_params_fn(account_id) {
        Some(p) => p,
        None => {
            eprintln!("sync engine: no connection params for account {account_id}");
            return;
        }
    };

    // Batch STORE flags operations: group consecutive StoreFlags ops
    let mut i = 0;
    while i < ops.len() {
        let op = &ops[i];

        if op.state == OperationState::Failed {
            i += 1;
            continue;
        }

        // Mark in-flight
        if let Err(e) = pending_ops_store::mark_in_flight(&conn, op.id) {
            eprintln!("sync engine: mark in-flight failed: {e}");
            i += 1;
            continue;
        }

        match op.kind {
            OperationKind::StoreFlags => {
                let payload: StoreFlagsPayload = match serde_json::from_str(&op.payload) {
                    Ok(p) => p,
                    Err(e) => {
                        let err_msg = format!("invalid payload: {e}");
                        let _ = pending_ops_store::mark_failed(&conn, op.id, &err_msg);
                        let _ = event_sender.send(SyncEvent::OperationFailed {
                            account_id: account_id.to_string(),
                            operation_id: op.id,
                            error: err_msg,
                        });
                        i += 1;
                        continue;
                    }
                };

                // Execute on server (blocking I/O in spawn_blocking)
                let flag_store_clone = flag_store.clone();
                let params_clone = params.clone();
                let folder = payload.folder_name.clone();
                let uid = payload.uid;
                let flags = payload.new_flags;

                let result = tokio::task::spawn_blocking(move || {
                    flag_store_clone.store_flags(&params_clone, &folder, uid, flags)
                })
                .await;

                match result {
                    Ok(Ok(())) => {
                        // Success - delete the operation
                        let _ = pending_ops_store::complete_op(&conn, op.id);
                        let _ = event_sender.send(SyncEvent::MessageFlagsChanged {
                            account_id: account_id.to_string(),
                            message_id: payload.message_id,
                            new_flags: payload.new_flags,
                        });
                    }
                    Ok(Err(err_msg)) => {
                        let sync_err = SyncError::Imap(err_msg.clone());
                        if is_transient_error(&sync_err) {
                            match pending_ops_store::requeue_op(&conn, op.id, &err_msg) {
                                Ok(retry_count) => {
                                    let delay = backoff_duration(retry_count - 1);
                                    tokio::time::sleep(delay).await;
                                }
                                Err(e) => {
                                    eprintln!("sync engine: requeue failed: {e}");
                                }
                            }
                        } else {
                            let _ = pending_ops_store::mark_failed(&conn, op.id, &err_msg);
                            let _ = event_sender.send(SyncEvent::OperationFailed {
                                account_id: account_id.to_string(),
                                operation_id: op.id,
                                error: err_msg,
                            });
                        }
                    }
                    Err(e) => {
                        let err_msg = format!("task join error: {e}");
                        let _ = pending_ops_store::requeue_op(&conn, op.id, &err_msg);
                    }
                }
            }
        }

        i += 1;
    }
}

/// Toggle the read/unread flag on a message. Updates the local database immediately
/// and inserts a pending operation for the sync engine.
/// Returns the new flags value.
pub(crate) fn toggle_message_read(
    db_path: &std::path::Path,
    message_id: i64,
    account_id: &str,
    uid: u32,
    folder_name: &str,
    current_flags: u32,
) -> Result<u32, SyncError> {
    let conn = open_and_migrate(db_path)?;

    let new_flags = if current_flags & FLAG_SEEN != 0 {
        current_flags & !FLAG_SEEN
    } else {
        current_flags | FLAG_SEEN
    };

    // Update local DB immediately
    crate::services::message_store::update_message_flags(&conn, message_id, new_flags)
        .map_err(SyncError::Database)?;

    // Insert pending operation
    let payload = StoreFlagsPayload {
        message_id,
        uid,
        folder_name: folder_name.to_string(),
        new_flags,
    };
    let payload_json =
        serde_json::to_string(&payload).map_err(|e| SyncError::PayloadParse(e.to_string()))?;

    pending_ops_store::insert_pending_op(
        &conn,
        account_id,
        &OperationKind::StoreFlags,
        &payload_json,
    )
    .map_err(SyncError::Database)?;

    Ok(new_flags)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::account::EncryptionMode;
    use crate::core::message::{FLAG_ANSWERED, FLAG_SEEN};
    use crate::core::pending_operation::OperationState;
    use crate::services::database::open_and_migrate;
    use crate::services::imap_client::ImapConnectParams;
    use tempfile::TempDir;

    fn setup_db() -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_and_migrate(&db_path).unwrap();
        conn.execute(
            "INSERT INTO accounts (id, display_name, protocol, host, port, encryption, auth_method, username, credential)
             VALUES ('acct-1', 'Test', 'Imap', 'imap.example.com', 993, 'SslTls', 'Plain', 'user', '')",
            [],
        ).unwrap();
        // Insert a folder and message for testing
        conn.execute(
            "INSERT INTO folders (id, account_id, name, attributes) VALUES (1, 'acct-1', 'INBOX', '')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO messages (id, account_id, uid, flags, size, content_hash) VALUES (1, 'acct-1', 100, 0, 1024, 'hash1')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO message_folders (message_id, folder_id) VALUES (1, 1)",
            [],
        )
        .unwrap();
        drop(conn);
        (dir, db_path)
    }

    fn make_test_params() -> ImapConnectParams {
        ImapConnectParams {
            host: "imap.example.com".to_string(),
            port: 993,
            encryption: EncryptionMode::SslTls,
            username: "user".to_string(),
            password: "pass".to_string(),
            accepted_fingerprint: None,
            insecure: false,
            account_id: "acct-1".to_string(),
        }
    }

    #[test]
    fn toggle_read_sets_seen_and_creates_pending_op() {
        let (_dir, db_path) = setup_db();

        let new_flags = toggle_message_read(&db_path, 1, "acct-1", 100, "INBOX", 0).unwrap();
        assert_eq!(new_flags, FLAG_SEEN);

        // Verify message flags updated in DB
        let conn = open_and_migrate(&db_path).unwrap();
        let flags: u32 = conn
            .query_row("SELECT flags FROM messages WHERE id = 1", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(flags, FLAG_SEEN);

        // Verify pending operation created
        let ops = pending_ops_store::load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].kind, OperationKind::StoreFlags);
        assert_eq!(ops[0].state, OperationState::Pending);

        let payload: StoreFlagsPayload = serde_json::from_str(&ops[0].payload).unwrap();
        assert_eq!(payload.message_id, 1);
        assert_eq!(payload.uid, 100);
        assert_eq!(payload.new_flags, FLAG_SEEN);
    }

    #[test]
    fn toggle_read_unsets_seen_when_already_set() {
        let (_dir, db_path) = setup_db();

        // First set it to seen
        let conn = open_and_migrate(&db_path).unwrap();
        conn.execute(
            "UPDATE messages SET flags = ?1 WHERE id = 1",
            rusqlite::params![FLAG_SEEN | FLAG_ANSWERED],
        )
        .unwrap();
        drop(conn);

        let new_flags = toggle_message_read(
            &db_path,
            1,
            "acct-1",
            100,
            "INBOX",
            FLAG_SEEN | FLAG_ANSWERED,
        )
        .unwrap();
        assert_eq!(new_flags, FLAG_ANSWERED); // SEEN cleared, ANSWERED preserved
    }

    #[test]
    fn is_transient_error_classification() {
        assert!(is_transient_error(&SyncError::Imap(
            "connection timed out".to_string()
        )));
        assert!(is_transient_error(&SyncError::Imap(
            "network unreachable".to_string()
        )));
        assert!(!is_transient_error(&SyncError::Imap(
            "authentication failed".to_string()
        )));
        assert!(!is_transient_error(&SyncError::Imap(
            "auth rejected".to_string()
        )));
        assert!(!is_transient_error(&SyncError::Credential(
            "no credential".to_string()
        )));
        assert!(!is_transient_error(&SyncError::PayloadParse(
            "bad json".to_string()
        )));
    }

    #[test]
    fn backoff_duration_caps_at_one_hour() {
        assert_eq!(backoff_duration(0), std::time::Duration::from_secs(5));
        assert_eq!(backoff_duration(1), std::time::Duration::from_secs(30));
        assert_eq!(backoff_duration(2), std::time::Duration::from_secs(120));
        assert_eq!(backoff_duration(3), std::time::Duration::from_secs(600));
        assert_eq!(backoff_duration(4), std::time::Duration::from_secs(3600));
        assert_eq!(backoff_duration(100), std::time::Duration::from_secs(3600));
        // capped
    }

    #[tokio::test]
    async fn engine_processes_pending_ops_with_mock() {
        let (_dir, db_path) = setup_db();

        // Create a pending operation
        let conn = open_and_migrate(&db_path).unwrap();
        let payload = StoreFlagsPayload {
            message_id: 1,
            uid: 100,
            folder_name: "INBOX".to_string(),
            new_flags: FLAG_SEEN,
        };
        let payload_json = serde_json::to_string(&payload).unwrap();
        pending_ops_store::insert_pending_op(
            &conn,
            "acct-1",
            &OperationKind::StoreFlags,
            &payload_json,
        )
        .unwrap();
        drop(conn);

        let (event_tx, mut event_rx) = broadcast::channel::<SyncEvent>(16);
        let flag_store: Arc<dyn ImapFlagStore> = Arc::new(MockImapFlagStore { should_fail: None });
        let params = make_test_params();
        let account_params_fn: Arc<AccountParamsFn> = Arc::new(move |_| Some(params.clone()));

        process_account_ops(
            &db_path,
            "acct-1",
            &event_tx,
            flag_store.clone(),
            account_params_fn.as_ref(),
        )
        .await;

        // Operation should be completed (removed)
        let conn = open_and_migrate(&db_path).unwrap();
        let ops = pending_ops_store::load_pending_ops(&conn, "acct-1").unwrap();
        assert!(ops.is_empty());

        // Should have received a MessageFlagsChanged event
        let event = event_rx.try_recv().unwrap();
        match event {
            SyncEvent::MessageFlagsChanged {
                message_id,
                new_flags,
                ..
            } => {
                assert_eq!(message_id, 1);
                assert_eq!(new_flags, FLAG_SEEN);
            }
            _ => panic!("unexpected event"),
        }
    }

    #[tokio::test]
    async fn engine_handles_transient_failure() {
        let (_dir, db_path) = setup_db();

        let conn = open_and_migrate(&db_path).unwrap();
        let payload = StoreFlagsPayload {
            message_id: 1,
            uid: 100,
            folder_name: "INBOX".to_string(),
            new_flags: FLAG_SEEN,
        };
        let payload_json = serde_json::to_string(&payload).unwrap();
        pending_ops_store::insert_pending_op(
            &conn,
            "acct-1",
            &OperationKind::StoreFlags,
            &payload_json,
        )
        .unwrap();
        drop(conn);

        let (event_tx, _event_rx) = broadcast::channel::<SyncEvent>(16);
        let flag_store: Arc<dyn ImapFlagStore> = Arc::new(MockImapFlagStore {
            should_fail: Some("connection timed out".to_string()),
        });
        let params = make_test_params();
        let account_params_fn: Arc<AccountParamsFn> = Arc::new(move |_| Some(params.clone()));

        // Use a timeout so we don't wait for the full backoff
        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            process_account_ops(
                &db_path,
                "acct-1",
                &event_tx,
                flag_store.clone(),
                account_params_fn.as_ref(),
            ),
        )
        .await;

        // Operation should be requeued with retry_count > 0
        let conn = open_and_migrate(&db_path).unwrap();
        let ops = pending_ops_store::load_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].state, OperationState::Pending);
        assert!(ops[0].retry_count > 0);
        assert!(ops[0].last_error.as_ref().unwrap().contains("timed out"));
    }

    #[tokio::test]
    async fn engine_handles_permanent_failure() {
        let (_dir, db_path) = setup_db();

        let conn = open_and_migrate(&db_path).unwrap();
        let payload = StoreFlagsPayload {
            message_id: 1,
            uid: 100,
            folder_name: "INBOX".to_string(),
            new_flags: FLAG_SEEN,
        };
        let payload_json = serde_json::to_string(&payload).unwrap();
        pending_ops_store::insert_pending_op(
            &conn,
            "acct-1",
            &OperationKind::StoreFlags,
            &payload_json,
        )
        .unwrap();
        drop(conn);

        let (event_tx, mut event_rx) = broadcast::channel::<SyncEvent>(16);
        let flag_store: Arc<dyn ImapFlagStore> = Arc::new(MockImapFlagStore {
            should_fail: Some("authentication failed".to_string()),
        });
        let params = make_test_params();
        let account_params_fn: Arc<AccountParamsFn> = Arc::new(move |_| Some(params.clone()));

        process_account_ops(
            &db_path,
            "acct-1",
            &event_tx,
            flag_store.clone(),
            account_params_fn.as_ref(),
        )
        .await;

        // Operation should be marked failed (not in pending ops)
        let conn = open_and_migrate(&db_path).unwrap();
        let ops = pending_ops_store::load_pending_ops(&conn, "acct-1").unwrap();
        assert!(ops.is_empty());

        // Should have received an OperationFailed event
        let event = event_rx.try_recv().unwrap();
        match event {
            SyncEvent::OperationFailed { error, .. } => {
                assert!(error.contains("authentication"));
            }
            _ => panic!("unexpected event"),
        }
    }

    #[tokio::test]
    async fn engine_start_and_notify() {
        let (_dir, db_path) = setup_db();

        // Create a pending operation
        let conn = open_and_migrate(&db_path).unwrap();
        let payload = StoreFlagsPayload {
            message_id: 1,
            uid: 100,
            folder_name: "INBOX".to_string(),
            new_flags: FLAG_SEEN,
        };
        let payload_json = serde_json::to_string(&payload).unwrap();
        pending_ops_store::insert_pending_op(
            &conn,
            "acct-1",
            &OperationKind::StoreFlags,
            &payload_json,
        )
        .unwrap();
        drop(conn);

        let (event_tx, mut event_rx) = broadcast::channel::<SyncEvent>(16);
        let flag_store: Arc<dyn ImapFlagStore> = Arc::new(MockImapFlagStore { should_fail: None });
        let params = make_test_params();
        let account_params_fn: Arc<AccountParamsFn> = Arc::new(move |_| Some(params.clone()));

        let handle = start_sync_engine(db_path.clone(), event_tx, flag_store, account_params_fn);
        handle.notify_account("acct-1");

        // Wait for the event
        let event = tokio::time::timeout(std::time::Duration::from_secs(5), event_rx.recv())
            .await
            .unwrap()
            .unwrap();

        match event {
            SyncEvent::MessageFlagsChanged { message_id, .. } => {
                assert_eq!(message_id, 1);
            }
            _ => panic!("unexpected event"),
        }

        // Engine handle dropped here, triggering shutdown
        drop(handle);
    }

    #[test]
    fn stress_test_toggle_1000_messages() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_and_migrate(&db_path).unwrap();
        conn.execute(
            "INSERT INTO accounts (id, display_name, protocol, host, port, encryption, auth_method, username, credential)
             VALUES ('acct-1', 'Test', 'Imap', 'imap.example.com', 993, 'SslTls', 'Plain', 'user', '')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO folders (id, account_id, name, attributes) VALUES (1, 'acct-1', 'INBOX', '')",
            [],
        ).unwrap();

        // Insert 1000 messages
        for i in 1..=1000 {
            conn.execute(
                "INSERT INTO messages (id, account_id, uid, flags, size, content_hash)
                 VALUES (?1, 'acct-1', ?2, 0, 100, ?3)",
                rusqlite::params![i as i64, i as u32, format!("hash_{i}")],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO message_folders (message_id, folder_id) VALUES (?1, 1)",
                rusqlite::params![i as i64],
            )
            .unwrap();
        }
        drop(conn);

        // Toggle all 1000 messages - should not block
        let start = std::time::Instant::now();
        for i in 1..=1000 {
            toggle_message_read(&db_path, i, "acct-1", i as u32, "INBOX", 0).unwrap();
        }
        let elapsed = start.elapsed();

        // Verify all 1000 have pending ops
        let conn = open_and_migrate(&db_path).unwrap();
        let count = pending_ops_store::count_pending_ops(&conn, "acct-1").unwrap();
        assert_eq!(count, 1000);

        // Should complete in well under a second on any reasonable hardware
        assert!(
            elapsed.as_secs() < 10,
            "1000 toggles took too long: {elapsed:?}"
        );
    }
}
