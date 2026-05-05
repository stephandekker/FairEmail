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
    pub scopes: Vec<String>,
    pub client_id: Option<String>,
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
}
