use crate::core::account::{Account, AuthMethod, Protocol};
use crate::core::oauth_signin::is_oauth_provider;
use crate::core::provider::{OAuthConfig, ProviderDatabase};

/// Parameters for re-authorizing an existing account (FR-32, FR-33, FR-34).
#[derive(Debug, Clone)]
pub struct ReauthParams {
    /// The username to match against existing accounts (FR-34).
    pub username: String,
    /// The incoming-server protocol type to match (FR-34).
    pub protocol: Protocol,
    /// The new credential (password or OAuth token) (FR-33).
    pub new_credential: String,
    /// The new authentication method (may change from password to OAuth or vice versa).
    pub new_auth_method: AuthMethod,
}

/// Errors from the re-authorization flow.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum ReauthError {
    #[error("no matching account found for username '{username}' with protocol {protocol}")]
    NoMatchingAccount {
        username: String,
        protocol: Protocol,
    },
    #[error("credential must not be empty")]
    EmptyCredential,
}

/// Determine the OAuth configuration for re-authorization of an existing account (FR-25).
///
/// Looks up the provider by the account's IMAP hostname and email domain,
/// returning the OAuth configuration if the account uses OAuth2 and the
/// provider supports it.
pub fn find_oauth_config_for_reauth(
    account: &Account,
    provider_db: &ProviderDatabase,
) -> Option<OAuthConfig> {
    if account.auth_method() != AuthMethod::OAuth2 {
        return None;
    }

    // Try lookup by hostname first.
    if let Some(provider) = provider_db.lookup_by_hostname(account.host()) {
        if provider.oauth.is_some() && is_oauth_provider(&provider.id) {
            return provider.oauth.clone();
        }
    }

    // Fallback: try lookup by email domain from the username.
    let username = account.username();
    if let Some(domain) = username.split('@').nth(1) {
        if let Some(candidate) = provider_db.lookup_by_domain(domain) {
            if candidate.provider.oauth.is_some() && is_oauth_provider(&candidate.provider.id) {
                return candidate.provider.oauth.clone();
            }
        }
    }

    None
}

/// Find an existing account matching by username and incoming-server protocol type (FR-34).
///
/// Returns the index of the first matching account, or `None` if no match is found.
pub fn find_matching_account(
    accounts: &[Account],
    username: &str,
    protocol: Protocol,
) -> Option<usize> {
    accounts
        .iter()
        .position(|a| a.username() == username && a.protocol() == protocol)
}

