//! IDLE service: runs the per-account IDLE/poll loop on the sync engine's
//! tokio runtime. Uses the state machine from `core::idle_manager` and
//! delegates I/O to `imap_client::run_idle_cycle` (or a mock for tests).

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::broadcast;

use crate::core::connection_log::{ConnectionLogEventType, ConnectionLogRecord};
use crate::core::content_store::ContentStore;
use crate::core::idle_manager::{
    self, IdleAction, IdleEvent, IdleState, DEFAULT_POLL_INTERVAL_SECS, IDLE_RENEWAL_SECS,
};
use crate::core::sync_event::SyncEvent;
use crate::services::connection_log_store::append_connection_logs;
use crate::services::database::open_and_migrate;
use crate::services::imap_client::{IdleWaitResult, ImapConnectParams};
use crate::services::message_fetch::incremental_sync_folder;
use crate::services::sync_state_store::load_sync_state;

/// Trait abstracting IDLE I/O for testability.
pub(crate) trait IdleWaiter: Send + Sync {
    /// Run one IDLE cycle (connect, login, SELECT, IDLE, wait, DONE, logout).
    /// Blocks for up to `timeout`.
    fn idle_wait(
        &self,
        params: &ImapConnectParams,
        folder_name: &str,
        timeout: Duration,
    ) -> (IdleWaitResult, Vec<ConnectionLogRecord>);
}

/// Real implementation that connects to the IMAP server.
pub(crate) struct RealIdleWaiter;

impl IdleWaiter for RealIdleWaiter {
    fn idle_wait(
        &self,
        params: &ImapConnectParams,
        folder_name: &str,
        timeout: Duration,
    ) -> (IdleWaitResult, Vec<ConnectionLogRecord>) {
        crate::services::imap_client::run_idle_cycle(params, folder_name, timeout)
    }
}

/// Type alias for the account-params lookup function.
type AccountParamsFn = dyn Fn(&str) -> Option<ImapConnectParams> + Send + Sync;

