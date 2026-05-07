use super::provider::{
    MatchScore, Provider, ProviderCandidate, ProviderDatabase, ProviderEncryption, ServerConfig,
};

/// Known NS patterns mapped to provider IDs for DNS NS matching (FR-10.2).
const NS_PATTERNS: &[(&str, &str)] = &[
    ("google.com", "gmail"),
    ("googledomains.com", "gmail"),
    ("outlook.com", "outlook"),
    ("microsoft.com", "office365"),
    ("yahoo.com", "yahoo"),
    ("yahoodns.net", "yahoo"),
    ("icloud.com", "icloud"),
    ("apple.com", "icloud"),
    ("yandex.ru", "yandex"),
    ("yandex.net", "yandex"),
    ("mail.ru", "mailru"),
    ("zoho.com", "zoho"),
];

/// A trait abstracting DNS resolution so we can test without real network calls.
pub trait DnsResolver {
    /// Look up NS records for a domain. Returns name-server hostnames.
    fn lookup_ns(&self, domain: &str) -> Result<Vec<String>, DnsError>;
    /// Look up MX records for a domain. Returns (priority, exchange) pairs.
    fn lookup_mx(&self, domain: &str) -> Result<Vec<(u16, String)>, DnsError>;
    /// Look up SRV records for a service name. Returns (priority, weight, port, target) tuples.
    fn lookup_srv(&self, name: &str) -> Result<Vec<SrvRecord>, DnsError>;
}

/// A single SRV record result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SrvRecord {
    pub priority: u16,
    pub weight: u16,
    pub port: u16,
    pub target: String,
}

/// Errors from DNS resolution.
#[derive(Debug, Clone, thiserror::Error)]
pub enum DnsError {
    #[error("no records found")]
    NoRecords,
    #[error("DNS lookup failed: {0}")]
    LookupFailed(String),
}

/// Result of the full DNS discovery pipeline.
#[derive(Debug, Clone)]
pub struct DnsDiscoveryResult {
    /// All candidates found, sorted by score (highest first).
    pub candidates: Vec<ProviderCandidate>,
}

/// Run all DNS discovery strategies for the given domain.
/// Returns candidates ordered by confidence score (highest first).
pub fn discover_by_dns(
    domain: &str,
    resolver: &dyn DnsResolver,
    provider_db: &ProviderDatabase,
) -> DnsDiscoveryResult {
    let mut candidates = Vec::new();

    // Strategy 1: NS record match (FR-10.2)
    if let Some(candidate) = discover_by_ns(domain, resolver, provider_db) {
        candidates.push(candidate);
    }

    // Strategy 2: MX record match (FR-10.3)
    if let Some(candidate) = discover_by_mx(domain, resolver, provider_db) {
        // Only add if not already found by NS with higher score
        if !candidates
            .iter()
            .any(|c| c.provider.id == candidate.provider.id)
        {
            candidates.push(candidate);
        }
    }

    // Strategy 3: RFC 6186 SRV discovery (FR-10.4)
    if let Some(candidate) = discover_by_srv(domain, resolver) {
        candidates.push(candidate);
    }

    candidates.sort_by(|a, b| b.score.value().partial_cmp(&a.score.value()).unwrap());

    DnsDiscoveryResult { candidates }
}

/// FR-10.2: Look up NS records and match against known provider patterns.
/// If a match is found, attempt SRV discovery first, falling back to the matched provider entry.
pub(crate) fn discover_by_ns(
    domain: &str,
    resolver: &dyn DnsResolver,
    provider_db: &ProviderDatabase,
) -> Option<ProviderCandidate> {
    let ns_records = resolver.lookup_ns(domain).ok()?;

    for ns in &ns_records {
        let ns_lower = ns.to_lowercase();
        for &(pattern, provider_id) in NS_PATTERNS {
            if ns_lower.ends_with(pattern) || ns_lower.ends_with(&format!("{pattern}.")) {
                // Found NS match — try SRV first per spec
                if let Some(srv_candidate) = discover_by_srv(domain, resolver) {
                    return Some(ProviderCandidate {
                        provider: srv_candidate.provider,
                        score: MatchScore::DNS_NS, // Upgrade to NS score since NS confirmed provider
                    });
                }
                // Fall back to the matched provider from bundled database
                if let Some(provider) = find_provider_by_id(provider_db, provider_id) {
                    return Some(ProviderCandidate {
                        provider,
                        score: MatchScore::DNS_NS,
                    });
                }
            }
        }
    }

    None
}

