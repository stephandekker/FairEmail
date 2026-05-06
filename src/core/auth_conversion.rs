use crate::core::account::{Account, AuthMethod};
use crate::core::oauth_signin::is_oauth_provider;
use crate::core::provider::{OAuthConfig, ProviderDatabase};

/// Find the OAuth configuration for an account that currently uses password
/// authentication, so that it can be converted to OAuth (FR-30, US-17).
///
/// Returns `None` if the account already uses OAuth2 or if the provider
/// does not support OAuth.
pub fn find_oauth_config_for_conversion(
    account: &Account,
    provider_db: &ProviderDatabase,
) -> Option<OAuthConfig> {
    if account.auth_method() == AuthMethod::OAuth2 {
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

/// Check whether an OAuth account can be converted to password authentication (FR-30).
///
/// Returns `true` when the account currently uses OAuth2 — the only
/// prerequisite for converting to password auth.
pub fn can_convert_to_password(account: &Account) -> bool {
    account.auth_method() == AuthMethod::OAuth2
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::account::{
        Account, AuthMethod, EncryptionMode, NewAccountParams, Protocol, SmtpConfig,
    };

    fn make_account(auth_method: AuthMethod, username: &str, host: &str) -> Account {
        Account::new(NewAccountParams {
            display_name: "Test".into(),
            protocol: Protocol::Imap,
            host: host.into(),
            port: 993,
            encryption: EncryptionMode::SslTls,
            auth_method,
            username: username.into(),
            credential: "cred".into(),
            smtp: Some(SmtpConfig {
                host: "smtp.example.com".into(),
                port: 465,
                encryption: EncryptionMode::SslTls,
                auth_method,
                username: username.into(),
                credential: "cred".into(),
            }),
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
    fn find_oauth_config_for_password_gmail_account() {
        let db = ProviderDatabase::bundled();
        let acct = make_account(AuthMethod::Plain, "user@gmail.com", "imap.gmail.com");
        let config = find_oauth_config_for_conversion(&acct, &db);
        assert!(config.is_some());
        let config = config.unwrap();
        assert!(!config.auth_url.is_empty());
        assert!(!config.token_url.is_empty());
    }

    #[test]
    fn find_oauth_config_returns_none_for_oauth_account() {
        let db = ProviderDatabase::bundled();
        let acct = make_account(AuthMethod::OAuth2, "user@gmail.com", "imap.gmail.com");
        let config = find_oauth_config_for_conversion(&acct, &db);
        assert!(config.is_none());
    }

    #[test]
    fn find_oauth_config_returns_none_for_unknown_provider() {
        let db = ProviderDatabase::bundled();
        let acct = make_account(
            AuthMethod::Plain,
            "user@custom.example.com",
            "imap.custom.example.com",
        );
        let config = find_oauth_config_for_conversion(&acct, &db);
        assert!(config.is_none());
    }

    #[test]
    fn find_oauth_config_uses_domain_fallback() {
        let db = ProviderDatabase::bundled();
        let acct = make_account(
            AuthMethod::Login,
            "user@gmail.com",
            "custom-imap.example.com",
        );
        let config = find_oauth_config_for_conversion(&acct, &db);
        assert!(config.is_some());
    }

    #[test]
    fn can_convert_to_password_returns_true_for_oauth() {
        let acct = make_account(AuthMethod::OAuth2, "user@gmail.com", "imap.gmail.com");
        assert!(can_convert_to_password(&acct));
    }

    #[test]
    fn can_convert_to_password_returns_false_for_plain() {
        let acct = make_account(AuthMethod::Plain, "user@gmail.com", "imap.gmail.com");
        assert!(!can_convert_to_password(&acct));
    }

    #[test]
    fn can_convert_to_password_returns_false_for_login() {
        let acct = make_account(AuthMethod::Login, "user@example.com", "imap.example.com");
        assert!(!can_convert_to_password(&acct));
    }

    #[test]
    fn conversion_preserves_account_properties() {
        // Verify that the conversion functions only inspect auth_method —
        // they never mutate the account, so all properties remain intact.
        let acct = make_account(AuthMethod::Plain, "user@gmail.com", "imap.gmail.com");
        let original_id = acct.id();
        let original_host = acct.host().to_string();
        let original_username = acct.username().to_string();

        let db = ProviderDatabase::bundled();
        let _ = find_oauth_config_for_conversion(&acct, &db);

        assert_eq!(acct.id(), original_id);
        assert_eq!(acct.host(), original_host);
        assert_eq!(acct.username(), original_username);
        assert_eq!(acct.auth_method(), AuthMethod::Plain);
    }
}