/// Run the IDLE/poll loop for one account. Blocks until shutdown.
///
/// This is an async function meant to be spawned on the sync engine's tokio runtime.
pub(crate) async fn run_idle_loop(
    account_id: String,
    db_path: PathBuf,
    account_params_fn: Arc<AccountParamsFn>,
    event_sender: broadcast::Sender<SyncEvent>,
    content_store: Arc<dyn ContentStore + Send + Sync>,
    idle_waiter: Arc<dyn IdleWaiter>,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
    // Check IDLE capability from the database.
    let idle_supported = check_idle_supported(&db_path, &account_id).unwrap_or_default();

    let mut state = IdleState::Initial;
    let (new_state, mut action) =
        idle_manager::next_state(&state, &IdleEvent::InitialSyncDone { idle_supported });
    state = new_state;

    loop {
        match action {
            IdleAction::EnterIdle | IdleAction::RenewIdle => {
                let params = match account_params_fn(&account_id) {
                    Some(p) => p,
                    None => {
                        // No params — can't connect. Wait and retry.
                        let (s, a) = idle_manager::next_state(&state, &IdleEvent::Disconnected);
                        state = s;
                        action = a;
                        continue;
                    }
                };

                let waiter = idle_waiter.clone();
                let folder = "INBOX".to_string();
                let timeout = Duration::from_secs(IDLE_RENEWAL_SECS);

                // Run the blocking IDLE cycle on a worker thread.
                let idle_result = tokio::select! {
                    _ = shutdown_rx.changed() => {
                        let (s, a) = idle_manager::next_state(&state, &IdleEvent::Shutdown);
                        state = s;
                        action = a;
                        continue;
                    }
                    result = tokio::task::spawn_blocking(move || {
                        waiter.idle_wait(&params, &folder, timeout)
                    }) => {
                        match result {
                            Ok(r) => r,
                            Err(e) => (IdleWaitResult::Error(format!("task error: {e}")), Vec::new()),
                        }
                    }
                };

                let (wait_result, logs) = idle_result;

                // Persist connection logs.
                write_logs(&db_path, &logs);

                // Map IDLE result to state machine event.
                let event = match &wait_result {
                    IdleWaitResult::NewMessages => IdleEvent::NewMail,
                    IdleWaitResult::FlagsOrExpunge => IdleEvent::FlagChange,
                    IdleWaitResult::Timeout => IdleEvent::RenewalTimeout,
                    IdleWaitResult::Error(_) => IdleEvent::Disconnected,
                };

                let (s, a) = idle_manager::next_state(&state, &event);
                state = s;
                action = a;
            }

            IdleAction::TriggerSync => {
                // Run incremental sync on a worker thread.
                let db = db_path.clone();
                let acct = account_id.clone();
                let cs = content_store.clone();
                let params_fn = account_params_fn.clone();
                let ev_tx = event_sender.clone();

                let sync_ok = tokio::select! {
                    _ = shutdown_rx.changed() => {
                        let (s, a) = idle_manager::next_state(&state, &IdleEvent::Shutdown);
                        state = s;
                        action = a;
                        continue;
                    }
                    result = tokio::task::spawn_blocking(move || {
                        do_incremental_sync(&db, &acct, cs.as_ref(), params_fn.as_ref(), &ev_tx)
                    }) => {
                        result.unwrap_or_default()
                    }
                };

                let event = if sync_ok {
                    IdleEvent::SyncCompleted
                } else {
                    IdleEvent::Disconnected
                };
                let (s, a) = idle_manager::next_state(&state, &event);
                state = s;
                action = a;
            }

            IdleAction::ReconnectAfter(delay) => {
                // Log the reconnect attempt.
                let log = ConnectionLogRecord::new(
                    account_id.clone(),
                    ConnectionLogEventType::Reconnect,
                    format!("Reconnecting in {}s", delay.as_secs()),
                );
                write_logs(&db_path, &[log]);

                // Wait for the backoff duration or shutdown.
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        let (s, a) = idle_manager::next_state(&state, &IdleEvent::Shutdown);
                        state = s;
                        action = a;
                        continue;
                    }
                    _ = tokio::time::sleep(delay) => {}
                }

                // Check if IDLE is still supported (may have changed).
                let idle_supported = check_idle_supported(&db_path, &account_id).unwrap_or(false);

                // Try to reconnect by checking if params are available.
                let can_connect = account_params_fn(&account_id).is_some();

                if can_connect {
                    let (s, a) = idle_manager::next_state(
                        &state,
                        &IdleEvent::Reconnected { idle_supported },
                    );
                    state = s;
                    action = a;
                } else {
                    let (s, a) = idle_manager::next_state(&state, &IdleEvent::Disconnected);
                    state = s;
                    action = a;
                }
            }

            IdleAction::StartPolling | IdleAction::PollNow => {
                if matches!(action, IdleAction::PollNow) {
                    // Run incremental sync.
                    let db = db_path.clone();
                    let acct = account_id.clone();
                    let cs = content_store.clone();
                    let params_fn = account_params_fn.clone();
                    let ev_tx = event_sender.clone();

                    let sync_ok = tokio::select! {
                        _ = shutdown_rx.changed() => {
                            let (s, a) = idle_manager::next_state(&state, &IdleEvent::Shutdown);
                            state = s;
                            action = a;
                            continue;
                        }
                        result = tokio::task::spawn_blocking(move || {
                            do_incremental_sync(&db, &acct, cs.as_ref(), params_fn.as_ref(), &ev_tx)
                        }) => {
                            result.unwrap_or_default()
                        }
                    };

                    if !sync_ok {
                        let (s, a) = idle_manager::next_state(&state, &IdleEvent::Disconnected);
                        state = s;
                        action = a;
                        continue;
                    }
                }

                // Wait for the poll interval or shutdown.
                let poll_interval = Duration::from_secs(DEFAULT_POLL_INTERVAL_SECS);
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        let (s, a) = idle_manager::next_state(&state, &IdleEvent::Shutdown);
                        state = s;
                        action = a;
                        continue;
                    }
                    _ = tokio::time::sleep(poll_interval) => {}
                }

                let (s, a) = idle_manager::next_state(&state, &IdleEvent::PollTimeout);
                state = s;
                action = a;
            }

            IdleAction::Stop | IdleAction::None => {
                break;
            }
        }

        if state == IdleState::Stopped {
            break;
        }
    }
}

/// Check whether the account's cached capabilities include IDLE.
fn check_idle_supported(db_path: &std::path::Path, account_id: &str) -> Option<bool> {
    let conn = open_and_migrate(db_path).ok()?;
    let sync_state = load_sync_state(&conn, account_id).ok()??;
    Some(sync_state.idle_supported)
}

