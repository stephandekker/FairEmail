use crate::core::imap_check::ImapCheckError;
use crate::core::provider::Provider;
use crate::core::smtp_check::{ConnectivityCheckError, SmtpCheckError};

/// General FAQ / support link shown on any error (FR-24).
pub const GENERAL_SUPPORT_URL: &str = "https://github.com/nicefair/FairEmail/wiki/FAQ";

/// Outlook-specific guidance URL (FR-22).
const OUTLOOK_GUIDANCE_URL: &str =
    "https://support.microsoft.com/en-us/office/pop-imap-and-smtp-settings-8361e398-8af4-4e97-b147-6c6c4ac95353";

/// Known Outlook/Hotmail/Live domains for provider-specific guidance (FR-22).
const OUTLOOK_DOMAINS: &[&str] = &[
    "outlook.com",
    "hotmail.com",
    "live.com",
    "msn.com",
    "hotmail.co.uk",
    "hotmail.fr",
    "hotmail.de",
    "hotmail.it",
    "hotmail.es",
    "live.co.uk",
    "live.fr",
    "outlook.de",
    "outlook.fr",
    "outlook.co.uk",
];

/// The category of error that occurred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthErrorKind {
    /// Authentication credentials were rejected.
    AuthenticationFailed,
    /// Could not connect to the server.
    ConnectionFailed,
    /// A non-auth, non-connection error.
    Other,
}

/// A provider-specific hint to show alongside the error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderHint {
    /// Whether app-specific passwords are required for this provider (FR-15k).
    pub app_password_required: bool,
    /// The provider's documentation URL, if available (FR-24).
    pub documentation_url: Option<String>,
    /// Whether this is an Outlook/Hotmail/Live domain (FR-22).
    pub is_outlook_provider: bool,
    /// Outlook-specific guidance text (FR-22).
    pub outlook_guidance: Option<String>,
}

/// A user-friendly auth error with provider-specific guidance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthErrorPresentation {
    /// Non-technical summary message for the user (FR-25).
    pub user_message: String,
    /// The raw server error string, hidden by default (FR-25 "show details").
    pub raw_details: Option<String>,
    /// The category of error.
    pub error_kind: AuthErrorKind,
    /// Provider-specific hints and links (FR-21, FR-22, FR-24).
    pub provider_hint: Option<ProviderHint>,
    /// General support/FAQ link (FR-24).
    pub support_url: String,
}

/// Check whether a domain is an Outlook/Hotmail/Live domain (FR-22).
pub fn is_outlook_domain(domain: &str) -> bool {
    let lower = domain.to_lowercase();
    OUTLOOK_DOMAINS.iter().any(|d| *d == lower)
}

/// Extract the domain from an email address.
fn email_domain(email: &str) -> Option<&str> {
    let at_pos = email.rfind('@')?;
    let domain = &email[at_pos + 1..];
    if domain.is_empty() {
        None
    } else {
        Some(domain)
    }
}

/// Build provider hint from a provider entry and email address.
pub fn build_provider_hint(provider: &Provider, email: &str) -> ProviderHint {
    let is_outlook = email_domain(email)
        .map(is_outlook_domain)
        .unwrap_or(false);

    let outlook_guidance = if is_outlook {
        Some(
            "Microsoft accounts may require OAuth2 authentication or an app-specific password. \
             Check your Microsoft account security settings."
                .to_string(),
        )
    } else {
        None
    };

    ProviderHint {
        app_password_required: provider.app_password_required,
        documentation_url: provider.documentation_url.clone(),
        is_outlook_provider: is_outlook,
        outlook_guidance,
    }
}

/// Build a user-friendly error presentation from an IMAP check error.
pub fn present_imap_error(
    error: &ImapCheckError,
    provider: Option<&Provider>,
    email: &str,
) -> AuthErrorPresentation {
    let (user_message, raw_details, error_kind) = match error {
        ImapCheckError::AuthenticationFailed => (
            "Could not sign in to your email account. \
             Please check your email address and password."
                .to_string(),
            None,
            AuthErrorKind::AuthenticationFailed,
        ),
        ImapCheckError::ConnectionFailed(details) => (
            "Could not connect to the email server. \
             Please check your internet connection and try again."
                .to_string(),
            Some(details.clone()),
            AuthErrorKind::ConnectionFailed,
        ),
        ImapCheckError::FolderListFailed(details) => (
            "Connected successfully, but could not retrieve your mailbox folders.".to_string(),
            Some(details.clone()),
            AuthErrorKind::Other,
        ),
        ImapCheckError::UntrustedCertificate(_) => (
            "The server's security certificate could not be verified.".to_string(),
            None,
            AuthErrorKind::Other,
        ),
    };

    let provider_hint = provider.map(|p| build_provider_hint(p, email));

    AuthErrorPresentation {
        user_message,
        raw_details,
        error_kind,
        provider_hint,
        support_url: GENERAL_SUPPORT_URL.to_string(),
    }
}

