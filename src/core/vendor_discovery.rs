use super::ispdb_discovery::{parse_autoconfig_xml, AutoconfigError, HttpClient};
use super::provider::{MatchScore, ProviderCandidate};

/// Well-known autoconfig URL patterns to try for vendor-hosted autodiscovery.
/// These follow the Mozilla autoconfig protocol hosted by the domain owner.
const AUTOCONFIG_URL_TEMPLATES: &[&str] = &[
    "https://autoconfig.{domain}/mail/config-v1.1.xml",
    "https://{domain}/.well-known/autoconfig/mail/config-v1.1.xml",
];

/// Attempt vendor-specific autodiscovery for the given domain (FR-10.6).
///
/// Tries well-known autoconfig endpoints hosted by the domain itself.
/// Only the domain is used in the query — the user's password is never
/// transmitted (FR-38).
pub(crate) fn discover_by_vendor(
    domain: &str,
    client: &dyn HttpClient,
) -> Result<ProviderCandidate, AutoconfigError> {
    let mut last_err = AutoconfigError::HttpFailed("no autoconfig URLs tried".to_string());

    for template in AUTOCONFIG_URL_TEMPLATES {
        let url = template.replace("{domain}", domain);
        match client.get(&url) {
            Ok(body) => match parse_autoconfig_xml(&body, domain) {
                Ok(provider) => {
                    return Ok(ProviderCandidate {
                        provider,
                        score: MatchScore::VENDOR_AUTODISCOVERY,
                    });
                }
                Err(e) => last_err = e,
            },
            Err(e) => last_err = e,
        }
    }

    Err(last_err)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::provider::{ProviderEncryption, UsernameType};
    use std::collections::HashMap;

    /// Mock HTTP client for vendor autodiscovery tests.
    struct MockHttp {
        responses: HashMap<String, Result<String, AutoconfigError>>,
    }

    impl MockHttp {
        fn new() -> Self {
            Self {
                responses: HashMap::new(),
            }
        }

        fn with_response(mut self, url: &str, body: &str) -> Self {
            self.responses.insert(url.to_string(), Ok(body.to_string()));
            self
        }

        fn with_error(mut self, url: &str, msg: &str) -> Self {
            self.responses.insert(
                url.to_string(),
                Err(AutoconfigError::HttpFailed(msg.to_string())),
            );
            self
        }
    }

    impl HttpClient for MockHttp {
        fn get(&self, url: &str) -> Result<String, AutoconfigError> {
            self.responses
                .get(url)
                .cloned()
                .unwrap_or(Err(AutoconfigError::HttpFailed(format!(
                    "no mock for {url}"
                ))))
        }
    }

    const VENDOR_AUTOCONFIG: &str = r#"<?xml version="1.0"?>
<clientConfig version="1.1">
  <emailProvider id="corpmail.example.com">
    <domain>corpmail.example.com</domain>
    <displayName>CorpMail</displayName>
    <incomingServer type="imap">
      <hostname>imap.corpmail.example.com</hostname>
      <port>993</port>
      <socketType>SSL</socketType>
      <username>%EMAILADDRESS%</username>
    </incomingServer>
    <outgoingServer type="smtp">
      <hostname>smtp.corpmail.example.com</hostname>
      <port>587</port>
      <socketType>STARTTLS</socketType>
      <username>%EMAILADDRESS%</username>
    </outgoingServer>
  </emailProvider>
</clientConfig>"#;

    // --- Score ordering (FR-11) ---

    #[test]
    fn test_vendor_score_below_ispdb() {
        assert!(MatchScore::ISPDB > MatchScore::VENDOR_AUTODISCOVERY);
    }

    #[test]
    fn test_vendor_score_below_dns() {
        assert!(MatchScore::DNS_SRV > MatchScore::VENDOR_AUTODISCOVERY);
        assert!(MatchScore::DNS_MX > MatchScore::VENDOR_AUTODISCOVERY);
        assert!(MatchScore::DNS_NS > MatchScore::VENDOR_AUTODISCOVERY);
    }

    // --- Vendor autodiscovery (FR-10.6) ---

    #[test]
    fn test_vendor_discovery_first_url_success() {
        let client = MockHttp::new().with_response(
            "https://autoconfig.corpmail.example.com/mail/config-v1.1.xml",
            VENDOR_AUTOCONFIG,
        );

        let candidate = discover_by_vendor("corpmail.example.com", &client).unwrap();
        assert_eq!(candidate.score, MatchScore::VENDOR_AUTODISCOVERY);
        assert_eq!(
            candidate.provider.incoming.hostname,
            "imap.corpmail.example.com"
        );
        assert_eq!(candidate.provider.incoming.port, 993);
        assert_eq!(
            candidate.provider.incoming.encryption,
            ProviderEncryption::SslTls
        );
        assert_eq!(
            candidate.provider.outgoing.hostname,
            "smtp.corpmail.example.com"
        );
        assert_eq!(candidate.provider.outgoing.port, 587);
        assert_eq!(
            candidate.provider.outgoing.encryption,
            ProviderEncryption::StartTls
        );
        assert_eq!(candidate.provider.username_type, UsernameType::EmailAddress);
    }

    #[test]
    fn test_vendor_discovery_fallback_to_second_url() {
        let client = MockHttp::new()
            .with_error(
                "https://autoconfig.fallback.example.com/mail/config-v1.1.xml",
                "connection refused",
            )
            .with_response(
                "https://fallback.example.com/.well-known/autoconfig/mail/config-v1.1.xml",
                VENDOR_AUTOCONFIG,
            );

        let candidate = discover_by_vendor("fallback.example.com", &client).unwrap();
        assert_eq!(candidate.score, MatchScore::VENDOR_AUTODISCOVERY);
        assert_eq!(
            candidate.provider.incoming.hostname,
            "imap.corpmail.example.com"
        );
    }

    #[test]
    fn test_vendor_discovery_all_urls_fail() {
        let client = MockHttp::new()
            .with_error(
                "https://autoconfig.nope.example.com/mail/config-v1.1.xml",
                "timeout",
            )
            .with_error(
                "https://nope.example.com/.well-known/autoconfig/mail/config-v1.1.xml",
                "404",
            );

        let result = discover_by_vendor("nope.example.com", &client);
        assert!(result.is_err());
    }

    // --- Privacy (FR-38) ---

    #[test]
    fn test_vendor_urls_contain_only_domain() {
        for template in AUTOCONFIG_URL_TEMPLATES {
            let url = template.replace("{domain}", "example.com");
            assert!(!url.contains('@'), "URL should not contain email: {url}");
            assert!(
                !url.contains("password"),
                "URL should not contain password: {url}"
            );
        }
    }
}
