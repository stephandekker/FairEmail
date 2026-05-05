use super::provider::{
    MatchScore, MaxTlsVersion, Provider, ProviderCandidate, ProviderEncryption, ServerConfig,
    UsernameType,
};

/// Errors from ISPDB / autoconfig operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum AutoconfigError {
    #[error("HTTP request failed: {0}")]
    HttpFailed(String),
    #[error("invalid autoconfig XML: {0}")]
    InvalidXml(String),
    #[error("no IMAP server found in autoconfig")]
    NoImapServer,
}

/// Trait abstracting HTTP GET requests so we can test without real network calls.
pub(crate) trait HttpClient {
    /// Perform an HTTP GET request and return the response body as a string.
    /// Returns Err if the request fails or returns a non-success status.
    fn get(&self, url: &str) -> Result<String, AutoconfigError>;
}

/// The Thunderbird ISPDB base URL.
const ISPDB_BASE_URL: &str = "https://autoconfig.thunderbird.net/v1.1";

/// Query the Thunderbird ISPDB for the given domain (FR-10.5).
///
/// Only the domain is sent — the user's password is never transmitted (FR-38).
pub(crate) fn discover_by_ispdb(
    domain: &str,
    client: &dyn HttpClient,
) -> Result<ProviderCandidate, AutoconfigError> {
    let url = format!("{ISPDB_BASE_URL}/{domain}");
    let body = client.get(&url)?;
    let provider = parse_autoconfig_xml(&body, domain)?;
    Ok(ProviderCandidate {
        provider,
        score: MatchScore::ISPDB,
    })
}

/// Parse Mozilla autoconfig XML into a `Provider`.
///
/// Handles the `<clientConfig>` / `<emailProvider>` format used by both the
/// Thunderbird ISPDB and vendor-hosted autoconfig endpoints.
pub(crate) fn parse_autoconfig_xml(xml: &str, domain: &str) -> Result<Provider, AutoconfigError> {
    let incoming =
        parse_server_block(xml, "incomingServer").ok_or(AutoconfigError::NoImapServer)?;
    let outgoing = parse_server_block(xml, "outgoingServer").unwrap_or(ServerConfig {
        hostname: format!("smtp.{domain}"),
        port: 465,
        encryption: ProviderEncryption::SslTls,
    });

    let username_type = if contains_username_tag(xml, "%EMAILLOCALPART%") {
        UsernameType::LocalPart
    } else {
        UsernameType::EmailAddress
    };

    let id = extract_tag_content(xml, "displayName")
        .map(|n| n.to_lowercase().replace(' ', "-"))
        .unwrap_or_else(|| format!("ispdb-{domain}"));
    let display_name =
        extract_tag_content(xml, "displayName").unwrap_or_else(|| domain.to_string());

    Ok(Provider {
        id,
        display_name,
        domain_patterns: vec![domain.to_string()],
        mx_patterns: vec![],
        incoming,
        outgoing,
        username_type,
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
    })
}

// ---------------------------------------------------------------------------
// Simple XML helpers — sufficient for the well-known autoconfig schema.
// ---------------------------------------------------------------------------

/// Extract the text content of the first occurrence of `<tag>…</tag>`.
fn extract_tag_content(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    let content = xml[start..end].trim().to_string();
    if content.is_empty() {
        None
    } else {
        Some(content)
    }
}

/// Extract tag content within a specific section of XML.
fn extract_tag_in_section(section: &str, tag: &str) -> Option<String> {
    extract_tag_content(section, tag)
}

/// Check whether a `<username>` tag in the XML contains the given placeholder.
fn contains_username_tag(xml: &str, placeholder: &str) -> bool {
    extract_tag_content(xml, "username").is_some_and(|u| u.contains(placeholder))
}

