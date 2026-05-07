use super::account::EncryptionMode;
use super::provider::{derive_username, Provider, ProviderDatabase, ProviderEncryption};

/// Pre-filled server settings for both inbound (IMAP) and outbound (SMTP)
/// extracted from a matched provider entry (FR-15, FR-16, FR-17, FR-19).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerSettingsPrefill {
    /// IMAP server hostname.
    pub imap_host: String,
    /// IMAP server port.
    pub imap_port: u16,
    /// IMAP connection encryption mode.
    pub imap_encryption: EncryptionMode,
    /// SMTP server hostname.
    pub smtp_host: String,
    /// SMTP server port.
    pub smtp_port: u16,
    /// SMTP connection encryption mode.
    pub smtp_encryption: EncryptionMode,
    /// Derived login username (FR-19).
    pub username: String,
}

/// Convert provider encryption to account encryption mode.
fn to_encryption_mode(enc: ProviderEncryption) -> EncryptionMode {
    match enc {
        ProviderEncryption::None => EncryptionMode::None,
        ProviderEncryption::SslTls => EncryptionMode::SslTls,
        ProviderEncryption::StartTls => EncryptionMode::StartTls,
    }
}

/// Extract pre-filled server settings from a provider entry (FR-15, FR-16, FR-17, FR-19).
///
/// When a provider defines multiple server entries of the same type,
/// the first (primary) entry is used (FR-16). The current data model
/// stores a single incoming and outgoing config per provider, so this
/// is inherently deterministic.
///
/// The `email` parameter is used to derive the login username according
/// to the provider's `username_type` (FR-18, FR-19).
pub(crate) fn prefill_from_provider(provider: &Provider, email: &str) -> ServerSettingsPrefill {
    ServerSettingsPrefill {
        imap_host: provider.incoming.hostname.clone(),
        imap_port: provider.incoming.port,
        imap_encryption: to_encryption_mode(provider.incoming.encryption),
        smtp_host: provider.outgoing.hostname.clone(),
        smtp_port: provider.outgoing.port,
        smtp_encryption: to_encryption_mode(provider.outgoing.encryption),
        username: derive_username(email, &provider.username_type),
    }
}

/// Look up a provider by email address and return pre-filled IMAP and SMTP
/// server settings including the derived username (FR-15, FR-17, FR-19).
///
/// Returns `None` if no provider matches the email domain.
/// This uses only the bundled provider database, so it works fully offline (AC-2).
pub(crate) fn prefill_from_email(
    email: &str,
    db: &ProviderDatabase,
) -> Option<ServerSettingsPrefill> {
    let candidate = db.lookup_by_email(email)?;
    Some(prefill_from_provider(&candidate.provider, email))
}

