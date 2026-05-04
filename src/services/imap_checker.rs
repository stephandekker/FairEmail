use crate::core::imap_check::{
    build_imap_success, resolve_username_candidates, ImapCheckError, ImapCheckResult,
    ImapConnectionParams,
};
use crate::core::provider::Provider;

/// Trait for performing IMAP connectivity checks.
/// Implementations handle the actual network I/O (connect, authenticate, list folders).
pub trait ImapChecker {
    /// Perform a full IMAP connectivity check:
    /// 1. Connect to the server using the provider's incoming settings.
    /// 2. Try each username candidate until one authenticates successfully (FR-18).
    /// 3. Enumerate server-side folders and detect system-folder roles.
    fn check_imap(&self, email: &str, password: &str, provider: &Provider) -> ImapCheckResult;
}

/// Mock implementation of `ImapChecker` for testing.
///
/// Behavior:
/// - Hosts containing "unreachable" → `ConnectionFailed`
/// - Hosts containing "authfail" → `AuthenticationFailed` (all usernames fail)
/// - Hosts containing "listfail" → authenticated ok but folder listing fails
/// - Password "wrong" → `AuthenticationFailed`
/// - Username "failfirst@" prefix in email → first candidate fails, second succeeds
/// - Otherwise → success with a standard set of folders
pub struct MockImapChecker;

impl ImapChecker for MockImapChecker {
    fn check_imap(&self, email: &str, password: &str, provider: &Provider) -> ImapCheckResult {
        let params = ImapConnectionParams::from_provider(provider);

        // Simulate connection failure
        if params.host.to_lowercase().contains("unreachable") {
            return Err(ImapCheckError::ConnectionFailed(
                "could not connect to host".to_string(),
            ));
        }

        // Simulate folder listing failure
        let list_fails = params.host.to_lowercase().contains("listfail");

        // Simulate auth failure for all candidates
        if params.host.to_lowercase().contains("authfail") || password == "wrong" {
            return Err(ImapCheckError::AuthenticationFailed);
        }

        let candidates = resolve_username_candidates(email, provider);

        // Simulate first-candidate-fails scenario
        let authenticated_username = if email.starts_with("failfirst@") {
            // First candidate fails, second succeeds
            if candidates.len() > 1 {
                candidates[1].value().to_string()
            } else {
                return Err(ImapCheckError::AuthenticationFailed);
            }
        } else {
            candidates[0].value().to_string()
        };

        if list_fails {
            return Err(ImapCheckError::FolderListFailed(
                "LIST command failed".to_string(),
            ));
        }

        // Return a standard folder set
        let raw_folders = vec![
            ("INBOX".to_string(), "".to_string()),
            ("Sent".to_string(), "\\Sent".to_string()),
            ("Drafts".to_string(), "\\Drafts".to_string()),
            ("Trash".to_string(), "\\Trash".to_string()),
            ("Junk".to_string(), "\\Junk".to_string()),
            ("Archive".to_string(), "\\Archive".to_string()),
        ];

        Ok(build_imap_success(authenticated_username, raw_folders))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::provider::{MaxTlsVersion, ProviderEncryption, ServerConfig, UsernameType};

    fn test_provider(username_type: UsernameType) -> Provider {
        Provider {
            id: "test".to_string(),
            display_name: "Test".to_string(),
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
        }
    }

    #[test]
    fn successful_check_returns_folders_and_system_roles() {
        let checker = MockImapChecker;
        let provider = test_provider(UsernameType::EmailAddress);
        let result = checker
            .check_imap("user@example.com", "secret", &provider)
            .unwrap();

        assert_eq!(result.authenticated_username, "user@example.com");
        assert!(result.has_inbox);
        assert_eq!(result.folders.len(), 6);
        assert_eq!(result.system_folders.sent, Some("Sent".to_string()));
        assert_eq!(result.system_folders.drafts, Some("Drafts".to_string()));
        assert_eq!(result.system_folders.trash, Some("Trash".to_string()));
        assert_eq!(result.system_folders.junk, Some("Junk".to_string()));
        assert_eq!(result.system_folders.archive, Some("Archive".to_string()));
    }

    #[test]
    fn connection_failure_for_unreachable_host() {
        let checker = MockImapChecker;
        let mut provider = test_provider(UsernameType::EmailAddress);
        provider.incoming.hostname = "unreachable.example.com".to_string();
        let result = checker.check_imap("user@example.com", "secret", &provider);
        assert!(matches!(result, Err(ImapCheckError::ConnectionFailed(_))));
    }

    #[test]
    fn auth_failure_for_authfail_host() {
        let checker = MockImapChecker;
        let mut provider = test_provider(UsernameType::EmailAddress);
        provider.incoming.hostname = "authfail.example.com".to_string();
        let result = checker.check_imap("user@example.com", "secret", &provider);
        assert!(matches!(result, Err(ImapCheckError::AuthenticationFailed)));
    }

    #[test]
    fn auth_failure_for_wrong_password() {
        let checker = MockImapChecker;
        let provider = test_provider(UsernameType::EmailAddress);
        let result = checker.check_imap("user@example.com", "wrong", &provider);
        assert!(matches!(result, Err(ImapCheckError::AuthenticationFailed)));
    }

    #[test]
    fn fallback_username_on_first_failure() {
        let checker = MockImapChecker;
        let provider = test_provider(UsernameType::EmailAddress);
        // "failfirst@" prefix triggers first-candidate-fails behavior
        let result = checker
            .check_imap("failfirst@example.com", "secret", &provider)
            .unwrap();

        // Primary is EmailAddress, so fallback is LocalPart
        assert_eq!(result.authenticated_username, "failfirst");
    }

    #[test]
    fn fallback_username_local_part_primary() {
        let checker = MockImapChecker;
        let provider = test_provider(UsernameType::LocalPart);
        let result = checker
            .check_imap("failfirst@example.com", "secret", &provider)
            .unwrap();

        // Primary is LocalPart, so fallback is EmailAddress
        assert_eq!(result.authenticated_username, "failfirst@example.com");
    }

    #[test]
    fn folder_list_failure() {
        let checker = MockImapChecker;
        let mut provider = test_provider(UsernameType::EmailAddress);
        provider.incoming.hostname = "listfail.example.com".to_string();
        let result = checker.check_imap("user@example.com", "secret", &provider);
        assert!(matches!(result, Err(ImapCheckError::FolderListFailed(_))));
    }
}