/// FR-10.3: Look up MX records and match against known provider MX patterns.
///
/// When the device is offline the DNS lookup returns an error which is
/// silently converted to `None` — no network error is surfaced.
pub(crate) fn discover_by_mx(
    domain: &str,
    resolver: &dyn DnsResolver,
    provider_db: &ProviderDatabase,
) -> Option<ProviderCandidate> {
    let mx_records = resolver.lookup_mx(domain).ok()?;
    provider_db.lookup_by_mx_records(&mx_records)
}

/// Check if an MX hostname matches a pattern (supports *.domain.com wildcards).
#[cfg(test)]
fn matches_mx_pattern(exchange: &str, pattern: &str) -> bool {
    let pattern_lower = pattern.to_lowercase();
    if let Some(suffix) = pattern_lower.strip_prefix("*.") {
        // Wildcard: exchange must end with the suffix (and have at least one more label)
        exchange.ends_with(suffix)
            && exchange.len() > suffix.len()
            && exchange.as_bytes()[exchange.len() - suffix.len() - 1] == b'.'
    } else {
        // Exact match or root-domain equivalence
        exchange == pattern_lower || exchange.ends_with(&format!(".{pattern_lower}"))
    }
}

/// FR-10.4: RFC 6186 SRV record discovery.
/// Queries _imaps._tcp, _imap._tcp, _submissions._tcp, _submission._tcp.
pub(crate) fn discover_by_srv(
    domain: &str,
    resolver: &dyn DnsResolver,
) -> Option<ProviderCandidate> {
    let incoming = resolve_imap_srv(domain, resolver)?;
    let outgoing = resolve_smtp_srv(domain, resolver).unwrap_or(ServerConfig {
        hostname: format!("smtp.{domain}"),
        port: 465,
        encryption: ProviderEncryption::SslTls,
    });

    let provider = Provider {
        id: format!("srv-{domain}"),
        display_name: domain.to_string(),
        domain_patterns: vec![domain.to_string()],
        mx_patterns: vec![],
        incoming,
        outgoing,
        username_type: super::provider::UsernameType::EmailAddress,
        keep_alive_interval: 15,
        noop_keep_alive: false,
        partial_fetch: true,
        max_tls_version: super::provider::MaxTlsVersion::Tls1_3,
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
        graph: None,
        debug_only: false,
        variant_of: None,
    };

    Some(ProviderCandidate {
        provider,
        score: MatchScore::DNS_SRV,
    })
}

/// Resolve IMAP server from SRV records (prefer _imaps._tcp over _imap._tcp).
fn resolve_imap_srv(domain: &str, resolver: &dyn DnsResolver) -> Option<ServerConfig> {
    // Try IMAPS (implicit TLS) first
    if let Ok(records) = resolver.lookup_srv(&format!("_imaps._tcp.{domain}")) {
        if let Some(best) = pick_best_srv(&records) {
            return Some(ServerConfig {
                hostname: best.target.trim_end_matches('.').to_string(),
                port: best.port,
                encryption: ProviderEncryption::SslTls,
            });
        }
    }

    // Fall back to IMAP (STARTTLS)
    if let Ok(records) = resolver.lookup_srv(&format!("_imap._tcp.{domain}")) {
        if let Some(best) = pick_best_srv(&records) {
            return Some(ServerConfig {
                hostname: best.target.trim_end_matches('.').to_string(),
                port: best.port,
                encryption: ProviderEncryption::StartTls,
            });
        }
    }

    None
}

