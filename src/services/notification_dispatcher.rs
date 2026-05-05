//! Notification dispatcher — subscribes to [`SyncEvent`] broadcast events and
//! dispatches freedesktop desktop notifications for new-mail arrivals (FR-46,
//! FR-47, FR-48).

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::broadcast;

use crate::core::SyncEvent;

/// Trait abstracting account/folder notification-flag lookups so the dispatcher
/// can be tested without a real database.
pub(crate) trait NotificationFlagStore: Send + Sync {
    /// Whether notifications are enabled for the given account.
    fn account_notifications_enabled(&self, account_id: &str) -> bool;
    /// The display name for the given account (for notification body text).
    fn account_display_name(&self, account_id: &str) -> Option<String>;
    /// Whether notifications are enabled for the given folder on the account.
    fn folder_notifications_enabled(&self, account_id: &str, folder_name: &str) -> bool;
}

/// Trait abstracting the desktop notification backend so unit tests do not
/// require a session bus.
pub(crate) trait NotificationSender: Send + Sync {
    /// Show a desktop notification. Returns an error string on failure.
    fn show(&self, account_name: &str, folder_name: &str, count: usize) -> Result<(), String>;
}

/// Real [`NotificationSender`] backed by `notify-rust`.
pub(crate) struct FreedesktopNotificationSender;

impl NotificationSender for FreedesktopNotificationSender {
    fn show(&self, account_name: &str, folder_name: &str, count: usize) -> Result<(), String> {
        let body = if count == 1 {
            format!("1 new message in {folder_name} on {account_name}")
        } else {
            format!("{count} new messages in {folder_name} on {account_name}")
        };
        notify_rust::Notification::new()
            .appname("FairEmail")
            .summary("New mail")
            .body(&body)
            .hint(notify_rust::Hint::Category(format!(
                "account-{account_name}"
            )))
            .icon("mail-unread")
            .show()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// Real [`NotificationFlagStore`] backed by SQLite.
pub(crate) struct SqliteNotificationFlagStore {
    db_path: PathBuf,
}

impl SqliteNotificationFlagStore {
    pub(crate) fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }
}

impl NotificationFlagStore for SqliteNotificationFlagStore {
    fn account_notifications_enabled(&self, account_id: &str) -> bool {
        let Ok(conn) = rusqlite::Connection::open(&self.db_path) else {
            return false;
        };
        conn.query_row(
            "SELECT notifications_enabled FROM accounts WHERE id = ?1",
            rusqlite::params![account_id],
            |row| row.get::<_, i32>(0),
        )
        .map(|v| v != 0)
        .unwrap_or(false)
    }

    fn account_display_name(&self, account_id: &str) -> Option<String> {
        let conn = rusqlite::Connection::open(&self.db_path).ok()?;
        conn.query_row(
            "SELECT display_name FROM accounts WHERE id = ?1",
            rusqlite::params![account_id],
            |row| row.get::<_, String>(0),
        )
        .ok()
    }

    fn folder_notifications_enabled(&self, account_id: &str, folder_name: &str) -> bool {
        let Ok(conn) = rusqlite::Connection::open(&self.db_path) else {
            return true;
        };
        crate::services::folder_store::is_folder_notifications_enabled(
            &conn,
            account_id,
            folder_name,
        )
        .unwrap_or(true)
    }
}

/// Spawn a tokio task that listens on the broadcast channel for
/// [`SyncEvent::NewMailReceived`] events and dispatches desktop notifications.
///
/// The task runs until the broadcast sender is dropped (all senders gone) or
/// the returned [`tokio::task::JoinHandle`] is aborted.
pub(crate) fn spawn_notification_listener(
    mut rx: broadcast::Receiver<SyncEvent>,
    flag_store: Arc<dyn NotificationFlagStore>,
    sender: Arc<dyn NotificationSender>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(SyncEvent::NewMailReceived {
                    account_id,
                    folder_name,
                    bodies_fetched,
                }) => {
                    dispatch_notification(
                        &*flag_store,
                        &*sender,
                        &account_id,
                        &folder_name,
                        bodies_fetched,
                    );
                }
                Ok(_) => { /* ignore non-new-mail events */ }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!("notification dispatcher: skipped {n} events (lagged)");
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    })
}