/// Parse a `<incomingServer>` or `<outgoingServer>` block into a `ServerConfig`.
fn parse_server_block(xml: &str, block_tag: &str) -> Option<ServerConfig> {
    // Find the block boundaries
    let open_prefix = format!("<{block_tag}");
    let close = format!("</{block_tag}>");
    let block_start = xml.find(&open_prefix)?;
    let block_end = xml[block_start..].find(&close)? + block_start + close.len();
    let section = &xml[block_start..block_end];

    let hostname = extract_tag_in_section(section, "hostname")?;
    let port: u16 = extract_tag_in_section(section, "port")?.parse().ok()?;
    let socket_type = extract_tag_in_section(section, "socketType").unwrap_or_default();
    let encryption = match socket_type.to_uppercase().as_str() {
        "SSL" | "TLS" => ProviderEncryption::SslTls,
        "STARTTLS" => ProviderEncryption::StartTls,
        _ => ProviderEncryption::SslTls,
    };

    Some(ServerConfig {
        hostname,
        port,
        encryption,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Mock HTTP client that returns canned responses.
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

    const SAMPLE_AUTOCONFIG: &str = r#"<?xml version="1.0"?>
<clientConfig version="1.1">
  <emailProvider id="example.com">
    <domain>example.com</domain>
    <displayName>Example Mail</displayName>
    <incomingServer type="imap">
      <hostname>imap.example.com</hostname>
      <port>993</port>
      <socketType>SSL</socketType>
      <username>%EMAILADDRESS%</username>
      <authentication>password-cleartext</authentication>
    </incomingServer>
    <outgoingServer type="smtp">
      <hostname>smtp.example.com</hostname>
      <port>465</port>
      <socketType>SSL</socketType>
      <username>%EMAILADDRESS%</username>
      <authentication>password-cleartext</authentication>
    </outgoingServer>
  </emailProvider>
</clientConfig>"#;

    const STARTTLS_AUTOCONFIG: &str = r#"<?xml version="1.0"?>
<clientConfig version="1.1">
  <emailProvider id="starttls.example.com">
    <domain>starttls.example.com</domain>
    <displayName>StartTLS Provider</displayName>
    <incomingServer type="imap">
      <hostname>mail.starttls.example.com</hostname>
      <port>143</port>
      <socketType>STARTTLS</socketType>
      <username>%EMAILLOCALPART%</username>
    </incomingServer>
    <outgoingServer type="smtp">
      <hostname>mail.starttls.example.com</hostname>
      <port>587</port>
      <socketType>STARTTLS</socketType>
      <username>%EMAILLOCALPART%</username>
    </outgoingServer>
  </emailProvider>
</clientConfig>"#;

    // --- Score ordering (FR-11) ---

    #[test]
    fn test_ispdb_score_below_dns() {
        assert!(MatchScore::DNS_SRV > MatchScore::ISPDB);
        assert!(MatchScore::DNS_MX > MatchScore::ISPDB);
        assert!(MatchScore::DNS_NS > MatchScore::ISPDB);
    }

    #[test]
    fn test_ispdb_score_above_vendor() {
        assert!(MatchScore::ISPDB > MatchScore::VENDOR_AUTODISCOVERY);
    }

    // --- ISPDB discovery (FR-10.5) ---

    #[test]
    fn test_ispdb_success() {
        let client = MockHttp::new().with_response(
            "https://autoconfig.thunderbird.net/v1.1/example.com",
            SAMPLE_AUTOCONFIG,
        );

        let result = discover_by_ispdb("example.com", &client);
        assert!(result.is_ok());

        let candidate = result.unwrap();
        assert_eq!(candidate.score, MatchScore::ISPDB);
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
    }

    #[test]
    fn test_ispdb_starttls_and_localpart_username() {
        let client = MockHttp::new().with_response(
            "https://autoconfig.thunderbird.net/v1.1/starttls.example.com",
            STARTTLS_AUTOCONFIG,
        );

        let candidate = discover_by_ispdb("starttls.example.com", &client).unwrap();
        assert_eq!(
            candidate.provider.incoming.encryption,
            ProviderEncryption::StartTls
        );
        assert_eq!(candidate.provider.incoming.port, 143);
        assert_eq!(
            candidate.provider.outgoing.encryption,
            ProviderEncryption::StartTls
        );
        assert_eq!(candidate.provider.outgoing.port, 587);
        assert_eq!(candidate.provider.username_type, UsernameType::LocalPart);
    }

    #[test]
    fn test_ispdb_http_failure() {
        let client = MockHttp::new().with_error(
            "https://autoconfig.thunderbird.net/v1.1/unknown.example.com",
            "404 Not Found",
        );

        let result = discover_by_ispdb("unknown.example.com", &client);
        assert!(result.is_err());
    }

    #[test]
    fn test_ispdb_invalid_xml() {
        let client = MockHttp::new().with_response(
            "https://autoconfig.thunderbird.net/v1.1/bad.com",
            "<html>Not autoconfig</html>",
        );

        let result = discover_by_ispdb("bad.com", &client);
        assert!(result.is_err());
    }

    // --- Privacy (FR-38) ---

    #[test]
    fn test_ispdb_url_contains_only_domain() {
        // Verify the URL format never includes a password or email
        let url = format!("{ISPDB_BASE_URL}/{}", "example.com");
        assert!(!url.contains('@'));
        assert!(!url.contains("password"));
        assert_eq!(url, "https://autoconfig.thunderbird.net/v1.1/example.com");
    }

    // --- XML parsing ---

    #[test]
    fn test_parse_autoconfig_xml_valid() {
        let provider = parse_autoconfig_xml(SAMPLE_AUTOCONFIG, "example.com").unwrap();
        assert_eq!(provider.display_name, "Example Mail");
        assert_eq!(provider.incoming.hostname, "imap.example.com");
        assert_eq!(provider.outgoing.hostname, "smtp.example.com");
        assert_eq!(provider.username_type, UsernameType::EmailAddress);
    }

    #[test]
    fn test_parse_autoconfig_xml_no_incoming_server() {
        let xml = r#"<?xml version="1.0"?>
<clientConfig>
  <emailProvider>
    <outgoingServer type="smtp">
      <hostname>smtp.example.com</hostname>
      <port>465</port>
      <socketType>SSL</socketType>
    </outgoingServer>
  </emailProvider>
</clientConfig>"#;

        let result = parse_autoconfig_xml(xml, "example.com");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AutoconfigError::NoImapServer));
    }

    #[test]
    fn test_parse_autoconfig_xml_missing_outgoing_uses_defaults() {
        let xml = r#"<?xml version="1.0"?>
<clientConfig>
  <emailProvider>
    <incomingServer type="imap">
      <hostname>imap.example.com</hostname>
      <port>993</port>
      <socketType>SSL</socketType>
    </incomingServer>
  </emailProvider>
</clientConfig>"#;

        let provider = parse_autoconfig_xml(xml, "example.com").unwrap();
        assert_eq!(provider.incoming.hostname, "imap.example.com");
        // Outgoing falls back to smtp.{domain}:465
        assert_eq!(provider.outgoing.hostname, "smtp.example.com");
        assert_eq!(provider.outgoing.port, 465);
    }

    // --- AC-5: Domain with ISPDB entry but no SRV produces valid settings ---

    #[test]
    fn test_ispdb_produces_valid_settings_without_srv() {
        let client = MockHttp::new().with_response(
            "https://autoconfig.thunderbird.net/v1.1/smallisp.com",
            r#"<?xml version="1.0"?>
<clientConfig version="1.1">
  <emailProvider id="smallisp.com">
    <domain>smallisp.com</domain>
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

        let candidate = discover_by_ispdb("smallisp.com", &client).unwrap();

        // Valid IMAP settings
        assert_eq!(candidate.provider.incoming.hostname, "mail.smallisp.com");
        assert_eq!(candidate.provider.incoming.port, 993);
        assert_eq!(
            candidate.provider.incoming.encryption,
            ProviderEncryption::SslTls
        );
        // Valid SMTP settings
        assert_eq!(candidate.provider.outgoing.hostname, "mail.smallisp.com");
        assert_eq!(candidate.provider.outgoing.port, 465);
        assert_eq!(
            candidate.provider.outgoing.encryption,
            ProviderEncryption::SslTls
        );
        // Score is ISPDB (below DNS)
        assert_eq!(candidate.score, MatchScore::ISPDB);
    }
}
