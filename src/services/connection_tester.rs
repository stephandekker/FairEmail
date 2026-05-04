use crate::core::connection_test::{
    ConnectionTestError, ConnectionTestRequest, ConnectionTestResult, ServerTestOutcome,
};

/// Trait for testing mail server connections.
/// Implementations handle the actual network I/O.
pub trait ConnectionTester {
    fn test_connection(
        &self,
        request: &ConnectionTestRequest,
    ) -> Result<ConnectionTestResult, ConnectionTestError>;
}

/// Mock implementation that simulates connection testing.
/// Always succeeds for well-formed requests (validates params only).
/// Returns failure for hosts containing "unreachable", "authfail", or "certfail".
/// Returns `Offline` error for hosts containing "offline" (simulates device-offline).
pub struct MockConnectionTester;

impl ConnectionTester for MockConnectionTester {
    fn test_connection(
        &self,
        request: &ConnectionTestRequest,
    ) -> Result<ConnectionTestResult, ConnectionTestError> {
        request.validate()?;

        // Check for simulated offline state before attempting connections.
        if request.incoming.host.to_lowercase().contains("offline") {
            return Err(ConnectionTestError::Offline);
        }
        if let Some(ref outgoing) = request.outgoing {
            if outgoing.host.to_lowercase().contains("offline") {
                return Err(ConnectionTestError::Offline);
            }
        }

        let incoming = test_server_mock(&request.incoming.host);
        let outgoing = request.outgoing.as_ref().map(|o| test_server_mock(&o.host));

        Ok(ConnectionTestResult { incoming, outgoing })
    }
}

fn test_server_mock(host: &str) -> ServerTestOutcome {
    let host_lower = host.to_lowercase();
    if host_lower.contains("unreachable") {
        ServerTestOutcome::Failure("unreachable host".into())
    } else if host_lower.contains("authfail") {
        ServerTestOutcome::Failure("authentication failed".into())
    } else if host_lower.contains("certfail") {
        ServerTestOutcome::Failure("certificate error".into())
    } else {
        ServerTestOutcome::Success
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::account::{AuthMethod, EncryptionMode, Protocol};
    use crate::core::connection_test::ServerConnectionParams;

    fn valid_incoming() -> ServerConnectionParams {
        ServerConnectionParams {
            host: "imap.example.com".into(),
            port: 993,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "secret".into(),
        }
    }

    #[test]
    fn mock_succeeds_for_valid_request() {
        let tester = MockConnectionTester;
        let req = ConnectionTestRequest {
            incoming: valid_incoming(),
            incoming_protocol: Protocol::Imap,
            outgoing: None,
        };
        let result = tester.test_connection(&req).unwrap();
        assert!(result.is_success());
    }

    #[test]
    fn mock_succeeds_for_pop3() {
        let tester = MockConnectionTester;
        let req = ConnectionTestRequest {
            incoming: valid_incoming(),
            incoming_protocol: Protocol::Pop3,
            outgoing: None,
        };
        let result = tester.test_connection(&req).unwrap();
        assert!(result.is_success());
    }

    #[test]
    fn mock_succeeds_with_smtp() {
        let tester = MockConnectionTester;
        let smtp = ServerConnectionParams {
            host: "smtp.example.com".into(),
            port: 587,
            encryption: EncryptionMode::StartTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "secret".into(),
        };
        let req = ConnectionTestRequest {
            incoming: valid_incoming(),
            incoming_protocol: Protocol::Imap,
            outgoing: Some(smtp),
        };
        let result = tester.test_connection(&req).unwrap();
        assert!(result.is_success());
    }

    #[test]
    fn mock_fails_for_unreachable_host() {
        let tester = MockConnectionTester;
        let mut incoming = valid_incoming();
        incoming.host = "unreachable.example.com".into();
        let req = ConnectionTestRequest {
            incoming,
            incoming_protocol: Protocol::Imap,
            outgoing: None,
        };
        let result = tester.test_connection(&req).unwrap();
        assert!(!result.is_success());
        assert!(result.summary().contains("unreachable host"));
    }

    #[test]
    fn mock_fails_for_auth_failure() {
        let tester = MockConnectionTester;
        let mut incoming = valid_incoming();
        incoming.host = "authfail.example.com".into();
        let req = ConnectionTestRequest {
            incoming,
            incoming_protocol: Protocol::Imap,
            outgoing: None,
        };
        let result = tester.test_connection(&req).unwrap();
        assert!(result.summary().contains("authentication failed"));
    }

    #[test]
    fn mock_fails_for_cert_error() {
        let tester = MockConnectionTester;
        let smtp = ServerConnectionParams {
            host: "certfail.smtp.example.com".into(),
            port: 465,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Login,
            username: "user@example.com".into(),
            credential: "secret".into(),
        };
        let req = ConnectionTestRequest {
            incoming: valid_incoming(),
            incoming_protocol: Protocol::Imap,
            outgoing: Some(smtp),
        };
        let result = tester.test_connection(&req).unwrap();
        assert!(!result.is_success());
        assert!(result.summary().contains("certificate error"));
    }

    #[test]
    fn mock_rejects_empty_host() {
        let tester = MockConnectionTester;
        let mut incoming = valid_incoming();
        incoming.host = "".into();
        let req = ConnectionTestRequest {
            incoming,
            incoming_protocol: Protocol::Imap,
            outgoing: None,
        };
        assert!(matches!(
            tester.test_connection(&req),
            Err(ConnectionTestError::EmptyHost)
        ));
    }

    #[test]
    fn mock_returns_offline_for_incoming_offline_host() {
        let tester = MockConnectionTester;
        let mut incoming = valid_incoming();
        incoming.host = "offline.example.com".into();
        let req = ConnectionTestRequest {
            incoming,
            incoming_protocol: Protocol::Imap,
            outgoing: None,
        };
        assert!(matches!(
            tester.test_connection(&req),
            Err(ConnectionTestError::Offline)
        ));
    }

    #[test]
    fn mock_returns_offline_for_outgoing_offline_host() {
        let tester = MockConnectionTester;
        let smtp = ServerConnectionParams {
            host: "offline.smtp.example.com".into(),
            port: 587,
            encryption: EncryptionMode::StartTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "secret".into(),
        };
        let req = ConnectionTestRequest {
            incoming: valid_incoming(),
            incoming_protocol: Protocol::Pop3,
            outgoing: Some(smtp),
        };
        assert!(matches!(
            tester.test_connection(&req),
            Err(ConnectionTestError::Offline)
        ));
    }

    #[test]
    fn mock_rejects_empty_credential() {
        let tester = MockConnectionTester;
        let mut incoming = valid_incoming();
        incoming.credential = "  ".into();
        let req = ConnectionTestRequest {
            incoming,
            incoming_protocol: Protocol::Imap,
            outgoing: None,
        };
        assert!(matches!(
            tester.test_connection(&req),
            Err(ConnectionTestError::EmptyCredential)
        ));
    }
}
