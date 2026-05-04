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
/// Used until a real notification backend is chosen.
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
