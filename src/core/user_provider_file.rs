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
}
