// Domain-based auto-configuration for inbound server (FR-23 through FR-28).
//
// Orchestrates discovery strategies in order:
// 1. Bundled provider database by domain match
// 2. DNS NS record lookup
// 3. DNS SRV records (RFC 6186)
// 4. DNS MX record lookup
// 5. Well-known auto-configuration XML endpoints (ISPDB + vendor)
// 6. Port scanning as a last resort

use super::dns_discovery::DnsResolver;
use super::ispdb_discovery::HttpClient;
use super::port_scanning::PortProber;
use super::provider::{ProviderDatabase, ProviderEncryption};

/// Result of a successful auto-config discovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoConfigResult {
    pub hostname: String,
    pub port: u16,
    pub encryption: ProviderEncryption,
}

/// Error when all auto-config strategies fail.
#[derive(Debug, Clone, thiserror::Error)]
pub enum AutoConfigError {
    #[error("Could not determine server settings for this domain")]
    AllStrategiesFailed,
    #[error("Domain is empty")]
    EmptyDomain,
}

/// Attempt to discover inbound server settings for the given domain.
///
/// Tries each strategy in order and returns the first successful result.
/// On failure, returns an error and no fields should be modified.
pub(crate) fn discover_inbound(
    domain: &str,
    provider_db: &ProviderDatabase,
    resolver: &dyn DnsResolver,
    http_client: &dyn HttpClient,
    port_prober: &dyn PortProber,
) -> Result<AutoConfigResult, AutoConfigError> {
    let domain = domain.trim().to_lowercase();
    if domain.is_empty() {
        return Err(AutoConfigError::EmptyDomain);
    }

    // 1. Bundled provider database
    if let Some(candidate) = provider_db.lookup_by_domain(&domain) {
        return Ok(to_result(&candidate.provider.incoming));
    }

    // 2. DNS NS record lookup
    if let Some(candidate) = super::dns_discovery::discover_by_ns(&domain, resolver, provider_db) {
        return Ok(to_result(&candidate.provider.incoming));
    }

    // 3. DNS SRV records (RFC 6186)
    if let Some(candidate) = super::dns_discovery::discover_by_srv(&domain, resolver) {
        return Ok(to_result(&candidate.provider.incoming));
    }

    // 4. DNS MX record lookup
    if let Some(candidate) = super::dns_discovery::discover_by_mx(&domain, resolver, provider_db) {
        return Ok(to_result(&candidate.provider.incoming));
    }

    // 5. Well-known auto-configuration XML endpoints
    if let Ok(candidate) = super::ispdb_discovery::discover_by_ispdb(&domain, http_client) {
        return Ok(to_result(&candidate.provider.incoming));
    }
    if let Ok(candidate) = super::vendor_discovery::discover_by_vendor(&domain, http_client) {
        return Ok(to_result(&candidate.provider.incoming));
    }

    // 6. Port scanning as last resort
    if let Ok(result) =
        super::port_scanning::discover_by_port_scan(&domain, resolver, port_prober, None)
    {
        if let Some(candidate) = result.candidates.first() {
            return Ok(AutoConfigResult {
                hostname: candidate.provider.incoming.hostname.clone(),
                port: candidate.provider.incoming.port,
                encryption: candidate.provider.incoming.encryption,
            });
        }
    }

    Err(AutoConfigError::AllStrategiesFailed)
}

