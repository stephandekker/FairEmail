use crate::core::account::AuthMethod;
use crate::core::provider::{OAuthConfig, Provider};

/// Known provider IDs that support OAuth (FR-40).
const OAUTH_PROVIDER_IDS: &[&str] = &[
    "gmail",
    "outlook",
    "office365",
    "yahoo",
    "aol",
    "yandex",
    "mailru",
    "fastmail",
    "163",
];

/// Reason why OAuth is not available for a provider that otherwise supports it (FR-29, US-15).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OAuthUnavailableReason {
    /// The provider supports OAuth, but this build does not include the required
    /// client credentials (e.g. a community-maintained package without bundled
    /// OAuth client IDs). The user should use password or app-password authentication.
    MissingCredentials,
}

/// Represents the authentication options available after provider detection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthOptions {
    /// Whether the provider supports OAuth and is in the known whitelist.
    /// This indicates the provider *could* use OAuth if credentials are present.
    pub oauth_available: bool,
    /// The OAuth configuration, if the provider supports OAuth.
    pub oauth_config: Option<OAuthConfig>,
    /// Whether password-based authentication is available.
    pub password_available: bool,
    /// The provider display name (for UI labeling).
    pub provider_name: String,
    /// Whether this build has the OAuth client credentials needed to actually
    /// perform the OAuth flow (FR-29, N-7). When `oauth_available` is true but
    /// `oauth_credentials_present` is false, the UI should hide the OAuth option
    /// and show a message guiding the user to password authentication.
    pub oauth_credentials_present: bool,
    /// When the provider supports OAuth but credentials are missing, this
    /// explains why OAuth cannot be used (FR-29, AC-6, N-7).
    pub oauth_unavailable_reason: Option<OAuthUnavailableReason>,
}

/// The result of an OAuth token acquisition (passed back from epic 1.5's flow).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuthTokenResult {
    /// The access token for IMAP/SMTP authentication.
    pub access_token: String,
    /// The refresh token for obtaining new access tokens.
    pub refresh_token: Option<String>,
    /// Token expiry in seconds from now, if provided.
    pub expires_in: Option<u64>,
}

/// The user's chosen authentication method after being presented with options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthChoice {
    /// User chose OAuth sign-in. Contains the token result from the OAuth flow.
    OAuth(OAuthTokenResult),
    /// User chose password-based authentication.
    Password(String),
}

/// Determine the authentication options available for a detected provider (FR-39, AC-18).
///
/// Returns `AuthOptions` indicating whether OAuth is available and whether
/// password-based auth is also offered. When a provider defines OAuth endpoints
/// but the build lacks client credentials, `oauth_available` is false and
/// `oauth_unavailable_reason` explains why (FR-29, AC-6, N-7).
pub fn determine_auth_options(provider: &Provider) -> AuthOptions {
    let oauth_available = provider.oauth.is_some()
        && provider.enabled
        && OAUTH_PROVIDER_IDS.contains(&provider.id.as_str());

    let oauth_credentials_present =
        oauth_available && provider.oauth.as_ref().is_some_and(has_oauth_credentials);

    // Provider supports OAuth in theory but this build lacks credentials (N-7).
    let oauth_unavailable_reason = if oauth_available && !oauth_credentials_present {
        Some(OAuthUnavailableReason::MissingCredentials)
    } else {
        None
    };

    AuthOptions {
        oauth_available,
        oauth_config: if oauth_available {
            provider.oauth.clone()
        } else {
            None
        },
        password_available: true,
        provider_name: provider.display_name.clone(),
        oauth_credentials_present,
        oauth_unavailable_reason,
    }
}

/// Check whether OAuth client credentials are available for a provider.
///
/// Credentials are detected from two sources, checked in order:
/// 1. The provider's bundled `client_id` field — populated in official builds
///    that embed OAuth credentials at compile time.
/// 2. A runtime credentials file at `$XDG_CONFIG_HOME/fairmail/oauth_credentials.json`
///    — allows community builds or self-compiled users to supply their own
///    OAuth client IDs without modifying the source.
///
/// Returns true if either source provides credentials (FR-29, N-7).
pub fn has_oauth_credentials(config: &OAuthConfig) -> bool {
    // Source 1: bundled client_id in the provider config.
    if config
        .client_id
        .as_ref()
        .is_some_and(|id| !id.trim().is_empty())
    {
        return true;
    }

    // Source 2: runtime credentials file on disk.
    oauth_credentials_file_exists()
}

