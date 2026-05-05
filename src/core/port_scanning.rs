// Port-scanning fallback strategy (FR-10.7).
//
// Last-resort detection: when all other strategies fail, attempt TCP connections
// to well-known IMAP/SMTP ports on the MX host or the domain itself.
//
// **Security note (OQ-2):** Port scanning produces candidates with the lowest
// confidence score. When a user proceeds with a port-scanned candidate, their
// password will be sent to whatever server answers on the probed ports. Callers
// should inform the user that this is a best-guess configuration before
// authentication is attempted.

use super::detection_progress::DetectionStep;
use super::dns_discovery::{DnsError, DnsResolver};
use super::provider::{
    MatchScore, MaxTlsVersion, Provider, ProviderCandidate, ProviderEncryption, ServerConfig,
    UsernameType,
};

/// Ports to probe for incoming IMAP, ordered by preference (most secure first).
const IMAP_PORTS: &[(u16, ProviderEncryption)] = &[
    (993, ProviderEncryption::SslTls),
    (143, ProviderEncryption::StartTls),
];

/// Ports to probe for outgoing SMTP, ordered by preference (most secure first).
const SMTP_PORTS: &[(u16, ProviderEncryption)] = &[
    (465, ProviderEncryption::SslTls),
    (587, ProviderEncryption::StartTls),
];

/// Trait abstracting TCP connection probing so tests can avoid real network I/O.
pub trait PortProber {
    /// Returns `true` if a TCP connection to `host:port` can be established.
    fn probe(&self, host: &str, port: u16) -> bool;
}

/// Errors that can occur during port-scan discovery.
#[derive(Debug, Clone, thiserror::Error)]
pub enum PortScanError {
    #[error("no open IMAP ports found on {host}")]
    ImapPortClosed { host: String },
    #[error("no open SMTP ports found on {host}")]
    SmtpPortClosed { host: String },
    #[error("could not determine target host for port scanning")]
    TargetHostUnresolvable,
}

/// Result of port-scan discovery containing candidates for the host(s) probed.
#[derive(Debug, Clone)]
pub struct PortScanResult {
    /// Candidates found, sorted by score (highest first).
    /// In practice port scanning always yields a single candidate per host.
    pub candidates: Vec<ProviderCandidate>,
}

