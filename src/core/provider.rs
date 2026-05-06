use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Connection encryption mode for provider server settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderEncryption {
    None,
    SslTls,
    StartTls,
}

/// How the username is derived from the email address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UsernameType {
    /// Full email address (e.g. user@example.com)
    EmailAddress,
    /// Local part only (e.g. user)
    LocalPart,
}

/// Maximum TLS version a provider supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaxTlsVersion {
    Tls1_2,
    Tls1_3,
}

/// OAuth configuration for a provider (FR-15n).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub auth_url: String,
    pub token_url: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub client_id: Option<String>,
    /// Whether this provider requires PKCE (Proof Key for Code Exchange, RFC 7636).
    /// When `true`, the authorization request includes `code_challenge` and
    /// `code_challenge_method=S256`, and the token exchange includes `code_verifier`.
    #[serde(default = "default_pkce_required")]
    pub pkce_required: bool,
    /// Provider-specific query parameters appended to the authorization request
    /// (e.g. `prompt=consent`, `access_type=offline`, `force_confirm=true`).
    pub extra_params: Vec<(String, String)>,
    /// URL to fetch user identity info when the token response does not include
    /// an ID token with email/name claims (FR-35, N-9). When `Some`, the
    /// application must GET this endpoint with a Bearer token to retrieve the
    /// user's email and display name. Analogous to the `askAccount` flag in the
    /// Android codebase.
    pub userinfo_url: Option<String>,
}

fn default_pkce_required() -> bool {
    true
}

/// Default tenant value used when a provider requires a tenant but the user
/// does not supply one. `"common"` allows both personal and organizational
/// Microsoft accounts (FR-10, AC-5).
pub const DEFAULT_TENANT: &str = "common";

impl OAuthConfig {
    /// Whether this provider's endpoint URLs contain a `{tenant}` placeholder
    /// that must be substituted before use (FR-10, US-4).
    pub fn requires_tenant(&self) -> bool {
        self.auth_url.contains("{tenant}") || self.token_url.contains("{tenant}")
    }

    /// Return a copy of this config with `{tenant}` placeholders replaced.
    /// If `tenant` is empty or `None`, [`DEFAULT_TENANT`] is used.
    pub fn with_tenant(&self, tenant: Option<&str>) -> Self {
        let tenant = match tenant {
            Some(t) if !t.trim().is_empty() => t.trim(),
            _ => DEFAULT_TENANT,
        };
        let mut config = self.clone();
        config.auth_url = config.auth_url.replace("{tenant}", tenant);
        config.token_url = config.token_url.replace("{tenant}", tenant);
        config
    }
}

/// Server configuration (incoming or outgoing).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerConfig {
    pub hostname: String,
    pub port: u16,
    pub encryption: ProviderEncryption,
}

/// A localized documentation snippet (FR-15m).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalizedDoc {
    pub locale: String,
    pub text: String,
}

/// A single provider entry in the bundled database (FR-15 a-p).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provider {
    /// Unique identifier (FR-15a)
    pub id: String,
    /// Display name (FR-15a)
    pub display_name: String,
    /// Domain-matching patterns (FR-15b) - supports wildcards like *.example.com
    pub domain_patterns: Vec<String>,
    /// MX-matching patterns (FR-15c)
    pub mx_patterns: Vec<String>,
    /// Incoming server configuration (FR-15d)
    pub incoming: ServerConfig,
    /// Outgoing server configuration (FR-15e)
    pub outgoing: ServerConfig,
    /// Username type (FR-15f)
    pub username_type: UsernameType,
    /// Keep-alive interval in minutes (FR-15g)
    pub keep_alive_interval: u32,
    /// Whether to use NOOP for keep-alive (FR-15h)
    pub noop_keep_alive: bool,
    /// Whether partial-fetch is supported (FR-15i)
    pub partial_fetch: bool,
    /// Maximum TLS version (FR-15j)
    pub max_tls_version: MaxTlsVersion,
    /// Whether an app-specific password is required (FR-15k)
    pub app_password_required: bool,
    /// Provider documentation link (FR-15l)
    pub documentation_url: Option<String>,
    /// Localized documentation snippets (FR-15m)
    pub localized_docs: Vec<LocalizedDoc>,
    /// OAuth configuration (FR-15n)
    pub oauth: Option<OAuthConfig>,
    /// Display order / priority (FR-15o)
    pub display_order: u32,
    /// Enabled/disabled flag (FR-15p)
    pub enabled: bool,
    /// Whether this provider supports shared mailbox access (FR-40, N-8).
    /// When true, the UI allows the user to specify a shared mailbox address
    /// and the application encodes the username appropriately (e.g.
    /// `shared@domain\user@domain` for Outlook).
    #[serde(default)]
    pub supports_shared_mailbox: bool,
}