/// Decide whether to dispatch a notification and do so if appropriate.
fn dispatch_notification(
    flags: &dyn NotificationFlagStore,
    sender: &dyn NotificationSender,
    account_id: &str,
    folder_name: &str,
    count: usize,
) {
    if count == 0 {
        return;
    }
    if !flags.account_notifications_enabled(account_id) {
        return;
    }
    if !flags.folder_notifications_enabled(account_id, folder_name) {
        return;
    }
    let account_name = flags
        .account_display_name(account_id)
        .unwrap_or_else(|| account_id.to_string());
    if let Err(e) = sender.show(&account_name, folder_name, count) {
        eprintln!("notification dispatcher: failed to show notification: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Mock flag store for testing.
    struct MockFlagStore {
        account_enabled: bool,
        folder_enabled: bool,
        display_name: Option<String>,
    }

    impl NotificationFlagStore for MockFlagStore {
        fn account_notifications_enabled(&self, _account_id: &str) -> bool {
            self.account_enabled
        }
        fn account_display_name(&self, _account_id: &str) -> Option<String> {
            self.display_name.clone()
        }
        fn folder_notifications_enabled(&self, _account_id: &str, _folder_name: &str) -> bool {
            self.folder_enabled
        }
    }

    /// Mock notification sender that records calls.
    struct MockSender {
        calls: Mutex<Vec<(String, String, usize)>>,
    }

    impl MockSender {
        fn new() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
            }
        }
        fn call_count(&self) -> usize {
            self.calls.lock().unwrap().len()
        }
        fn last_call(&self) -> Option<(String, String, usize)> {
            self.calls.lock().unwrap().last().cloned()
        }
    }

    impl NotificationSender for MockSender {
        fn show(&self, account_name: &str, folder_name: &str, count: usize) -> Result<(), String> {
            self.calls.lock().unwrap().push((
                account_name.to_string(),
                folder_name.to_string(),
                count,
            ));
            Ok(())
        }
    }

    #[test]
    fn dispatches_when_both_flags_enabled() {
        let flags = MockFlagStore {
            account_enabled: true,
            folder_enabled: true,
            display_name: Some("Work".to_string()),
        };
        let sender = MockSender::new();
        dispatch_notification(&flags, &sender, "acct-1", "INBOX", 3);
        assert_eq!(sender.call_count(), 1);
        let (name, folder, count) = sender.last_call().unwrap();
        assert_eq!(name, "Work");
        assert_eq!(folder, "INBOX");
        assert_eq!(count, 3);
    }

    #[test]
    fn suppressed_when_account_disabled() {
        let flags = MockFlagStore {
            account_enabled: false,
            folder_enabled: true,
            display_name: Some("Work".to_string()),
        };
        let sender = MockSender::new();
        dispatch_notification(&flags, &sender, "acct-1", "INBOX", 1);
        assert_eq!(sender.call_count(), 0);
    }

    #[test]
    fn suppressed_when_folder_disabled() {
        let flags = MockFlagStore {
            account_enabled: true,
            folder_enabled: false,
            display_name: Some("Work".to_string()),
        };
        let sender = MockSender::new();
        dispatch_notification(&flags, &sender, "acct-1", "INBOX", 1);
        assert_eq!(sender.call_count(), 0);
    }

    #[test]
    fn no_notification_for_zero_count() {
        let flags = MockFlagStore {
            account_enabled: true,
            folder_enabled: true,
            display_name: Some("Work".to_string()),
        };
        let sender = MockSender::new();
        dispatch_notification(&flags, &sender, "acct-1", "INBOX", 0);
        assert_eq!(sender.call_count(), 0);
    }

    #[test]
    fn uses_account_id_when_display_name_missing() {
        let flags = MockFlagStore {
            account_enabled: true,
            folder_enabled: true,
            display_name: None,
        };
        let sender = MockSender::new();
        dispatch_notification(&flags, &sender, "acct-1", "INBOX", 1);
        assert_eq!(sender.call_count(), 1);
        let (name, _, _) = sender.last_call().unwrap();
        assert_eq!(name, "acct-1");
    }

    #[tokio::test]
    async fn listener_dispatches_new_mail_event() {
        let (tx, rx) = broadcast::channel::<SyncEvent>(16);
        let flags = Arc::new(MockFlagStore {
            account_enabled: true,
            folder_enabled: true,
            display_name: Some("Personal".to_string()),
        });
        let sender = Arc::new(MockSender::new());
        let sender_dyn: Arc<dyn NotificationSender> = Arc::clone(&sender) as _;
        let handle = spawn_notification_listener(rx, flags, sender_dyn);

        tx.send(SyncEvent::NewMailReceived {
            account_id: "acct-1".to_string(),
            folder_name: "INBOX".to_string(),
            bodies_fetched: 2,
        })
        .unwrap();

        // Drop sender so the listener loop terminates.
        drop(tx);
        handle.await.unwrap();

        assert_eq!(sender.call_count(), 1);
        let (name, folder, count) = sender.last_call().unwrap();
        assert_eq!(name, "Personal");
        assert_eq!(folder, "INBOX");
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn listener_ignores_non_new_mail_events() {
        let (tx, rx) = broadcast::channel::<SyncEvent>(16);
        let flags = Arc::new(MockFlagStore {
            account_enabled: true,
            folder_enabled: true,
            display_name: Some("Work".to_string()),
        });
        let sender = Arc::new(MockSender::new());
        let sender_dyn: Arc<dyn NotificationSender> = Arc::clone(&sender) as _;
        let handle = spawn_notification_listener(rx, flags, sender_dyn);

        tx.send(SyncEvent::FolderListChanged {
            account_id: "acct-1".to_string(),
        })
        .unwrap();

        drop(tx);
        handle.await.unwrap();

        assert_eq!(sender.call_count(), 0);
    }

    #[tokio::test]
    async fn listener_respects_account_flag() {
        let (tx, rx) = broadcast::channel::<SyncEvent>(16);
        let flags = Arc::new(MockFlagStore {
            account_enabled: false,
            folder_enabled: true,
            display_name: Some("Work".to_string()),
        });
        let sender = Arc::new(MockSender::new());
        let sender_dyn: Arc<dyn NotificationSender> = Arc::clone(&sender) as _;
        let handle = spawn_notification_listener(rx, flags, sender_dyn);

        tx.send(SyncEvent::NewMailReceived {
            account_id: "acct-1".to_string(),
            folder_name: "INBOX".to_string(),
            bodies_fetched: 1,
        })
        .unwrap();

        drop(tx);
        handle.await.unwrap();

        assert_eq!(sender.call_count(), 0);
    }

    #[tokio::test]
    async fn listener_respects_folder_flag() {
        let (tx, rx) = broadcast::channel::<SyncEvent>(16);
        let flags = Arc::new(MockFlagStore {
            account_enabled: true,
            folder_enabled: false,
            display_name: Some("Work".to_string()),
        });
        let sender = Arc::new(MockSender::new());
        let sender_dyn: Arc<dyn NotificationSender> = Arc::clone(&sender) as _;
        let handle = spawn_notification_listener(rx, flags, sender_dyn);

        tx.send(SyncEvent::NewMailReceived {
            account_id: "acct-1".to_string(),
            folder_name: "INBOX".to_string(),
            bodies_fetched: 1,
        })
        .unwrap();

        drop(tx);
        handle.await.unwrap();

        assert_eq!(sender.call_count(), 0);
    }
}