/// Attempt port-scan discovery for the given domain (FR-10.7).
///
/// Determines the target host (MX record preferred, falling back to the domain
/// itself), then probes standard IMAP and SMTP ports. Reports progress via the
/// optional callback.
///
/// Returns a candidate with `MatchScore::PORT_SCAN` — the lowest confidence of
/// all detection strategies.
pub(crate) fn discover_by_port_scan(
    domain: &str,
    resolver: &dyn DnsResolver,
    prober: &dyn PortProber,
    progress: Option<&dyn Fn(DetectionStep)>,
) -> Result<PortScanResult, PortScanError> {
    let host = determine_target_host(domain, resolver)?;

    // Report progress: scanning ports on the resolved host.
    if let Some(cb) = progress {
        cb(DetectionStep::ScanPorts { host: host.clone() });
    }

    // Probe IMAP ports
    let imap = probe_first_open(&host, IMAP_PORTS, prober)
        .ok_or_else(|| PortScanError::ImapPortClosed { host: host.clone() })?;

    // Probe SMTP ports
    let smtp = probe_first_open(&host, SMTP_PORTS, prober)
        .ok_or_else(|| PortScanError::SmtpPortClosed { host: host.clone() })?;

    let provider = Provider {
        id: format!("port-scan-{domain}"),
        display_name: domain.to_string(),
        domain_patterns: vec![domain.to_string()],
        mx_patterns: vec![],
        incoming: ServerConfig {
            hostname: host.clone(),
            port: imap.0,
            encryption: imap.1,
        },
        outgoing: ServerConfig {
            hostname: host,
            port: smtp.0,
            encryption: smtp.1,
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
    };

    Ok(PortScanResult {
        candidates: vec![ProviderCandidate {
            provider,
            score: MatchScore::PORT_SCAN,
        }],
    })
}

/// Determine the host to probe: prefer the highest-priority MX record,
/// fall back to the domain itself.
fn determine_target_host(
    domain: &str,
    resolver: &dyn DnsResolver,
) -> Result<String, PortScanError> {
    match resolver.lookup_mx(domain) {
        Ok(mut mx_records) if !mx_records.is_empty() => {
            // Sort by priority (lowest value = highest priority)
            mx_records.sort_by_key(|(prio, _)| *prio);
            let host = mx_records[0].1.trim_end_matches('.').to_string();
            if host.is_empty() {
                Ok(domain.to_string())
            } else {
                Ok(host)
            }
        }
        Ok(_) | Err(DnsError::NoRecords) => {
            // No MX records — fall back to the domain itself
            Ok(domain.to_string())
        }
        Err(DnsError::LookupFailed(_)) => {
            // DNS failure — still try the domain itself as a last resort
            Ok(domain.to_string())
        }
    }
}

/// Probe a list of (port, encryption) pairs and return the first that is open.
fn probe_first_open(
    host: &str,
    ports: &[(u16, ProviderEncryption)],
    prober: &dyn PortProber,
) -> Option<(u16, ProviderEncryption)> {
    for &(port, encryption) in ports {
        if prober.probe(host, port) {
            return Some((port, encryption));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// Mock DNS resolver for port-scan tests.
    struct MockResolver {
        mx_records: Result<Vec<(u16, String)>, DnsError>,
    }

    impl MockResolver {
        fn with_mx(records: Vec<(u16, String)>) -> Self {
            Self {
                mx_records: Ok(records),
            }
        }

        fn with_no_records() -> Self {
            Self {
                mx_records: Err(DnsError::NoRecords),
            }
        }

        fn with_lookup_failure() -> Self {
            Self {
                mx_records: Err(DnsError::LookupFailed("timeout".to_string())),
            }
        }
    }

    impl DnsResolver for MockResolver {
        fn lookup_ns(&self, _domain: &str) -> Result<Vec<String>, DnsError> {
            Err(DnsError::NoRecords)
        }

        fn lookup_mx(&self, _domain: &str) -> Result<Vec<(u16, String)>, DnsError> {
            self.mx_records.clone()
        }

        fn lookup_srv(
            &self,
            _name: &str,
        ) -> Result<Vec<super::super::dns_discovery::SrvRecord>, DnsError> {
            Err(DnsError::NoRecords)
        }
    }

    /// Mock port prober that allows configuring which host:port pairs are open.
    struct MockProber {
        open_ports: HashSet<(String, u16)>,
    }

    impl MockProber {
        fn new() -> Self {
            Self {
                open_ports: HashSet::new(),
            }
        }

        fn with_open(mut self, host: &str, port: u16) -> Self {
            self.open_ports.insert((host.to_string(), port));
            self
        }
    }

    impl PortProber for MockProber {
        fn probe(&self, host: &str, port: u16) -> bool {
            self.open_ports.contains(&(host.to_string(), port))
        }
    }

    // --- Score ordering (FR-11) ---

    #[test]
    fn test_port_scan_score_is_lowest() {
        assert!(MatchScore::VENDOR_AUTODISCOVERY > MatchScore::PORT_SCAN);
        assert!(MatchScore::ISPDB > MatchScore::PORT_SCAN);
        assert!(MatchScore::DNS_SRV > MatchScore::PORT_SCAN);
        assert!(MatchScore::DNS_MX > MatchScore::PORT_SCAN);
        assert!(MatchScore::DNS_NS > MatchScore::PORT_SCAN);
        assert!(MatchScore::BUNDLED_WILDCARD > MatchScore::PORT_SCAN);
        assert!(MatchScore::BUNDLED_EXACT > MatchScore::PORT_SCAN);
    }

    #[test]
    fn test_port_scan_score_value() {
        assert_eq!(MatchScore::PORT_SCAN.value(), 0.10);
    }

    // --- Target host resolution ---

    #[test]
    fn test_uses_mx_host_when_available() {
        let resolver = MockResolver::with_mx(vec![
            (20, "backup.mail.example.com.".to_string()),
            (10, "primary.mail.example.com.".to_string()),
        ]);
        let prober = MockProber::new()
            .with_open("primary.mail.example.com", 993)
            .with_open("primary.mail.example.com", 465);

        let result = discover_by_port_scan("example.com", &resolver, &prober, None).unwrap();
        assert_eq!(
            result.candidates[0].provider.incoming.hostname,
            "primary.mail.example.com"
        );
    }

    #[test]
    fn test_falls_back_to_domain_when_no_mx() {
        let resolver = MockResolver::with_no_records();
        let prober = MockProber::new()
            .with_open("example.com", 993)
            .with_open("example.com", 587);

        let result = discover_by_port_scan("example.com", &resolver, &prober, None).unwrap();
        assert_eq!(
            result.candidates[0].provider.incoming.hostname,
            "example.com"
        );
    }

    #[test]
    fn test_falls_back_to_domain_on_dns_failure() {
        let resolver = MockResolver::with_lookup_failure();
        let prober = MockProber::new()
            .with_open("example.com", 143)
            .with_open("example.com", 587);

        let result = discover_by_port_scan("example.com", &resolver, &prober, None).unwrap();
        assert_eq!(
            result.candidates[0].provider.incoming.hostname,
            "example.com"
        );
        assert_eq!(result.candidates[0].provider.incoming.port, 143);
        assert_eq!(
            result.candidates[0].provider.incoming.encryption,
            ProviderEncryption::StartTls
        );
    }

    // --- Port probing ---

    #[test]
    fn test_prefers_ssl_tls_over_starttls_for_imap() {
        let resolver = MockResolver::with_no_records();
        let prober = MockProber::new()
            .with_open("example.com", 993)
            .with_open("example.com", 143)
            .with_open("example.com", 587);

        let result = discover_by_port_scan("example.com", &resolver, &prober, None).unwrap();
        assert_eq!(result.candidates[0].provider.incoming.port, 993);
        assert_eq!(
            result.candidates[0].provider.incoming.encryption,
            ProviderEncryption::SslTls
        );
    }

    #[test]
    fn test_prefers_ssl_tls_over_starttls_for_smtp() {
        let resolver = MockResolver::with_no_records();
        let prober = MockProber::new()
            .with_open("example.com", 993)
            .with_open("example.com", 465)
            .with_open("example.com", 587);

        let result = discover_by_port_scan("example.com", &resolver, &prober, None).unwrap();
        assert_eq!(result.candidates[0].provider.outgoing.port, 465);
        assert_eq!(
            result.candidates[0].provider.outgoing.encryption,
            ProviderEncryption::SslTls
        );
    }

    #[test]
    fn test_falls_back_to_starttls_imap() {
        let resolver = MockResolver::with_no_records();
        let prober = MockProber::new()
            .with_open("example.com", 143)
            .with_open("example.com", 587);

        let result = discover_by_port_scan("example.com", &resolver, &prober, None).unwrap();
        assert_eq!(result.candidates[0].provider.incoming.port, 143);
        assert_eq!(
            result.candidates[0].provider.incoming.encryption,
            ProviderEncryption::StartTls
        );
    }

    #[test]
    fn test_error_when_no_imap_port_open() {
        let resolver = MockResolver::with_no_records();
        let prober = MockProber::new().with_open("example.com", 587);

        let result = discover_by_port_scan("example.com", &resolver, &prober, None);
        assert!(matches!(result, Err(PortScanError::ImapPortClosed { .. })));
    }

    #[test]
    fn test_error_when_no_smtp_port_open() {
        let resolver = MockResolver::with_no_records();
        let prober = MockProber::new().with_open("example.com", 993);

        let result = discover_by_port_scan("example.com", &resolver, &prober, None);
        assert!(matches!(result, Err(PortScanError::SmtpPortClosed { .. })));
    }

    // --- Progress feedback (FR-14) ---

    #[test]
    fn test_progress_callback_reports_scan_ports() {
        let resolver = MockResolver::with_no_records();
        let prober = MockProber::new()
            .with_open("example.com", 993)
            .with_open("example.com", 465);

        let reported = std::cell::RefCell::new(Vec::new());
        let cb = |step: DetectionStep| {
            reported.borrow_mut().push(step);
        };

        discover_by_port_scan("example.com", &resolver, &prober, Some(&cb)).unwrap();

        let steps = reported.borrow();
        assert_eq!(steps.len(), 1);
        assert_eq!(
            steps[0],
            DetectionStep::ScanPorts {
                host: "example.com".to_string()
            }
        );
    }

    #[test]
    fn test_progress_callback_uses_mx_host() {
        let resolver = MockResolver::with_mx(vec![(10, "mx.example.com.".to_string())]);
        let prober = MockProber::new()
            .with_open("mx.example.com", 993)
            .with_open("mx.example.com", 465);

        let reported = std::cell::RefCell::new(Vec::new());
        let cb = |step: DetectionStep| {
            reported.borrow_mut().push(step);
        };

        discover_by_port_scan("example.com", &resolver, &prober, Some(&cb)).unwrap();

        let steps = reported.borrow();
        assert_eq!(
            steps[0],
            DetectionStep::ScanPorts {
                host: "mx.example.com".to_string()
            }
        );
    }

    // --- Candidate structure ---

    #[test]
    fn test_candidate_has_correct_score() {
        let resolver = MockResolver::with_no_records();
        let prober = MockProber::new()
            .with_open("example.com", 993)
            .with_open("example.com", 587);

        let result = discover_by_port_scan("example.com", &resolver, &prober, None).unwrap();
        assert_eq!(result.candidates.len(), 1);
        assert_eq!(result.candidates[0].score, MatchScore::PORT_SCAN);
    }

    #[test]
    fn test_candidate_uses_email_address_username() {
        let resolver = MockResolver::with_no_records();
        let prober = MockProber::new()
            .with_open("example.com", 993)
            .with_open("example.com", 587);

        let result = discover_by_port_scan("example.com", &resolver, &prober, None).unwrap();
        assert_eq!(
            result.candidates[0].provider.username_type,
            UsernameType::EmailAddress
        );
    }

    #[test]
    fn test_candidate_provider_id_includes_domain() {
        let resolver = MockResolver::with_no_records();
        let prober = MockProber::new()
            .with_open("mymail.org", 993)
            .with_open("mymail.org", 465);

        let result = discover_by_port_scan("mymail.org", &resolver, &prober, None).unwrap();
        assert_eq!(result.candidates[0].provider.id, "port-scan-mymail.org");
    }

    // --- MX trailing dot handling ---

    #[test]
    fn test_mx_trailing_dot_stripped() {
        let resolver = MockResolver::with_mx(vec![(10, "mail.example.com.".to_string())]);
        let prober = MockProber::new()
            .with_open("mail.example.com", 993)
            .with_open("mail.example.com", 465);

        let result = discover_by_port_scan("example.com", &resolver, &prober, None).unwrap();
        assert_eq!(
            result.candidates[0].provider.incoming.hostname,
            "mail.example.com"
        );
        assert!(!result.candidates[0]
            .provider
            .incoming
            .hostname
            .ends_with('.'));
    }
}
