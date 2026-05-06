use crate::core::inbound_test_diagnostics::ConnectionDiagnostic;
use crate::core::provider::ProviderDatabase;
use crate::core::smtp_check::SmtpCheckError;

/// Diagnose an SMTP check error and produce an actionable diagnostic.
///
/// When `hostname` is provided, the provider database is consulted
/// to attach a documentation link if the provider has one.
pub fn diagnose_smtp_error(
    error: &SmtpCheckError,
    hostname: Option<&str>,
    provider_db: &ProviderDatabase,
) -> ConnectionDiagnostic {
    let doc_url = hostname.and_then(|h| {
        provider_db
            .lookup_by_hostname(h)
            .and_then(|p| p.documentation_url.clone())
    });

    let (message, guidance) = match error {
        SmtpCheckError::ConnectionFailed(detail) => (
            format!("Connection failed: {detail}"),
            "Could not establish a connection to the SMTP server. Verify the hostname and port, and check your network connection.".to_string(),
        ),
        SmtpCheckError::AuthenticationFailed => (
            "Authentication failed: wrong credentials".to_string(),
            "The SMTP server rejected the username or password. Double-check your credentials. If your provider requires an app-specific password, generate one in your provider's security settings.".to_string(),
        ),
        SmtpCheckError::UntrustedCertificate(info) => (
            format!(
                "Untrusted certificate from server \"{}\"",
                info.server_hostname
            ),
            "The server presented a certificate that could not be verified. This may indicate a self-signed certificate or a misconfigured server. Verify that you are connecting to the correct server.".to_string(),
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
    use crate::core::certificate::CertificateInfo;
    use crate::core::provider::{
        MaxTlsVersion, Provider, ProviderEncryption, ServerConfig, UsernameType,
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
            documentation_url: Some("https://help.testprovider.com/smtp-setup".to_string()),
            localized_docs: vec![],
            oauth: None,
            display_order: 0,
            enabled: true,
            supports_shared_mailbox: false,
        };
        ProviderDatabase::new(vec![provider])
    }

    #[test]
    fn connection_failed_shows_detail() {
        let err = SmtpCheckError::ConnectionFailed("connection refused".to_string());
        let diag = diagnose_smtp_error(&err, None, &empty_db());
        assert!(diag.message.contains("connection refused"));
        assert!(diag.guidance.contains("SMTP server"));
    }

    #[test]
    fn auth_failure_shows_credentials_hint() {
        let err = SmtpCheckError::AuthenticationFailed;
        let diag = diagnose_smtp_error(&err, None, &empty_db());
        assert!(diag.message.contains("wrong credentials"));
        assert!(diag.guidance.contains("app-specific password"));
    }

    #[test]
    fn untrusted_certificate_shows_hostname() {
        let err = SmtpCheckError::UntrustedCertificate(Box::new(CertificateInfo {
            fingerprint: "AA:BB:CC".to_string(),
            dns_names: vec!["smtp.example.com".to_string()],
            server_hostname: "smtp.example.com".to_string(),
        }));
        let diag = diagnose_smtp_error(&err, None, &empty_db());
        assert!(diag.message.contains("smtp.example.com"));
        assert!(diag.guidance.contains("certificate"));
    }

    #[test]
    fn provider_doc_link_included_when_hostname_matches() {
        let err = SmtpCheckError::AuthenticationFailed;
        let db = db_with_docs();
        let diag = diagnose_smtp_error(&err, Some("smtp.testprovider.com"), &db);
        assert_eq!(
            diag.documentation_url,
            Some("https://help.testprovider.com/smtp-setup".to_string())
        );
    }

    #[test]
    fn no_doc_link_when_provider_not_found() {
        let err = SmtpCheckError::AuthenticationFailed;
        let db = db_with_docs();
        let diag = diagnose_smtp_error(&err, Some("smtp.unknown.com"), &db);
        assert!(diag.documentation_url.is_none());
    }

    #[test]
    fn no_doc_link_when_hostname_not_provided() {
        let err = SmtpCheckError::AuthenticationFailed;
        let db = db_with_docs();
        let diag = diagnose_smtp_error(&err, None, &db);
        assert!(diag.documentation_url.is_none());
    }

    #[test]
    fn display_text_combines_message_and_guidance() {
        let err = SmtpCheckError::ConnectionFailed("timeout".to_string());
        let diag = diagnose_smtp_error(&err, None, &empty_db());
        let text = diag.display_text();
        assert!(text.contains(&diag.message));
        assert!(text.contains(&diag.guidance));
    }
}