/// Build a user-friendly error presentation from an SMTP check error.
pub fn present_smtp_error(
    error: &SmtpCheckError,
    provider: Option<&Provider>,
    email: &str,
) -> AuthErrorPresentation {
    let (user_message, raw_details, error_kind) = match error {
        SmtpCheckError::AuthenticationFailed => (
            "Could not sign in to the outgoing mail server. \
             Please check your email address and password."
                .to_string(),
            None,
            AuthErrorKind::AuthenticationFailed,
        ),
        SmtpCheckError::ConnectionFailed(details) => (
            "Could not connect to the outgoing mail server. \
             Please check your internet connection and try again."
                .to_string(),
            Some(details.clone()),
            AuthErrorKind::ConnectionFailed,
        ),
        SmtpCheckError::UntrustedCertificate(_) => (
            "The outgoing server's security certificate could not be verified.".to_string(),
            None,
            AuthErrorKind::Other,
        ),
    };

    let provider_hint = provider.map(|p| build_provider_hint(p, email));

    AuthErrorPresentation {
        user_message,
        raw_details,
        error_kind,
        provider_hint,
        support_url: GENERAL_SUPPORT_URL.to_string(),
    }
}

/// Build a user-friendly error presentation from a combined connectivity check error.
pub fn present_connectivity_error(
    error: &ConnectivityCheckError,
    provider: Option<&Provider>,
    email: &str,
) -> AuthErrorPresentation {
    match error {
        ConnectivityCheckError::ImapFailed(imap_err) => {
            present_imap_error(imap_err, provider, email)
        }
        ConnectivityCheckError::SmtpFailed(smtp_err) => {
            present_smtp_error(smtp_err, provider, email)
        }
        ConnectivityCheckError::BothFailed { imap, smtp } => {
            // Present the most actionable error — prefer auth failures over connection failures
            let imap_is_auth = matches!(imap, ImapCheckError::AuthenticationFailed);
            let smtp_is_auth = matches!(smtp, SmtpCheckError::AuthenticationFailed);

            if imap_is_auth || smtp_is_auth {
                // Auth failure: show auth-focused message
                let user_message = "Could not sign in to your email account. \
                     Please check your email address and password."
                    .to_string();
                let provider_hint = provider.map(|p| build_provider_hint(p, email));

                AuthErrorPresentation {
                    user_message,
                    raw_details: None,
                    error_kind: AuthErrorKind::AuthenticationFailed,
                    provider_hint,
                    support_url: GENERAL_SUPPORT_URL.to_string(),
                }
            } else {
                // Connection failure: show connection-focused message
                let raw = format!("IMAP: {}; SMTP: {}", imap, smtp);
                let provider_hint = provider.map(|p| build_provider_hint(p, email));

                AuthErrorPresentation {
                    user_message: "Could not connect to the email servers. \
                         Please check your internet connection and try again."
                        .to_string(),
                    raw_details: Some(raw),
                    error_kind: AuthErrorKind::ConnectionFailed,
                    provider_hint,
                    support_url: GENERAL_SUPPORT_URL.to_string(),
                }
            }
        }
    }
}

/// Get the Outlook-specific guidance URL (FR-22, FR-24).
pub fn outlook_documentation_url() -> &'static str {
    OUTLOOK_GUIDANCE_URL
}

