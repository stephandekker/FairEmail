use std::time::Duration;

use crate::core::account::{AuthMethod, EncryptionMode, FolderRole, Protocol};
use crate::core::imap_check::ImapFolder;

/// Timeout for the inbound connection test.
pub const INBOUND_TEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Parameters for an inbound server connection test.
#[derive(Debug, Clone)]
pub struct InboundTestParams {
    pub host: String,
    pub port: u16,
    pub encryption: EncryptionMode,
    pub auth_method: AuthMethod,
    pub username: String,
    pub credential: String,
    pub protocol: Protocol,
    /// When true, skip certificate verification (FR-11, FR-12).
    /// Also relaxes username/credential requirements (FR-18, FR-19).
    pub insecure: bool,
    /// Pinned certificate fingerprint (SHA-256). When set, the connection
    /// accepts the server certificate only if its fingerprint matches (FR-15).
    pub accepted_fingerprint: Option<String>,
    /// Path to a client certificate file (PKCS#12) for mutual TLS (FR-9, FR-19).
    /// When set, the password/credential field is no longer required.
    pub client_certificate: Option<String>,
    /// When true, require DANE (TLSA) verification for TLS connections (FR-13).
    pub dane: bool,
    /// When true, require DNSSEC-validated DNS resolution (FR-14).
    pub dnssec: bool,
    /// Optional authentication realm for SASL/NTLM domain (FR-10).
    pub auth_realm: Option<String>,
}

/// The result of a successful inbound connection test.
#[derive(Debug, Clone)]
pub struct InboundTestSuccess {
    /// Folders discovered on the server (IMAP only).
    pub folders: Vec<ImapFolder>,
    /// Whether the server supports IDLE (push notifications).
    pub idle_supported: bool,
    /// Whether the server supports UTF-8 (IMAP UTF8=ACCEPT).
    pub utf8_supported: bool,
}

/// Errors that can occur during an inbound connection test.
#[derive(Debug, Clone, thiserror::Error)]
pub enum InboundTestError {
    #[error("host must not be empty")]
    EmptyHost,
    #[error("username must not be empty")]
    EmptyUsername,
    #[error("credential must not be empty")]
    EmptyCredential,
    #[error("connection timed out")]
    Timeout,
    #[error("connection failed: {0}")]
    ConnectionFailed(String),
    #[error("authentication failed")]
    AuthenticationFailed,
    #[error("DNS resolution failed for host: {0}")]
    DnsResolutionFailed(String),
    #[error("connection refused by {host}:{port}")]
    ConnectionRefused { host: String, port: u16 },
    #[error("TLS/SSL handshake failed: {message}")]
    TlsHandshakeFailed {
        message: String,
        /// Certificate fingerprint extracted from the untrusted certificate, if available.
        fingerprint: Option<String>,
    },
    #[error("protocol mismatch: {0}")]
    ProtocolMismatch(String),
    #[error("DNSSEC validation failed: {0}")]
    DnssecFailed(String),
    #[error("DANE verification failed: {0}")]
    DaneFailed(String),
}

/// The result type for inbound connection tests.
pub type InboundTestResult = Result<InboundTestSuccess, InboundTestError>;

impl InboundTestParams {
    /// Validate that the parameters are sufficient to attempt a connection.
    ///
    /// When `insecure` is true, username and credential are not required (FR-18, FR-19).
    pub fn validate(&self) -> Result<(), InboundTestError> {
        if self.host.trim().is_empty() {
            return Err(InboundTestError::EmptyHost);
        }
        if !self.insecure {
            if self.username.trim().is_empty() {
                return Err(InboundTestError::EmptyUsername);
            }
            // Password is not required when a client certificate is selected (FR-19).
            if self.credential.trim().is_empty() && self.client_certificate.is_none() {
                return Err(InboundTestError::EmptyCredential);
            }
        }
        Ok(())
    }
}