/// Look up a provider by domain and return pre-filled IMAP and SMTP
/// server settings (FR-15, FR-17, FR-19).
///
/// Returns `None` if no provider matches the domain.
/// This uses only the bundled provider database, so it works fully offline (AC-2).
/// The `email` parameter is used to derive the login username.
pub(crate) fn prefill_from_domain(
    email: &str,
    domain: &str,
    db: &ProviderDatabase,
) -> Option<ServerSettingsPrefill> {
    let candidate = db.lookup_by_domain(domain)?;
    Some(prefill_from_provider(&candidate.provider, email))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::provider::{
        MaxTlsVersion, Provider, ProviderDatabase, ProviderEncryption, ServerConfig, UsernameType,
    };

    fn make_test_provider(id: &str, domains: &[&str]) -> Provider {
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
            disable_ip_connections: false,
            requires_manual_enablement: false,
            documentation_url: None,
            localized_docs: vec![],
            oauth: None,
            display_order: 0,
            enabled: true,
            supports_shared_mailbox: false,
            subtitle: None,
            registration_url: None,
            app_password_url: None,
            graph: None,
            debug_only: false,
            variant_of: None,
        }
    }

    // --- AC: When a provider is matched by domain, IMAP host/port/encryption are pre-filled ---

    #[test]
    fn prefill_from_email_returns_imap_settings() {
        let db = ProviderDatabase::new(vec![make_test_provider("example", &["example.com"])]);
        let prefill = prefill_from_email("user@example.com", &db).unwrap();
        assert_eq!(prefill.imap_host, "imap.example.com");
        assert_eq!(prefill.imap_port, 993);
        assert_eq!(prefill.imap_encryption, EncryptionMode::SslTls);
    }

    // --- AC: When a provider is matched by domain, SMTP host/port/encryption are pre-filled ---

    #[test]
    fn prefill_from_email_returns_smtp_settings() {
        let db = ProviderDatabase::new(vec![make_test_provider("example", &["example.com"])]);
        let prefill = prefill_from_email("user@example.com", &db).unwrap();
        assert_eq!(prefill.smtp_host, "smtp.example.com");
        assert_eq!(prefill.smtp_port, 465);
        assert_eq!(prefill.smtp_encryption, EncryptionMode::SslTls);
    }

    // --- AC: Each server entry specifies exactly one of SSL/TLS or STARTTLS ---

    #[test]
    fn encryption_ssl_tls_maps_correctly() {
        assert_eq!(
            to_encryption_mode(ProviderEncryption::SslTls),
            EncryptionMode::SslTls
        );
    }

    #[test]
    fn encryption_starttls_maps_correctly() {
        assert_eq!(
            to_encryption_mode(ProviderEncryption::StartTls),
            EncryptionMode::StartTls
        );
    }

    #[test]
    fn encryption_none_maps_correctly() {
        assert_eq!(
            to_encryption_mode(ProviderEncryption::None),
            EncryptionMode::None
        );
    }

    #[test]
    fn provider_with_starttls_prefills_starttls() {
        let mut provider = make_test_provider("starttls", &["starttls.example.com"]);
        provider.incoming.encryption = ProviderEncryption::StartTls;
        provider.incoming.port = 143;
        provider.outgoing.encryption = ProviderEncryption::StartTls;
        provider.outgoing.port = 587;
        let db = ProviderDatabase::new(vec![provider]);

        let prefill = prefill_from_email("user@starttls.example.com", &db).unwrap();
        assert_eq!(prefill.imap_encryption, EncryptionMode::StartTls);
        assert_eq!(prefill.imap_port, 143);
        assert_eq!(prefill.smtp_encryption, EncryptionMode::StartTls);
        assert_eq!(prefill.smtp_port, 587);
    }

    // --- AC: Multiple server entries → deterministic selection (first entry wins) ---

    #[test]
    fn prefill_from_provider_is_deterministic() {
        let provider = make_test_provider("multi", &["multi.com"]);
        let first = prefill_from_provider(&provider, "user@multi.com");
        let second = prefill_from_provider(&provider, "user@multi.com");
        assert_eq!(first, second);
    }

    // --- AC: No match returns None ---

    #[test]
    fn prefill_from_email_returns_none_for_unknown_domain() {
        let db = ProviderDatabase::new(vec![make_test_provider("example", &["example.com"])]);
        assert!(prefill_from_email("user@unknown.com", &db).is_none());
    }

    #[test]
    fn prefill_from_domain_returns_none_for_unknown() {
        let db = ProviderDatabase::new(vec![make_test_provider("example", &["example.com"])]);
        assert!(prefill_from_domain("user@unknown.com", "unknown.com", &db).is_none());
    }

    // --- AC-1: Gmail pre-fills imap.gmail.com:993/SSL and smtp.gmail.com:465/SSL ---

    #[test]
    fn gmail_prefills_correct_imap_settings() {
        let db = ProviderDatabase::bundled();
        let prefill = prefill_from_email("user@gmail.com", &db).unwrap();
        assert_eq!(prefill.imap_host, "imap.gmail.com");
        assert_eq!(prefill.imap_port, 993);
        assert_eq!(prefill.imap_encryption, EncryptionMode::SslTls);
    }

    #[test]
    fn gmail_prefills_correct_smtp_settings() {
        let db = ProviderDatabase::bundled();
        let prefill = prefill_from_email("user@gmail.com", &db).unwrap();
        assert_eq!(prefill.smtp_host, "smtp.gmail.com");
        assert_eq!(prefill.smtp_port, 465);
        assert_eq!(prefill.smtp_encryption, EncryptionMode::SslTls);
    }

    // --- AC-2: Offline pre-fill (bundled database requires no network) ---

    #[test]
    fn gmail_prefill_works_offline_with_bundled_db() {
        // The bundled database is compiled into the binary, so lookup
        // succeeds without any network access — verifying AC-2.
        let db = ProviderDatabase::bundled();
        let prefill = prefill_from_email("alice@gmail.com", &db);
        assert!(prefill.is_some(), "Bundled lookup must succeed offline");
    }

    // --- Additional coverage: domain-based lookup ---

    #[test]
    fn prefill_from_domain_returns_settings() {
        let db = ProviderDatabase::new(vec![make_test_provider("example", &["example.com"])]);
        let prefill = prefill_from_domain("user@example.com", "example.com", &db).unwrap();
        assert_eq!(prefill.imap_host, "imap.example.com");
        assert_eq!(prefill.smtp_host, "smtp.example.com");
    }

    #[test]
    fn prefill_from_domain_case_insensitive() {
        let db = ProviderDatabase::new(vec![make_test_provider("example", &["example.com"])]);
        let prefill = prefill_from_domain("user@EXAMPLE.COM", "EXAMPLE.COM", &db).unwrap();
        assert_eq!(prefill.imap_host, "imap.example.com");
    }

    #[test]
    fn prefill_from_email_case_insensitive() {
        let db = ProviderDatabase::bundled();
        let prefill = prefill_from_email("User@GMAIL.COM", &db).unwrap();
        assert_eq!(prefill.imap_host, "imap.gmail.com");
        assert_eq!(prefill.smtp_host, "smtp.gmail.com");
    }

    // --- Disabled provider is not matched ---

    #[test]
    fn disabled_provider_returns_none() {
        let mut provider = make_test_provider("disabled", &["disabled.com"]);
        provider.enabled = false;
        let db = ProviderDatabase::new(vec![provider]);
        assert!(prefill_from_email("user@disabled.com", &db).is_none());
    }

    // --- AC: Default username format uses full email address (FR-19) ---

    #[test]
    fn default_username_type_prefills_full_email() {
        let provider = make_test_provider("example", &["example.com"]);
        assert_eq!(provider.username_type, UsernameType::EmailAddress);
        let prefill = prefill_from_provider(&provider, "alice@example.com");
        assert_eq!(prefill.username, "alice@example.com");
    }

    // --- AC-15: Local-part-only username derivation ---

    #[test]
    fn local_part_username_prefills_local_only() {
        let mut provider = make_test_provider("localonly", &["localonly.com"]);
        provider.username_type = UsernameType::LocalPart;
        let prefill = prefill_from_provider(&provider, "alice@localonly.com");
        assert_eq!(prefill.username, "alice");
    }

    // --- AC: Custom template username derivation ---

    #[test]
    fn custom_template_username_uses_local_placeholder() {
        let mut provider = make_test_provider("custom", &["custom.com"]);
        provider.username_type = UsernameType::CustomTemplate("{local}+mail@{domain}".to_string());
        let prefill = prefill_from_provider(&provider, "alice@custom.com");
        assert_eq!(prefill.username, "alice+mail@custom.com");
    }

    #[test]
    fn custom_template_username_uses_email_placeholder() {
        let mut provider = make_test_provider("custom", &["custom.com"]);
        provider.username_type = UsernameType::CustomTemplate("prefix-{email}".to_string());
        let prefill = prefill_from_provider(&provider, "alice@custom.com");
        assert_eq!(prefill.username, "prefix-alice@custom.com");
    }

    // --- AC: Username is pre-filled via email lookup path ---

    #[test]
    fn prefill_from_email_includes_derived_username() {
        let db = ProviderDatabase::new(vec![make_test_provider("example", &["example.com"])]);
        let prefill = prefill_from_email("bob@example.com", &db).unwrap();
        assert_eq!(prefill.username, "bob@example.com");
    }

    #[test]
    fn prefill_from_email_local_part_provider() {
        let mut provider = make_test_provider("lp", &["lp.com"]);
        provider.username_type = UsernameType::LocalPart;
        let db = ProviderDatabase::new(vec![provider]);
        let prefill = prefill_from_email("alice@lp.com", &db).unwrap();
        assert_eq!(prefill.username, "alice");
    }

    // --- AC: Username is pre-filled via domain lookup path ---

    #[test]
    fn prefill_from_domain_includes_derived_username() {
        let db = ProviderDatabase::new(vec![make_test_provider("example", &["example.com"])]);
        let prefill = prefill_from_domain("bob@example.com", "example.com", &db).unwrap();
        assert_eq!(prefill.username, "bob@example.com");
    }
}