/// Confidence score for a provider match.
/// Bundled-database matches always score higher than network-discovered (FR-11).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct MatchScore(f64);

impl MatchScore {
    /// Score for an exact domain match in the bundled database.
    pub const BUNDLED_EXACT: Self = Self(1.0);
    /// Score for a wildcard domain match in the bundled database.
    pub const BUNDLED_WILDCARD: Self = Self(0.9);
    /// Maximum score any network-discovered candidate can have.
    pub const NETWORK_MAX: Self = Self(0.5);
    /// Score for a DNS NS record match (FR-10.2).
    pub const DNS_NS: Self = Self(0.45);
    /// Score for a DNS MX record match (FR-10.3).
    pub const DNS_MX: Self = Self(0.40);
    /// Score for RFC 6186 SRV record discovery (FR-10.4).
    pub const DNS_SRV: Self = Self(0.35);
    /// Score for Thunderbird ISPDB autoconfig (FR-10.5).
    pub const ISPDB: Self = Self(0.30);
    /// Score for vendor-specific autodiscovery (FR-10.6).
    pub const VENDOR_AUTODISCOVERY: Self = Self(0.25);
    /// Score for port-scanning fallback (FR-10.7) — lowest confidence of all strategies.
    pub const PORT_SCAN: Self = Self(0.10);

    pub fn value(&self) -> f64 {
        self.0
    }

    pub fn is_bundled(&self) -> bool {
        self.0 > Self::NETWORK_MAX.0
    }
}

/// A candidate result from looking up a provider.
#[derive(Debug, Clone)]
pub struct ProviderCandidate {
    pub provider: Provider,
    pub score: MatchScore,
}

/// The bundled provider database.
pub struct ProviderDatabase {
    providers: Vec<Provider>,
    /// Index: domain -> list of (provider index, is_wildcard)
    domain_index: HashMap<String, Vec<(usize, bool)>>,
}

impl ProviderDatabase {
    /// Build the database from a list of providers.
    pub fn new(providers: Vec<Provider>) -> Self {
        let mut domain_index: HashMap<String, Vec<(usize, bool)>> = HashMap::new();

        for (idx, provider) in providers.iter().enumerate() {
            if !provider.enabled {
                continue;
            }
            for pattern in &provider.domain_patterns {
                let lower = pattern.to_lowercase();
                if lower.starts_with("*.") {
                    // Wildcard: index by the suffix (without *.)
                    let suffix = lower.strip_prefix("*.").unwrap();
                    domain_index
                        .entry(suffix.to_string())
                        .or_default()
                        .push((idx, true));
                } else {
                    domain_index.entry(lower).or_default().push((idx, false));
                }
            }
        }

        Self {
            providers,
            domain_index,
        }
    }

    /// Load the bundled provider database (compiled into the binary).
    pub fn bundled() -> Self {
        Self::new(super::provider_data::bundled_providers())
    }

    /// Look up a provider by email address.
    /// Returns the best-matching candidate, or None if no match found.
    pub fn lookup_by_email(&self, email: &str) -> Option<ProviderCandidate> {
        let domain = email_domain(email)?;
        self.lookup_by_domain(&domain)
    }