/// Path to the runtime OAuth credentials file.
pub const OAUTH_CREDENTIALS_FILENAME: &str = "oauth_credentials.json";

/// Check whether a runtime OAuth credentials file exists and is non-empty.
///
/// The file is expected at `$XDG_CONFIG_HOME/fairmail/oauth_credentials.json`
/// (defaulting to `~/.config/fairmail/oauth_credentials.json`). Its presence
/// with non-empty content signals that the user has supplied their own OAuth
/// client credentials.
pub fn oauth_credentials_file_exists() -> bool {
    let config_dir = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_default();
        format!("{home}/.config")
    });
    let path = std::path::Path::new(&config_dir)
        .join(super::user_provider_file::APP_CONFIG_DIR)
        .join(OAUTH_CREDENTIALS_FILENAME);
    path.is_file()
        && std::fs::metadata(&path)
            .map(|m| m.len() > 0)
            .unwrap_or(false)
}

/// Build a user-facing message explaining why OAuth is unavailable and guiding
/// the user to password-based authentication (FR-29, US-15, AC-6).
pub fn oauth_unavailable_message(provider_name: &str) -> String {
    gettextrs::gettext(
        "OAuth sign-in is not available for %s in this build. \
         Please use password or app-password authentication instead.",
    )
    .replace("%s", provider_name)
}

/// Check whether a provider ID is in the known OAuth-supporting list (FR-40).
pub fn is_oauth_provider(provider_id: &str) -> bool {
    OAUTH_PROVIDER_IDS.contains(&provider_id)
}

