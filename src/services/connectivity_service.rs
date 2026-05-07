//! Network connectivity monitor that triggers operation queue replay.
//!
//! Watches `gio::NetworkMonitor` for connectivity changes. When an
//! offline-to-online transition is detected, it enumerates all accounts
//! with pending operations and notifies the sync engine to process them.

use std::path::PathBuf;
use std::sync::Mutex;

use tokio::sync::mpsc::UnboundedSender;

use crate::core::connectivity::{ConnectivityState, ConnectivityTracker};
use crate::services::database::open_and_migrate;
use crate::services::pending_ops_store;

/// Replay all pending operations by notifying the sync engine for each
/// account that has queued work.
///
/// This is the core replay logic, separated for testability.
pub(crate) fn replay_pending_ops(
    db_path: &std::path::Path,
    notify_tx: &UnboundedSender<String>,
) -> Result<usize, String> {
    let conn = open_and_migrate(db_path).map_err(|e| format!("failed to open db: {e}"))?;
    let accounts = pending_ops_store::list_accounts_with_pending_ops(&conn)
        .map_err(|e| format!("failed to list accounts: {e}"))?;
    let count = accounts.len();
    for account_id in accounts {
        let _ = notify_tx.send(account_id);
    }
    Ok(count)
}

/// Handle to stop the connectivity monitor.
pub(crate) struct ConnectivityMonitorHandle {
    // Signal handler ID, kept alive so we can disconnect later.
    #[cfg(feature = "ui")]
    _signal_id: Option<glib::SignalHandlerId>,
    #[cfg(not(feature = "ui"))]
    _phantom: (),
}

/// Start monitoring network connectivity changes.
///
/// When an offline-to-online transition occurs, enumerates all accounts
/// with pending operations and notifies the sync engine to replay them.
///
/// Returns a handle that keeps the monitor alive; drop it to stop.
#[cfg(feature = "ui")]
pub(crate) fn start_connectivity_monitor(
    db_path: PathBuf,
    notify_tx: UnboundedSender<String>,
) -> ConnectivityMonitorHandle {
    use gtk4::gio;
    use gtk4::prelude::NetworkMonitorExt;

    let monitor = gio::NetworkMonitor::default();
    let initial_state = if monitor.is_network_available() {
        ConnectivityState::Online
    } else {
        ConnectivityState::Offline
    };

    let tracker = std::sync::Arc::new(Mutex::new(ConnectivityTracker::new(initial_state)));

    let signal_id = monitor.connect_network_changed(move |_monitor, available| {
        let current = if available {
            ConnectivityState::Online
        } else {
            ConnectivityState::Offline
        };

        let should_replay = {
            let mut t = tracker.lock().expect("connectivity tracker lock poisoned");
            t.update(current)
        };

        if should_replay {
            match replay_pending_ops(&db_path, &notify_tx) {
                Ok(count) => {
                    if count > 0 {
                        eprintln!(
                            "connectivity restored: replaying operations for {count} account(s)"
                        );
                    }
                }
                Err(e) => {
                    eprintln!("connectivity restored but replay failed: {e}");
                }
            }
        }
    });

    ConnectivityMonitorHandle {
        _signal_id: Some(signal_id),
    }
}

/// Headless / test build: no-op monitor.
#[cfg(not(feature = "ui"))]
pub(crate) fn start_connectivity_monitor(
    _db_path: PathBuf,
    _notify_tx: UnboundedSender<String>,
) -> ConnectivityMonitorHandle {
    ConnectivityMonitorHandle { _phantom: () }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::pending_operation::OperationKind;
    use crate::services::database::open_and_migrate;
    use crate::services::pending_ops_store::insert_pending_op;
    use tempfile::TempDir;

    fn setup_db_with_accounts(account_ids: &[&str]) -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_and_migrate(&db_path).unwrap();
        for id in account_ids {
            conn.execute(
                "INSERT INTO accounts (id, display_name, protocol, host, port, encryption, auth_method, username, credential)
                 VALUES (?1, 'Test', 'Imap', 'imap.example.com', 993, 'SslTls', 'Plain', 'user', '')",
                rusqlite::params![id],
            ).unwrap();
        }
        (dir, db_path)
    }

    #[test]
    fn replay_notifies_accounts_with_pending_ops() {
        let (_dir, db_path) = setup_db_with_accounts(&["acct-1", "acct-2", "acct-3"]);
        let conn = open_and_migrate(&db_path).unwrap();
        insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, "{}").unwrap();
        insert_pending_op(&conn, "acct-2", &OperationKind::MoveMessage, "{}").unwrap();
        // acct-3 has no pending ops
        drop(conn);

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let count = replay_pending_ops(&db_path, &tx).unwrap();
        assert_eq!(count, 2);

        let mut notified = Vec::new();
        while let Ok(id) = rx.try_recv() {
            notified.push(id);
        }
        notified.sort();
        assert_eq!(notified, vec!["acct-1", "acct-2"]);
    }

    #[test]
    fn replay_with_no_pending_ops_notifies_nobody() {
        let (_dir, db_path) = setup_db_with_accounts(&["acct-1"]);

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let count = replay_pending_ops(&db_path, &tx).unwrap();
        assert_eq!(count, 0);
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn replay_handles_1000_queued_ops_single_account() {
        let (_dir, db_path) = setup_db_with_accounts(&["acct-1"]);
        let conn = open_and_migrate(&db_path).unwrap();
        for i in 0..1000 {
            let payload = format!(r#"{{"n":{i}}}"#);
            insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, &payload).unwrap();
        }
        drop(conn);

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let count = replay_pending_ops(&db_path, &tx).unwrap();
        // Only 1 account notified (not 1000 notifications)
        assert_eq!(count, 1);
        assert_eq!(rx.try_recv().unwrap(), "acct-1");
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn replay_handles_1000_queued_ops_across_accounts() {
        let ids: Vec<String> = (0..100).map(|i| format!("acct-{i}")).collect();
        let id_refs: Vec<&str> = ids.iter().map(|s| s.as_str()).collect();
        let (_dir, db_path) = setup_db_with_accounts(&id_refs);
        let conn = open_and_migrate(&db_path).unwrap();
        // 10 ops per account = 1000 total
        for id in &ids {
            for j in 0..10 {
                let payload = format!(r#"{{"n":{j}}}"#);
                insert_pending_op(&conn, id, &OperationKind::StoreFlags, &payload).unwrap();
            }
        }
        drop(conn);

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let count = replay_pending_ops(&db_path, &tx).unwrap();
        assert_eq!(count, 100);

        let mut notified = Vec::new();
        while let Ok(id) = rx.try_recv() {
            notified.push(id);
        }
        assert_eq!(notified.len(), 100);
    }

    #[test]
    fn replay_skips_failed_ops() {
        let (_dir, db_path) = setup_db_with_accounts(&["acct-1"]);
        let conn = open_and_migrate(&db_path).unwrap();
        let id = insert_pending_op(&conn, "acct-1", &OperationKind::StoreFlags, "{}").unwrap();
        crate::services::pending_ops_store::mark_failed(&conn, id, "permanent error").unwrap();
        drop(conn);

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let count = replay_pending_ops(&db_path, &tx).unwrap();
        assert_eq!(count, 0);
        assert!(rx.try_recv().is_err());
    }
}
