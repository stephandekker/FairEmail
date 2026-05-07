// Discovery Pipeline with Trust Scoring (FR-9, N-2).
//
// Collects configuration candidates from all available discovery methods,
// assigns trust scores, and returns the highest-scoring result. The bundled
// catalogue always outranks network-derived configurations.
//
// Score hierarchy (highest to lowest):
//   Bundled exact domain match  (1.0)
//   Bundled wildcard match      (0.9)
//   Bundled MX match            (0.4) — via MatchScore::DNS_MX on catalogue lookup
//   DNS NS                      (0.45)
//   DNS MX                      (0.40)
//   DNS SRV                     (0.35)
//   ISPDB                       (0.30)
//   Vendor autodiscovery        (0.25)
//   Port scan                   (0.10)

use super::dns_discovery::DnsResolver;
use super::ispdb_discovery::HttpClient;
use super::port_scanning::PortProber;
use super::provider::{merge_network_with_bundled, ProviderCandidate, ProviderDatabase};

/// A ranked discovery result from the pipeline.
#[derive(Debug, Clone)]
pub struct DiscoveryResult {
    /// All candidates found, sorted by trust score (highest first).
    pub candidates: Vec<ProviderCandidate>,
}

impl DiscoveryResult {
    /// The winning candidate (highest trust score), if any.
    pub fn best(&self) -> Option<&ProviderCandidate> {
        self.candidates.first()
    }
}