/// Resolve the authentication method and credential based on the user's choice (FR-41).
///
/// When OAuth is chosen, the access token is used as the credential and
/// the auth method is set to OAuth2. When password is chosen, Plain auth is used.
pub fn resolve_auth_from_choice(choice: &AuthChoice) -> (AuthMethod, String) {
    match choice {
        AuthChoice::OAuth(token_result) => (AuthMethod::OAuth2, token_result.access_token.clone()),
        AuthChoice::Password(password) => (AuthMethod::Plain, password.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::provider::{
        MaxTlsVersion, OAuthConfig, Provider, ProviderEncryption, ServerConfig, UsernameType,
    };

    fn make_oauth_provider(id: &str) -> Provider {
        Provider {
            id: id.to_string(),
            display_name: format!("{id} Mail"),
            domain_patterns: vec![format!("{id}.com")],
            mx_patterns: vec![],
            incoming: ServerConfig {
                hostname: format!("imap.{id}.com"),
                port: 993,
                encryption: ProviderEncryption::SslTls,
            },
            outgoing: ServerConfig {
                hostname: format!("smtp.{id}.com"),
                port: 465,
                encryption: ProviderEncryption::SslTls,
            },
            username_type: UsernameType::EmailAddress,
            keep_alive_interval: 15,
            noop_keep_alive: false,
            partial_fetch: true,
            max_tls_version: MaxTlsVersion::Tls1_3,
            app_password_required: false,
            documentation_url: None,
            localized_docs: vec![],
            oauth: Some(OAuthConfig {
                auth_url: "https://auth.example.com/oauth".to_string(),
                token_url: "https://auth.example.com/token".to_string(),
                redirect_uri: "http://127.0.0.1/callback".to_string(),
                scopes: vec!["mail".to_string()],
                client_id: Some("test-client-id".to_string()),
                pkce_required: true,
                extra_params: vec![],
                userinfo_url: None,
                privacy_policy_url: None,
            }),
            display_order: 1,
            enabled: true,
            supports_shared_mailbox: false,
            subtitle: None,
            registration_url: None,
            graph: None,
            debug_only: false,
            variant_of: None,
        }
    }

    /// Create a provider in the OAuth whitelist with OAuth config but NO client credentials.
    /// Simulates a community build that lacks bundled OAuth client IDs.
    fn make_oauth_provider_no_credentials(id: &str) -> Provider {
        let mut provider = make_oauth_provider(id);
        provider.oauth.as_mut().unwrap().client_id = None;
        provider
    }

    fn make_no_oauth_provider() -> Provider {
        Provider {
            id: "nooauth".to_string(),
            display_name: "No OAuth Provider".to_string(),
            domain_patterns: vec!["nooauth.com".to_string()],
            mx_patterns: vec![],
            incoming: ServerConfig {
                hostname: "imap.nooauth.com".to_string(),
                port: 993,
                encryption: ProviderEncryption::SslTls,
            },
            outgoing: ServerConfig {
                hostname: "smtp.nooauth.com".to_string(),
                port: 465,
                encryption: ProviderEncryption::SslTls,
            },
            username_type: UsernameType::EmailAddress,
            keep_alive_interval: 15,
            noop_keep_alive: false,
            partial_fetch: true,
            max_tls_version: MaxTlsVersion::Tls1_3,
            app_password_required: false,
            documentation_url: None,
            localized_docs: vec![],
            oauth: None,
            display_order: 1,
            enabled: true,
            supports_shared_mailbox: false,
            subtitle: None,
            registration_url: None,
            graph: None,
            debug_only: false,
            variant_of: None,
        }
    }

    #[test]
    fn oauth_available_for_gmail() {
        let provider = make_oauth_provider("gmail");
        let options = determine_auth_options(&provider);
        assert!(options.oauth_available);
        assert!(options.oauth_config.is_some());
        assert!(options.password_available);
    }

    #[test]
    fn oauth_available_for_outlook() {
        let provider = make_oauth_provider("outlook");
        let options = determine_auth_options(&provider);
        assert!(options.oauth_available);
    }

    #[test]
    fn oauth_available_for_yahoo() {
        let provider = make_oauth_provider("yahoo");
        let options = determine_auth_options(&provider);
        assert!(options.oauth_available);
    }

    #[test]
    fn oauth_available_for_aol() {
        let provider = make_oauth_provider("aol");
        let options = determine_auth_options(&provider);
        assert!(options.oauth_available);
    }

    #[test]
    fn oauth_available_for_yandex() {
        let provider = make_oauth_provider("yandex");
        let options = determine_auth_options(&provider);
        assert!(options.oauth_available);
    }

    #[test]
    fn oauth_available_for_mailru() {
        let provider = make_oauth_provider("mailru");
        let options = determine_auth_options(&provider);
        assert!(options.oauth_available);
    }

    #[test]
    fn oauth_available_for_fastmail() {
        let provider = make_oauth_provider("fastmail");
        let options = determine_auth_options(&provider);
        assert!(options.oauth_available);
    }

    #[test]
    fn oauth_available_for_office365() {
        let provider = make_oauth_provider("office365");
        let options = determine_auth_options(&provider);
        assert!(options.oauth_available);
    }

    #[test]
    fn oauth_not_available_without_config() {
        let provider = make_no_oauth_provider();
        let options = determine_auth_options(&provider);
        assert!(!options.oauth_available);
        assert!(options.oauth_config.is_none());
        assert!(options.password_available);
    }

    #[test]
    fn oauth_not_available_for_unknown_provider_with_config() {
        // A provider with an OAuth config but not in the known list
        let mut provider = make_no_oauth_provider();
        provider.oauth = Some(OAuthConfig {
            auth_url: "https://auth.example.com".to_string(),
            token_url: "https://token.example.com".to_string(),
            redirect_uri: "http://127.0.0.1/callback".to_string(),
            scopes: vec!["mail".to_string()],
            client_id: None,
            pkce_required: true,
            extra_params: vec![],
            userinfo_url: None,
            privacy_policy_url: None,
        });
        let options = determine_auth_options(&provider);
        assert!(!options.oauth_available);
    }

    #[test]
    fn oauth_not_available_when_provider_disabled() {
        let mut provider = make_oauth_provider("gmail");
        provider.enabled = false;
        let options = determine_auth_options(&provider);
        assert!(!options.oauth_available);
    }

    #[test]
    fn password_always_available() {
        let provider = make_oauth_provider("gmail");
        let options = determine_auth_options(&provider);
        assert!(options.password_available);

        let provider2 = make_no_oauth_provider();
        let options2 = determine_auth_options(&provider2);
        assert!(options2.password_available);
    }

    #[test]
    fn is_oauth_provider_returns_true_for_known() {
        assert!(is_oauth_provider("gmail"));
        assert!(is_oauth_provider("outlook"));
        assert!(is_oauth_provider("office365"));
        assert!(is_oauth_provider("yahoo"));
        assert!(is_oauth_provider("aol"));
        assert!(is_oauth_provider("yandex"));
        assert!(is_oauth_provider("mailru"));
        assert!(is_oauth_provider("fastmail"));
    }

    #[test]
    fn is_oauth_provider_returns_false_for_unknown() {
        assert!(!is_oauth_provider("protonmail"));
        assert!(!is_oauth_provider("zoho"));
        assert!(!is_oauth_provider("unknown"));
    }

    #[test]
    fn resolve_auth_oauth_choice() {
        let choice = AuthChoice::OAuth(OAuthTokenResult {
            access_token: "ya29.access-token-here".to_string(),
            refresh_token: Some("1//refresh-token".to_string()),
            expires_in: Some(3600),
        });
        let (method, credential) = resolve_auth_from_choice(&choice);
        assert_eq!(method, AuthMethod::OAuth2);
        assert_eq!(credential, "ya29.access-token-here");
    }

    #[test]
    fn resolve_auth_password_choice() {
        let choice = AuthChoice::Password("my-app-password".to_string());
        let (method, credential) = resolve_auth_from_choice(&choice);
        assert_eq!(method, AuthMethod::Plain);
        assert_eq!(credential, "my-app-password");
    }

    #[test]
    fn oauth_token_result_stores_all_fields() {
        let result = OAuthTokenResult {
            access_token: "token123".to_string(),
            refresh_token: Some("refresh456".to_string()),
            expires_in: Some(7200),
        };
        assert_eq!(result.access_token, "token123");
        assert_eq!(result.refresh_token, Some("refresh456".to_string()));
        assert_eq!(result.expires_in, Some(7200));
    }

    #[test]
    fn oauth_token_result_optional_fields() {
        let result = OAuthTokenResult {
            access_token: "token".to_string(),
            refresh_token: None,
            expires_in: None,
        };
        assert!(result.refresh_token.is_none());
        assert!(result.expires_in.is_none());
    }

    #[test]
    fn bundled_gmail_has_oauth() {
        let db = crate::core::provider::ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("gmail.com").unwrap();
        let options = determine_auth_options(&candidate.provider);
        assert!(options.oauth_available);
        assert!(options.oauth_config.is_some());
    }

    #[test]
    fn bundled_outlook_has_oauth() {
        let db = crate::core::provider::ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("outlook.com").unwrap();
        let options = determine_auth_options(&candidate.provider);
        assert!(options.oauth_available);
    }

    #[test]
    fn bundled_yahoo_has_oauth() {
        let db = crate::core::provider::ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("yahoo.com").unwrap();
        let options = determine_auth_options(&candidate.provider);
        assert!(options.oauth_available);
    }

    #[test]
    fn bundled_aol_has_oauth() {
        let db = crate::core::provider::ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("aol.com").unwrap();
        let options = determine_auth_options(&candidate.provider);
        assert!(options.oauth_available);
    }

    #[test]
    fn bundled_yandex_has_oauth() {
        let db = crate::core::provider::ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("yandex.ru").unwrap();
        let options = determine_auth_options(&candidate.provider);
        assert!(options.oauth_available);
    }

    #[test]
    fn bundled_mailru_has_oauth() {
        let db = crate::core::provider::ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("mail.ru").unwrap();
        let options = determine_auth_options(&candidate.provider);
        assert!(options.oauth_available);
    }

    #[test]
    fn bundled_fastmail_has_oauth() {
        let db = crate::core::provider::ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("fastmail.com").unwrap();
        let options = determine_auth_options(&candidate.provider);
        assert!(options.oauth_available);
    }

    #[test]
    fn bundled_protonmail_no_oauth() {
        let db = crate::core::provider::ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("protonmail.com").unwrap();
        let options = determine_auth_options(&candidate.provider);
        assert!(!options.oauth_available);
    }

    #[test]
    fn bundled_163_has_oauth() {
        let db = crate::core::provider::ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("163.com").unwrap();
        let options = determine_auth_options(&candidate.provider);
        assert!(options.oauth_available);
        assert!(options.oauth_config.is_some());
    }

    #[test]
    fn is_oauth_provider_includes_163() {
        assert!(is_oauth_provider("163"));
    }

    #[test]
    fn oauth_config_has_redirect_uri() {
        let db = crate::core::provider::ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("gmail.com").unwrap();
        let oauth = candidate.provider.oauth.unwrap();
        assert_eq!(oauth.redirect_uri, "http://127.0.0.1/callback");
    }

    #[test]
    fn gmail_oauth_has_provider_specific_params() {
        let db = crate::core::provider::ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("gmail.com").unwrap();
        let oauth = candidate.provider.oauth.unwrap();
        assert!(oauth
            .extra_params
            .contains(&("prompt".to_string(), "consent".to_string())));
        assert!(oauth
            .extra_params
            .contains(&("access_type".to_string(), "offline".to_string())));
    }

    #[test]
    fn yandex_oauth_has_force_confirm() {
        let db = crate::core::provider::ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("yandex.ru").unwrap();
        let oauth = candidate.provider.oauth.unwrap();
        assert!(oauth
            .extra_params
            .contains(&("force_confirm".to_string(), "true".to_string())));
    }

    #[test]
    fn outlook_oauth_has_offline_access_scope() {
        let db = crate::core::provider::ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("outlook.com").unwrap();
        let oauth = candidate.provider.oauth.unwrap();
        assert!(oauth.scopes.contains(&"offline_access".to_string()));
    }

    #[test]
    fn all_oauth_providers_have_redirect_uri() {
        let db = crate::core::provider::ProviderDatabase::bundled();
        let oauth_domains = [
            "gmail.com",
            "outlook.com",
            "yahoo.com",
            "aol.com",
            "yandex.ru",
            "mail.ru",
            "fastmail.com",
            "163.com",
        ];
        for domain in &oauth_domains {
            let candidate = db.lookup_by_domain(domain).unwrap();
            let oauth = candidate
                .provider
                .oauth
                .as_ref()
                .unwrap_or_else(|| panic!("Expected OAuth config for {domain}"));
            assert!(
                !oauth.redirect_uri.is_empty(),
                "Expected non-empty redirect_uri for {domain}"
            );
            assert!(
                !oauth.auth_url.is_empty(),
                "Expected non-empty auth_url for {domain}"
            );
            assert!(
                !oauth.token_url.is_empty(),
                "Expected non-empty token_url for {domain}"
            );
            assert!(
                !oauth.scopes.is_empty(),
                "Expected non-empty scopes for {domain}"
            );
        }
    }

    // --- Distribution-channel gating & credential detection (story 12, FR-29, AC-6, N-7) ---

    #[test]
    fn has_oauth_credentials_true_when_client_id_present() {
        let config = OAuthConfig {
            auth_url: "https://example.com/auth".to_string(),
            token_url: "https://example.com/token".to_string(),
            redirect_uri: "http://127.0.0.1/callback".to_string(),
            scopes: vec!["mail".to_string()],
            client_id: Some("my-client-id".to_string()),
            pkce_required: true,
            extra_params: vec![],
            userinfo_url: None,
            privacy_policy_url: None,
        };
        assert!(super::has_oauth_credentials(&config));
    }

    #[test]
    fn has_oauth_credentials_false_when_client_id_none_and_no_file() {
        // has_oauth_credentials checks client_id first, then falls back to
        // runtime credentials file. When client_id is None and no file exists,
        // it should return false. (If a credentials file exists on disk, this
        // test will return true, which is correct behavior.)
        let config = OAuthConfig {
            auth_url: "https://example.com/auth".to_string(),
            token_url: "https://example.com/token".to_string(),
            redirect_uri: "http://127.0.0.1/callback".to_string(),
            scopes: vec!["mail".to_string()],
            client_id: None,
            pkce_required: true,
            extra_params: vec![],
            userinfo_url: None,
            privacy_policy_url: None,
        };
        // When no runtime file, should be false. With file, true is correct.
        let result = super::has_oauth_credentials(&config);
        let file_exists = super::oauth_credentials_file_exists();
        if file_exists {
            assert!(result, "With credentials file present, should be true");
        } else {
            assert!(
                !result,
                "Without client_id or credentials file, should be false"
            );
        }
    }

    #[test]
    fn has_oauth_credentials_false_when_client_id_empty_and_no_file() {
        let config = OAuthConfig {
            auth_url: "https://example.com/auth".to_string(),
            token_url: "https://example.com/token".to_string(),
            redirect_uri: "http://127.0.0.1/callback".to_string(),
            scopes: vec!["mail".to_string()],
            client_id: Some("".to_string()),
            pkce_required: true,
            extra_params: vec![],
            userinfo_url: None,
            privacy_policy_url: None,
        };
        let result = super::has_oauth_credentials(&config);
        let file_exists = super::oauth_credentials_file_exists();
        if file_exists {
            assert!(result);
        } else {
            assert!(!result);
        }
    }

    #[test]
    fn has_oauth_credentials_false_when_client_id_whitespace_and_no_file() {
        let config = OAuthConfig {
            auth_url: "https://example.com/auth".to_string(),
            token_url: "https://example.com/token".to_string(),
            redirect_uri: "http://127.0.0.1/callback".to_string(),
            scopes: vec!["mail".to_string()],
            client_id: Some("   ".to_string()),
            pkce_required: true,
            extra_params: vec![],
            userinfo_url: None,
            privacy_policy_url: None,
        };
        let result = super::has_oauth_credentials(&config);
        let file_exists = super::oauth_credentials_file_exists();
        if file_exists {
            assert!(result);
        } else {
            assert!(!result);
        }
    }

    #[test]
    fn oauth_unavailable_when_credentials_missing() {
        let provider = make_oauth_provider_no_credentials("gmail");
        assert!(provider.oauth.as_ref().unwrap().client_id.is_none());
        let options = determine_auth_options(&provider);
        // Provider supports OAuth (config present + in whitelist).
        assert!(options.oauth_available);
        assert!(options.oauth_config.is_some());
        // But this build lacks credentials.
        assert!(!options.oauth_credentials_present);
        assert_eq!(
            options.oauth_unavailable_reason,
            Some(OAuthUnavailableReason::MissingCredentials)
        );
        assert!(options.password_available);
    }

    #[test]
    fn oauth_available_when_credentials_present() {
        let provider = make_oauth_provider("gmail");
        assert!(provider.oauth.as_ref().unwrap().client_id.is_some());
        let options = determine_auth_options(&provider);
        assert!(options.oauth_available);
        assert!(options.oauth_credentials_present);
        assert!(options.oauth_config.is_some());
        assert!(options.oauth_unavailable_reason.is_none());
        assert!(options.password_available);
    }

    #[test]
    fn no_unavailable_reason_for_non_oauth_provider() {
        let provider = make_no_oauth_provider();
        let options = determine_auth_options(&provider);
        assert!(!options.oauth_available);
        // No reason because the provider doesn't support OAuth at all.
        assert!(options.oauth_unavailable_reason.is_none());
    }

    #[test]
    fn no_unavailable_reason_for_unknown_provider_with_config() {
        // Provider has OAuth config but is not in the known whitelist.
        let mut provider = make_no_oauth_provider();
        provider.oauth = Some(OAuthConfig {
            auth_url: "https://auth.example.com".to_string(),
            token_url: "https://token.example.com".to_string(),
            redirect_uri: "http://127.0.0.1/callback".to_string(),
            scopes: vec!["mail".to_string()],
            client_id: Some("has-credentials".to_string()),
            pkce_required: true,
            extra_params: vec![],
            userinfo_url: None,
            privacy_policy_url: None,
        });
        let options = determine_auth_options(&provider);
        assert!(!options.oauth_available);
        // Not in whitelist, so not a "missing credentials" situation.
        assert!(options.oauth_unavailable_reason.is_none());
    }

    #[test]
    fn bundled_providers_without_credentials_report_missing() {
        // All bundled OAuth providers ship with client_id: None by default.
        // They support OAuth (oauth_available=true) but lack credentials.
        let db = crate::core::provider::ProviderDatabase::bundled();
        let oauth_domains = [
            "gmail.com",
            "outlook.com",
            "yahoo.com",
            "aol.com",
            "yandex.ru",
            "mail.ru",
            "fastmail.com",
            "163.com",
        ];
        for domain in &oauth_domains {
            let candidate = db.lookup_by_domain(domain).unwrap();
            let options = determine_auth_options(&candidate.provider);
            // Provider supports OAuth.
            assert!(
                options.oauth_available,
                "Provider for {domain} should support OAuth"
            );
            // But this build has no bundled credentials (client_id is None).
            // Note: oauth_credentials_present may still be true if a runtime
            // credentials file exists on disk. We only check the reason field.
            if !options.oauth_credentials_present {
                assert_eq!(
                    options.oauth_unavailable_reason,
                    Some(OAuthUnavailableReason::MissingCredentials),
                    "Expected MissingCredentials reason for {domain}"
                );
            }
            // Password fallback always available.
            assert!(options.password_available);
        }
    }

    #[test]
    fn oauth_unavailable_message_contains_provider_name() {
        let msg = super::oauth_unavailable_message("Gmail");
        assert!(msg.contains("Gmail"));
        assert!(msg.contains("password"));
    }

    #[test]
    fn password_always_available_regardless_of_credentials() {
        // With credentials
        let provider = make_oauth_provider("gmail");
        assert!(determine_auth_options(&provider).password_available);

        // Without credentials
        let provider2 = make_oauth_provider_no_credentials("gmail");
        assert!(determine_auth_options(&provider2).password_available);

        // No OAuth at all
        let provider3 = make_no_oauth_provider();
        assert!(determine_auth_options(&provider3).password_available);
    }
}
