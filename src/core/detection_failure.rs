// Detection failure fallback (FR-23, FR-24, FR-25, US-18).
//
// When auto-detection fails entirely (no provider settings could be
// determined from any strategy), this module provides the non-technical
// message and support link shown to the user.

use crate::core::auth_error::GENERAL_SUPPORT_URL;

/// Presentation data for the detection-failure fallback screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectionFailureFallback {
    /// Non-technical message explaining that auto-detection failed (FR-25).
    pub user_message: String,
    /// General support / FAQ link (FR-24).
    pub support_url: String,
}

/// Build the fallback presentation shown when all detection strategies fail.
///
/// The message is deliberately non-technical and does not expose raw error
/// details (FR-25). A general support link is included (FR-24).
pub fn build_detection_failure_fallback() -> DetectionFailureFallback {
    DetectionFailureFallback {
        user_message: "We could not automatically determine your email provider's settings. \
             You can set up your account manually instead."
            .to_string(),
        support_url: GENERAL_SUPPORT_URL.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_message_is_non_technical() {
        let fb = build_detection_failure_fallback();
        // Must not contain raw protocol/error jargon (FR-25).
        assert!(!fb.user_message.contains("IMAP"));
        assert!(!fb.user_message.contains("SMTP"));
        assert!(!fb.user_message.contains("DNS"));
        assert!(!fb.user_message.contains("TCP"));
        assert!(!fb.user_message.contains("SSL"));
        assert!(!fb.user_message.contains("error"));
        assert!(!fb.user_message.contains("exception"));
    }

    #[test]
    fn fallback_message_mentions_manual_setup() {
        let fb = build_detection_failure_fallback();
        assert!(
            fb.user_message.to_lowercase().contains("manually"),
            "Message should guide user toward manual setup"
        );
    }

    #[test]
    fn fallback_message_is_non_empty() {
        let fb = build_detection_failure_fallback();
        assert!(!fb.user_message.is_empty());
    }

    #[test]
    fn support_url_is_present() {
        let fb = build_detection_failure_fallback();
        assert!(!fb.support_url.is_empty());
        assert_eq!(fb.support_url, GENERAL_SUPPORT_URL);
    }

    #[test]
    fn support_url_starts_with_https() {
        let fb = build_detection_failure_fallback();
        assert!(
            fb.support_url.starts_with("https://"),
            "Support URL should use HTTPS"
        );
    }
}