/// Run all discovery methods for the given domain and return candidates ranked
/// by trust score (FR-9). The pipeline does not short-circuit — it collects
/// results from every available method and picks the winner by score (N-2).
///
/// When a network-discovered candidate's server hostname matches a bundled
/// provider entry, the bundled entry's full configuration is substituted while
/// preserving the original discovery score.
pub(crate) fn discover_all(
    domain: &str,
    provider_db: &ProviderDatabase,
    resolver: &dyn DnsResolver,
    http_client: &dyn HttpClient,
    port_prober: &dyn PortProber,
) -> DiscoveryResult {
    let domain = domain.trim().to_lowercase();
    if domain.is_empty() {
        return DiscoveryResult { candidates: vec![] };
    }

    let mut candidates: Vec<ProviderCandidate> = Vec::new();

    // 1. Bundled provider database — domain match
    if let Some(candidate) = provider_db.lookup_by_domain(&domain) {
        candidates.push(candidate);
    }

    // 2. Bundled provider database — MX-based match
    if let Ok(mx_records) = resolver.lookup_mx(&domain) {
        if let Some(candidate) = provider_db.lookup_by_mx_records(&mx_records) {
            // Only add if not a duplicate of the domain match
            if !candidates
                .iter()
                .any(|c| c.provider.id == candidate.provider.id)
            {
                candidates.push(candidate);
            }
        }
    }

    // 3. DNS NS discovery
    if let Some(candidate) = super::dns_discovery::discover_by_ns(&domain, resolver, provider_db) {
        if !candidates
            .iter()
            .any(|c| c.provider.id == candidate.provider.id)
        {
            candidates.push(merge_network_with_bundled(candidate, provider_db));
        }
    }

    // 4. DNS SRV discovery
    if let Some(candidate) = super::dns_discovery::discover_by_srv(&domain, resolver) {
        candidates.push(merge_network_with_bundled(candidate, provider_db));
    }

    // 5. ISPDB
    if let Ok(candidate) = super::ispdb_discovery::discover_by_ispdb(&domain, http_client) {
        candidates.push(merge_network_with_bundled(candidate, provider_db));
    }

    // 6. Vendor autodiscovery
    if let Ok(candidate) = super::vendor_discovery::discover_by_vendor(&domain, http_client) {
        candidates.push(merge_network_with_bundled(candidate, provider_db));
    }

    // 7. Port scanning (last resort)
    if let Ok(result) =
        super::port_scanning::discover_by_port_scan(&domain, resolver, port_prober, None)
    {
        for candidate in result.candidates {
            candidates.push(merge_network_with_bundled(candidate, provider_db));
        }
    }

    // Sort by trust score descending. For determinism (AC-4), break ties by
    // provider ID (lexicographic) so the same inputs always produce the same winner.
    candidates.sort_by(|a, b| {
        b.score
            .value()
            .partial_cmp(&a.score.value())
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.provider.id.cmp(&b.provider.id))
    });

    DiscoveryResult { candidates }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::dns_discovery::{DnsError, SrvRecord};
    use crate::core::ispdb_discovery::AutoconfigError;
    use crate::core::provider::MatchScore;
    use crate::core::provider::{
        MaxTlsVersion, Provider, ProviderEncryption, ServerConfig, UsernameType,
    };

    // --- Mock infrastructure ---

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

    struct MockHttpClient {
        responses: std::collections::HashMap<String, String>,
    }
    impl MockHttpClient {
        fn new() -> Self {
            Self {
                responses: std::collections::HashMap::new(),
            }
        }
        fn with_response(mut self, url: &str, body: &str) -> Self {
            self.responses.insert(url.to_string(), body.to_string());
            self
        }
    }
    impl HttpClient for MockHttpClient {
        fn get(&self, url: &str) -> Result<String, AutoconfigError> {
            self.responses
                .get(url)
                .cloned()
                .ok_or_else(|| AutoconfigError::HttpFailed(format!("no mock for {url}")))
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
            subtitle: None,
            registration_url: None,
            graph: None,
            debug_only: false,
            variant_of: None,
        }
    }

    // --- AC-1: Bundled catalogue matches receive higher trust score ---

    #[test]
    fn bundled_score_higher_than_all_network_methods() {
        assert!(MatchScore::BUNDLED_EXACT > MatchScore::DNS_NS);
        assert!(MatchScore::BUNDLED_EXACT > MatchScore::DNS_MX);
        assert!(MatchScore::BUNDLED_EXACT > MatchScore::DNS_SRV);
        assert!(MatchScore::BUNDLED_EXACT > MatchScore::ISPDB);
        assert!(MatchScore::BUNDLED_EXACT > MatchScore::VENDOR_AUTODISCOVERY);
        assert!(MatchScore::BUNDLED_EXACT > MatchScore::PORT_SCAN);

        assert!(MatchScore::BUNDLED_WILDCARD > MatchScore::DNS_NS);
        assert!(MatchScore::BUNDLED_WILDCARD > MatchScore::DNS_MX);
        assert!(MatchScore::BUNDLED_WILDCARD > MatchScore::DNS_SRV);
        assert!(MatchScore::BUNDLED_WILDCARD > MatchScore::ISPDB);
        assert!(MatchScore::BUNDLED_WILDCARD > MatchScore::VENDOR_AUTODISCOVERY);
        assert!(MatchScore::BUNDLED_WILDCARD > MatchScore::PORT_SCAN);
    }

    // --- AC-2: Bundled wins when both bundled and network return results ---

    #[test]
    fn bundled_wins_over_dns_srv_for_same_domain() {
        let db = ProviderDatabase::bundled();
        // gmail.com is in the bundled DB; also provide SRV records pointing elsewhere
        let resolver = MockResolver::empty().with_srv(
            "_imaps._tcp.gmail.com",
            vec![SrvRecord {
                priority: 0,
                weight: 1,
                port: 993,
                target: "alt-imap.example.com.".to_string(),
            }],
        );

        let result = discover_all(
            "gmail.com",
            &db,
            &resolver,
            &FailingHttpClient,
            &FailingPortProber,
        );

        let best = result.best().unwrap();
        assert!(best.score.is_bundled());
        assert_eq!(best.provider.incoming.hostname, "imap.gmail.com");
    }

    #[test]
    fn bundled_wins_over_ispdb_for_same_domain() {
        let db = ProviderDatabase::new(vec![make_test_provider("testprov", &["testprov.com"])]);
        let http = MockHttpClient::new().with_response(
            "https://autoconfig.thunderbird.net/v1.1/testprov.com",
            r#"<?xml version="1.0"?>
<clientConfig version="1.1">
  <emailProvider id="testprov.com">
    <displayName>TestProv ISPDB</displayName>
    <incomingServer type="imap">
      <hostname>ispdb-imap.testprov.com</hostname>
      <port>993</port>
      <socketType>SSL</socketType>
      <username>%EMAILADDRESS%</username>
    </incomingServer>
    <outgoingServer type="smtp">
      <hostname>ispdb-smtp.testprov.com</hostname>
      <port>465</port>
      <socketType>SSL</socketType>
      <username>%EMAILADDRESS%</username>
    </outgoingServer>
  </emailProvider>
</clientConfig>"#,
        );

        let result = discover_all(
            "testprov.com",
            &db,
            &MockResolver::empty(),
            &http,
            &FailingPortProber,
        );

        let best = result.best().unwrap();
        assert!(best.score.is_bundled());
        assert_eq!(best.provider.incoming.hostname, "imap.testprov.com");
    }

    // --- AC-3: Network fallback when bundled has no entry ---

    #[test]
    fn network_fallback_dns_srv_when_no_bundled_entry() {
        let db = ProviderDatabase::new(vec![make_test_provider("other", &["other.com"])]);
        let resolver = MockResolver::empty().with_srv(
            "_imaps._tcp.unknown.example.com",
            vec![SrvRecord {
                priority: 0,
                weight: 1,
                port: 993,
                target: "imap.unknown.example.com.".to_string(),
            }],
        );

        let result = discover_all(
            "unknown.example.com",
            &db,
            &resolver,
            &FailingHttpClient,
            &FailingPortProber,
        );

        let best = result.best().unwrap();
        assert_eq!(best.score, MatchScore::DNS_SRV);
        assert_eq!(best.provider.incoming.hostname, "imap.unknown.example.com");
    }

    #[test]
    fn network_fallback_ispdb_when_no_bundled_entry() {
        let db = ProviderDatabase::new(vec![]);
        let http = MockHttpClient::new().with_response(
            "https://autoconfig.thunderbird.net/v1.1/smallisp.com",
            r#"<?xml version="1.0"?>
<clientConfig version="1.1">
  <emailProvider id="smallisp.com">
    <displayName>Small ISP</displayName>
    <incomingServer type="imap">
      <hostname>mail.smallisp.com</hostname>
      <port>993</port>
      <socketType>SSL</socketType>
      <username>%EMAILADDRESS%</username>
    </incomingServer>
    <outgoingServer type="smtp">
      <hostname>mail.smallisp.com</hostname>
      <port>465</port>
      <socketType>SSL</socketType>
      <username>%EMAILADDRESS%</username>
    </outgoingServer>
  </emailProvider>
</clientConfig>"#,
        );

        let result = discover_all(
            "smallisp.com",
            &db,
            &MockResolver::empty(),
            &http,
            &FailingPortProber,
        );

        let best = result.best().unwrap();
        assert_eq!(best.score, MatchScore::ISPDB);
        assert_eq!(best.provider.incoming.hostname, "mail.smallisp.com");
    }

    #[test]
    fn network_fallback_port_scan_when_no_bundled_entry() {
        let db = ProviderDatabase::new(vec![]);
        let prober = OpenPortProber::new(&[("fallback.com", 993), ("fallback.com", 465)]);

        let result = discover_all(
            "fallback.com",
            &db,
            &MockResolver::empty(),
            &FailingHttpClient,
            &prober,
        );

        let best = result.best().unwrap();
        assert_eq!(best.score, MatchScore::PORT_SCAN);
        assert_eq!(best.provider.incoming.hostname, "fallback.com");
    }

    // --- AC-4: Deterministic ranking ---

    #[test]
    fn deterministic_ranking_same_inputs_same_output() {
        let db = ProviderDatabase::bundled();
        let resolver = MockResolver::empty()
            .with_ns(&["ns1.google.com"])
            .with_mx(&[(10, "aspmx.l.google.com")]);

        // Run the pipeline multiple times
        let results: Vec<_> = (0..10)
            .map(|_| {
                discover_all(
                    "custom.example.com",
                    &db,
                    &resolver,
                    &FailingHttpClient,
                    &FailingPortProber,
                )
            })
            .collect();

        // All runs should produce the same winner
        let first_best = results[0].best().unwrap();
        for result in &results[1..] {
            let best = result.best().unwrap();
            assert_eq!(best.provider.id, first_best.provider.id);
            assert_eq!(best.score, first_best.score);
        }
    }

    #[test]
    fn deterministic_tie_breaking_by_provider_id() {
        // Create two providers with the same score class
        let mut p1 = make_test_provider("alpha", &[]);
        p1.mx_patterns = vec!["*.alpha-hosting.com".to_string()];
        let mut p2 = make_test_provider("beta", &[]);
        p2.mx_patterns = vec!["*.alpha-hosting.com".to_string()];

        // With the bundled DB containing both, an MX lookup for alpha-hosting.com
        // will match whichever is first in the compiled patterns. The tie-breaker
        // ensures deterministic output regardless of HashMap iteration order.
        let db = ProviderDatabase::new(vec![p1, p2]);

        let resolver = MockResolver::empty().with_mx(&[(10, "mx.alpha-hosting.com")]);

        let result = discover_all(
            "unknown-domain.example",
            &db,
            &resolver,
            &FailingHttpClient,
            &FailingPortProber,
        );

        // The result should be deterministic
        assert!(result.best().is_some());
        let best_id = result.best().unwrap().provider.id.clone();

        // Run again — same result
        let result2 = discover_all(
            "unknown-domain.example",
            &db,
            &resolver,
            &FailingHttpClient,
            &FailingPortProber,
        );
        assert_eq!(result2.best().unwrap().provider.id, best_id);
    }

    // --- Pipeline collects from all methods ---

    #[test]
    fn pipeline_collects_multiple_candidates() {
        let db = ProviderDatabase::new(vec![]);
        let resolver = MockResolver::empty().with_srv(
            "_imaps._tcp.multi.example.com",
            vec![SrvRecord {
                priority: 0,
                weight: 1,
                port: 993,
                target: "imap.multi.example.com.".to_string(),
            }],
        );
        let http = MockHttpClient::new().with_response(
            "https://autoconfig.thunderbird.net/v1.1/multi.example.com",
            r#"<?xml version="1.0"?>
<clientConfig version="1.1">
  <emailProvider id="multi.example.com">
    <displayName>Multi</displayName>
    <incomingServer type="imap">
      <hostname>ispdb.multi.example.com</hostname>
      <port>993</port>
      <socketType>SSL</socketType>
      <username>%EMAILADDRESS%</username>
    </incomingServer>
    <outgoingServer type="smtp">
      <hostname>smtp.multi.example.com</hostname>
      <port>465</port>
      <socketType>SSL</socketType>
      <username>%EMAILADDRESS%</username>
    </outgoingServer>
  </emailProvider>
</clientConfig>"#,
        );

        let result = discover_all(
            "multi.example.com",
            &db,
            &resolver,
            &http,
            &FailingPortProber,
        );

        // Should have at least 2 candidates (SRV + ISPDB)
        assert!(result.candidates.len() >= 2);
        // SRV should rank higher than ISPDB
        assert!(result.candidates[0].score.value() >= result.candidates[1].score.value());
    }

    // --- Empty domain ---

    #[test]
    fn empty_domain_returns_no_candidates() {
        let db = ProviderDatabase::new(vec![]);
        let result = discover_all(
            "",
            &db,
            &MockResolver::empty(),
            &FailingHttpClient,
            &FailingPortProber,
        );
        assert!(result.candidates.is_empty());
        assert!(result.best().is_none());
    }

    #[test]
    fn whitespace_domain_returns_no_candidates() {
        let db = ProviderDatabase::new(vec![]);
        let result = discover_all(
            "   ",
            &db,
            &MockResolver::empty(),
            &FailingHttpClient,
            &FailingPortProber,
        );
        assert!(result.candidates.is_empty());
    }

    // --- Score ordering is preserved end-to-end ---

    #[test]
    fn score_ordering_bundled_gt_dns_gt_ispdb_gt_port_scan() {
        // This test verifies the trust hierarchy is maintained in the sorted output
        assert!(MatchScore::BUNDLED_EXACT.value() > MatchScore::BUNDLED_WILDCARD.value());
        assert!(MatchScore::BUNDLED_WILDCARD.value() > MatchScore::NETWORK_MAX.value());
        assert!(MatchScore::DNS_NS.value() > MatchScore::DNS_MX.value());
        assert!(MatchScore::DNS_MX.value() > MatchScore::DNS_SRV.value());
        assert!(MatchScore::DNS_SRV.value() > MatchScore::ISPDB.value());
        assert!(MatchScore::ISPDB.value() > MatchScore::VENDOR_AUTODISCOVERY.value());
        assert!(MatchScore::VENDOR_AUTODISCOVERY.value() > MatchScore::PORT_SCAN.value());
    }

    // --- Network candidates get merged with bundled when hostname matches ---

    #[test]
    fn network_candidate_merged_with_bundled_on_hostname_match() {
        let db = ProviderDatabase::bundled();
        // Provide an ISPDB result whose hostname matches gmail in bundled DB
        let http = MockHttpClient::new().with_response(
            "https://autoconfig.thunderbird.net/v1.1/custom-gmail-domain.example",
            r#"<?xml version="1.0"?>
<clientConfig version="1.1">
  <emailProvider id="custom-gmail-domain.example">
    <displayName>Custom Gmail</displayName>
    <incomingServer type="imap">
      <hostname>imap.gmail.com</hostname>
      <port>993</port>
      <socketType>SSL</socketType>
      <username>%EMAILADDRESS%</username>
    </incomingServer>
    <outgoingServer type="smtp">
      <hostname>smtp.gmail.com</hostname>
      <port>465</port>
      <socketType>SSL</socketType>
      <username>%EMAILADDRESS%</username>
    </outgoingServer>
  </emailProvider>
</clientConfig>"#,
        );

        let result = discover_all(
            "custom-gmail-domain.example",
            &db,
            &MockResolver::empty(),
            &http,
            &FailingPortProber,
        );

        let best = result.best().unwrap();
        // Score is ISPDB (network-derived) but provider data is from bundled gmail
        assert_eq!(best.score, MatchScore::ISPDB);
        assert_eq!(best.provider.id, "gmail");
    }
}
