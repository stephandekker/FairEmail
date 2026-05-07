use crate::core::account::EncryptionMode;
use crate::core::certificate::CertificateInfo;
use crate::core::provider::{Provider, ProviderEncryption};

/// The encryption mode to use for an SMTP connection, derived from provider settings.
fn encryption_from_provider(enc: ProviderEncryption) -> EncryptionMode {
    match enc {
        ProviderEncryption::None => EncryptionMode::None,
        ProviderEncryption::SslTls => EncryptionMode::SslTls,
        ProviderEncryption::StartTls => EncryptionMode::StartTls,
    }
}

/// Parameters for connecting to an SMTP server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmtpConnectionParams {
    pub host: String,
    pub port: u16,
    pub encryption: EncryptionMode,
}

impl SmtpConnectionParams {
    /// Build connection params from a provider's outgoing server settings.
    pub fn from_provider(provider: &Provider) -> Self {
        Self {
            host: provider.outgoing.hostname.clone(),
            port: provider.outgoing.port,
            encryption: encryption_from_provider(provider.outgoing.encryption),
        }
    }
}

/// The result of a successful SMTP connectivity check.
#[derive(Debug, Clone)]
pub struct SmtpCheckSuccess {
    /// The username that successfully authenticated.
    pub authenticated_username: String,
    /// The server's advertised maximum message size in bytes, if any (FR-17.6).
    pub max_message_size: Option<u64>,
}

/// The reason an SMTP check failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SmtpCheckError {
    #[error("connection failed: {0}")]
    ConnectionFailed(String),
    #[error("authentication failed for all username formats")]
    AuthenticationFailed,
    #[error("no common authentication mechanism supported by both client and server")]
    MechanismUnavailable,
    #[error("all compatible authentication mechanisms have been disabled in settings")]
    AllMechanismsDisabled,
    #[error("authentication token expired or revoked: {0}")]
    TokenExpired(String),
    #[error("server error during authentication: {0}")]
    ServerError(String),
    #[error("untrusted certificate from server")]
    UntrustedCertificate(Box<CertificateInfo>),
}

/// The overall result of an SMTP connectivity check.
pub type SmtpCheckResult = Result<SmtpCheckSuccess, SmtpCheckError>;

/// Combined result of both IMAP and SMTP connectivity checks.
///
/// Both checks must succeed for the account to be saved (AC-7, Design Note N-2).
#[derive(Debug, Clone)]
pub struct ConnectivityCheckResult {
    pub imap: crate::core::imap_check::ImapCheckSuccess,
    pub smtp: SmtpCheckSuccess,
}

