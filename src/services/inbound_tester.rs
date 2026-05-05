use crate::core::account::{FolderRole, Protocol};
use crate::core::imap_check::ImapFolder;
use crate::core::inbound_test::{
    InboundTestError, InboundTestParams, InboundTestResult, InboundTestSuccess,
};

/// Trait for testing inbound mail server connections.
/// Implementations handle the actual network I/O.
pub trait InboundTester {
    fn test_inbound(&self, params: &InboundTestParams) -> InboundTestResult;
}

/// Mock implementation that simulates an inbound connection test.
/// Returns a realistic folder list for IMAP, reports IDLE and UTF-8 support.
/// Simulates failures for hosts containing "unreachable", "authfail", or "timeout".
pub struct MockInboundTester;

impl InboundTester for MockInboundTester {
    fn test_inbound(&self, params: &InboundTestParams) -> InboundTestResult {
        params.validate()?;

        let host_lower = params.host.to_lowercase();

        // Simulate various failure modes.
        if host_lower.contains("dnsfail") {
            return Err(InboundTestError::DnsResolutionFailed(params.host.clone()));
        }
        if host_lower.contains("refused") {
            return Err(InboundTestError::ConnectionRefused {
                host: params.host.clone(),
                port: params.port,
            });
        }
        if host_lower.contains("timeout") {
            return Err(InboundTestError::Timeout);
        }
        if host_lower.contains("tlsfail") {
            // Simulate: the server has a self-signed cert with this fingerprint.
            let mock_fingerprint =
                "AB:CD:EF:01:23:45:67:89:AB:CD:EF:01:23:45:67:89:AB:CD:EF:01:23:45:67:89:AB:CD:EF:01:23:45:67:89".to_string();
            // If the caller has pinned this exact fingerprint, the connection succeeds.
            if params
                .accepted_fingerprint
                .as_ref()
                .is_some_and(|fp| fp == &mock_fingerprint)
            {
                // Fall through to success path.
            } else {
                return Err(InboundTestError::TlsHandshakeFailed {
                    message: "certificate verification failed: self-signed certificate".into(),
                    fingerprint: Some(mock_fingerprint),
                });
            }
        }
        if host_lower.contains("unreachable") {
            return Err(InboundTestError::ConnectionFailed(
                "could not connect to host".into(),
            ));
        }
        if host_lower.contains("authfail") {
            return Err(InboundTestError::AuthenticationFailed);
        }
        if host_lower.contains("protomismatch") {
            return Err(InboundTestError::ProtocolMismatch(
                "expected IMAP greeting, got POP3 banner".into(),
            ));
        }

        match params.protocol {
            Protocol::Imap => Ok(mock_imap_success(&host_lower)),
            Protocol::Pop3 => Ok(InboundTestSuccess {
                folders: vec![],
                idle_supported: false,
                utf8_supported: false,
            }),
        }
    }
}