/// Re-authorize an existing account by updating only credentials and sync-enabled flag (FR-33).
///
/// This function:
/// - Matches an existing account by username and incoming-server protocol type (FR-34)
/// - Updates only the credential (password or OAuth token) and auth method
/// - Re-enables synchronization (FR-33)
/// - Preserves all other account properties: folder structure, sync settings,
///   identities, rules, display name, color, etc. (AC-13)
///
/// Returns the ID of the re-authorized account on success.
pub fn reauthorize_account(
    accounts: &mut [Account],
    params: ReauthParams,
) -> Result<uuid::Uuid, ReauthError> {
    if params.new_credential.trim().is_empty() {
        return Err(ReauthError::EmptyCredential);
    }

    let idx =
        find_matching_account(accounts, &params.username, params.protocol).ok_or_else(|| {
            ReauthError::NoMatchingAccount {
                username: params.username.clone(),
                protocol: params.protocol,
            }
        })?;

    let account = &mut accounts[idx];
    account.update_credentials(params.new_credential, params.new_auth_method);
    account.set_sync_enabled(true);

    Ok(account.id())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::account::{
        Account, AuthMethod, EncryptionMode, NewAccountParams, Protocol, SmtpConfig, SwipeAction,
        SwipeDefaults, SystemFolders,
    };

    fn make_account_with_details(
        username: &str,
        protocol: Protocol,
        credential: &str,
        sync_enabled: bool,
    ) -> Account {
        let mut acct = Account::new(NewAccountParams {
            display_name: "Test Account".into(),
            protocol,
            host: "imap.example.com".into(),
            port: 993,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Plain,
            username: username.into(),
            credential: credential.into(),
            smtp: Some(SmtpConfig {
                host: "smtp.example.com".into(),
                port: 465,
                encryption: EncryptionMode::SslTls,
                auth_method: AuthMethod::Plain,
                username: username.into(),
                credential: credential.into(),
            }),
            pop3_settings: None,
            color: None,
            avatar_path: None,
            category: Some("Work".into()),
            sync_enabled,
            on_demand: false,
            polling_interval_minutes: Some(15),
            unmetered_only: true,
            vpn_only: false,
            schedule_exempt: false,
            system_folders: Some(SystemFolders {
                drafts: Some("Drafts".into()),
                sent: Some("Sent".into()),
                archive: Some("Archive".into()),
                trash: Some("Trash".into()),
                junk: Some("Junk".into()),
            }),
            swipe_defaults: Some(SwipeDefaults {
                swipe_left: SwipeAction::Archive,
                swipe_right: SwipeAction::Delete,
                default_move_to: Some("Archive".into()),
            }),
            notifications_enabled: true,
            security_settings: None,
            fetch_settings: None,
            keep_alive_settings: None,
            oauth_tenant: None,
            shared_mailbox: None,
        })
        .unwrap();
        if !sync_enabled {
            acct.set_sync_enabled(false);
        }
        acct
    }

    #[test]
    fn finds_matching_account_by_username_and_protocol() {
        let accounts = vec![
            make_account_with_details("user@example.com", Protocol::Imap, "old-pass", true),
            make_account_with_details("other@example.com", Protocol::Imap, "pass2", true),
        ];

        let idx = find_matching_account(&accounts, "user@example.com", Protocol::Imap);
        assert_eq!(idx, Some(0));

        let idx = find_matching_account(&accounts, "other@example.com", Protocol::Imap);
        assert_eq!(idx, Some(1));
    }

    #[test]
    fn no_match_when_username_differs() {
        let accounts = vec![make_account_with_details(
            "user@example.com",
            Protocol::Imap,
            "pass",
            true,
        )];

        let idx = find_matching_account(&accounts, "different@example.com", Protocol::Imap);
        assert_eq!(idx, None);
    }

    #[test]
    fn no_match_when_protocol_differs() {
        let accounts = vec![make_account_with_details(
            "user@example.com",
            Protocol::Imap,
            "pass",
            true,
        )];

        let idx = find_matching_account(&accounts, "user@example.com", Protocol::Pop3);
        assert_eq!(idx, None);
    }

    #[test]
    fn reauth_updates_only_credentials_and_sync_flag() {
        let mut accounts = vec![make_account_with_details(
            "user@example.com",
            Protocol::Imap,
            "old-pass",
            false,
        )];

        // Capture pre-reauth state for properties that must be preserved.
        let original_id = accounts[0].id();
        let original_display_name = accounts[0].display_name().to_string();
        let original_host = accounts[0].host().to_string();
        let original_port = accounts[0].port();
        let original_category = accounts[0].category().map(|s| s.to_string());
        let original_system_folders = accounts[0].system_folders().cloned();
        let original_swipe_defaults = accounts[0].swipe_defaults().cloned();
        let original_polling = accounts[0].polling_interval_minutes();
        let original_unmetered = accounts[0].unmetered_only();
        let original_notifications = accounts[0].notifications_enabled();

        let result = reauthorize_account(
            &mut accounts,
            ReauthParams {
                username: "user@example.com".into(),
                protocol: Protocol::Imap,
                new_credential: "new-token".into(),
                new_auth_method: AuthMethod::OAuth2,
            },
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), original_id);

        let acct = &accounts[0];
        // Credential and auth method updated.
        assert_eq!(acct.credential(), "new-token");
        assert_eq!(acct.auth_method(), AuthMethod::OAuth2);
        // Sync re-enabled.
        assert!(acct.sync_enabled());

        // All other properties preserved (AC-13).
        assert_eq!(acct.id(), original_id);
        assert_eq!(acct.display_name(), original_display_name);
        assert_eq!(acct.host(), original_host);
        assert_eq!(acct.port(), original_port);
        assert_eq!(acct.category().map(|s| s.to_string()), original_category);
        assert_eq!(acct.system_folders().cloned(), original_system_folders);
        assert_eq!(acct.swipe_defaults().cloned(), original_swipe_defaults);
        assert_eq!(acct.polling_interval_minutes(), original_polling);
        assert_eq!(acct.unmetered_only(), original_unmetered);
        assert_eq!(acct.notifications_enabled(), original_notifications);
    }

    #[test]
    fn reauth_error_when_no_matching_account() {
        let mut accounts = vec![make_account_with_details(
            "user@example.com",
            Protocol::Imap,
            "pass",
            true,
        )];

        let result = reauthorize_account(
            &mut accounts,
            ReauthParams {
                username: "nonexistent@example.com".into(),
                protocol: Protocol::Imap,
                new_credential: "new-pass".into(),
                new_auth_method: AuthMethod::Plain,
            },
        );

        assert_eq!(
            result,
            Err(ReauthError::NoMatchingAccount {
                username: "nonexistent@example.com".into(),
                protocol: Protocol::Imap,
            })
        );
    }

    #[test]
    fn reauth_error_when_credential_empty() {
        let mut accounts = vec![make_account_with_details(
            "user@example.com",
            Protocol::Imap,
            "pass",
            true,
        )];

        let result = reauthorize_account(
            &mut accounts,
            ReauthParams {
                username: "user@example.com".into(),
                protocol: Protocol::Imap,
                new_credential: "   ".into(),
                new_auth_method: AuthMethod::Plain,
            },
        );

        assert_eq!(result, Err(ReauthError::EmptyCredential));
    }

    #[test]
    fn reauth_preserves_folder_structure() {
        let mut accounts = vec![make_account_with_details(
            "user@example.com",
            Protocol::Imap,
            "old",
            true,
        )];

        let original_folders = accounts[0].system_folders().cloned();

        reauthorize_account(
            &mut accounts,
            ReauthParams {
                username: "user@example.com".into(),
                protocol: Protocol::Imap,
                new_credential: "new".into(),
                new_auth_method: AuthMethod::Plain,
            },
        )
        .unwrap();

        assert_eq!(accounts[0].system_folders().cloned(), original_folders);
    }

    #[test]
    fn reauth_preserves_sync_settings() {
        let mut accounts = vec![make_account_with_details(
            "user@example.com",
            Protocol::Imap,
            "old",
            false,
        )];

        // Sync settings like polling interval, unmetered_only, on_demand should be preserved.
        let original_polling = accounts[0].polling_interval_minutes();
        let original_unmetered = accounts[0].unmetered_only();
        let original_on_demand = accounts[0].on_demand();

        reauthorize_account(
            &mut accounts,
            ReauthParams {
                username: "user@example.com".into(),
                protocol: Protocol::Imap,
                new_credential: "new".into(),
                new_auth_method: AuthMethod::Plain,
            },
        )
        .unwrap();

        assert_eq!(accounts[0].polling_interval_minutes(), original_polling);
        assert_eq!(accounts[0].unmetered_only(), original_unmetered);
        assert_eq!(accounts[0].on_demand(), original_on_demand);
        // But sync_enabled is re-enabled.
        assert!(accounts[0].sync_enabled());
    }

    #[test]
    fn reauth_preserves_identity_settings_unchanged() {
        // Identity settings are stored separately (SendingIdentity), so the account's
        // properties that relate to identity (display_name, notifications, etc.) must remain.
        let mut accounts = vec![make_account_with_details(
            "user@example.com",
            Protocol::Imap,
            "old",
            true,
        )];

        let original_display = accounts[0].display_name().to_string();
        let original_notifications = accounts[0].notifications_enabled();

        reauthorize_account(
            &mut accounts,
            ReauthParams {
                username: "user@example.com".into(),
                protocol: Protocol::Imap,
                new_credential: "refreshed-token".into(),
                new_auth_method: AuthMethod::OAuth2,
            },
        )
        .unwrap();

        assert_eq!(accounts[0].display_name(), original_display);
        assert_eq!(accounts[0].notifications_enabled(), original_notifications);
    }

    // -- find_oauth_config_for_reauth tests (FR-25) --

    fn make_oauth_account(username: &str, host: &str) -> Account {
        Account::new(NewAccountParams {
            display_name: "OAuth Account".into(),
            protocol: Protocol::Imap,
            host: host.into(),
            port: 993,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::OAuth2,
            username: username.into(),
            credential: "access-token".into(),
            smtp: None,
            pop3_settings: None,
            color: None,
            avatar_path: None,
            category: None,
            sync_enabled: true,
            on_demand: false,
            polling_interval_minutes: None,
            unmetered_only: false,
            vpn_only: false,
            schedule_exempt: false,
            system_folders: None,
            swipe_defaults: None,
            notifications_enabled: true,
            security_settings: None,
            fetch_settings: None,
            keep_alive_settings: None,
            oauth_tenant: None,
            shared_mailbox: None,
        })
        .unwrap()
    }

    #[test]
    fn find_oauth_config_for_gmail_account() {
        let db = ProviderDatabase::bundled();
        let acct = make_oauth_account("user@gmail.com", "imap.gmail.com");
        let config = find_oauth_config_for_reauth(&acct, &db);
        assert!(config.is_some());
        let config = config.unwrap();
        assert!(!config.auth_url.is_empty());
        assert!(!config.token_url.is_empty());
    }

    #[test]
    fn find_oauth_config_for_outlook_account() {
        let db = ProviderDatabase::bundled();
        let acct = make_oauth_account("user@outlook.com", "outlook.office365.com");
        let config = find_oauth_config_for_reauth(&acct, &db);
        assert!(config.is_some());
    }

    #[test]
    fn find_oauth_config_returns_none_for_plain_auth() {
        let db = ProviderDatabase::bundled();
        let acct = Account::new(NewAccountParams {
            display_name: "Plain Account".into(),
            protocol: Protocol::Imap,
            host: "imap.gmail.com".into(),
            port: 993,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Plain,
            username: "user@gmail.com".into(),
            credential: "password".into(),
            smtp: None,
            pop3_settings: None,
            color: None,
            avatar_path: None,
            category: None,
            sync_enabled: true,
            on_demand: false,
            polling_interval_minutes: None,
            unmetered_only: false,
            vpn_only: false,
            schedule_exempt: false,
            system_folders: None,
            swipe_defaults: None,
            notifications_enabled: true,
            security_settings: None,
            fetch_settings: None,
            keep_alive_settings: None,
            oauth_tenant: None,
            shared_mailbox: None,
        })
        .unwrap();
        let config = find_oauth_config_for_reauth(&acct, &db);
        assert!(config.is_none());
    }

    #[test]
    fn find_oauth_config_returns_none_for_unknown_provider() {
        let db = ProviderDatabase::bundled();
        let acct = make_oauth_account("user@custom.example.com", "imap.custom.example.com");
        let config = find_oauth_config_for_reauth(&acct, &db);
        assert!(config.is_none());
    }

    #[test]
    fn find_oauth_config_uses_domain_fallback() {
        let db = ProviderDatabase::bundled();
        // Use a non-matching hostname but a valid email domain.
        let acct = make_oauth_account("user@gmail.com", "custom-imap-server.example.com");
        let config = find_oauth_config_for_reauth(&acct, &db);
        assert!(config.is_some());
    }
}
