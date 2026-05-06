//! User-supplied provider file support (FR-16, NFR-5).
//!
//! Allows loading an additional provider file that augments or overrides
//! the bundled provider database. User-supplied entries participate in
//! the same domain-matching and score-based ranking as bundled entries.

use super::provider::{Provider, ProviderDatabase};
use thiserror::Error;

/// Errors that can occur when loading a user-supplied provider file.
#[derive(Debug, Error)]
pub enum UserProviderFileError {
    #[error("failed to read user provider file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse user provider file: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("validation error: {0}")]
    Validation(String),
}

/// Validation error detail for a single provider entry.
#[derive(Debug, Clone)]
pub struct ProviderValidationError {
    pub provider_id: String,
    pub message: String,
}

impl std::fmt::Display for ProviderValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "provider '{}': {}", self.provider_id, self.message)
    }
}

/// Validate a list of parsed providers.
///
/// Checks that each provider has a non-empty `id` and `display_name`, and if
/// OAuth is configured, that the required fields (`auth_url`, `token_url`, and
/// at least one scope) are present and non-empty.
///
/// Returns `Ok(())` if all providers pass validation, or a list of errors.
pub fn validate_provider_configs(
    providers: &[Provider],
) -> Result<(), Vec<ProviderValidationError>> {
    let mut errors = Vec::new();

    for provider in providers {
        if provider.id.trim().is_empty() {
            errors.push(ProviderValidationError {
                provider_id: "(empty)".to_string(),
                message: "id must not be empty".to_string(),
            });
            continue;
        }
        if provider.display_name.trim().is_empty() {
            errors.push(ProviderValidationError {
                provider_id: provider.id.clone(),
                message: "display_name must not be empty".to_string(),
            });
        }
        if let Some(ref oauth) = provider.oauth {
            if oauth.auth_url.trim().is_empty() {
                errors.push(ProviderValidationError {
                    provider_id: provider.id.clone(),
                    message: "oauth.auth_url must not be empty".to_string(),
                });
            }
            if oauth.token_url.trim().is_empty() {
                errors.push(ProviderValidationError {
                    provider_id: provider.id.clone(),
                    message: "oauth.token_url must not be empty".to_string(),
                });
            }
            if oauth.scopes.is_empty() {
                errors.push(ProviderValidationError {
                    provider_id: provider.id.clone(),
                    message: "oauth.scopes must contain at least one scope".to_string(),
                });
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Parse and validate a user-supplied provider file from JSON content.
///
/// Returns the parsed providers if parsing and validation both succeed.
pub fn parse_and_validate_provider_file(
    content: &str,
) -> Result<Vec<Provider>, UserProviderFileError> {
    let providers = parse_user_provider_file(content)?;
    validate_provider_configs(&providers).map_err(|errs| {
        let msgs: Vec<String> = errs.iter().map(|e| e.to_string()).collect();
        UserProviderFileError::Validation(msgs.join("; "))
    })?;
    Ok(providers)
}

/// Merge user-supplied providers into the bundled database.
///
/// User-supplied entries override bundled entries with the same `id`.
/// New entries (IDs not present in bundled) are appended.
/// The resulting database participates in normal domain-matching and scoring.
pub fn merge_user_providers(
    bundled_providers: Vec<Provider>,
    user_providers: Vec<Provider>,
) -> Vec<Provider> {
    let mut merged = bundled_providers;

    for user_entry in user_providers {
        if let Some(pos) = merged.iter().position(|p| p.id == user_entry.id) {
            // Override existing bundled entry
            merged[pos] = user_entry;
        } else {
            // Add new entry
            merged.push(user_entry);
        }
    }

    merged
}

/// Parse a user-supplied provider file from JSON content.
pub fn parse_user_provider_file(content: &str) -> Result<Vec<Provider>, UserProviderFileError> {
    let providers: Vec<Provider> = serde_json::from_str(content)?;
    Ok(providers)
}

/// Build a ProviderDatabase from bundled + user-supplied providers.
///
/// If `user_content` is `Some`, the JSON is parsed and merged.
/// If `None`, returns the bundled database unchanged.
pub fn build_merged_database(
    user_content: Option<&str>,
) -> Result<ProviderDatabase, UserProviderFileError> {
    let bundled = super::provider_data::bundled_providers();

    match user_content {
        Some(content) => {
            let user_providers = parse_user_provider_file(content)?;
            let merged = merge_user_providers(bundled, user_providers);
            Ok(ProviderDatabase::new(merged))
        }
        None => Ok(ProviderDatabase::new(bundled)),
    }
}

/// The default filename for the user-supplied provider file.
pub const USER_PROVIDER_FILENAME: &str = "providers.json";

/// The application config directory name.
pub const APP_CONFIG_DIR: &str = "fairmail";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::provider::{
        MatchScore, MaxTlsVersion, ProviderEncryption, ServerConfig, UsernameType,
    };

    fn make_provider(id: &str, domains: &[&str]) -> Provider {
        Provider {
            id: id.to_string(),
            display_name: id.to_string(),
            domain_patterns: domains.iter().map(|s| s.to_string()).collect(),
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
            oauth: None,
            display_order: 0,
            enabled: true,
            supports_shared_mailbox: false,
        }
    }

    #[test]
    fn test_merge_adds_new_providers() {
        let bundled = vec![make_provider("gmail", &["gmail.com"])];
        let user = vec![make_provider("corpmail", &["corp.example.com"])];

        let merged = merge_user_providers(bundled, user);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[1].id, "corpmail");
    }

    #[test]
    fn test_merge_overrides_existing_provider() {
        let bundled = vec![make_provider("gmail", &["gmail.com"])];
        let mut override_gmail = make_provider("gmail", &["gmail.com", "custom-gmail.com"]);
        override_gmail.incoming.port = 143;
        let user = vec![override_gmail];

        let merged = merge_user_providers(bundled, user);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].id, "gmail");
        assert_eq!(merged[0].incoming.port, 143);
        assert_eq!(merged[0].domain_patterns.len(), 2);
    }

    #[test]
    fn test_merge_preserves_bundled_when_no_user_providers() {
        let bundled = vec![
            make_provider("gmail", &["gmail.com"]),
            make_provider("outlook", &["outlook.com"]),
        ];
        let user = vec![];

        let merged = merge_user_providers(bundled, user);
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn test_parse_valid_json() {
        let json = r#"[
            {
                "id": "corpmail",
                "display_name": "Corporate Mail",
                "domain_patterns": ["corp.example.com", "*.corp.example.com"],
                "mx_patterns": [],
                "incoming": {
                    "hostname": "imap.corp.example.com",
                    "port": 993,
                    "encryption": "SslTls"
                },
                "outgoing": {
                    "hostname": "smtp.corp.example.com",
                    "port": 465,
                    "encryption": "SslTls"
                },
                "username_type": "EmailAddress",
                "keep_alive_interval": 15,
                "noop_keep_alive": false,
                "partial_fetch": true,
                "max_tls_version": "Tls1_3",
                "app_password_required": false,
                "documentation_url": null,
                "localized_docs": [],
                "oauth": null,
                "display_order": 200,
                "enabled": true
            }
        ]"#;

        let providers = parse_user_provider_file(json).unwrap();
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].id, "corpmail");
        assert_eq!(providers[0].domain_patterns.len(), 2);
    }

    #[test]
    fn test_parse_invalid_json() {
        let result = parse_user_provider_file("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_user_providers_participate_in_domain_matching() {
        let json = r#"[
            {
                "id": "corpmail",
                "display_name": "Corporate Mail",
                "domain_patterns": ["corp.example.com", "*.corp.example.com"],
                "mx_patterns": [],
                "incoming": {
                    "hostname": "imap.corp.example.com",
                    "port": 993,
                    "encryption": "SslTls"
                },
                "outgoing": {
                    "hostname": "smtp.corp.example.com",
                    "port": 465,
                    "encryption": "SslTls"
                },
                "username_type": "EmailAddress",
                "keep_alive_interval": 15,
                "noop_keep_alive": false,
                "partial_fetch": true,
                "max_tls_version": "Tls1_3",
                "app_password_required": false,
                "documentation_url": null,
                "localized_docs": [],
                "oauth": null,
                "display_order": 200,
                "enabled": true
            }
        ]"#;

        let db = build_merged_database(Some(json)).unwrap();

        // User-supplied provider participates in exact domain matching
        let candidate = db.lookup_by_email("user@corp.example.com").unwrap();
        assert_eq!(candidate.provider.id, "corpmail");
        assert_eq!(candidate.score, MatchScore::BUNDLED_EXACT);

        // User-supplied provider participates in wildcard matching
        let candidate = db.lookup_by_email("user@sub.corp.example.com").unwrap();
        assert_eq!(candidate.provider.id, "corpmail");
        assert_eq!(candidate.score, MatchScore::BUNDLED_WILDCARD);
    }

    #[test]
    fn test_build_merged_database_no_user_file() {
        let db = build_merged_database(None).unwrap();
        // Should behave exactly like bundled database
        assert!(db.provider_count() >= 150);
        assert!(db.lookup_by_domain("gmail.com").is_some());
    }

    #[test]
    fn test_user_override_participates_in_scoring() {
        // Override gmail with different settings — should still match via domain
        let json = r#"[
            {
                "id": "gmail",
                "display_name": "Gmail Custom",
                "domain_patterns": ["gmail.com", "googlemail.com"],
                "mx_patterns": [],
                "incoming": {
                    "hostname": "custom-imap.gmail.com",
                    "port": 993,
                    "encryption": "SslTls"
                },
                "outgoing": {
                    "hostname": "custom-smtp.gmail.com",
                    "port": 465,
                    "encryption": "SslTls"
                },
                "username_type": "EmailAddress",
                "keep_alive_interval": 20,
                "noop_keep_alive": true,
                "partial_fetch": true,
                "max_tls_version": "Tls1_3",
                "app_password_required": false,
                "documentation_url": null,
                "localized_docs": [],
                "oauth": null,
                "display_order": 1,
                "enabled": true
            }
        ]"#;

        let db = build_merged_database(Some(json)).unwrap();
        let candidate = db.lookup_by_email("user@gmail.com").unwrap();
        assert_eq!(candidate.provider.id, "gmail");
        assert_eq!(candidate.provider.display_name, "Gmail Custom");
        assert_eq!(candidate.provider.keep_alive_interval, 20);
        assert_eq!(candidate.score, MatchScore::BUNDLED_EXACT);
    }

    #[test]
    fn test_validate_valid_provider_without_oauth() {
        let providers = vec![make_provider("test", &["test.com"])];
        assert!(validate_provider_configs(&providers).is_ok());
    }

    #[test]
    fn test_validate_empty_id_rejected() {
        let mut p = make_provider("", &["test.com"]);
        p.id = "".to_string();
        let result = validate_provider_configs(&[p]);
        assert!(result.is_err());
        assert!(result.unwrap_err()[0]
            .message
            .contains("id must not be empty"));
    }

    #[test]
    fn test_validate_empty_display_name_rejected() {
        let mut p = make_provider("test", &["test.com"]);
        p.display_name = "".to_string();
        let result = validate_provider_configs(&[p]);
        assert!(result.is_err());
        assert!(result.unwrap_err()[0]
            .message
            .contains("display_name must not be empty"));
    }

    #[test]
    fn test_validate_oauth_missing_auth_url() {
        let mut p = make_provider("test", &["test.com"]);
        p.oauth = Some(crate::core::provider::OAuthConfig {
            auth_url: "".to_string(),
            token_url: "https://example.com/token".to_string(),
            redirect_uri: "http://127.0.0.1".to_string(),
            scopes: vec!["openid".to_string()],
            client_id: None,
            extra_params: vec![],
            userinfo_url: None,
        });
        let result = validate_provider_configs(&[p]);
        assert!(result.is_err());
        assert!(result.unwrap_err()[0]
            .message
            .contains("auth_url must not be empty"));
    }

    #[test]
    fn test_validate_oauth_missing_token_url() {
        let mut p = make_provider("test", &["test.com"]);
        p.oauth = Some(crate::core::provider::OAuthConfig {
            auth_url: "https://example.com/auth".to_string(),
            token_url: "".to_string(),
            redirect_uri: "http://127.0.0.1".to_string(),
            scopes: vec!["openid".to_string()],
            client_id: None,
            extra_params: vec![],
            userinfo_url: None,
        });
        let result = validate_provider_configs(&[p]);
        assert!(result.is_err());
        assert!(result.unwrap_err()[0]
            .message
            .contains("token_url must not be empty"));
    }

    #[test]
    fn test_validate_oauth_empty_scopes() {
        let mut p = make_provider("test", &["test.com"]);
        p.oauth = Some(crate::core::provider::OAuthConfig {
            auth_url: "https://example.com/auth".to_string(),
            token_url: "https://example.com/token".to_string(),
            redirect_uri: "http://127.0.0.1".to_string(),
            scopes: vec![],
            client_id: None,
            extra_params: vec![],
            userinfo_url: None,
        });
        let result = validate_provider_configs(&[p]);
        assert!(result.is_err());
        assert!(result.unwrap_err()[0]
            .message
            .contains("at least one scope"));
    }

    #[test]
    fn test_validate_valid_oauth_provider() {
        let mut p = make_provider("keycloak", &["corp.example.com"]);
        p.oauth = Some(crate::core::provider::OAuthConfig {
            auth_url: "https://auth.corp.example.com/auth".to_string(),
            token_url: "https://auth.corp.example.com/token".to_string(),
            redirect_uri: "http://127.0.0.1".to_string(),
            scopes: vec!["openid".to_string(), "email".to_string()],
            client_id: Some("fairmail-desktop".to_string()),
            extra_params: vec![],
            userinfo_url: Some("https://auth.corp.example.com/userinfo".to_string()),
        });
        assert!(validate_provider_configs(&[p]).is_ok());
    }

    #[test]
    fn test_parse_and_validate_valid_file() {
        let json = r#"[{
            "id": "corpmail",
            "display_name": "Corp Mail",
            "domain_patterns": ["corp.example.com"],
            "mx_patterns": [],
            "incoming": {"hostname": "imap.corp.example.com", "port": 993, "encryption": "SslTls"},
            "outgoing": {"hostname": "smtp.corp.example.com", "port": 465, "encryption": "SslTls"},
            "username_type": "EmailAddress",
            "keep_alive_interval": 15,
            "noop_keep_alive": false,
            "partial_fetch": true,
            "max_tls_version": "Tls1_3",
            "app_password_required": false,
            "documentation_url": null,
            "localized_docs": [],
            "oauth": null,
            "display_order": 200,
            "enabled": true
        }]"#;
        let result = parse_and_validate_provider_file(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_parse_and_validate_rejects_invalid_oauth() {
        let json = r#"[{
            "id": "bad-oauth",
            "display_name": "Bad OAuth",
            "domain_patterns": ["bad.example.com"],
            "mx_patterns": [],
            "incoming": {"hostname": "imap.bad.example.com", "port": 993, "encryption": "SslTls"},
            "outgoing": {"hostname": "smtp.bad.example.com", "port": 465, "encryption": "SslTls"},
            "username_type": "EmailAddress",
            "keep_alive_interval": 15,
            "noop_keep_alive": false,
            "partial_fetch": true,
            "max_tls_version": "Tls1_3",
            "app_password_required": false,
            "documentation_url": null,
            "localized_docs": [],
            "oauth": {"auth_url": "", "token_url": "", "redirect_uri": "", "scopes": [], "client_id": null, "extra_params": [], "userinfo_url": null},
            "display_order": 200,
            "enabled": true
        }]"#;
        let result = parse_and_validate_provider_file(json);
        assert!(result.is_err());
    }
}
