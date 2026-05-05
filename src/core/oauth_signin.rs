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
];

/// Represents the authentication options available after provider detection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthOptions {
    /// Whether OAuth sign-in is available for this provider.
    pub oauth_available: bool,
    /// The OAuth configuration, if available.
    pub oauth_config: Option<OAuthConfig>,
    /// Whether password-based authentication is available.
    pub password_available: bool,
    /// The provider display name (for UI labeling).
    pub provider_name: String,
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
/// password-based auth is also offered.
pub fn determine_auth_options(provider: &Provider) -> AuthOptions {
    let oauth_available = provider.oauth.is_some()
        && provider.enabled
        && OAUTH_PROVIDER_IDS.contains(&provider.id.as_str());

    AuthOptions {
        oauth_available,
        oauth_config: if oauth_available {
            provider.oauth.clone()
        } else {
            None
        },
        password_available: true,
        provider_name: provider.display_name.clone(),
    }
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
                scopes: vec!["mail".to_string()],
                client_id: None,
            }),
            display_order: 1,
            enabled: true,
        }
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
            scopes: vec!["mail".to_string()],
            client_id: None,
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
}
