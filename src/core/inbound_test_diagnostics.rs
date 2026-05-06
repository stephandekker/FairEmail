use crate::core::inbound_test::InboundTestError;
use crate::core::provider::ProviderDatabase;

/// A diagnosed connection error with an actionable user-facing message
/// and an optional link to provider documentation.
#[derive(Debug, Clone)]
pub struct ConnectionDiagnostic {
    /// The primary error message describing what went wrong.
    pub message: String,
    /// Actionable guidance for the user to resolve the issue.
    pub guidance: String,
    /// Optional URL to provider setup documentation.
    pub documentation_url: Option<String>,
}

impl ConnectionDiagnostic {
    /// Produce a full display string combining message and guidance.
    pub fn display_text(&self) -> String {
        format!("{}\n\n{}", self.message, self.guidance)
    }
}

/// Diagnose an inbound test error and produce an actionable diagnostic.
///
/// When `hostname` is provided, the provider database is consulted
/// to attach a documentation link if the provider has one.
pub fn diagnose_error(
    error: &InboundTestError,
    hostname: Option<&str>,
    provider_db: &ProviderDatabase,
) -> ConnectionDiagnostic {
    let doc_url = hostname.and_then(|h| {
        provider_db
            .lookup_by_hostname(h)
            .and_then(|p| p.documentation_url.clone())
    });

    let (message, guidance) = match error {
        InboundTestError::DnsResolutionFailed(host) => (
            format!("Unknown host: Could not resolve \"{host}\""),
            "Check the hostname for typos. Make sure the server address is correct and that your network connection is working.".to_string(),
        ),
        InboundTestError::ConnectionRefused { host, port } => (
            format!("Connection refused by {host} on port {port}"),
            format!(
                "The server at {host}:{port} is not accepting connections. Verify the hostname and port are correct. The server may be down or a firewall may be blocking the connection."
            ),
        ),
        InboundTestError::Timeout => (
            "Connection timed out".to_string(),
            "The server did not respond within 30 seconds. Check that the hostname and port are correct, and that no firewall is blocking the connection.".to_string(),
        ),
        InboundTestError::TlsHandshakeFailed { message, .. } => (
            format!("TLS/SSL error: {message}"),
            "The secure connection could not be established. This may indicate a certificate problem or a protocol mismatch. Try a different encryption mode, or verify the server supports TLS on this port.".to_string(),
        ),
        InboundTestError::AuthenticationFailed => (
            "Authentication failed: wrong credentials".to_string(),
            "The server rejected the username or password. Double-check your credentials. If your provider requires an app-specific password, generate one in your provider's security settings.".to_string(),
        ),
        InboundTestError::ProtocolMismatch(detail) => (
            format!("Protocol mismatch: {detail}"),
            "The server responded with an unexpected protocol. This usually means you are connecting to the wrong port (e.g. using an IMAP client on a POP3 port). Check your protocol and port settings.".to_string(),
        ),
        InboundTestError::ConnectionFailed(detail) => (
            format!("Connection failed: {detail}"),
            "Could not establish a connection to the server. Verify the hostname and port, and check your network connection.".to_string(),
        ),
        InboundTestError::EmptyHost => (
            "No hostname specified".to_string(),
            "Enter the server hostname to test the connection.".to_string(),
        ),
        InboundTestError::EmptyUsername => (
            "No username specified".to_string(),
            "Enter your username (usually your email address) to test the connection.".to_string(),
        ),
        InboundTestError::DnssecFailed(detail) => (
            format!("DNSSEC validation failed: {detail}"),
            "The DNS response for this server could not be validated with DNSSEC. The server's domain may not have DNSSEC signing enabled. Disable the DNSSEC toggle if you do not require DNSSEC validation.".to_string(),
        ),
        InboundTestError::DaneFailed(detail) => (
            format!("DANE verification failed: {detail}"),
            "The server's TLS certificate could not be verified against TLSA DNS records. The server may not publish TLSA records, or the records may not match the certificate. Disable the DANE toggle if you do not require DANE verification.".to_string(),
        ),
        InboundTestError::EmptyCredential => (
            "No password specified".to_string(),
            "Enter your password to test the connection.".to_string(),
        ),
    };

    ConnectionDiagnostic {
        message,
        guidance,
        documentation_url: doc_url,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::provider::{
        MaxTlsVersion, Provider, ProviderDatabase, ProviderEncryption, ServerConfig, UsernameType,
    };

    fn empty_db() -> ProviderDatabase {
        ProviderDatabase::new(vec![])
    }

    fn db_with_docs() -> ProviderDatabase {
        let provider = Provider {
            id: "testprovider".to_string(),
            display_name: "Test Provider".to_string(),
            domain_patterns: vec!["testprovider.com".to_string()],
            mx_patterns: vec![],
            incoming: ServerConfig {
                hostname: "imap.testprovider.com".to_string(),
                port: 993,
                encryption: ProviderEncryption::SslTls,
            },
            outgoing: ServerConfig {
                hostname: "smtp.testprovider.com".to_string(),
                port: 465,
                encryption: ProviderEncryption::SslTls,
            },
            username_type: UsernameType::EmailAddress,
            keep_alive_interval: 15,
            noop_keep_alive: false,
            partial_fetch: true,
            max_tls_version: MaxTlsVersion::Tls1_3,
            app_password_required: false,
            documentation_url: Some("https://help.testprovider.com/imap-setup".to_string()),
            localized_docs: vec![],
            oauth: None,
            display_order: 0,
            enabled: true,
            supports_shared_mailbox: false,
        };
        ProviderDatabase::new(vec![provider])
    }

    #[test]
    fn dns_resolution_error_shows_hostname() {
        let err = InboundTestError::DnsResolutionFailed("mail.example.com".to_string());
        let diag = diagnose_error(&err, None, &empty_db());
        assert!(diag.message.contains("mail.example.com"));
        assert!(diag.message.contains("Unknown host"));
        assert!(diag.guidance.contains("typos"));
    }

    #[test]
    fn connection_refused_shows_host_and_port() {
        let err = InboundTestError::ConnectionRefused {
            host: "imap.example.com".to_string(),
            port: 993,
        };
        let diag = diagnose_error(&err, None, &empty_db());
        assert!(diag.message.contains("imap.example.com"));
        assert!(diag.message.contains("993"));
        assert!(diag.guidance.contains("port"));
    }

    #[test]
    fn tls_handshake_error_shows_guidance() {
        let err = InboundTestError::TlsHandshakeFailed {
            message: "certificate expired".to_string(),
            fingerprint: None,
        };
        let diag = diagnose_error(&err, None, &empty_db());
        assert!(diag.message.contains("certificate expired"));
        assert!(diag.guidance.contains("encryption mode"));
    }

    #[test]
    fn auth_failure_shows_wrong_credentials() {
        let err = InboundTestError::AuthenticationFailed;
        let diag = diagnose_error(&err, None, &empty_db());
        assert!(diag.message.contains("wrong credentials"));
        assert!(diag.guidance.contains("app-specific password"));
    }

    #[test]
    fn protocol_mismatch_shows_hint() {
        let err =
            InboundTestError::ProtocolMismatch("expected IMAP greeting, got POP3".to_string());
        let diag = diagnose_error(&err, None, &empty_db());
        assert!(diag.message.contains("Protocol mismatch"));
        assert!(diag.guidance.contains("wrong port"));
    }

    #[test]
    fn timeout_error_is_actionable() {
        let err = InboundTestError::Timeout;
        let diag = diagnose_error(&err, None, &empty_db());
        assert!(diag.message.contains("timed out"));
        assert!(diag.guidance.contains("30 seconds"));
    }

    #[test]
    fn provider_doc_link_included_when_hostname_matches() {
        let err = InboundTestError::AuthenticationFailed;
        let db = db_with_docs();
        let diag = diagnose_error(&err, Some("imap.testprovider.com"), &db);
        assert_eq!(
            diag.documentation_url,
            Some("https://help.testprovider.com/imap-setup".to_string())
        );
    }

    #[test]
    fn no_doc_link_when_provider_not_found() {
        let err = InboundTestError::AuthenticationFailed;
        let db = db_with_docs();
        let diag = diagnose_error(&err, Some("imap.unknown.com"), &db);
        assert!(diag.documentation_url.is_none());
    }

    #[test]
    fn no_doc_link_when_hostname_not_provided() {
        let err = InboundTestError::AuthenticationFailed;
        let db = db_with_docs();
        let diag = diagnose_error(&err, None, &db);
        assert!(diag.documentation_url.is_none());
    }

    #[test]
    fn display_text_combines_message_and_guidance() {
        let err = InboundTestError::DnsResolutionFailed("mail.test.com".to_string());
        let diag = diagnose_error(&err, None, &empty_db());
        let text = diag.display_text();
        assert!(text.contains(&diag.message));
        assert!(text.contains(&diag.guidance));
    }

    #[test]
    fn connection_failed_generic_is_actionable() {
        let err = InboundTestError::ConnectionFailed("network unreachable".to_string());
        let diag = diagnose_error(&err, None, &empty_db());
        assert!(diag.message.contains("network unreachable"));
        assert!(diag.guidance.contains("network connection"));
    }

    #[test]
    fn dnssec_error_shows_detail() {
        let err =
            InboundTestError::DnssecFailed("DNSSEC validation failed for mail.test.com".into());
        let diag = diagnose_error(&err, None, &empty_db());
        assert!(diag.message.contains("DNSSEC"));
        assert!(diag.guidance.contains("DNSSEC"));
    }

    #[test]
    fn dane_error_shows_detail() {
        let err = InboundTestError::DaneFailed("no matching TLSA record".into());
        let diag = diagnose_error(&err, None, &empty_db());
        assert!(diag.message.contains("DANE"));
        assert!(diag.guidance.contains("TLSA"));
    }
}