/// Error when the combined connectivity check fails.
///
/// If either IMAP or SMTP fails, the account must NOT be saved.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ConnectivityCheckError {
    #[error("IMAP check failed: {0}")]
    ImapFailed(#[from] crate::core::imap_check::ImapCheckError),
    #[error("SMTP check failed: {0}")]
    SmtpFailed(#[from] SmtpCheckError),
    #[error("IMAP check failed: {imap}; SMTP check failed: {smtp}")]
    BothFailed {
        imap: crate::core::imap_check::ImapCheckError,
        smtp: SmtpCheckError,
    },
}

/// Build a combined connectivity check result from individual IMAP and SMTP results.
///
/// Returns `Ok` only if both succeed. If either or both fail, the appropriate error
/// variant is returned — the account must not be saved in any failure case.
pub fn combine_connectivity_results(
    imap_result: crate::core::imap_check::ImapCheckResult,
    smtp_result: SmtpCheckResult,
) -> Result<ConnectivityCheckResult, ConnectivityCheckError> {
    match (imap_result, smtp_result) {
        (Ok(imap), Ok(smtp)) => Ok(ConnectivityCheckResult { imap, smtp }),
        (Err(imap_err), Err(smtp_err)) => Err(ConnectivityCheckError::BothFailed {
            imap: imap_err,
            smtp: smtp_err,
        }),
        (Err(imap_err), Ok(_)) => Err(ConnectivityCheckError::ImapFailed(imap_err)),
        (Ok(_), Err(smtp_err)) => Err(ConnectivityCheckError::SmtpFailed(smtp_err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::imap_check::{build_imap_success, ImapCheckError};
    use crate::core::provider::{MaxTlsVersion, ServerConfig, UsernameType};

    fn make_provider(encryption: ProviderEncryption, port: u16) -> Provider {
        Provider {
            id: "test".to_string(),
            display_name: "Test Provider".to_string(),
            domain_patterns: vec!["example.com".to_string()],
            mx_patterns: vec![],
            incoming: ServerConfig {
                hostname: "imap.example.com".to_string(),
                port: 993,
                encryption: ProviderEncryption::SslTls,
            },
            outgoing: ServerConfig {
                hostname: "smtp.example.com".to_string(),
                port,
                encryption,
            },
            username_type: UsernameType::EmailAddress,
            keep_alive_interval: 15,
            noop_keep_alive: false,
            partial_fetch: true,
            max_tls_version: MaxTlsVersion::Tls1_3,
            app_password_required: false,
            disable_ip_connections: false,
            requires_manual_enablement: false,
            documentation_url: None,
            localized_docs: vec![],
            oauth: None,
            display_order: 0,
            enabled: true,
            supports_shared_mailbox: false,
            subtitle: None,
            registration_url: None,
            app_password_url: None,
            graph: None,
            debug_only: false,
            variant_of: None,
        }
    }

    #[test]
    fn connection_params_from_provider_ssl() {
        let provider = make_provider(ProviderEncryption::SslTls, 465);
        let params = SmtpConnectionParams::from_provider(&provider);
        assert_eq!(params.host, "smtp.example.com");
        assert_eq!(params.port, 465);
        assert_eq!(params.encryption, EncryptionMode::SslTls);
    }

    #[test]
    fn connection_params_from_provider_starttls() {
        let provider = make_provider(ProviderEncryption::StartTls, 587);
        let params = SmtpConnectionParams::from_provider(&provider);
        assert_eq!(params.host, "smtp.example.com");
        assert_eq!(params.port, 587);
        assert_eq!(params.encryption, EncryptionMode::StartTls);
    }

    #[test]
    fn connection_params_from_provider_none() {
        let provider = make_provider(ProviderEncryption::None, 25);
        let params = SmtpConnectionParams::from_provider(&provider);
        assert_eq!(params.encryption, EncryptionMode::None);
        assert_eq!(params.port, 25);
    }

    fn sample_imap_success() -> crate::core::imap_check::ImapCheckSuccess {
        build_imap_success(
            "user@example.com".to_string(),
            vec![("INBOX".to_string(), "".to_string())],
        )
    }

    fn sample_smtp_success() -> SmtpCheckSuccess {
        SmtpCheckSuccess {
            authenticated_username: "user@example.com".to_string(),
            max_message_size: Some(25_000_000),
        }
    }

    #[test]
    fn combine_both_succeed() {
        let result =
            combine_connectivity_results(Ok(sample_imap_success()), Ok(sample_smtp_success()));
        assert!(result.is_ok());
        let combined = result.unwrap();
        assert_eq!(combined.imap.authenticated_username, "user@example.com");
        assert_eq!(combined.smtp.max_message_size, Some(25_000_000));
    }

    #[test]
    fn combine_imap_fails_smtp_succeeds_returns_error() {
        let result = combine_connectivity_results(
            Err(ImapCheckError::AuthenticationFailed),
            Ok(sample_smtp_success()),
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConnectivityCheckError::ImapFailed(ImapCheckError::AuthenticationFailed)
        ));
    }

    #[test]
    fn combine_imap_succeeds_smtp_fails_returns_error() {
        let result = combine_connectivity_results(
            Ok(sample_imap_success()),
            Err(SmtpCheckError::ConnectionFailed("timeout".to_string())),
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConnectivityCheckError::SmtpFailed(SmtpCheckError::ConnectionFailed(_))
        ));
    }

    #[test]
    fn combine_both_fail_returns_both_failed() {
        let result = combine_connectivity_results(
            Err(ImapCheckError::ConnectionFailed("refused".to_string())),
            Err(SmtpCheckError::AuthenticationFailed),
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConnectivityCheckError::BothFailed { .. }
        ));
    }

    #[test]
    fn combine_smtp_no_max_size() {
        let smtp = SmtpCheckSuccess {
            authenticated_username: "user@example.com".to_string(),
            max_message_size: None,
        };
        let result = combine_connectivity_results(Ok(sample_imap_success()), Ok(smtp));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().smtp.max_message_size, None);
    }

    #[test]
    fn connectivity_check_error_display() {
        let err = ConnectivityCheckError::SmtpFailed(SmtpCheckError::ConnectionFailed(
            "connection refused".to_string(),
        ));
        let msg = err.to_string();
        assert!(msg.contains("SMTP"));
        assert!(msg.contains("connection refused"));
    }

    #[test]
    fn connectivity_check_error_display_both() {
        let err = ConnectivityCheckError::BothFailed {
            imap: ImapCheckError::AuthenticationFailed,
            smtp: SmtpCheckError::ConnectionFailed("timeout".to_string()),
        };
        let msg = err.to_string();
        assert!(msg.contains("IMAP"));
        assert!(msg.contains("SMTP"));
    }
}