/// Persist connection log entries to the database.
fn write_logs(db_path: &std::path::Path, logs: &[ConnectionLogRecord]) {
    if logs.is_empty() {
        return;
    }
    if let Ok(conn) = open_and_migrate(db_path) {
        let _ = append_connection_logs(&conn, logs);
    }
}

/// Run incremental sync for the INBOX folder of an account.
/// Returns true on success, false on error.
fn do_incremental_sync(
    db_path: &std::path::Path,
    account_id: &str,
    content_store: &dyn ContentStore,
    account_params_fn: &(dyn Fn(&str) -> Option<ImapConnectParams> + Send + Sync),
    event_sender: &broadcast::Sender<SyncEvent>,
) -> bool {
    let conn = match open_and_migrate(db_path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    let params = match account_params_fn(account_id) {
        Some(p) => p,
        None => return false,
    };

    match incremental_sync_folder(&conn, content_store, &params, "INBOX") {
        Ok(result) => {
            // Broadcast sync events (flag changes from server).
            for ev in result.events {
                let _ = event_sender.send(ev);
            }

            // Notify about new mail.
            if result.bodies_fetched > 0 {
                let _ = event_sender.send(SyncEvent::NewMailReceived {
                    account_id: account_id.to_string(),
                    folder_name: "INBOX".to_string(),
                    bodies_fetched: result.bodies_fetched,
                });
            }

            true
        }
        Err(_) => false,
    }
}

/// Mock IDLE waiter for testing.
#[cfg(test)]
pub(crate) struct MockIdleWaiter {
    pub results: std::sync::Mutex<Vec<IdleWaitResult>>,
}

#[cfg(test)]
impl MockIdleWaiter {
    pub fn new(results: Vec<IdleWaitResult>) -> Self {
        Self {
            results: std::sync::Mutex::new(results),
        }
    }
}

#[cfg(test)]
impl IdleWaiter for MockIdleWaiter {
    fn idle_wait(
        &self,
        params: &ImapConnectParams,
        _folder_name: &str,
        _timeout: Duration,
    ) -> (IdleWaitResult, Vec<ConnectionLogRecord>) {
        let result = {
            let mut results = self.results.lock().unwrap();
            if results.is_empty() {
                // Default: simulate disconnect after all results consumed.
                IdleWaitResult::Error("mock: no more results".to_string())
            } else {
                results.remove(0)
            }
        };

        let mut logs = vec![ConnectionLogRecord::new(
            params.account_id.clone(),
            ConnectionLogEventType::IdleEnter,
            "Mock IDLE enter".to_string(),
        )];
        logs.push(ConnectionLogRecord::new(
            params.account_id.clone(),
            ConnectionLogEventType::IdleExit,
            format!("Mock IDLE exit: {result:?}"),
        ));

        (result, logs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::account::EncryptionMode;
    use crate::core::content_store::ContentStoreError;
    use crate::services::database::open_and_migrate;
    use crate::services::fs_content_store::sha256_hex;
    use crate::services::sync_state_store::upsert_sync_state;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use tempfile::TempDir;

    /// Thread-safe in-memory content store for async tests.
    struct SyncContentStore {
        data: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl SyncContentStore {
        fn new() -> Self {
            Self {
                data: Mutex::new(HashMap::new()),
            }
        }
    }

    impl ContentStore for SyncContentStore {
        fn put(&self, data: &[u8]) -> Result<String, ContentStoreError> {
            let hash = sha256_hex(data);
            self.data
                .lock()
                .unwrap()
                .insert(hash.clone(), data.to_vec());
            Ok(hash)
        }

        fn get(&self, hash: &str) -> Result<Vec<u8>, ContentStoreError> {
            self.data
                .lock()
                .unwrap()
                .get(hash)
                .cloned()
                .ok_or_else(|| ContentStoreError::NotFound(hash.to_string()))
        }

        fn delete(&self, hash: &str) -> Result<(), ContentStoreError> {
            self.data.lock().unwrap().remove(hash);
            Ok(())
        }

        fn exists(&self, hash: &str) -> Result<bool, ContentStoreError> {
            Ok(self.data.lock().unwrap().contains_key(hash))
        }
    }

    fn setup_db() -> (TempDir, PathBuf) {
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
        drop(conn);
        (dir, db_path)
    }

    type TestParamsFn = Arc<dyn Fn(&str) -> Option<ImapConnectParams> + Send + Sync>;

    fn setup_sync_state(db_path: &std::path::Path, idle_supported: bool) {
        let conn = open_and_migrate(db_path).unwrap();
        let state = crate::core::sync_state::SyncState {
            account_id: "acct-1".to_string(),
            idle_supported,
            condstore_supported: false,
            ..Default::default()
        };
        upsert_sync_state(&conn, &state).unwrap();
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
            client_certificate: None,
        }
    }

    #[tokio::test]
    async fn idle_enters_after_initial_sync_with_idle_support() {
        let (_dir, db_path) = setup_db();
        setup_sync_state(&db_path, true);

        // Mock: first call returns NewMessages, second returns error (to stop loop).
        let waiter = Arc::new(MockIdleWaiter::new(vec![
            IdleWaitResult::NewMessages,
            IdleWaitResult::Error("done".to_string()),
            IdleWaitResult::Error("done".to_string()),
            IdleWaitResult::Error("done".to_string()),
        ]));

        let (event_tx, _event_rx) = broadcast::channel::<SyncEvent>(16);
        let content_store: Arc<dyn ContentStore + Send + Sync> = Arc::new(SyncContentStore::new());
        let params = make_test_params();
        let params_fn: TestParamsFn = Arc::new(move |_| Some(params.clone()));

        let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

        // Run with a timeout to prevent infinite loop.
        let _ = tokio::time::timeout(
            Duration::from_secs(3),
            run_idle_loop(
                "acct-1".to_string(),
                db_path.clone(),
                params_fn,
                event_tx,
                content_store,
                waiter,
                shutdown_rx,
            ),
        )
        .await;

        // Verify IDLE enter was logged.
        let conn = open_and_migrate(&db_path).unwrap();
        let logs =
            crate::services::connection_log_store::load_connection_logs(&conn, "acct-1", 100)
                .unwrap();
        let idle_enters: Vec<_> = logs
            .iter()
            .filter(|l| l.event_type == ConnectionLogEventType::IdleEnter)
            .collect();
        assert!(
            !idle_enters.is_empty(),
            "Expected IDLE enter log entry, got: {logs:?}"
        );
    }

    #[tokio::test]
    async fn idle_renewal_exits_and_re_enters() {
        let (_dir, db_path) = setup_db();
        setup_sync_state(&db_path, true);

        // Mock: first returns Timeout (renewal), second returns error (stop).
        let waiter = Arc::new(MockIdleWaiter::new(vec![
            IdleWaitResult::Timeout,
            IdleWaitResult::Error("done".to_string()),
            IdleWaitResult::Error("done".to_string()),
            IdleWaitResult::Error("done".to_string()),
        ]));

        let (event_tx, _) = broadcast::channel::<SyncEvent>(16);
        let content_store: Arc<dyn ContentStore + Send + Sync> = Arc::new(SyncContentStore::new());
        let params = make_test_params();
        let params_fn: TestParamsFn = Arc::new(move |_| Some(params.clone()));

        let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

        let _ = tokio::time::timeout(
            Duration::from_secs(3),
            run_idle_loop(
                "acct-1".to_string(),
                db_path.clone(),
                params_fn,
                event_tx,
                content_store,
                waiter,
                shutdown_rx,
            ),
        )
        .await;

        // Verify multiple IDLE enter/exit cycles were logged (renewal).
        let conn = open_and_migrate(&db_path).unwrap();
        let logs =
            crate::services::connection_log_store::load_connection_logs(&conn, "acct-1", 100)
                .unwrap();
        let idle_enters = logs
            .iter()
            .filter(|l| l.event_type == ConnectionLogEventType::IdleEnter)
            .count();
        // At least 2: initial enter + renewal enter.
        assert!(
            idle_enters >= 2,
            "Expected at least 2 IDLE enters for renewal, got {idle_enters}"
        );
    }

    #[tokio::test]
    async fn disconnect_triggers_reconnect_with_backoff() {
        let (_dir, db_path) = setup_db();
        setup_sync_state(&db_path, true);

        // Mock: returns errors to simulate disconnects.
        let waiter = Arc::new(MockIdleWaiter::new(vec![
            IdleWaitResult::Error("network drop".to_string()),
            IdleWaitResult::Error("still down".to_string()),
            IdleWaitResult::Error("still down".to_string()),
        ]));

        let (event_tx, _) = broadcast::channel::<SyncEvent>(16);
        let content_store: Arc<dyn ContentStore + Send + Sync> = Arc::new(SyncContentStore::new());
        let params = make_test_params();
        let params_fn: TestParamsFn = Arc::new(move |_| Some(params.clone()));

        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

        let handle = tokio::spawn(run_idle_loop(
            "acct-1".to_string(),
            db_path.clone(),
            params_fn,
            event_tx,
            content_store,
            waiter,
            shutdown_rx,
        ));

        // Give it time for the first disconnect + reconnect attempt (5s backoff).
        tokio::time::sleep(Duration::from_secs(7)).await;

        // Shut down.
        let _ = shutdown_tx.send(true);
        let _ = tokio::time::timeout(Duration::from_secs(2), handle).await;

        // Verify reconnect log entries.
        let conn = open_and_migrate(&db_path).unwrap();
        let logs =
            crate::services::connection_log_store::load_connection_logs(&conn, "acct-1", 100)
                .unwrap();
        let reconnects = logs
            .iter()
            .filter(|l| l.event_type == ConnectionLogEventType::Reconnect)
            .count();
        assert!(
            reconnects >= 1,
            "Expected reconnect log entries, got {reconnects}"
        );
    }

    #[tokio::test]
    async fn no_idle_support_falls_back_to_polling() {
        let (_dir, db_path) = setup_db();
        setup_sync_state(&db_path, false); // No IDLE support.

        // The waiter should not be called for polling (polling uses incremental sync, not IDLE).
        let waiter = Arc::new(MockIdleWaiter::new(vec![]));

        let (event_tx, _) = broadcast::channel::<SyncEvent>(16);
        let content_store: Arc<dyn ContentStore + Send + Sync> = Arc::new(SyncContentStore::new());
        let params = make_test_params();
        let params_fn: TestParamsFn = Arc::new(move |_| Some(params.clone()));

        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

        // Run for a short period — the loop should enter polling mode.
        let handle = tokio::spawn(run_idle_loop(
            "acct-1".to_string(),
            db_path.clone(),
            params_fn,
            event_tx,
            content_store,
            waiter,
            shutdown_rx,
        ));

        // Give it a moment to enter polling state, then shut down.
        tokio::time::sleep(Duration::from_millis(100)).await;
        let _ = shutdown_tx.send(true);
        let _ = tokio::time::timeout(Duration::from_secs(2), handle).await;

        // The test passes if the loop entered polling (didn't crash)
        // and the IDLE waiter was never called (no IDLE enter logs from the mock).
        let conn = open_and_migrate(&db_path).unwrap();
        let logs =
            crate::services::connection_log_store::load_connection_logs(&conn, "acct-1", 100)
                .unwrap();
        let idle_enters = logs
            .iter()
            .filter(|l| l.event_type == ConnectionLogEventType::IdleEnter)
            .count();
        assert_eq!(
            idle_enters, 0,
            "Polling mode should not produce IDLE enter logs"
        );
    }

    #[tokio::test]
    async fn shutdown_stops_idle_loop() {
        let (_dir, db_path) = setup_db();
        setup_sync_state(&db_path, true);

        // Mock that blocks forever (simulating long IDLE wait).
        let waiter = Arc::new(MockIdleWaiter::new(vec![
            IdleWaitResult::Timeout,
            IdleWaitResult::Timeout,
            IdleWaitResult::Timeout,
        ]));

        let (event_tx, _) = broadcast::channel::<SyncEvent>(16);
        let content_store: Arc<dyn ContentStore + Send + Sync> = Arc::new(SyncContentStore::new());
        let params = make_test_params();
        let params_fn: TestParamsFn = Arc::new(move |_| Some(params.clone()));

        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

        let handle = tokio::spawn(run_idle_loop(
            "acct-1".to_string(),
            db_path.clone(),
            params_fn,
            event_tx,
            content_store,
            waiter,
            shutdown_rx,
        ));

        // Signal shutdown.
        let _ = shutdown_tx.send(true);

        // Should terminate promptly.
        let result = tokio::time::timeout(Duration::from_secs(5), handle).await;
        assert!(result.is_ok(), "IDLE loop should stop on shutdown");
    }

    #[test]
    fn check_idle_supported_from_db() {
        let (_dir, db_path) = setup_db();
        setup_sync_state(&db_path, true);
        assert_eq!(check_idle_supported(&db_path, "acct-1"), Some(true));

        // Non-existent account.
        assert_eq!(check_idle_supported(&db_path, "nonexistent"), None);
    }
}
