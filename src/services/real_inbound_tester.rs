//! Real implementation of `InboundTester` using `async-imap`.

use crate::core::account::Protocol;
use crate::core::inbound_test::{
    InboundTestError, InboundTestParams, InboundTestResult, InboundTestSuccess,
};
use crate::core::sync_state::SyncState;
use crate::services::imap_client::{run_imap_session, ImapConnectParams};
use crate::services::inbound_tester::InboundTester;

/// Real inbound tester that connects to a live IMAP server.
pub struct RealInboundTester;

/// Extended result from a real inbound test including sync state.
pub struct RealInboundTestResult {
    /// The standard inbound test result.
    pub result: InboundTestSuccess,
    /// Capability cache to persist in sync_state table.
    pub sync_state: SyncState,
    /// Connection logs generated during the test.
    pub logs: Vec<crate::core::connection_log::ConnectionLogRecord>,
}

impl InboundTester for RealInboundTester {
    fn test_inbound(&self, params: &InboundTestParams) -> InboundTestResult {
        params.validate()?;

        if params.protocol == Protocol::Pop3 {
            // POP3 not implemented in this story (decision D-10).
            return Ok(InboundTestSuccess {
                folders: vec![],
                idle_supported: false,
                utf8_supported: false,
            });
        }

        let connect_params = ImapConnectParams {
            host: params.host.clone(),
            port: params.port,
            encryption: params.encryption,
            username: params.username.clone(),
            password: params.credential.clone(),
            accepted_fingerprint: params.accepted_fingerprint.clone(),
            insecure: params.insecure,
            account_id: String::new(),
            client_certificate: params.client_certificate.clone(),
            dane: params.dane,
            dnssec: params.dnssec,
        };

        match run_imap_session(&connect_params) {
            Ok(session_result) => {
                let sync =
                    SyncState::from_capabilities(String::new(), &session_result.capabilities);
                Ok(InboundTestSuccess {
                    folders: session_result.folders,
                    idle_supported: sync.idle_supported,
                    utf8_supported: sync.utf8_accept,
                })
            }
            Err(e) => Err(InboundTestError::from(e)),
        }
    }
}

impl RealInboundTester {
    /// Extended test that also returns sync_state and logs for persistence.
    pub fn test_inbound_extended(
        &self,
        params: &InboundTestParams,
        account_id: &str,
    ) -> Result<RealInboundTestResult, InboundTestError> {
        params.validate()?;

        if params.protocol == Protocol::Pop3 {
            return Ok(RealInboundTestResult {
                result: InboundTestSuccess {
                    folders: vec![],
                    idle_supported: false,
                    utf8_supported: false,
                },
                sync_state: SyncState::default(),
                logs: vec![],
            });
        }

        let connect_params = ImapConnectParams {
            host: params.host.clone(),
            port: params.port,
            encryption: params.encryption,
            username: params.username.clone(),
            password: params.credential.clone(),
            accepted_fingerprint: params.accepted_fingerprint.clone(),
            insecure: params.insecure,
            account_id: account_id.to_string(),
            client_certificate: params.client_certificate.clone(),
            dane: params.dane,
            dnssec: params.dnssec,
        };

        match run_imap_session(&connect_params) {
            Ok(session_result) => {
                let sync = SyncState::from_capabilities(
                    account_id.to_string(),
                    &session_result.capabilities,
                );
                Ok(RealInboundTestResult {
                    result: InboundTestSuccess {
                        folders: session_result.folders,
                        idle_supported: sync.idle_supported,
                        utf8_supported: sync.utf8_accept,
                    },
                    sync_state: sync,
                    logs: session_result.logs,
                })
            }
            Err(e) => Err(InboundTestError::from(e)),
        }
    }
}