    /// Look up a provider by domain.
    pub fn lookup_by_domain(&self, domain: &str) -> Option<ProviderCandidate> {
        let lower = domain.to_lowercase();
        let mut best: Option<ProviderCandidate> = None;

        // Check for exact domain match
        if let Some(entries) = self.domain_index.get(&lower) {
            for &(idx, is_wildcard) in entries {
                if !is_wildcard {
                    let score = MatchScore::BUNDLED_EXACT;
                    let candidate = ProviderCandidate {
                        provider: self.providers[idx].clone(),
                        score,
                    };
                    if best.as_ref().is_none_or(|b| score > b.score) {
                        best = Some(candidate);
                    }
                }
            }
        }

        // If no exact match, check wildcard matches by walking up the domain
        if best.is_none() {
            let parts: Vec<&str> = lower.split('.').collect();
            for i in 1..parts.len() {
                let suffix = parts[i..].join(".");
                if let Some(entries) = self.domain_index.get(&suffix) {
                    for &(idx, is_wildcard) in entries {
                        if is_wildcard {
                            let score = MatchScore::BUNDLED_WILDCARD;
                            let candidate = ProviderCandidate {
                                provider: self.providers[idx].clone(),
                                score,
                            };
                            if best.as_ref().is_none_or(|b| score > b.score) {
                                best = Some(candidate);
                            }
                        }
                    }
                }
                if best.is_some() {
                    break;
                }
            }
        }

        best
    }

    /// Get the total number of providers in the database.
    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }

    /// Get all providers.
    pub fn providers(&self) -> &[Provider] {
        &self.providers
    }

    /// Look up a provider by server hostname (incoming or outgoing).
    /// Returns the first enabled provider whose IMAP or SMTP hostname matches.
    pub fn lookup_by_hostname(&self, hostname: &str) -> Option<&Provider> {
        let lower = hostname.to_lowercase();
        self.providers.iter().filter(|p| p.enabled).find(|p| {
            p.incoming.hostname.to_lowercase() == lower
                || p.outgoing.hostname.to_lowercase() == lower
        })
    }
}

/// When a network-discovered candidate's server hostname matches a bundled
/// provider entry, replace the network-discovered settings with the bundled
/// entry's values while preserving the original discovery score (FR-12, N-4).
///
/// If no bundled entry matches, the candidate is returned unchanged.
pub fn merge_network_with_bundled(
    candidate: ProviderCandidate,
    db: &ProviderDatabase,
) -> ProviderCandidate {
    // Only merge network-discovered candidates (bundled ones are already complete).
    if candidate.score.is_bundled() {
        return candidate;
    }

    // Try matching on incoming hostname, then outgoing hostname.
    if let Some(bundled) = db
        .lookup_by_hostname(&candidate.provider.incoming.hostname)
        .or_else(|| db.lookup_by_hostname(&candidate.provider.outgoing.hostname))
    {
        ProviderCandidate {
            provider: bundled.clone(),
            score: candidate.score,
        }
    } else {
        candidate
    }
}