fn mock_imap_success(host_lower: &str) -> InboundTestSuccess {
    let folders = vec![
        ImapFolder {
            name: "INBOX".into(),
            attributes: "".into(),
            role: None,
        },
        ImapFolder {
            name: "Drafts".into(),
            attributes: "\\Drafts".into(),
            role: Some(FolderRole::Drafts),
        },
        ImapFolder {
            name: "Sent".into(),
            attributes: "\\Sent".into(),
            role: Some(FolderRole::Sent),
        },
        ImapFolder {
            name: "Trash".into(),
            attributes: "\\Trash".into(),
            role: Some(FolderRole::Trash),
        },
        ImapFolder {
            name: "Spam".into(),
            attributes: "\\Junk".into(),
            role: Some(FolderRole::Junk),
        },
        ImapFolder {
            name: "Archive".into(),
            attributes: "\\Archive".into(),
            role: Some(FolderRole::Archive),
        },
    ];

    // Simulate: hosts containing "noidle" don't support IDLE.
    let idle_supported = !host_lower.contains("noidle");
    // Simulate: hosts containing "noutf8" don't support UTF-8.
    let utf8_supported = !host_lower.contains("noutf8");

    InboundTestSuccess {
        folders,
        idle_supported,
        utf8_supported,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::account::{AuthMethod, EncryptionMode};

    fn valid_params() -> InboundTestParams {
        InboundTestParams {
            host: "imap.example.com".into(),
            port: 993,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "secret".into(),
            protocol: Protocol::Imap,
            insecure: false,
            accepted_fingerprint: None,
            client_certificate: None,
        }
    }

    #[test]
    fn mock_imap_returns_folders() {
        let tester = MockInboundTester;
        let result = tester.test_inbound(&valid_params()).unwrap();
        assert!(!result.folders.is_empty());
        assert!(result.idle_supported);
        assert!(result.utf8_supported);
        // Should contain INBOX
        assert!(result.folders.iter().any(|f| f.name == "INBOX"));
        // Should detect Sent role
        assert!(result
            .folders
            .iter()
            .any(|f| f.role == Some(FolderRole::Sent)));
    }

    #[test]
    fn mock_pop3_returns_empty_folders() {
        let tester = MockInboundTester;
        let mut params = valid_params();
        params.protocol = Protocol::Pop3;
        let result = tester.test_inbound(&params).unwrap();
        assert!(result.folders.is_empty());
        assert!(!result.idle_supported);
    }

    #[test]
    fn mock_timeout_host() {
        let tester = MockInboundTester;
        let mut params = valid_params();
        params.host = "timeout.example.com".into();
        let err = tester.test_inbound(&params).unwrap_err();
        assert!(matches!(err, InboundTestError::Timeout));
    }

    #[test]
    fn mock_unreachable_host() {
        let tester = MockInboundTester;
        let mut params = valid_params();
        params.host = "unreachable.example.com".into();
        let err = tester.test_inbound(&params).unwrap_err();
        assert!(matches!(err, InboundTestError::ConnectionFailed(_)));
    }

    #[test]
    fn mock_authfail_host() {
        let tester = MockInboundTester;
        let mut params = valid_params();
        params.host = "authfail.example.com".into();
        let err = tester.test_inbound(&params).unwrap_err();
        assert!(matches!(err, InboundTestError::AuthenticationFailed));
    }

    #[test]
    fn mock_noidle_host() {
        let tester = MockInboundTester;
        let mut params = valid_params();
        params.host = "noidle.example.com".into();
        let result = tester.test_inbound(&params).unwrap();
        assert!(!result.idle_supported);
        assert!(result.utf8_supported);
    }

    #[test]
    fn mock_noutf8_host() {
        let tester = MockInboundTester;
        let mut params = valid_params();
        params.host = "noutf8.example.com".into();
        let result = tester.test_inbound(&params).unwrap();
        assert!(result.idle_supported);
        assert!(!result.utf8_supported);
    }

    #[test]
    fn mock_validates_params() {
        let tester = MockInboundTester;
        let mut params = valid_params();
        params.host = "".into();
        assert!(matches!(
            tester.test_inbound(&params),
            Err(InboundTestError::EmptyHost)
        ));
    }

    #[test]
    fn mock_tlsfail_returns_fingerprint() {
        let tester = MockInboundTester;
        let mut params = valid_params();
        params.host = "tlsfail.example.com".into();
        let err = tester.test_inbound(&params).unwrap_err();
        match err {
            InboundTestError::TlsHandshakeFailed {
                fingerprint: Some(fp),
                ..
            } => {
                assert!(fp.contains(':'), "fingerprint should be colon-separated");
            }
            other => panic!("expected TlsHandshakeFailed with fingerprint, got: {other:?}"),
        }
    }

    #[test]
    fn mock_tlsfail_succeeds_with_accepted_fingerprint() {
        let tester = MockInboundTester;
        let mut params = valid_params();
        params.host = "tlsfail.example.com".into();
        params.accepted_fingerprint = Some(
            "AB:CD:EF:01:23:45:67:89:AB:CD:EF:01:23:45:67:89:AB:CD:EF:01:23:45:67:89:AB:CD:EF:01:23:45:67:89".to_string(),
        );
        let result = tester.test_inbound(&params);
        assert!(
            result.is_ok(),
            "should succeed with matching pinned fingerprint"
        );
    }

    #[test]
    fn mock_tlsfail_fails_with_wrong_fingerprint() {
        let tester = MockInboundTester;
        let mut params = valid_params();
        params.host = "tlsfail.example.com".into();
        params.accepted_fingerprint = Some("00:11:22:33".to_string());
        let err = tester.test_inbound(&params).unwrap_err();
        assert!(matches!(err, InboundTestError::TlsHandshakeFailed { .. }));
    }
}
