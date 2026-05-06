//! OAuth token revocation notification logic.
//!
//! Provides types and functions for building user-facing notifications
//! when an OAuth refresh token is revoked or expired (US-5, AC-1, AC-2, AC-4).
//! This module is UI-free so it can be unit-tested without a display server.

use uuid::Uuid;

use crate::core::auth_error::ProviderHint;
use crate::core::token_refresh::RefreshOutcome;

/// An event indicating that an account's OAuth authorization has been
/// revoked or its refresh token has expired and the user must re-authorize.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RevocationEvent {
    /// The account that needs re-authorization.
    pub account_id: Uuid,
    /// Human-readable account name for notification display.
    pub account_display_name: String,
    /// The reason from the token refresh service.
    pub reason: String,
}

/// A user-facing notification built from a revocation event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RevocationNotification {
    /// The account that needs re-authorization.
    pub account_id: Uuid,
    /// Short title for the notification banner.
    pub title: String,
    /// Label for the re-authorize action button.
    pub action_label: String,
    /// Optional provider-specific guidance (NFR-7).
    pub provider_guidance: Option<String>,
}

/// Build a user-facing notification from a revocation event.
pub(crate) fn build_revocation_notification(event: &RevocationEvent) -> RevocationNotification {
    let title = format!(
        "Authorization expired for {}. Please sign in again to restore access.",
        event.account_display_name,
    );

    RevocationNotification {
        account_id: event.account_id,
        title,
        action_label: "Re-authorize".to_string(),
        provider_guidance: None,
    }
}

/// Build a notification with provider-specific guidance (NFR-7).
///
/// If the provider has deprecated password access, the guidance directs
/// the user toward OAuth or app-specific passwords.
pub(crate) fn build_revocation_notification_with_hint(
    event: &RevocationEvent,
    hint: &ProviderHint,
) -> RevocationNotification {
    let mut notification = build_revocation_notification(event);

    let guidance = if hint.app_password_required {
        Some(
            "This provider requires OAuth or an app-specific password. \
             Use the Re-authorize button to sign in with your browser."
                .to_string(),
        )
    } else if hint.is_outlook_provider {
        Some(
            "Microsoft accounts require OAuth authentication. \
             Use the Re-authorize button to sign in with your Microsoft account."
                .to_string(),
        )
    } else {
        None
    };

    notification.provider_guidance = guidance;
    notification
}

/// Extract a `RevocationEvent` from a `RefreshOutcome`, if it indicates
/// that re-authorization is needed.
pub(crate) fn revocation_event_from_outcome(
    outcome: &RefreshOutcome,
    account_id: Uuid,
    account_display_name: &str,
) -> Option<RevocationEvent> {
    match outcome {
        RefreshOutcome::NeedsReauthorization { reason } => Some(RevocationEvent {
            account_id,
            account_display_name: account_display_name.to_string(),
            reason: reason.clone(),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_event() -> RevocationEvent {
        RevocationEvent {
            account_id: Uuid::nil(),
            account_display_name: "user@gmail.com".to_string(),
            reason: "invalid_grant".to_string(),
        }
    }

    #[test]
    fn build_notification_includes_account_name() {
        let event = sample_event();
        let notification = build_revocation_notification(&event);
        assert!(notification.title.contains("user@gmail.com"));
    }

    #[test]
    fn build_notification_has_reauthorize_action() {
        let event = sample_event();
        let notification = build_revocation_notification(&event);
        assert_eq!(notification.action_label, "Re-authorize");
    }

    #[test]
    fn build_notification_account_id_matches() {
        let event = sample_event();
        let notification = build_revocation_notification(&event);
        assert_eq!(notification.account_id, event.account_id);
    }

    #[test]
    fn build_notification_no_guidance_by_default() {
        let event = sample_event();
        let notification = build_revocation_notification(&event);
        assert!(notification.provider_guidance.is_none());
    }

    #[test]
    fn build_notification_with_app_password_hint() {
        let event = sample_event();
        let hint = ProviderHint {
            app_password_required: true,
            documentation_url: Some("https://example.com/docs".to_string()),
            is_outlook_provider: false,
            outlook_guidance: None,
            supports_oauth: false,
            deprecated_mechanism_guidance: None,
        };
        let notification = build_revocation_notification_with_hint(&event, &hint);
        assert!(notification.provider_guidance.is_some());
        let guidance = notification.provider_guidance.unwrap();
        assert!(guidance.contains("app-specific password"));
    }

    #[test]
    fn build_notification_with_outlook_hint() {
        let event = RevocationEvent {
            account_id: Uuid::nil(),
            account_display_name: "user@outlook.com".to_string(),
            reason: "invalid_grant".to_string(),
        };
        let hint = ProviderHint {
            app_password_required: false,
            documentation_url: None,
            is_outlook_provider: true,
            outlook_guidance: Some("Use OAuth".to_string()),
            supports_oauth: false,
            deprecated_mechanism_guidance: None,
        };
        let notification = build_revocation_notification_with_hint(&event, &hint);
        assert!(notification.provider_guidance.is_some());
        let guidance = notification.provider_guidance.unwrap();
        assert!(guidance.contains("Microsoft"));
    }

    #[test]
    fn build_notification_with_no_special_hint() {
        let event = sample_event();
        let hint = ProviderHint {
            app_password_required: false,
            documentation_url: None,
            is_outlook_provider: false,
            outlook_guidance: None,
            supports_oauth: false,
            deprecated_mechanism_guidance: None,
        };
        let notification = build_revocation_notification_with_hint(&event, &hint);
        assert!(notification.provider_guidance.is_none());
    }

    #[test]
    fn revocation_event_from_needs_reauth_outcome() {
        let outcome = RefreshOutcome::NeedsReauthorization {
            reason: "Token revoked".to_string(),
        };
        let event = revocation_event_from_outcome(&outcome, Uuid::nil(), "test@example.com");
        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.account_display_name, "test@example.com");
        assert_eq!(event.reason, "Token revoked");
    }

    #[test]
    fn no_revocation_event_from_refreshed_outcome() {
        let outcome = RefreshOutcome::Refreshed;
        let event = revocation_event_from_outcome(&outcome, Uuid::nil(), "test@example.com");
        assert!(event.is_none());
    }

    #[test]
    fn no_revocation_event_from_skipped_outcome() {
        let outcome = RefreshOutcome::Skipped;
        let event = revocation_event_from_outcome(&outcome, Uuid::nil(), "test@example.com");
        assert!(event.is_none());
    }

    #[test]
    fn no_revocation_event_from_rate_limited_outcome() {
        let outcome = RefreshOutcome::RateLimited {
            reason: "Too many requests".to_string(),
        };
        let event = revocation_event_from_outcome(&outcome, Uuid::nil(), "test@example.com");
        assert!(event.is_none());
    }

    #[test]
    fn notification_title_mentions_sign_in() {
        let event = sample_event();
        let notification = build_revocation_notification(&event);
        assert!(notification.title.contains("sign in"));
    }

    #[test]
    fn notification_title_mentions_restore_access() {
        let event = sample_event();
        let notification = build_revocation_notification(&event);
        assert!(notification.title.contains("restore access"));
    }
}