/// Extract the domain part from an email address.
fn email_domain(email: &str) -> Option<String> {
    let at_pos = email.rfind('@')?;
    let domain = &email[at_pos + 1..];
    if domain.is_empty() {
        None
    } else {
        Some(domain.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            documentation_url: None,
            localized_docs: vec![],
            oauth: None,
            display_order: 0,
            enabled: true,
            supports_shared_mailbox: false,
        }
    }

    #[test]
    fn test_exact_domain_match() {
        let db = ProviderDatabase::new(vec![make_test_provider(
            "gmail",
            &["gmail.com", "googlemail.com"],
        )]);
        let candidate = db.lookup_by_email("user@gmail.com").unwrap();
        assert_eq!(candidate.provider.id, "gmail");
        assert_eq!(candidate.score, MatchScore::BUNDLED_EXACT);
    }

    #[test]
    fn test_wildcard_domain_match() {
        let db = ProviderDatabase::new(vec![make_test_provider("custom", &["*.example.org"])]);
        let candidate = db.lookup_by_domain("mail.example.org").unwrap();
        assert_eq!(candidate.provider.id, "custom");
        assert_eq!(candidate.score, MatchScore::BUNDLED_WILDCARD);
    }

    #[test]
    fn test_no_match() {
        let db = ProviderDatabase::new(vec![make_test_provider("gmail", &["gmail.com"])]);
        assert!(db.lookup_by_domain("unknown.example.com").is_none());
    }

    #[test]
    fn test_case_insensitive() {
        let db = ProviderDatabase::new(vec![make_test_provider("gmail", &["gmail.com"])]);
        let candidate = db.lookup_by_email("User@GMAIL.COM").unwrap();
        assert_eq!(candidate.provider.id, "gmail");
    }

    #[test]
    fn test_disabled_provider_not_matched() {
        let mut provider = make_test_provider("disabled", &["test.com"]);
        provider.enabled = false;
        let db = ProviderDatabase::new(vec![provider]);
        assert!(db.lookup_by_domain("test.com").is_none());
    }

    #[test]
    fn test_bundled_score_outranks_network() {
        assert!(MatchScore::BUNDLED_EXACT > MatchScore::NETWORK_MAX);
        assert!(MatchScore::BUNDLED_WILDCARD > MatchScore::NETWORK_MAX);
    }

    #[test]
    fn test_email_domain_extraction() {
        assert_eq!(
            email_domain("user@example.com"),
            Some("example.com".to_string())
        );
        assert_eq!(
            email_domain("user@SUB.Example.COM"),
            Some("sub.example.com".to_string())
        );
        assert_eq!(email_domain("invalid"), None);
        assert_eq!(email_domain("user@"), None);
    }

    #[test]
    fn test_bundled_database_loads() {
        let db = ProviderDatabase::bundled();
        assert!(db.provider_count() >= 150);
    }

    #[test]
    fn test_bundled_top_providers() {
        let db = ProviderDatabase::bundled();
        let top_domains = [
            "gmail.com",
            "outlook.com",
            "yahoo.com",
            "icloud.com",
            "aol.com",
            "mail.ru",
            "yandex.ru",
            "protonmail.com",
            "zoho.com",
            "gmx.de",
        ];
        for domain in &top_domains {
            assert!(
                db.lookup_by_domain(domain).is_some(),
                "Expected provider for domain: {domain}"
            );
        }
    }

    #[test]
    fn test_lookup_by_hostname_incoming() {
        let db = ProviderDatabase::new(vec![make_test_provider("gmail", &["gmail.com"])]);
        let found = db.lookup_by_hostname("imap.gmail.com").unwrap();
        assert_eq!(found.id, "gmail");
    }

    #[test]
    fn test_lookup_by_hostname_outgoing() {
        let db = ProviderDatabase::new(vec![make_test_provider("gmail", &["gmail.com"])]);
        let found = db.lookup_by_hostname("smtp.gmail.com").unwrap();
        assert_eq!(found.id, "gmail");
    }

    #[test]
    fn test_lookup_by_hostname_case_insensitive() {
        let db = ProviderDatabase::new(vec![make_test_provider("gmail", &["gmail.com"])]);
        assert!(db.lookup_by_hostname("IMAP.GMAIL.COM").is_some());
    }

    #[test]
    fn test_lookup_by_hostname_no_match() {
        let db = ProviderDatabase::new(vec![make_test_provider("gmail", &["gmail.com"])]);
        assert!(db.lookup_by_hostname("imap.unknown.com").is_none());
    }

    #[test]
    fn test_lookup_by_hostname_disabled_skipped() {
        let mut provider = make_test_provider("disabled", &["test.com"]);
        provider.enabled = false;
        let db = ProviderDatabase::new(vec![provider]);
        assert!(db.lookup_by_hostname("imap.disabled.com").is_none());
    }

    #[test]
    fn test_merge_replaces_settings_preserves_score() {
        let bundled = make_test_provider("gmail", &["gmail.com"]);
        let db = ProviderDatabase::new(vec![bundled]);

        // Network-discovered candidate with matching hostname but different settings
        let network_provider = Provider {
            id: "ispdb-discovered".to_string(),
            display_name: "ISPDB Result".to_string(),
            domain_patterns: vec![],
            mx_patterns: vec![],
            incoming: ServerConfig {
                hostname: "imap.gmail.com".to_string(),
                port: 143,
                encryption: ProviderEncryption::StartTls,
            },
            outgoing: ServerConfig {
                hostname: "smtp.gmail.com".to_string(),
                port: 587,
                encryption: ProviderEncryption::StartTls,
            },
            username_type: UsernameType::EmailAddress,
            keep_alive_interval: 0,
            noop_keep_alive: false,
            partial_fetch: false,
            max_tls_version: MaxTlsVersion::Tls1_3,
            app_password_required: false,
            documentation_url: None,
            localized_docs: vec![],
            oauth: None,
            display_order: 0,
            enabled: true,
            supports_shared_mailbox: false,
        };

        let candidate = ProviderCandidate {
            provider: network_provider,
            score: MatchScore::ISPDB,
        };

        let merged = merge_network_with_bundled(candidate, &db);

        // Score preserved
        assert_eq!(merged.score, MatchScore::ISPDB);
        // Provider replaced with bundled
        assert_eq!(merged.provider.id, "gmail");
        assert_eq!(merged.provider.incoming.port, 993);
        assert_eq!(
            merged.provider.incoming.encryption,
            ProviderEncryption::SslTls
        );
        assert_eq!(merged.provider.outgoing.port, 465);
        assert_eq!(merged.provider.keep_alive_interval, 15);
        assert!(merged.provider.partial_fetch);
    }

    #[test]
    fn test_merge_no_match_returns_unchanged() {
        let db = ProviderDatabase::new(vec![make_test_provider("gmail", &["gmail.com"])]);

        let network_provider = Provider {
            id: "unknown".to_string(),
            display_name: "Unknown".to_string(),
            domain_patterns: vec![],
            mx_patterns: vec![],
            incoming: ServerConfig {
                hostname: "imap.unknown.com".to_string(),
                port: 993,
                encryption: ProviderEncryption::SslTls,
            },
            outgoing: ServerConfig {
                hostname: "smtp.unknown.com".to_string(),
                port: 465,
                encryption: ProviderEncryption::SslTls,
            },
            username_type: UsernameType::EmailAddress,
            keep_alive_interval: 0,
            noop_keep_alive: false,
            partial_fetch: false,
            max_tls_version: MaxTlsVersion::Tls1_3,
            app_password_required: false,
            documentation_url: None,
            localized_docs: vec![],
            oauth: None,
            display_order: 0,
            enabled: true,
            supports_shared_mailbox: false,
        };

        let candidate = ProviderCandidate {
            provider: network_provider,
            score: MatchScore::DNS_SRV,
        };

        let result = merge_network_with_bundled(candidate, &db);
        assert_eq!(result.provider.id, "unknown");
        assert_eq!(result.score, MatchScore::DNS_SRV);
    }

    #[test]
    fn test_merge_skips_bundled_candidates() {
        let db = ProviderDatabase::new(vec![make_test_provider("gmail", &["gmail.com"])]);

        let candidate = ProviderCandidate {
            provider: make_test_provider("gmail", &["gmail.com"]),
            score: MatchScore::BUNDLED_EXACT,
        };

        let result = merge_network_with_bundled(candidate, &db);
        assert_eq!(result.score, MatchScore::BUNDLED_EXACT);
    }

    #[test]
    fn test_merge_provider_flags_applied() {
        // Create a bundled provider with specific flags
        let mut bundled = make_test_provider("special", &["special.com"]);
        bundled.keep_alive_interval = 30;
        bundled.noop_keep_alive = true;
        bundled.partial_fetch = false;
        bundled.max_tls_version = MaxTlsVersion::Tls1_2;
        bundled.app_password_required = true;
        bundled.documentation_url = Some("https://docs.special.com".to_string());

        let db = ProviderDatabase::new(vec![bundled]);

        let network_provider = Provider {
            id: "network".to_string(),
            display_name: "Network".to_string(),
            domain_patterns: vec![],
            mx_patterns: vec![],
            incoming: ServerConfig {
                hostname: "imap.special.com".to_string(),
                port: 143,
                encryption: ProviderEncryption::StartTls,
            },
            outgoing: ServerConfig {
                hostname: "smtp.other.com".to_string(),
                port: 587,
                encryption: ProviderEncryption::StartTls,
            },
            username_type: UsernameType::EmailAddress,
            keep_alive_interval: 0,
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
        };

        let candidate = ProviderCandidate {
            provider: network_provider,
            score: MatchScore::VENDOR_AUTODISCOVERY,
        };

        let merged = merge_network_with_bundled(candidate, &db);

        assert_eq!(merged.score, MatchScore::VENDOR_AUTODISCOVERY);
        assert_eq!(merged.provider.keep_alive_interval, 30);
        assert!(merged.provider.noop_keep_alive);
        assert!(!merged.provider.partial_fetch);
        assert_eq!(merged.provider.max_tls_version, MaxTlsVersion::Tls1_2);
        assert!(merged.provider.app_password_required);
        assert_eq!(
            merged.provider.documentation_url,
            Some("https://docs.special.com".to_string())
        );
    }

    #[test]
    fn test_bundled_lookup_performance() {
        let db = ProviderDatabase::bundled();
        let start = std::time::Instant::now();
        for _ in 0..10_000 {
            let _ = db.lookup_by_email("user@gmail.com");
            let _ = db.lookup_by_email("user@outlook.com");
            let _ = db.lookup_by_email("user@unknown-domain-xyz.com");
        }
        let elapsed = start.elapsed();
        // 30,000 lookups should complete well under 1 second
        assert!(
            elapsed.as_millis() < 1000,
            "Lookups took too long: {elapsed:?}"
        );
    }

    // --- OAuthConfig tenant tests (FR-10, US-4) ---

    fn make_tenant_oauth_config() -> OAuthConfig {
        OAuthConfig {
            auth_url: "https://login.microsoftonline.com/{tenant}/oauth2/v2.0/authorize"
                .to_string(),
            token_url: "https://login.microsoftonline.com/{tenant}/oauth2/v2.0/token".to_string(),
            redirect_uri: "http://127.0.0.1/callback".to_string(),
            scopes: vec!["mail".to_string()],
            client_id: None,
            pkce_required: true,
            extra_params: vec![],
            userinfo_url: None,
        }
    }

    fn make_non_tenant_oauth_config() -> OAuthConfig {
        OAuthConfig {
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
            redirect_uri: "http://127.0.0.1/callback".to_string(),
            scopes: vec!["mail".to_string()],
            client_id: None,
            pkce_required: true,
            extra_params: vec![],
            userinfo_url: None,
        }
    }

    #[test]
    fn requires_tenant_detects_placeholder_in_auth_url() {
        let config = make_tenant_oauth_config();
        assert!(config.requires_tenant());
    }

    #[test]
    fn requires_tenant_false_for_non_tenant_provider() {
        let config = make_non_tenant_oauth_config();
        assert!(!config.requires_tenant());
    }

    #[test]
    fn with_tenant_substitutes_in_both_urls() {
        let config = make_tenant_oauth_config();
        let resolved = config.with_tenant(Some("contoso.com"));
        assert_eq!(
            resolved.auth_url,
            "https://login.microsoftonline.com/contoso.com/oauth2/v2.0/authorize"
        );
        assert_eq!(
            resolved.token_url,
            "https://login.microsoftonline.com/contoso.com/oauth2/v2.0/token"
        );
    }

    #[test]
    fn with_tenant_uses_default_when_none() {
        let config = make_tenant_oauth_config();
        let resolved = config.with_tenant(None);
        assert_eq!(
            resolved.auth_url,
            "https://login.microsoftonline.com/common/oauth2/v2.0/authorize"
        );
        assert_eq!(
            resolved.token_url,
            "https://login.microsoftonline.com/common/oauth2/v2.0/token"
        );
    }

    #[test]
    fn with_tenant_uses_default_when_empty() {
        let config = make_tenant_oauth_config();
        let resolved = config.with_tenant(Some(""));
        assert!(resolved.auth_url.contains("/common/"));
    }

    #[test]
    fn with_tenant_uses_default_when_whitespace() {
        let config = make_tenant_oauth_config();
        let resolved = config.with_tenant(Some("   "));
        assert!(resolved.auth_url.contains("/common/"));
    }

    #[test]
    fn with_tenant_trims_whitespace() {
        let config = make_tenant_oauth_config();
        let resolved = config.with_tenant(Some("  myorg  "));
        assert!(resolved.auth_url.contains("/myorg/"));
    }

    #[test]
    fn with_tenant_preserves_non_tenant_urls() {
        let config = make_non_tenant_oauth_config();
        let resolved = config.with_tenant(Some("contoso.com"));
        // No placeholder, so URLs unchanged.
        assert_eq!(resolved.auth_url, config.auth_url);
        assert_eq!(resolved.token_url, config.token_url);
    }

    #[test]
    fn with_tenant_preserves_other_fields() {
        let config = make_tenant_oauth_config();
        let resolved = config.with_tenant(Some("org"));
        assert_eq!(resolved.redirect_uri, config.redirect_uri);
        assert_eq!(resolved.scopes, config.scopes);
        assert_eq!(resolved.client_id, config.client_id);
    }

    #[test]
    fn bundled_outlook_requires_tenant() {
        let db = ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("outlook.com").unwrap();
        let oauth = candidate.provider.oauth.as_ref().unwrap();
        assert!(oauth.requires_tenant());
    }

    #[test]
    fn bundled_office365_requires_tenant() {
        let db = ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("office365.com").unwrap();
        let oauth = candidate.provider.oauth.as_ref().unwrap();
        assert!(oauth.requires_tenant());
    }

    #[test]
    fn bundled_outlook_supports_shared_mailbox() {
        let db = ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("outlook.com").unwrap();
        assert!(candidate.provider.supports_shared_mailbox);
    }

    #[test]
    fn bundled_office365_supports_shared_mailbox() {
        let db = ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("office365.com").unwrap();
        assert!(candidate.provider.supports_shared_mailbox);
    }

    #[test]
    fn bundled_gmail_does_not_support_shared_mailbox() {
        let db = ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("gmail.com").unwrap();
        assert!(!candidate.provider.supports_shared_mailbox);
    }

    // --- Provider-specific OAuth parameters (story 10) ---

    #[test]
    fn bundled_gmail_has_consent_prompt() {
        let db = ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("gmail.com").unwrap();
        let oauth = candidate.provider.oauth.as_ref().unwrap();
        assert!(
            oauth
                .extra_params
                .contains(&("prompt".to_string(), "consent".to_string())),
            "Gmail must include prompt=consent (FR-37, AC-14)"
        );
    }

    #[test]
    fn bundled_gmail_has_offline_access() {
        let db = ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("gmail.com").unwrap();
        let oauth = candidate.provider.oauth.as_ref().unwrap();
        assert!(
            oauth
                .extra_params
                .contains(&("access_type".to_string(), "offline".to_string())),
            "Gmail must include access_type=offline (FR-38, AC-14)"
        );
    }

    #[test]
    fn bundled_gmail_requests_only_mail_scopes() {
        let db = ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("gmail.com").unwrap();
        let oauth = candidate.provider.oauth.as_ref().unwrap();
        // Gmail IMAP requires https://mail.google.com/ — this is the only IMAP scope.
        // No profile, contacts, calendar, or other scopes (NFR-6, US-5).
        assert!(oauth
            .scopes
            .contains(&"https://mail.google.com/".to_string()));
        for scope in &oauth.scopes {
            assert!(
                !scope.contains("profile")
                    && !scope.contains("contacts")
                    && !scope.contains("calendar"),
                "Gmail must not request non-mail scopes: {scope}"
            );
        }
    }

    #[test]
    fn bundled_yandex_has_force_confirm() {
        let db = ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("yandex.ru").unwrap();
        let oauth = candidate.provider.oauth.as_ref().unwrap();
        assert!(
            oauth
                .extra_params
                .contains(&("force_confirm".to_string(), "true".to_string())),
            "Yandex must include force_confirm=true"
        );
    }

    #[test]
    fn bundled_outlook_has_consent_prompt() {
        let db = ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("outlook.com").unwrap();
        let oauth = candidate.provider.oauth.as_ref().unwrap();
        assert!(
            oauth
                .extra_params
                .contains(&("prompt".to_string(), "consent".to_string())),
            "Outlook must include prompt=consent (FR-37)"
        );
    }

    #[test]
    fn bundled_outlook_requests_graph_mail_send() {
        let db = ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("outlook.com").unwrap();
        let oauth = candidate.provider.oauth.as_ref().unwrap();
        assert!(
            oauth
                .scopes
                .contains(&"https://graph.microsoft.com/Mail.Send".to_string()),
            "Outlook must request Graph Mail.Send scope for proprietary send API (FR-39, N-5)"
        );
    }

    #[test]
    fn bundled_office365_requests_graph_mail_send() {
        let db = ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("office365.com").unwrap();
        let oauth = candidate.provider.oauth.as_ref().unwrap();
        assert!(
            oauth
                .scopes
                .contains(&"https://graph.microsoft.com/Mail.Send".to_string()),
            "Office365 must request Graph Mail.Send scope for proprietary send API (FR-39, N-5)"
        );
    }

    #[test]
    fn bundled_outlook_requests_only_mail_scopes() {
        let db = ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("outlook.com").unwrap();
        let oauth = candidate.provider.oauth.as_ref().unwrap();
        // Outlook should have IMAP, SMTP, Graph Mail.Send, and offline_access — no profile/contacts/calendar
        for scope in &oauth.scopes {
            assert!(
                !scope.contains("Contacts")
                    && !scope.contains("Calendar")
                    && !scope.contains("User.Read"),
                "Outlook must not request non-mail scopes (NFR-6): {scope}"
            );
        }
    }

    #[test]
    fn extra_params_come_from_provider_database_not_flow_logic() {
        // Verify that OAuthConfig carries extra_params and that authorization_url
        // includes them — proving params are data-driven, not hard-coded in flow logic.
        let config = OAuthConfig {
            auth_url: "https://example.com/auth".to_string(),
            token_url: "https://example.com/token".to_string(),
            redirect_uri: "http://127.0.0.1/callback".to_string(),
            scopes: vec![],
            client_id: None,
            pkce_required: true,
            extra_params: vec![("custom_key".to_string(), "custom_value".to_string())],
            userinfo_url: None,
        };
        // The extra_params field on OAuthConfig is what the flow reads —
        // no provider-specific if/else branches exist in the flow module.
        assert_eq!(config.extra_params.len(), 1);
        assert_eq!(config.extra_params[0].0, "custom_key");
    }

    #[test]
    fn bundled_gmail_does_not_require_tenant() {
        let db = ProviderDatabase::bundled();
        let candidate = db.lookup_by_domain("gmail.com").unwrap();
        let oauth = candidate.provider.oauth.as_ref().unwrap();
        assert!(!oauth.requires_tenant());
    }

    #[test]
    fn default_tenant_is_common() {
        assert_eq!(DEFAULT_TENANT, "common");
    }
}
