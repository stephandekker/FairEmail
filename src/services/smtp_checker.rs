use crate::core::certificate::CertificateInfo;
use crate::core::imap_check::resolve_username_candidates;
use crate::core::provider::Provider;
use crate::core::smtp_check::{
    SmtpCheckError, SmtpCheckResult, SmtpCheckSuccess, SmtpConnectionParams,
};

/// Trait for performing SMTP connectivity checks.
/// Implementations handle the actual network I/O (connect, authenticate, query SIZE).
pub trait SmtpChecker {
    /// Perform a full SMTP connectivity check:
    /// 1. Connect to the server using the provider's outgoing settings (FR-17.4).
    /// 2. Try each username candidate until one authenticates successfully (FR-17.5).
    /// 3. Query the server's maximum message size if advertised (FR-17.6).
    ///
    /// `accepted_fingerprint` allows bypassing certificate validation when the user
    /// has previously accepted a specific certificate fingerprint (FR-19d).
    fn check_smtp(
        &self,
        email: &str,
        password: &str,
        provider: &Provider,
        accepted_fingerprint: Option<&str>,
    ) -> SmtpCheckResult;
}

/// Mock implementation of `SmtpChecker` for testing.
///
/// Behavior:
/// - Hosts containing "unreachable" -> `ConnectionFailed`
/// - Hosts containing "authfail" -> `AuthenticationFailed` (all usernames fail)
/// - Hosts containing "untrustedcert" -> `UntrustedCertificate` (unless accepted fingerprint matches)
/// - Password "wrong" -> `AuthenticationFailed`
/// - Username "failfirst@" prefix in email -> first candidate fails, second succeeds
/// - Hosts containing "nosize" -> success with no max message size advertised
/// - Otherwise -> success with max_message_size = 26_214_400 (25 MiB)
pub struct MockSmtpChecker;

/// The fingerprint the mock uses for simulated untrusted certificates.
pub const MOCK_SMTP_CERT_FINGERPRINT: &str =
    "FE:DC:BA:98:76:54:32:10:FE:DC:BA:98:76:54:32:10:FE:DC:BA:98:76:54:32:10:FE:DC:BA:98:76:54:32:10";