impl InboundTestSuccess {
    /// Format the folder list as a user-readable string.
    pub fn format_folder_list(&self) -> String {
        if self.folders.is_empty() {
            return String::new();
        }
        self.folders
            .iter()
            .map(|f| {
                if let Some(ref role) = f.role {
                    format!("{} ({})", f.name, format_folder_role(role))
                } else if f.name.eq_ignore_ascii_case("inbox") {
                    format!("{} (Inbox)", f.name)
                } else {
                    f.name.clone()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Format the capabilities summary line.
    pub fn format_capabilities(&self) -> String {
        let idle_status = if self.idle_supported {
            "IDLE: supported"
        } else {
            "IDLE: not supported (polling fallback will be used)"
        };
        let utf8_status = if self.utf8_supported {
            "UTF-8: supported"
        } else {
            "UTF-8: not supported"
        };
        format!("{idle_status}\n{utf8_status}")
    }
}

fn format_folder_role(role: &FolderRole) -> &'static str {
    match role {
        FolderRole::Drafts => "Drafts",
        FolderRole::Sent => "Sent",
        FolderRole::Archive => "Archive",
        FolderRole::Trash => "Trash",
        FolderRole::Junk => "Spam",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_params() -> InboundTestParams {
        InboundTestParams {
            host: "imap.example.com".into(),
            port: 993,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "password".into(),
            protocol: Protocol::Imap,
            insecure: false,
            accepted_fingerprint: None,
            client_certificate: None,
            dane: false,
            dnssec: false,
            auth_realm: None,
        }
    }

    #[test]
    fn validate_passes_for_valid_params() {
        assert!(valid_params().validate().is_ok());
    }

    #[test]
    fn validate_fails_empty_host() {
        let mut params = valid_params();
        params.host = "  ".into();
        assert!(matches!(
            params.validate(),
            Err(InboundTestError::EmptyHost)
        ));
    }

    #[test]
    fn validate_fails_empty_username() {
        let mut params = valid_params();
        params.username = "".into();
        assert!(matches!(
            params.validate(),
            Err(InboundTestError::EmptyUsername)
        ));
    }

    #[test]
    fn validate_fails_empty_credential() {
        let mut params = valid_params();
        params.credential = " ".into();
        assert!(matches!(
            params.validate(),
            Err(InboundTestError::EmptyCredential)
        ));
    }

    #[test]
    fn format_folder_list_empty() {
        let success = InboundTestSuccess {
            folders: vec![],
            idle_supported: true,
            utf8_supported: true,
        };
        assert_eq!(success.format_folder_list(), "");
    }

    #[test]
    fn format_folder_list_with_roles() {
        let success = InboundTestSuccess {
            folders: vec![
                ImapFolder {
                    name: "INBOX".into(),
                    attributes: "".into(),
                    role: None,
                },
                ImapFolder {
                    name: "Sent".into(),
                    attributes: "\\Sent".into(),
                    role: Some(FolderRole::Sent),
                },
                ImapFolder {
                    name: "Trash".into(),
                    attributes: "\\Trash".into(),
                    role: Some(FolderRole::Trash),
                },
                ImapFolder {
                    name: "Custom".into(),
                    attributes: "".into(),
                    role: None,
                },
            ],
            idle_supported: true,
            utf8_supported: false,
        };
        let list = success.format_folder_list();
        assert!(list.contains("INBOX (Inbox)"));
        assert!(list.contains("Sent (Sent)"));
        assert!(list.contains("Trash (Trash)"));
        assert!(list.contains("Custom"));
    }

    #[test]
    fn format_capabilities_idle_supported() {
        let success = InboundTestSuccess {
            folders: vec![],
            idle_supported: true,
            utf8_supported: true,
        };
        let caps = success.format_capabilities();
        assert!(caps.contains("IDLE: supported"));
        assert!(caps.contains("UTF-8: supported"));
    }

    #[test]
    fn validate_insecure_allows_empty_username() {
        let mut params = valid_params();
        params.insecure = true;
        params.username = "".into();
        assert!(params.validate().is_ok());
    }

    #[test]
    fn validate_insecure_allows_empty_credential() {
        let mut params = valid_params();
        params.insecure = true;
        params.credential = "".into();
        assert!(params.validate().is_ok());
    }

    #[test]
    fn validate_insecure_still_requires_host() {
        let mut params = valid_params();
        params.insecure = true;
        params.host = "".into();
        assert!(matches!(
            params.validate(),
            Err(InboundTestError::EmptyHost)
        ));
    }

    #[test]
    fn validate_client_certificate_allows_empty_credential() {
        let mut params = valid_params();
        params.credential = "".into();
        params.client_certificate = Some("/path/to/cert.p12".into());
        assert!(params.validate().is_ok());
    }

    #[test]
    fn validate_client_certificate_still_requires_username() {
        let mut params = valid_params();
        params.username = "".into();
        params.client_certificate = Some("/path/to/cert.p12".into());
        assert!(matches!(
            params.validate(),
            Err(InboundTestError::EmptyUsername)
        ));
    }

    #[test]
    fn format_capabilities_idle_not_supported() {
        let success = InboundTestSuccess {
            folders: vec![],
            idle_supported: false,
            utf8_supported: false,
        };
        let caps = success.format_capabilities();
        assert!(caps.contains("polling fallback"));
        assert!(caps.contains("UTF-8: not supported"));
    }

    #[test]
    fn dane_defaults_to_false() {
        let params = valid_params();
        assert!(!params.dane);
    }

    #[test]
    fn dnssec_defaults_to_false() {
        let params = valid_params();
        assert!(!params.dnssec);
    }

    #[test]
    fn validate_passes_with_dane_enabled() {
        let mut params = valid_params();
        params.dane = true;
        assert!(params.validate().is_ok());
    }

    #[test]
    fn validate_passes_with_dnssec_enabled() {
        let mut params = valid_params();
        params.dnssec = true;
        assert!(params.validate().is_ok());
    }

    #[test]
    fn validate_passes_with_both_dane_and_dnssec() {
        let mut params = valid_params();
        params.dane = true;
        params.dnssec = true;
        assert!(params.validate().is_ok());
    }
}
