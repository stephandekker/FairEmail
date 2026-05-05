use uuid::Uuid;

use crate::core::notification_channel_id;

/// Errors from the notification channel layer.
#[derive(Debug, thiserror::Error)]
pub enum NotificationChannelError {
    #[error("failed to register notification channel: {0}")]
    RegistrationFailed(String),
    #[error("failed to unregister notification channel: {0}")]
    UnregistrationFailed(String),
}

/// Trait for managing per-account notification channels (FR-40, FR-41).
///
/// On Linux, freedesktop.org notifications do not have persistent "channels"
/// like Android.  Implementations use the closest available mechanism — for
/// example, unique desktop-notification category strings or D-Bus notification
/// server hints — so the system can apply per-account sound, priority, and
/// behaviour settings.
pub trait NotificationChannelManager {
    /// Register (create) a notification channel for the given account.
    /// The channel identifier is derived from the account UUID via
    /// [`notification_channel_id`].
    fn register_channel(
        &self,
        account_id: Uuid,
        account_name: &str,
    ) -> Result<(), NotificationChannelError>;

    /// Unregister (remove) the notification channel for the given account.
    fn unregister_channel(&self, account_id: Uuid) -> Result<(), NotificationChannelError>;
}

/// Mock implementation that records calls but performs no real I/O.
#[derive(Debug, Default)]
pub struct MockNotificationChannelManager;

impl NotificationChannelManager for MockNotificationChannelManager {
    fn register_channel(
        &self,
        account_id: Uuid,
        _account_name: &str,
    ) -> Result<(), NotificationChannelError> {
        let _channel_id = notification_channel_id(account_id);
        // No-op in mock implementation.
        Ok(())
    }

    fn unregister_channel(&self, account_id: Uuid) -> Result<(), NotificationChannelError> {
        let _channel_id = notification_channel_id(account_id);
        // No-op in mock implementation.
        Ok(())
    }
}

/// Real freedesktop notification channel manager backed by `notify-rust`.
///
/// On register, it sends a transient notification so the desktop environment
/// learns about the application + category combination. On unregister it is
/// a no-op since freedesktop notifications are fire-and-forget.
#[derive(Debug, Default)]
pub struct FreedesktopNotificationChannelManager;

impl NotificationChannelManager for FreedesktopNotificationChannelManager {
    fn register_channel(
        &self,
        account_id: Uuid,
        account_name: &str,
    ) -> Result<(), NotificationChannelError> {
        let channel_id = notification_channel_id(account_id);
        // Send a silent, low-urgency notification so the desktop learns the
        // app-name + category pairing. This lets GNOME/KDE group future
        // notifications for this account.
        notify_rust::Notification::new()
            .appname("FairEmail")
            .summary(&format!("Notifications enabled for {account_name}"))
            .body("You will receive desktop notifications for new mail on this account.")
            .hint(notify_rust::Hint::Category(channel_id))
            .hint(notify_rust::Hint::SuppressSound(true))
            .urgency(notify_rust::Urgency::Low)
            .show()
            .map_err(|e| NotificationChannelError::RegistrationFailed(e.to_string()))?;
        Ok(())
    }

    fn unregister_channel(&self, account_id: Uuid) -> Result<(), NotificationChannelError> {
        let _channel_id = notification_channel_id(account_id);
        // Freedesktop notifications are fire-and-forget; there is no
        // persistent channel to remove.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_register_channel_succeeds() {
        let mgr = MockNotificationChannelManager;
        let id = Uuid::new_v4();
        assert!(mgr.register_channel(id, "Test Account").is_ok());
    }

    #[test]
    fn mock_unregister_channel_succeeds() {
        let mgr = MockNotificationChannelManager;
        let id = Uuid::new_v4();
        assert!(mgr.unregister_channel(id).is_ok());
    }
}