fn to_result(config: &super::provider::ServerConfig) -> AutoConfigResult {
    AutoConfigResult {
        hostname: config.hostname.clone(),
        port: config.port,
        encryption: config.encryption,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::dns_discovery::{DnsError, SrvRecord};
    use crate::core::ispdb_discovery::AutoconfigError;

    struct MockResolver {
        ns_records: Vec<String>,
        mx_records: Vec<(u16, String)>,
        srv_records: std::collections::HashMap<String, Vec<SrvRecord>>,
    }

    impl MockResolver {
        fn empty() -> Self {
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

    struct FailingHttpClient;
    impl HttpClient for FailingHttpClient {
        fn get(&self, _url: &str) -> Result<String, AutoconfigError> {
            Err(AutoconfigError::HttpFailed("mock: no network".to_string()))
        }
    }

    struct FailingPortProber;
    impl PortProber for FailingPortProber {
        fn probe(&self, _host: &str, _port: u16) -> bool {
            false
        }
    }

    struct OpenPortProber {
        open: std::collections::HashSet<(String, u16)>,
    }
    impl OpenPortProber {
        fn new(ports: &[(&str, u16)]) -> Self {
            Self {
                open: ports.iter().map(|(h, p)| (h.to_string(), *p)).collect(),
            }
        }
    }
    impl PortProber for OpenPortProber {
        fn probe(&self, host: &str, port: u16) -> bool {
            self.open.contains(&(host.to_string(), port))
        }
    }

    #[test]
    fn test_empty_domain_returns_error() {
        let db = ProviderDatabase::bundled();
        let result = discover_inbound(
            "",
            &db,
            &MockResolver::empty(),
            &FailingHttpClient,
            &FailingPortProber,
        );
        assert!(matches!(result, Err(AutoConfigError::EmptyDomain)));
    }

    #[test]
    fn test_whitespace_domain_returns_error() {
        let db = ProviderDatabase::bundled();
        let result = discover_inbound(
            "  ",
            &db,
            &MockResolver::empty(),
            &FailingHttpClient,
            &FailingPortProber,
        );
        assert!(matches!(result, Err(AutoConfigError::EmptyDomain)));
    }

    #[test]
    fn test_bundled_provider_match() {
        let db = ProviderDatabase::bundled();
        let result = discover_inbound(
            "gmail.com",
            &db,
            &MockResolver::empty(),
            &FailingHttpClient,
            &FailingPortProber,
        )
        .unwrap();
        assert_eq!(result.hostname, "imap.gmail.com");
        assert_eq!(result.port, 993);
        assert_eq!(result.encryption, ProviderEncryption::SslTls);
    }

    #[test]
    fn test_bundled_provider_case_insensitive() {
        let db = ProviderDatabase::bundled();
        let result = discover_inbound(
            "GMAIL.COM",
            &db,
            &MockResolver::empty(),
            &FailingHttpClient,
            &FailingPortProber,
        )
        .unwrap();
        assert_eq!(result.hostname, "imap.gmail.com");
    }

    #[test]
    fn test_dns_ns_fallback() {
        let db = ProviderDatabase::bundled();
        let resolver = MockResolver::empty().with_ns(&["ns1.google.com"]);
        let result = discover_inbound(
            "custom-domain.example.com",
            &db,
            &resolver,
            &FailingHttpClient,
            &FailingPortProber,
        )
        .unwrap();
        assert_eq!(result.hostname, "imap.gmail.com");
    }

    #[test]
    fn test_dns_srv_fallback() {
        let db = ProviderDatabase::bundled();
        let resolver = MockResolver::empty().with_srv(
            "_imaps._tcp.custom.example.org",
            vec![SrvRecord {
                priority: 0,
                weight: 1,
                port: 993,
                target: "mail.custom.example.org.".to_string(),
            }],
        );
        let result = discover_inbound(
            "custom.example.org",
            &db,
            &resolver,
            &FailingHttpClient,
            &FailingPortProber,
        )
        .unwrap();
        assert_eq!(result.hostname, "mail.custom.example.org");
        assert_eq!(result.port, 993);
        assert_eq!(result.encryption, ProviderEncryption::SslTls);
    }

    #[test]
    fn test_dns_mx_fallback() {
        let db = ProviderDatabase::bundled();
        let resolver = MockResolver::empty().with_mx(&[(10, "aspmx.l.google.com")]);
        let result = discover_inbound(
            "mx-only-domain.example.com",
            &db,
            &resolver,
            &FailingHttpClient,
            &FailingPortProber,
        )
        .unwrap();
        assert_eq!(result.hostname, "imap.gmail.com");
    }

    #[test]
    fn test_port_scan_fallback() {
        let db = ProviderDatabase::bundled();
        let prober = OpenPortProber::new(&[
            ("unknown-domain.example.com", 993),
            ("unknown-domain.example.com", 465),
        ]);
        let result = discover_inbound(
            "unknown-domain.example.com",
            &db,
            &MockResolver::empty(),
            &FailingHttpClient,
            &prober,
        )
        .unwrap();
        assert_eq!(result.hostname, "unknown-domain.example.com");
        assert_eq!(result.port, 993);
        assert_eq!(result.encryption, ProviderEncryption::SslTls);
    }

    #[test]
    fn test_all_strategies_fail() {
        let db = ProviderDatabase::bundled();
        let result = discover_inbound(
            "completely-unknown-domain.example.com",
            &db,
            &MockResolver::empty(),
            &FailingHttpClient,
            &FailingPortProber,
        );
        assert!(matches!(result, Err(AutoConfigError::AllStrategiesFailed)));
    }

    #[test]
    fn test_bundled_takes_priority_over_dns() {
        let db = ProviderDatabase::bundled();
        // Even with DNS records pointing elsewhere, bundled DB match wins
        let resolver = MockResolver::empty().with_srv(
            "_imaps._tcp.gmail.com",
            vec![SrvRecord {
                priority: 0,
                weight: 1,
                port: 993,
                target: "different-server.example.com.".to_string(),
            }],
        );
        let result = discover_inbound(
            "gmail.com",
            &db,
            &resolver,
            &FailingHttpClient,
            &FailingPortProber,
        )
        .unwrap();
        // Should use bundled DB result, not SRV
        assert_eq!(result.hostname, "imap.gmail.com");
    }

    #[test]
    fn test_result_contains_all_fields() {
        let db = ProviderDatabase::bundled();
        let result = discover_inbound(
            "outlook.com",
            &db,
            &MockResolver::empty(),
            &FailingHttpClient,
            &FailingPortProber,
        )
        .unwrap();
        assert!(!result.hostname.is_empty());
        assert!(result.port > 0);
    }

    #[test]
    fn test_domain_trimmed() {
        let db = ProviderDatabase::bundled();
        let result = discover_inbound(
            "  gmail.com  ",
            &db,
            &MockResolver::empty(),
            &FailingHttpClient,
            &FailingPortProber,
        )
        .unwrap();
        assert_eq!(result.hostname, "imap.gmail.com");
    }
}