/// Resolve SMTP server from SRV records (prefer _submissions._tcp over _submission._tcp).
fn resolve_smtp_srv(domain: &str, resolver: &dyn DnsResolver) -> Option<ServerConfig> {
    // Try submissions (implicit TLS) first
    if let Ok(records) = resolver.lookup_srv(&format!("_submissions._tcp.{domain}")) {
        if let Some(best) = pick_best_srv(&records) {
            return Some(ServerConfig {
                hostname: best.target.trim_end_matches('.').to_string(),
                port: best.port,
                encryption: ProviderEncryption::SslTls,
            });
        }
    }

    // Fall back to submission (STARTTLS)
    if let Ok(records) = resolver.lookup_srv(&format!("_submission._tcp.{domain}")) {
        if let Some(best) = pick_best_srv(&records) {
            return Some(ServerConfig {
                hostname: best.target.trim_end_matches('.').to_string(),
                port: best.port,
                encryption: ProviderEncryption::StartTls,
            });
        }
    }

    None
}

/// Pick the best SRV record (lowest priority, then highest weight).
/// Excludes records with target "." (indicates service not available).
fn pick_best_srv(records: &[SrvRecord]) -> Option<&SrvRecord> {
    records
        .iter()
        .filter(|r| r.target != "." && !r.target.is_empty())
        .min_by_key(|r| (r.priority, std::cmp::Reverse(r.weight)))
}