/// Format the app-password hint text for display (FR-21).
pub fn app_password_hint_text(provider: &Provider) -> Option<String> {
    if provider.app_password_required {
        Some(format!(
            "This email provider requires an app-specific password. \
             Please generate one in your {} account security settings.",
            provider.display_name
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::provider::{MaxTlsVersion, ProviderEncryption, ServerConfig, UsernameType};

    fn make_provider(id: &str, app_password_required: bool, doc_url: Option<&str>) -> Provider {
        Provider {
            id: id.to_string(),
            display_name: format!("Test {}", id),
            domain_patterns: vec![format!("{}.com", id)],
            mx_patterns: vec![],
            incoming: ServerConfig {
                hostname: format!("imap.{}.com", id),
                port: 993,
                encryption: ProviderEncryption::SslTls,
            },
            outgoing: ServerConfig {
                hostname: format!("smtp.{}.com", id),
                port: 465,
                encryption: ProviderEncryption::SslTls,
            },
            username_type: UsernameType::EmailAddress,
            keep_alive_interval: 15,
            noop_keep_alive: false,
            partial_fetch: true,
            max_tls_version: MaxTlsVersion::Tls1_3,
            app_password_required,
            documentation_url: doc_url.map(|s| s.to_string()),
            localized_docs: vec![],
            oauth: None,
            display_order: 0,
            enabled: true,
        }
    }

    #[test]
    fn auth_failure_shows_user_friendly_message() {
        let presentation = present_imap_error(
            &ImapCheckError::AuthenticationFailed,
            None,
            "user@example.com",
        );
        assert!(presentation
            .user_message
            .contains("check your email address and password"));
        assert_eq!(presentation.error_kind, AuthErrorKind::AuthenticationFailed);
        assert!(presentation.raw_details.is_none());
    }

    #[test]
    fn connection_failure_hides_raw_details() {
        let presentation = present_imap_error(
            &ImapCheckError::ConnectionFailed("TCP timeout after 30s".to_string()),
            None,
            "user@example.com",
        );
        assert!(presentation.user_message.contains("internet connection"));
        assert!(!presentation.user_message.contains("TCP timeout"));
        assert_eq!(
            presentation.raw_details,
            Some("TCP timeout after 30s".to_string())
        );
    }

    #[test]
    fn app_password_hint_included_when_required() {
        let provider = make_provider("yahoo", true, Some("https://help.yahoo.com/kb/app"));
        let presentation = present_imap_error(
            &ImapCheckError::AuthenticationFailed,
            Some(&provider),
            "user@yahoo.com",
        );
        let hint = presentation.provider_hint.unwrap();
        assert!(hint.app_password_required);
        assert_eq!(
            hint.documentation_url,
            Some("https://help.yahoo.com/kb/app".to_string())
        );
    }

    #[test]
    fn app_password_hint_text_generated_when_required() {
        let provider = make_provider("icloud", true, Some("https://support.apple.com"));
        let text = app_password_hint_text(&provider).unwrap();
        assert!(text.contains("app-specific password"));
        assert!(text.contains("Test icloud"));
    }

    #[test]
    fn app_password_hint_text_none_when_not_required() {
        let provider = make_provider("example", false, None);
        assert!(app_password_hint_text(&provider).is_none());
    }

    #[test]
    fn outlook_domain_detected() {
        assert!(is_outlook_domain("outlook.com"));
        assert!(is_outlook_domain("hotmail.com"));
        assert!(is_outlook_domain("live.com"));
        assert!(is_outlook_domain("msn.com"));
        assert!(is_outlook_domain("Outlook.Com"));
        assert!(!is_outlook_domain("gmail.com"));
        assert!(!is_outlook_domain("yahoo.com"));
    }

    #[test]
    fn outlook_provider_hint_includes_guidance() {
        let provider = make_provider("outlook", false, Some("https://support.microsoft.com"));
        let hint = build_provider_hint(&provider, "user@outlook.com");
        assert!(hint.is_outlook_provider);
        assert!(hint.outlook_guidance.is_some());
        assert!(hint
            .outlook_guidance
            .unwrap()
            .contains("Microsoft accounts"));
    }

    #[test]
    fn non_outlook_provider_no_outlook_guidance() {
        let provider = make_provider("gmail", false, None);
        let hint = build_provider_hint(&provider, "user@gmail.com");
        assert!(!hint.is_outlook_provider);
        assert!(hint.outlook_guidance.is_none());
    }

    #[test]
    fn provider_documentation_url_shown_when_available() {
        let provider = make_provider("test", false, Some("https://example.com/docs"));
        let presentation = present_imap_error(
            &ImapCheckError::AuthenticationFailed,
            Some(&provider),
            "user@test.com",
        );
        let hint = presentation.provider_hint.unwrap();
        assert_eq!(
            hint.documentation_url,
            Some("https://example.com/docs".to_string())
        );
    }

    #[test]
    fn general_support_url_always_present() {
        let presentation = present_imap_error(
            &ImapCheckError::AuthenticationFailed,
            None,
            "user@example.com",
        );
        assert_eq!(presentation.support_url, GENERAL_SUPPORT_URL);
    }

    #[test]
    fn smtp_auth_failure_user_friendly() {
        let presentation = present_smtp_error(
            &SmtpCheckError::AuthenticationFailed,
            None,
            "user@example.com",
        );
        assert!(presentation.user_message.contains("outgoing mail server"));
        assert!(presentation
            .user_message
            .contains("check your email address and password"));
        assert_eq!(presentation.error_kind, AuthErrorKind::AuthenticationFailed);
    }

    #[test]
    fn smtp_connection_failure_hides_details() {
        let presentation = present_smtp_error(
            &SmtpCheckError::ConnectionFailed("EHLO rejected".to_string()),
            None,
            "user@example.com",
        );
        assert!(!presentation.user_message.contains("EHLO"));
        assert_eq!(presentation.raw_details, Some("EHLO rejected".to_string()));
    }

    #[test]
    fn combined_error_auth_takes_priority() {
        let error = ConnectivityCheckError::BothFailed {
            imap: ImapCheckError::AuthenticationFailed,
            smtp: SmtpCheckError::ConnectionFailed("timeout".to_string()),
        };
        let presentation = present_connectivity_error(&error, None, "user@example.com");
        assert_eq!(presentation.error_kind, AuthErrorKind::AuthenticationFailed);
    }

    #[test]
    fn combined_error_connection_when_no_auth_failure() {
        let error = ConnectivityCheckError::BothFailed {
            imap: ImapCheckError::ConnectionFailed("refused".to_string()),
            smtp: SmtpCheckError::ConnectionFailed("timeout".to_string()),
        };
        let presentation = present_connectivity_error(&error, None, "user@example.com");
        assert_eq!(presentation.error_kind, AuthErrorKind::ConnectionFailed);
        assert!(presentation.raw_details.is_some());
    }

    #[test]
    fn outlook_email_triggers_specific_guidance() {
        let provider = make_provider(
            "outlook",
            false,
            Some("https://support.microsoft.com/en-us/office"),
        );
        let presentation = present_imap_error(
            &ImapCheckError::AuthenticationFailed,
            Some(&provider),
            "user@hotmail.com",
        );
        let hint = presentation.provider_hint.unwrap();
        assert!(hint.is_outlook_provider);
        assert!(hint.outlook_guidance.is_some());
        assert!(hint.documentation_url.is_some());
    }

    #[test]
    fn outlook_documentation_url_available() {
        let url = outlook_documentation_url();
        assert!(url.contains("microsoft.com"));
    }

    #[test]
    fn raw_details_hidden_in_user_message() {
        // Verify that raw server strings never leak into user_message
        let presentation = present_imap_error(
            &ImapCheckError::ConnectionFailed(
                "FATAL: SSL_connect error code 0x1408F10B".to_string(),
            ),
            None,
            "user@example.com",
        );
        assert!(!presentation.user_message.contains("SSL_connect"));
        assert!(!presentation.user_message.contains("0x1408F10B"));
        assert!(presentation
            .raw_details
            .as_ref()
            .unwrap()
            .contains("SSL_connect"));
    }

    #[test]
    fn folder_list_failure_presentation() {
        let presentation = present_imap_error(
            &ImapCheckError::FolderListFailed("LIST command failed".to_string()),
            None,
            "user@example.com",
        );
        assert!(presentation.user_message.contains("mailbox folders"));
        assert_eq!(presentation.error_kind, AuthErrorKind::Other);
        assert_eq!(
            presentation.raw_details,
            Some("LIST command failed".to_string())
        );
    }

    #[test]
    fn connectivity_error_imap_only_delegates() {
        let error = ConnectivityCheckError::ImapFailed(ImapCheckError::AuthenticationFailed);
        let presentation = present_connectivity_error(&error, None, "user@example.com");
        assert_eq!(presentation.error_kind, AuthErrorKind::AuthenticationFailed);
    }

    #[test]
    fn connectivity_error_smtp_only_delegates() {
        let error = ConnectivityCheckError::SmtpFailed(SmtpCheckError::AuthenticationFailed);
        let presentation = present_connectivity_error(&error, None, "user@example.com");
        assert_eq!(presentation.error_kind, AuthErrorKind::AuthenticationFailed);
        assert!(presentation.user_message.contains("outgoing mail server"));
    }
}
