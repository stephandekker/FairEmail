//! Real implementation of `ConnectionTester` using `async-imap`.

use crate::core::account::Protocol;
use crate::core::connection_test::{
    ConnectionTestError, ConnectionTestRequest, ConnectionTestResult, ServerTestOutcome,
};
use crate::services::connection_tester::ConnectionTester;
use crate::services::imap_client::{run_imap_session, ImapClientError, ImapConnectParams};

/// Real connection tester that connects to live servers.
pub struct RealConnectionTester;

impl ConnectionTester for RealConnectionTester {
    fn test_connection(
        &self,
        request: &ConnectionTestRequest,
    ) -> Result<ConnectionTestResult, ConnectionTestError> {
        request.validate()?;

        let incoming = test_incoming_server(request);

        // Outgoing SMTP testing is not part of this story; return Success as placeholder.
        let outgoing = request
            .outgoing
            .as_ref()
            .map(|_| ServerTestOutcome::Success);

        Ok(ConnectionTestResult { incoming, outgoing })
    }
}

fn test_incoming_server(request: &ConnectionTestRequest) -> ServerTestOutcome {
    if request.incoming_protocol == Protocol::Pop3 {
        // POP3 not implemented in this story (decision D-10).
        return ServerTestOutcome::Success;
    }

    let connect_params = ImapConnectParams {
        host: request.incoming.host.clone(),
        port: request.incoming.port,
        encryption: request.incoming.encryption,
        username: request.incoming.username.clone(),
        password: request.incoming.credential.clone(),
        accepted_fingerprint: None,
        insecure: false,
        account_id: String::new(),
        client_certificate: None,
        dane: false,
        dnssec: false,
        auth_realm: None,
        auth_method: request.incoming.auth_method,
    };

    match run_imap_session(&connect_params) {
        Ok(_) => ServerTestOutcome::Success,
        Err(e) => ServerTestOutcome::Failure(format_client_error(e)),
    }
}

fn format_client_error(e: ImapClientError) -> String {
    match e {
        ImapClientError::DnsResolution(host) => {
            format!("DNS resolution failed for {host}")
        }
        ImapClientError::ConnectionRefused { host, port } => {
            format!("Connection refused by {host}:{port}")
        }
        ImapClientError::Timeout => "Connection timed out".to_string(),
        ImapClientError::TlsHandshake(msg) => {
            format!("TLS handshake failed: {msg}")
        }
        ImapClientError::UntrustedCertificate(info) => {
            format!("Untrusted certificate (fingerprint: {})", info.fingerprint)
        }
        ImapClientError::AuthenticationFailed => "Authentication failed".to_string(),
        ImapClientError::ProtocolMismatch(msg) => {
            format!("Protocol mismatch: {msg}")
        }
        ImapClientError::FolderListFailed(msg) => {
            format!("Folder listing failed: {msg}")
        }
        ImapClientError::ConnectionFailed(msg) => msg,
        ImapClientError::DnssecFailed(msg) => {
            format!("DNSSEC validation failed: {msg}")
        }
        ImapClientError::DaneFailed(msg) => {
            format!("DANE verification failed: {msg}")
        }
    }
}
