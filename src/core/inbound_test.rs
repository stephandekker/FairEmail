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
    #[error("TLS/SSL handshake failed: {0}")]
    TlsHandshakeFailed(String),
    #[error("protocol mismatch: {0}")]
    ProtocolMismatch(String),
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
            if self.credential.trim().is_empty() {
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
}
