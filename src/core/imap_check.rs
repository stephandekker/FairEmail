use crate::core::account::{EncryptionMode, FolderRole, SystemFolders};
use crate::core::certificate::CertificateInfo;
use crate::core::provider::{Provider, ProviderEncryption, UsernameType};

/// The encryption mode to use for an IMAP connection, derived from provider settings.
fn encryption_from_provider(enc: ProviderEncryption) -> EncryptionMode {
    match enc {
        ProviderEncryption::None => EncryptionMode::None,
        ProviderEncryption::SslTls => EncryptionMode::SslTls,
        ProviderEncryption::StartTls => EncryptionMode::StartTls,
    }
}

/// Parameters for connecting to an IMAP server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImapConnectionParams {
    pub host: String,
    pub port: u16,
    pub encryption: EncryptionMode,
}

impl ImapConnectionParams {
    /// Build connection params from a provider's incoming server settings.
    pub fn from_provider(provider: &Provider) -> Self {
        Self {
            host: provider.incoming.hostname.clone(),
            port: provider.incoming.port,
            encryption: encryption_from_provider(provider.incoming.encryption),
        }
    }
}

/// A username candidate to try during authentication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsernameCandidate {
    /// Full email address (e.g. user@example.com)
    EmailAddress(String),
    /// Local part only (e.g. user)
    LocalPart(String),
}

impl UsernameCandidate {
    pub fn value(&self) -> &str {
        match self {
            Self::EmailAddress(v) | Self::LocalPart(v) => v,
        }
    }
}

/// Resolve the ordered list of username candidates to try, based on provider settings (FR-18).
/// The primary format from the provider is tried first, then the alternative.
pub fn resolve_username_candidates(email: &str, provider: &Provider) -> Vec<UsernameCandidate> {
    let local_part = email
        .rfind('@')
        .map(|pos| &email[..pos])
        .unwrap_or(email)
        .to_string();
    let full_email = email.to_string();

    match provider.username_type {
        UsernameType::EmailAddress => {
            vec![
                UsernameCandidate::EmailAddress(full_email),
                UsernameCandidate::LocalPart(local_part),
            ]
        }
        UsernameType::LocalPart => {
            vec![
                UsernameCandidate::LocalPart(local_part),
                UsernameCandidate::EmailAddress(full_email),
            ]
        }
    }
}

/// A folder reported by the IMAP server, with an optional detected role.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImapFolder {
    /// The full folder name as reported by the server.
    pub name: String,
    /// The IMAP attributes string (e.g. "\\Sent", "\\Trash").
    pub attributes: String,
    /// Detected system-folder role, if any.
    pub role: Option<FolderRole>,
}

/// Detect the system-folder role from an IMAP attribute string.
/// Also performs name-based heuristic detection for folders without explicit attributes.
pub fn detect_folder_role(name: &str, attributes: &str) -> Option<FolderRole> {
    // First try attribute-based detection (most reliable)
    match attributes {
        "\\Drafts" => return Some(FolderRole::Drafts),
        "\\Sent" => return Some(FolderRole::Sent),
        "\\Archive" => return Some(FolderRole::Archive),
        "\\Trash" => return Some(FolderRole::Trash),
        "\\Junk" => return Some(FolderRole::Junk),
        _ => {}
    }

    // Fallback: name-based heuristic detection
    let lower = name.to_lowercase();
    if lower == "inbox" {
        // Inbox is special — it doesn't have a FolderRole variant because it always exists.
        // We return None here; inbox detection is handled at the result level.
        return None;
    }
    if lower == "drafts" || lower == "draft" {
        return Some(FolderRole::Drafts);
    }
    if lower == "sent" || lower == "sent messages" || lower == "sent items" {
        return Some(FolderRole::Sent);
    }
    if lower == "archive" || lower == "archives" || lower == "all mail" {
        return Some(FolderRole::Archive);
    }
    if lower == "trash"
        || lower == "deleted"
        || lower == "deleted messages"
        || lower == "deleted items"
    {
        return Some(FolderRole::Trash);
    }
    if lower == "junk" || lower == "spam" || lower == "bulk mail" {
        return Some(FolderRole::Junk);
    }
    None
}

/// The result of a successful IMAP connectivity check.
#[derive(Debug, Clone)]
pub struct ImapCheckSuccess {
    /// The username that worked.
    pub authenticated_username: String,
    /// All folders found on the server.
    pub folders: Vec<ImapFolder>,
    /// Detected system folder assignments.
    pub system_folders: SystemFolders,
    /// Whether the Inbox was found.
    pub has_inbox: bool,
}

/// The reason an IMAP check failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ImapCheckError {
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
    #[error("folder listing failed: {0}")]
    FolderListFailed(String),
    #[error("untrusted certificate from server")]
    UntrustedCertificate(Box<CertificateInfo>),
}