/// Find a provider by ID in the database.
fn find_provider_by_id(db: &ProviderDatabase, id: &str) -> Option<Provider> {
    db.providers()
        .iter()
        .find(|p| p.id == id && p.enabled)
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A mock DNS resolver for testing.
    struct MockResolver {
        ns_records: Vec<String>,
        mx_records: Vec<(u16, String)>,
        srv_records: std::collections::HashMap<String, Vec<SrvRecord>>,
    }

    impl MockResolver {
        fn new() -> Self {
            Self {
                ns_records: vec![],
                mx_records: vec![],
                srv_records: std::collections::HashMap::new(),
            }
        }

        fn with_ns(mut self, records: &[&str]) -> Self {
            self.ns_records = records.iter().map(|s| s.to_string()).collect();
            self
        }

        fn with_mx(mut self, records: &[(u16, &str)]) -> Self {
            self.mx_records = records.iter().map(|(p, e)| (*p, e.to_string())).collect();
            self
        }

        fn with_srv(mut self, name: &str, records: Vec<SrvRecord>) -> Self {
            self.srv_records.insert(name.to_string(), records);
            self
        }
    }

    impl DnsResolver for MockResolver {
        fn lookup_ns(&self, _domain: &str) -> Result<Vec<String>, DnsError> {
            if self.ns_records.is_empty() {
                Err(DnsError::NoRecords)
            } else {
                Ok(self.ns_records.clone())
            }
        }

        fn lookup_mx(&self, _domain: &str) -> Result<Vec<(u16, String)>, DnsError> {
            if self.mx_records.is_empty() {
                Err(DnsError::NoRecords)
            } else {
                Ok(self.mx_records.clone())
            }
        }

        fn lookup_srv(&self, name: &str) -> Result<Vec<SrvRecord>, DnsError> {
            self.srv_records
                .get(name)
                .cloned()
                .ok_or(DnsError::NoRecords)
        }
    }

    fn test_db() -> ProviderDatabase {
        ProviderDatabase::bundled()
    }

    // --- Score ordering tests (FR-11) ---

    #[test]
    fn test_dns_scores_below_bundled() {
        assert!(MatchScore::BUNDLED_EXACT > MatchScore::DNS_NS);
        assert!(MatchScore::BUNDLED_WILDCARD > MatchScore::DNS_NS);
        assert!(MatchScore::NETWORK_MAX >= MatchScore::DNS_NS);
    }

    #[test]
    fn test_dns_score_ordering() {
        // NS > MX > SRV
        assert!(MatchScore::DNS_NS > MatchScore::DNS_MX);
        assert!(MatchScore::DNS_MX > MatchScore::DNS_SRV);
    }

    // --- NS discovery tests (FR-10.2) ---

    #[test]
    fn test_ns_match_google() {
        let resolver = MockResolver::new().with_ns(&["ns1.google.com", "ns2.google.com"]);
        let db = test_db();

        let result = discover_by_ns("customdomain.com", &resolver, &db);
        assert!(result.is_some());
        let candidate = result.unwrap();
        assert_eq!(candidate.provider.id, "gmail");
        assert_eq!(candidate.score, MatchScore::DNS_NS);
    }

    #[test]
    fn test_ns_match_outlook() {
        let resolver = MockResolver::new().with_ns(&["ns1.microsoft.com", "ns2.microsoft.com"]);
        let db = test_db();

        let result = discover_by_ns("mycompany.com", &resolver, &db);
        assert!(result.is_some());
        let candidate = result.unwrap();
        assert_eq!(candidate.provider.id, "office365");
        assert_eq!(candidate.score, MatchScore::DNS_NS);
    }

    #[test]
    fn test_ns_no_match() {
        let resolver = MockResolver::new().with_ns(&["ns1.cloudflare.com", "ns2.cloudflare.com"]);
        let db = test_db();

        let result = discover_by_ns("example.com", &resolver, &db);
        assert!(result.is_none());
    }

    #[test]
    fn test_ns_prefers_srv_when_available() {
        let resolver = MockResolver::new().with_ns(&["ns1.google.com"]).with_srv(
            "_imaps._tcp.example.com",
            vec![SrvRecord {
                priority: 10,
                weight: 1,
                port: 993,
                target: "imap.custom.example.com.".to_string(),
            }],
        );
        let db = test_db();

        let result = discover_by_ns("example.com", &resolver, &db);
        assert!(result.is_some());
        let candidate = result.unwrap();
        // Should use SRV-discovered host but with NS score
        assert_eq!(candidate.score, MatchScore::DNS_NS);
        assert_eq!(
            candidate.provider.incoming.hostname,
            "imap.custom.example.com"
        );
    }

    // --- MX discovery tests (FR-10.3) ---

    #[test]
    fn test_mx_match_google() {
        let resolver = MockResolver::new().with_mx(&[
            (10, "alt1.aspmx.l.google.com"),
            (20, "alt2.aspmx.l.google.com"),
        ]);
        let db = test_db();

        let result = discover_by_mx("customdomain.com", &resolver, &db);
        assert!(result.is_some());
        let candidate = result.unwrap();
        assert_eq!(candidate.provider.id, "gmail");
        assert_eq!(candidate.score, MatchScore::DNS_MX);
    }

    #[test]
    fn test_mx_match_outlook() {
        let resolver = MockResolver::new().with_mx(&[(10, "mail.protection.outlook.com")]);
        let db = test_db();

        let result = discover_by_mx("company.com", &resolver, &db);
        assert!(result.is_some());
        let candidate = result.unwrap();
        // office365 has *.outlook.com and *.protection.outlook.com mx patterns
        assert!(
            candidate.provider.id == "office365" || candidate.provider.id == "outlook",
            "Got provider: {}",
            candidate.provider.id
        );
        assert_eq!(candidate.score, MatchScore::DNS_MX);
    }

    #[test]
    fn test_mx_match_yahoo() {
        let resolver = MockResolver::new().with_mx(&[(10, "mx1.biz.mail.yahoodns.net")]);
        let db = test_db();

        let result = discover_by_mx("mybusiness.com", &resolver, &db);
        assert!(result.is_some());
        let candidate = result.unwrap();
        // Yahoo or AOL both use yahoodns.net
        assert!(
            candidate.provider.id == "yahoo" || candidate.provider.id == "aol",
            "Got provider: {}",
            candidate.provider.id
        );
        assert_eq!(candidate.score, MatchScore::DNS_MX);
    }

    #[test]
    fn test_mx_no_match() {
        let resolver = MockResolver::new().with_mx(&[(10, "mail.unknownprovider.example.org")]);
        let db = test_db();

        let result = discover_by_mx("example.org", &resolver, &db);
        assert!(result.is_none());
    }

    #[test]
    fn test_mx_trailing_dot() {
        let resolver = MockResolver::new().with_mx(&[(10, "mx.mail.yahoo.com.")]);
        let db = test_db();

        // Should handle trailing dot gracefully
        let result = discover_by_mx("somedomain.com", &resolver, &db);
        // yahoo uses *.yahoodns.net, not *.yahoo.com — so this might not match
        // but at minimum it shouldn't panic
        let _ = result;
    }

    // --- SRV discovery tests (FR-10.4) ---

    #[test]
    fn test_srv_imaps_discovery() {
        let resolver = MockResolver::new()
            .with_srv(
                "_imaps._tcp.example.com",
                vec![SrvRecord {
                    priority: 0,
                    weight: 1,
                    port: 993,
                    target: "imap.example.com.".to_string(),
                }],
            )
            .with_srv(
                "_submissions._tcp.example.com",
                vec![SrvRecord {
                    priority: 0,
                    weight: 1,
                    port: 465,
                    target: "smtp.example.com.".to_string(),
                }],
            );

        let result = discover_by_srv("example.com", &resolver);
        assert!(result.is_some());
        let candidate = result.unwrap();
        assert_eq!(candidate.provider.incoming.hostname, "imap.example.com");
        assert_eq!(candidate.provider.incoming.port, 993);
        assert_eq!(
            candidate.provider.incoming.encryption,
            ProviderEncryption::SslTls
        );
        assert_eq!(candidate.provider.outgoing.hostname, "smtp.example.com");
        assert_eq!(candidate.provider.outgoing.port, 465);
        assert_eq!(
            candidate.provider.outgoing.encryption,
            ProviderEncryption::SslTls
        );
        assert_eq!(candidate.score, MatchScore::DNS_SRV);
    }

    #[test]
    fn test_srv_imap_starttls_fallback() {
        let resolver = MockResolver::new().with_srv(
            "_imap._tcp.example.com",
            vec![SrvRecord {
                priority: 0,
                weight: 1,
                port: 143,
                target: "mail.example.com.".to_string(),
            }],
        );

        let result = discover_by_srv("example.com", &resolver);
        assert!(result.is_some());
        let candidate = result.unwrap();
        assert_eq!(candidate.provider.incoming.hostname, "mail.example.com");
        assert_eq!(candidate.provider.incoming.port, 143);
        assert_eq!(
            candidate.provider.incoming.encryption,
            ProviderEncryption::StartTls
        );
        // No SMTP SRV, should fall back to smtp.domain:465
        assert_eq!(candidate.provider.outgoing.hostname, "smtp.example.com");
        assert_eq!(candidate.provider.outgoing.port, 465);
    }

    #[test]
    fn test_srv_submission_starttls() {
        let resolver = MockResolver::new()
            .with_srv(
                "_imaps._tcp.example.com",
                vec![SrvRecord {
                    priority: 0,
                    weight: 1,
                    port: 993,
                    target: "imap.example.com.".to_string(),
                }],
            )
            .with_srv(
                "_submission._tcp.example.com",
                vec![SrvRecord {
                    priority: 0,
                    weight: 1,
                    port: 587,
                    target: "smtp.example.com.".to_string(),
                }],
            );

        let result = discover_by_srv("example.com", &resolver);
        assert!(result.is_some());
        let candidate = result.unwrap();
        assert_eq!(candidate.provider.outgoing.port, 587);
        assert_eq!(
            candidate.provider.outgoing.encryption,
            ProviderEncryption::StartTls
        );
    }

    #[test]
    fn test_srv_no_imap_records() {
        let resolver = MockResolver::new().with_srv(
            "_submissions._tcp.example.com",
            vec![SrvRecord {
                priority: 0,
                weight: 1,
                port: 465,
                target: "smtp.example.com.".to_string(),
            }],
        );

        // No IMAP SRV → no candidate (IMAP is required)
        let result = discover_by_srv("example.com", &resolver);
        assert!(result.is_none());
    }

    #[test]
    fn test_srv_dot_target_excluded() {
        let resolver = MockResolver::new().with_srv(
            "_imaps._tcp.example.com",
            vec![SrvRecord {
                priority: 0,
                weight: 1,
                port: 993,
                target: ".".to_string(),
            }],
        );

        // Target "." means service not available
        let result = discover_by_srv("example.com", &resolver);
        assert!(result.is_none());
    }

    #[test]
    fn test_srv_picks_lowest_priority() {
        let resolver = MockResolver::new().with_srv(
            "_imaps._tcp.example.com",
            vec![
                SrvRecord {
                    priority: 20,
                    weight: 1,
                    port: 993,
                    target: "backup.example.com.".to_string(),
                },
                SrvRecord {
                    priority: 5,
                    weight: 1,
                    port: 993,
                    target: "primary.example.com.".to_string(),
                },
            ],
        );

        let result = discover_by_srv("example.com", &resolver);
        assert!(result.is_some());
        assert_eq!(
            result.unwrap().provider.incoming.hostname,
            "primary.example.com"
        );
    }

    // --- Full pipeline tests ---

    #[test]
    fn test_discover_by_dns_all_strategies() {
        let resolver = MockResolver::new()
            .with_ns(&["ns1.google.com"])
            .with_mx(&[(10, "aspmx.l.google.com")])
            .with_srv(
                "_imaps._tcp.example.com",
                vec![SrvRecord {
                    priority: 0,
                    weight: 1,
                    port: 993,
                    target: "imap.example.com.".to_string(),
                }],
            );
        let db = test_db();

        let result = discover_by_dns("example.com", &resolver, &db);
        // Should have candidates, with NS-based first (highest score)
        assert!(!result.candidates.is_empty());
        assert_eq!(result.candidates[0].score, MatchScore::DNS_NS);
    }

    #[test]
    fn test_discover_by_dns_only_srv() {
        let resolver = MockResolver::new()
            .with_srv(
                "_imaps._tcp.custom.example.org",
                vec![SrvRecord {
                    priority: 0,
                    weight: 1,
                    port: 993,
                    target: "mail.custom.example.org.".to_string(),
                }],
            )
            .with_srv(
                "_submissions._tcp.custom.example.org",
                vec![SrvRecord {
                    priority: 0,
                    weight: 1,
                    port: 465,
                    target: "smtp.custom.example.org.".to_string(),
                }],
            );
        let db = test_db();

        let result = discover_by_dns("custom.example.org", &resolver, &db);
        assert_eq!(result.candidates.len(), 1);
        let candidate = &result.candidates[0];
        assert_eq!(candidate.score, MatchScore::DNS_SRV);
        assert_eq!(
            candidate.provider.incoming.hostname,
            "mail.custom.example.org"
        );
        assert_eq!(
            candidate.provider.outgoing.hostname,
            "smtp.custom.example.org"
        );
    }

    #[test]
    fn test_discover_by_dns_no_records() {
        let resolver = MockResolver::new();
        let db = test_db();

        let result = discover_by_dns("nonexistent.example.com", &resolver, &db);
        assert!(result.candidates.is_empty());
    }

    // --- AC-4: Custom domain with SRV records produces valid settings ---

    #[test]
    fn test_custom_domain_srv_produces_valid_settings() {
        let resolver = MockResolver::new()
            .with_srv(
                "_imaps._tcp.mycompany.io",
                vec![SrvRecord {
                    priority: 10,
                    weight: 5,
                    port: 993,
                    target: "imap.mycompany.io.".to_string(),
                }],
            )
            .with_srv(
                "_submissions._tcp.mycompany.io",
                vec![SrvRecord {
                    priority: 10,
                    weight: 5,
                    port: 465,
                    target: "smtp.mycompany.io.".to_string(),
                }],
            );
        let db = test_db();

        let result = discover_by_dns("mycompany.io", &resolver, &db);
        assert_eq!(result.candidates.len(), 1);

        let candidate = &result.candidates[0];
        // Valid IMAP settings
        assert_eq!(candidate.provider.incoming.hostname, "imap.mycompany.io");
        assert_eq!(candidate.provider.incoming.port, 993);
        assert_eq!(
            candidate.provider.incoming.encryption,
            ProviderEncryption::SslTls
        );
        // Valid SMTP settings
        assert_eq!(candidate.provider.outgoing.hostname, "smtp.mycompany.io");
        assert_eq!(candidate.provider.outgoing.port, 465);
        assert_eq!(
            candidate.provider.outgoing.encryption,
            ProviderEncryption::SslTls
        );
    }

    // --- Helper function tests ---

    #[test]
    fn test_matches_mx_pattern_wildcard() {
        assert!(matches_mx_pattern("mx1.google.com", "*.google.com"));
        assert!(matches_mx_pattern(
            "alt1.aspmx.l.google.com",
            "*.google.com"
        ));
        assert!(!matches_mx_pattern("google.com", "*.google.com"));
        assert!(!matches_mx_pattern("notgoogle.com", "*.google.com"));
    }

    #[test]
    fn test_matches_mx_pattern_exact() {
        assert!(matches_mx_pattern("mail.example.com", "mail.example.com"));
        assert!(!matches_mx_pattern("other.example.com", "mail.example.com"));
    }
}
