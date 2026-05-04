/// Information about an untrusted certificate presented by a server (FR-19).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertificateInfo {
    /// SHA-256 fingerprint of the certificate (FR-19a).
    pub fingerprint: String,
    /// DNS names (Subject Alternative Names) covered by the certificate (FR-19a).
    pub dns_names: Vec<String>,
    /// The hostname that was being connected to.
    pub server_hostname: String,
}

impl CertificateInfo {
    /// Check whether any of the certificate's DNS names match the server hostname (FR-19b).
    ///
    /// Returns `true` if there is a mismatch (i.e. the hostname is NOT covered).
    pub fn has_hostname_mismatch(&self) -> bool {
        !self
            .dns_names
            .iter()
            .any(|name| hostname_matches_pattern(&self.server_hostname, name))
    }

    /// Returns the list of DNS names that do NOT match the server hostname,
    /// useful for highlighting mismatches in the UI (FR-19b).
    pub fn mismatched_names(&self) -> Vec<&str> {
        if self.has_hostname_mismatch() {
            // All names are "mismatched" because none covers the hostname
            self.dns_names.iter().map(|s| s.as_str()).collect()
        } else {
            Vec::new()
        }
    }
}

/// Check if a hostname matches a certificate DNS name pattern.
///
/// Supports wildcard patterns like `*.example.com` which match exactly
/// one subdomain level (e.g. `mail.example.com` but not `a.b.example.com`).
fn hostname_matches_pattern(hostname: &str, pattern: &str) -> bool {
    let hostname = hostname.to_lowercase();
    let pattern = pattern.to_lowercase();

    if pattern.starts_with("*.") {
        let suffix = &pattern[1..]; // ".example.com"
                                    // Hostname must end with the suffix and the part before must not contain dots
        if let Some(prefix) = hostname.strip_suffix(suffix) {
            !prefix.contains('.') && !prefix.is_empty()
        } else {
            false
        }
    } else {
        hostname == pattern
    }
}

/// The user's decision when presented with an untrusted certificate (FR-19c).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CertificateDecision {
    /// Accept the certificate by fingerprint for this account (FR-19c).
    Accept,
    /// Reject the certificate and abort the connection.
    Reject,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_hostname_match() {
        assert!(hostname_matches_pattern(
            "mail.example.com",
            "mail.example.com"
        ));
    }

    #[test]
    fn exact_hostname_no_match() {
        assert!(!hostname_matches_pattern(
            "mail.example.com",
            "other.example.com"
        ));
    }

    #[test]
    fn wildcard_matches_single_subdomain() {
        assert!(hostname_matches_pattern(
            "mail.example.com",
            "*.example.com"
        ));
    }

    #[test]
    fn wildcard_does_not_match_deeper_subdomain() {
        assert!(!hostname_matches_pattern(
            "a.b.example.com",
            "*.example.com"
        ));
    }

    #[test]
    fn wildcard_does_not_match_bare_domain() {
        assert!(!hostname_matches_pattern("example.com", "*.example.com"));
    }

    #[test]
    fn case_insensitive_matching() {
        assert!(hostname_matches_pattern(
            "Mail.Example.COM",
            "mail.example.com"
        ));
        assert!(hostname_matches_pattern(
            "IMAP.EXAMPLE.COM",
            "*.example.com"
        ));
    }

    #[test]
    fn has_hostname_mismatch_when_no_names_match() {
        let info = CertificateInfo {
            fingerprint: "aa:bb:cc".to_string(),
            dns_names: vec!["other.example.com".to_string()],
            server_hostname: "mail.example.com".to_string(),
        };
        assert!(info.has_hostname_mismatch());
    }

    #[test]
    fn no_hostname_mismatch_when_exact_match() {
        let info = CertificateInfo {
            fingerprint: "aa:bb:cc".to_string(),
            dns_names: vec![
                "other.example.com".to_string(),
                "mail.example.com".to_string(),
            ],
            server_hostname: "mail.example.com".to_string(),
        };
        assert!(!info.has_hostname_mismatch());
    }

    #[test]
    fn no_hostname_mismatch_when_wildcard_covers() {
        let info = CertificateInfo {
            fingerprint: "aa:bb:cc".to_string(),
            dns_names: vec!["*.example.com".to_string()],
            server_hostname: "mail.example.com".to_string(),
        };
        assert!(!info.has_hostname_mismatch());
    }

    #[test]
    fn mismatched_names_returns_all_when_mismatch() {
        let info = CertificateInfo {
            fingerprint: "aa:bb:cc".to_string(),
            dns_names: vec![
                "other.example.com".to_string(),
                "another.example.com".to_string(),
            ],
            server_hostname: "mail.example.com".to_string(),
        };
        let mismatched = info.mismatched_names();
        assert_eq!(mismatched.len(), 2);
    }

    #[test]
    fn mismatched_names_returns_empty_when_match() {
        let info = CertificateInfo {
            fingerprint: "aa:bb:cc".to_string(),
            dns_names: vec!["mail.example.com".to_string()],
            server_hostname: "mail.example.com".to_string(),
        };
        let mismatched = info.mismatched_names();
        assert!(mismatched.is_empty());
    }

    #[test]
    fn empty_dns_names_is_mismatch() {
        let info = CertificateInfo {
            fingerprint: "aa:bb:cc".to_string(),
            dns_names: vec![],
            server_hostname: "mail.example.com".to_string(),
        };
        assert!(info.has_hostname_mismatch());
    }
}