impl SmtpChecker for MockSmtpChecker {
    fn check_smtp(
        &self,
        email: &str,
        password: &str,
        provider: &Provider,
        accepted_fingerprint: Option<&str>,
    ) -> SmtpCheckResult {
        let params = SmtpConnectionParams::from_provider(provider);

        // Simulate connection failure
        if params.host.to_lowercase().contains("unreachable") {
            return Err(SmtpCheckError::ConnectionFailed(
                "could not connect to host".to_string(),
            ));
        }

        // Simulate untrusted certificate (FR-19)
        if params.host.to_lowercase().contains("untrustedcert")
            && accepted_fingerprint != Some(MOCK_SMTP_CERT_FINGERPRINT)
        {
            return Err(SmtpCheckError::UntrustedCertificate(Box::new(
                CertificateInfo {
                    fingerprint: MOCK_SMTP_CERT_FINGERPRINT.to_string(),
                    dns_names: vec![
                        "*.smtp-server.example.net".to_string(),
                        "smtp-server.example.net".to_string(),
                    ],
                    server_hostname: params.host.clone(),
                },
            )));
        }

        // Simulate auth failure for all candidates
        if params.host.to_lowercase().contains("authfail") || password == "wrong" {
            return Err(SmtpCheckError::AuthenticationFailed);
        }

        let candidates = resolve_username_candidates(email, provider);

        // Simulate first-candidate-fails scenario
        let authenticated_username = if email.starts_with("failfirst@") {
            if candidates.len() > 1 {
                candidates[1].value().to_string()
            } else {
                return Err(SmtpCheckError::AuthenticationFailed);
            }
        } else {
            candidates[0].value().to_string()
        };

        // Determine max message size: hosts with "nosize" don't advertise it
        let max_message_size = if params.host.to_lowercase().contains("nosize") {
            None
        } else {
            Some(26_214_400) // 25 MiB
        };

        Ok(SmtpCheckSuccess {
            authenticated_username,
            max_message_size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::provider::{MaxTlsVersion, ProviderEncryption, ServerConfig, UsernameType};

    fn test_provider(username_type: UsernameType) -> Provider {
        Provider {
            id: "test".to_string(),
            display_name: "Test".to_string(),
            domain_patterns: vec!["example.com".to_string()],
            mx_patterns: vec![],
            incoming: ServerConfig {
                hostname: "imap.example.com".to_string(),
                port: 993,
                encryption: ProviderEncryption::SslTls,
            },
            outgoing: ServerConfig {
                hostname: "smtp.example.com".to_string(),
                port: 465,
                encryption: ProviderEncryption::SslTls,
            },
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
        }
    }

    #[test]
    fn successful_check_returns_username_and_size() {
        let checker = MockSmtpChecker;
        let provider = test_provider(UsernameType::EmailAddress);
        let result = checker
            .check_smtp("user@example.com", "secret", &provider, None)
            .unwrap();

        assert_eq!(result.authenticated_username, "user@example.com");
        assert_eq!(result.max_message_size, Some(26_214_400));
    }

    #[test]
    fn successful_check_no_size_advertised() {
        let checker = MockSmtpChecker;
        let mut provider = test_provider(UsernameType::EmailAddress);
        provider.outgoing.hostname = "nosize.smtp.example.com".to_string();
        let result = checker
            .check_smtp("user@example.com", "secret", &provider, None)
            .unwrap();

        assert_eq!(result.authenticated_username, "user@example.com");
        assert_eq!(result.max_message_size, None);
    }

    #[test]
    fn connection_failure_for_unreachable_host() {
        let checker = MockSmtpChecker;
        let mut provider = test_provider(UsernameType::EmailAddress);
        provider.outgoing.hostname = "unreachable.smtp.example.com".to_string();
        let result = checker.check_smtp("user@example.com", "secret", &provider, None);
        assert!(matches!(result, Err(SmtpCheckError::ConnectionFailed(_))));
    }

    #[test]
    fn auth_failure_for_authfail_host() {
        let checker = MockSmtpChecker;
        let mut provider = test_provider(UsernameType::EmailAddress);
        provider.outgoing.hostname = "authfail.smtp.example.com".to_string();
        let result = checker.check_smtp("user@example.com", "secret", &provider, None);
        assert!(matches!(result, Err(SmtpCheckError::AuthenticationFailed)));
    }

    #[test]
    fn auth_failure_for_wrong_password() {
        let checker = MockSmtpChecker;
        let provider = test_provider(UsernameType::EmailAddress);
        let result = checker.check_smtp("user@example.com", "wrong", &provider, None);
        assert!(matches!(result, Err(SmtpCheckError::AuthenticationFailed)));
    }

    #[test]
    fn fallback_username_on_first_failure() {
        let checker = MockSmtpChecker;
        let provider = test_provider(UsernameType::EmailAddress);
        let result = checker
            .check_smtp("failfirst@example.com", "secret", &provider, None)
            .unwrap();

        // Primary is EmailAddress, so fallback is LocalPart
        assert_eq!(result.authenticated_username, "failfirst");
    }

    #[test]
    fn fallback_username_local_part_primary() {
        let checker = MockSmtpChecker;
        let provider = test_provider(UsernameType::LocalPart);
        let result = checker
            .check_smtp("failfirst@example.com", "secret", &provider, None)
            .unwrap();

        // Primary is LocalPart, so fallback is EmailAddress
        assert_eq!(result.authenticated_username, "failfirst@example.com");
    }

    #[test]
    fn untrusted_certificate_error_for_untrustedcert_host() {
        let checker = MockSmtpChecker;
        let mut provider = test_provider(UsernameType::EmailAddress);
        provider.outgoing.hostname = "untrustedcert.smtp.example.com".to_string();
        let result = checker.check_smtp("user@example.com", "secret", &provider, None);
        assert!(matches!(
            result,
            Err(SmtpCheckError::UntrustedCertificate(_))
        ));
        if let Err(SmtpCheckError::UntrustedCertificate(ref info)) = result {
            assert_eq!(info.fingerprint, MOCK_SMTP_CERT_FINGERPRINT);
            assert!(!info.dns_names.is_empty());
        }
    }

    #[test]
    fn untrusted_certificate_bypassed_with_accepted_fingerprint() {
        let checker = MockSmtpChecker;
        let mut provider = test_provider(UsernameType::EmailAddress);
        provider.outgoing.hostname = "untrustedcert.smtp.example.com".to_string();
        let result = checker.check_smtp(
            "user@example.com",
            "secret",
            &provider,
            Some(MOCK_SMTP_CERT_FINGERPRINT),
        );
        assert!(result.is_ok());
    }
}