/// The overall result of an IMAP connectivity check.
pub type ImapCheckResult = Result<ImapCheckSuccess, ImapCheckError>;

/// Build the `ImapCheckSuccess` from a raw folder list (name + attributes pairs).
pub fn build_imap_success(
    authenticated_username: String,
    raw_folders: Vec<(String, String)>,
) -> ImapCheckSuccess {
    let mut folders = Vec::with_capacity(raw_folders.len());
    let mut has_inbox = false;
    let mut system_folders = SystemFolders::default();

    for (name, attributes) in &raw_folders {
        let role = detect_folder_role(name, attributes);

        // Check for inbox
        if name.eq_ignore_ascii_case("inbox") {
            has_inbox = true;
        }

        // Assign system folder
        match role {
            Some(FolderRole::Drafts) if system_folders.drafts.is_none() => {
                system_folders.drafts = Some(name.clone());
            }
            Some(FolderRole::Sent) if system_folders.sent.is_none() => {
                system_folders.sent = Some(name.clone());
            }
            Some(FolderRole::Archive) if system_folders.archive.is_none() => {
                system_folders.archive = Some(name.clone());
            }
            Some(FolderRole::Trash) if system_folders.trash.is_none() => {
                system_folders.trash = Some(name.clone());
            }
            Some(FolderRole::Junk) if system_folders.junk.is_none() => {
                system_folders.junk = Some(name.clone());
            }
            _ => {}
        }

        folders.push(ImapFolder {
            name: name.clone(),
            attributes: attributes.clone(),
            role,
        });
    }

    ImapCheckSuccess {
        authenticated_username,
        folders,
        system_folders,
        has_inbox,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::provider::{MaxTlsVersion, ServerConfig};

    fn make_provider(username_type: UsernameType) -> Provider {
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
                port: 465,
                encryption: ProviderEncryption::SslTls,
            },
            username_type,
            keep_alive_interval: 15,
            noop_keep_alive: false,
            partial_fetch: true,
            max_tls_version: MaxTlsVersion::Tls1_3,
            app_password_required: false,
            documentation_url: None,
            localized_docs: vec![],
            oauth: None,
            display_order: 0,
            enabled: true,
            supports_shared_mailbox: false,
            subtitle: None,
            registration_url: None,
            graph: None,
        }
    }

    #[test]
    fn connection_params_from_provider() {
        let provider = make_provider(UsernameType::EmailAddress);
        let params = ImapConnectionParams::from_provider(&provider);
        assert_eq!(params.host, "imap.example.com");
        assert_eq!(params.port, 993);
        assert_eq!(params.encryption, EncryptionMode::SslTls);
    }

    #[test]
    fn connection_params_starttls() {
        let mut provider = make_provider(UsernameType::EmailAddress);
        provider.incoming.encryption = ProviderEncryption::StartTls;
        provider.incoming.port = 143;
        let params = ImapConnectionParams::from_provider(&provider);
        assert_eq!(params.encryption, EncryptionMode::StartTls);
        assert_eq!(params.port, 143);
    }

    #[test]
    fn username_candidates_email_address_primary() {
        let provider = make_provider(UsernameType::EmailAddress);
        let candidates = resolve_username_candidates("user@example.com", &provider);
        assert_eq!(candidates.len(), 2);
        assert_eq!(
            candidates[0],
            UsernameCandidate::EmailAddress("user@example.com".to_string())
        );
        assert_eq!(
            candidates[1],
            UsernameCandidate::LocalPart("user".to_string())
        );
    }

    #[test]
    fn username_candidates_local_part_primary() {
        let provider = make_provider(UsernameType::LocalPart);
        let candidates = resolve_username_candidates("user@example.com", &provider);
        assert_eq!(candidates.len(), 2);
        assert_eq!(
            candidates[0],
            UsernameCandidate::LocalPart("user".to_string())
        );
        assert_eq!(
            candidates[1],
            UsernameCandidate::EmailAddress("user@example.com".to_string())
        );
    }

    #[test]
    fn username_candidates_no_at_sign() {
        let provider = make_provider(UsernameType::EmailAddress);
        let candidates = resolve_username_candidates("localonly", &provider);
        assert_eq!(
            candidates[0],
            UsernameCandidate::EmailAddress("localonly".to_string())
        );
        assert_eq!(
            candidates[1],
            UsernameCandidate::LocalPart("localonly".to_string())
        );
    }

    #[test]
    fn detect_role_from_attributes() {
        assert_eq!(
            detect_folder_role("Whatever", "\\Sent"),
            Some(FolderRole::Sent)
        );
        assert_eq!(
            detect_folder_role("Whatever", "\\Drafts"),
            Some(FolderRole::Drafts)
        );
        assert_eq!(
            detect_folder_role("Whatever", "\\Archive"),
            Some(FolderRole::Archive)
        );
        assert_eq!(
            detect_folder_role("Whatever", "\\Trash"),
            Some(FolderRole::Trash)
        );
        assert_eq!(
            detect_folder_role("Whatever", "\\Junk"),
            Some(FolderRole::Junk)
        );
    }

    #[test]
    fn detect_role_from_name_heuristic() {
        assert_eq!(detect_folder_role("Sent", ""), Some(FolderRole::Sent));
        assert_eq!(
            detect_folder_role("Sent Messages", ""),
            Some(FolderRole::Sent)
        );
        assert_eq!(detect_folder_role("Sent Items", ""), Some(FolderRole::Sent));
        assert_eq!(detect_folder_role("Drafts", ""), Some(FolderRole::Drafts));
        assert_eq!(detect_folder_role("Draft", ""), Some(FolderRole::Drafts));
        assert_eq!(detect_folder_role("Archive", ""), Some(FolderRole::Archive));
        assert_eq!(
            detect_folder_role("Archives", ""),
            Some(FolderRole::Archive)
        );
        assert_eq!(
            detect_folder_role("All Mail", ""),
            Some(FolderRole::Archive)
        );
        assert_eq!(detect_folder_role("Trash", ""), Some(FolderRole::Trash));
        assert_eq!(detect_folder_role("Deleted", ""), Some(FolderRole::Trash));
        assert_eq!(
            detect_folder_role("Deleted Messages", ""),
            Some(FolderRole::Trash)
        );
        assert_eq!(
            detect_folder_role("Deleted Items", ""),
            Some(FolderRole::Trash)
        );
        assert_eq!(detect_folder_role("Junk", ""), Some(FolderRole::Junk));
        assert_eq!(detect_folder_role("Spam", ""), Some(FolderRole::Junk));
        assert_eq!(detect_folder_role("Bulk Mail", ""), Some(FolderRole::Junk));
    }

    #[test]
    fn detect_role_unknown_returns_none() {
        assert_eq!(detect_folder_role("My Custom Folder", ""), None);
        assert_eq!(detect_folder_role("INBOX", ""), None);
    }

    #[test]
    fn build_success_detects_inbox_and_system_folders() {
        let raw_folders = vec![
            ("INBOX".to_string(), "".to_string()),
            ("Sent".to_string(), "\\Sent".to_string()),
            ("Drafts".to_string(), "\\Drafts".to_string()),
            ("Trash".to_string(), "\\Trash".to_string()),
            ("Spam".to_string(), "\\Junk".to_string()),
            ("Archive".to_string(), "\\Archive".to_string()),
            ("Custom".to_string(), "".to_string()),
        ];

        let result = build_imap_success("user@example.com".to_string(), raw_folders);
        assert!(result.has_inbox);
        assert_eq!(result.authenticated_username, "user@example.com");
        assert_eq!(result.folders.len(), 7);
        assert_eq!(result.system_folders.sent, Some("Sent".to_string()));
        assert_eq!(result.system_folders.drafts, Some("Drafts".to_string()));
        assert_eq!(result.system_folders.trash, Some("Trash".to_string()));
        assert_eq!(result.system_folders.junk, Some("Spam".to_string()));
        assert_eq!(result.system_folders.archive, Some("Archive".to_string()));
    }

    #[test]
    fn build_success_name_heuristic_fallback() {
        let raw_folders = vec![
            ("INBOX".to_string(), "".to_string()),
            ("Sent Messages".to_string(), "".to_string()),
            ("Trash".to_string(), "".to_string()),
        ];

        let result = build_imap_success("user@example.com".to_string(), raw_folders);
        assert!(result.has_inbox);
        assert_eq!(
            result.system_folders.sent,
            Some("Sent Messages".to_string())
        );
        assert_eq!(result.system_folders.trash, Some("Trash".to_string()));
    }

    #[test]
    fn build_success_no_inbox() {
        let raw_folders = vec![("Sent".to_string(), "\\Sent".to_string())];
        let result = build_imap_success("user@example.com".to_string(), raw_folders);
        assert!(!result.has_inbox);
    }

    #[test]
    fn build_success_first_role_wins() {
        // If two folders have the same role, the first one wins
        let raw_folders = vec![
            ("Sent".to_string(), "\\Sent".to_string()),
            ("Sent Items".to_string(), "\\Sent".to_string()),
        ];
        let result = build_imap_success("user@example.com".to_string(), raw_folders);
        assert_eq!(result.system_folders.sent, Some("Sent".to_string()));
    }
}
