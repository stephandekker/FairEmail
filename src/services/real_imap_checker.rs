//! Real implementation of `ImapChecker` using `async-imap`.

use crate::core::imap_check::{
    build_imap_success, resolve_username_candidates, ImapCheckError, ImapCheckResult,
    ImapConnectionParams,
};
use crate::core::provider::Provider;
use crate::services::imap_checker::ImapChecker;
use crate::services::imap_client::{
    run_imap_session, ImapClientError, ImapConnectParams, ImapSessionResult,
};

/// Real IMAP checker that connects to a live server.
pub struct RealImapChecker;

impl ImapChecker for RealImapChecker {
    fn check_imap(
        &self,
        email: &str,
        password: &str,
        provider: &Provider,
        accepted_fingerprint: Option<&str>,
    ) -> ImapCheckResult {
        let params = ImapConnectionParams::from_provider(provider);
        let candidates = resolve_username_candidates(email, provider);

        let mut last_error = None;

        for candidate in &candidates {
            let connect_params = ImapConnectParams {
                host: params.host.clone(),
                port: params.port,
                encryption: params.encryption,
                username: candidate.value().to_string(),
                password: password.to_string(),
                accepted_fingerprint: accepted_fingerprint.map(|s| s.to_string()),
                insecure: accepted_fingerprint.is_some(),
                account_id: String::new(),
                client_certificate: None,
                dane: false,
                dnssec: false,
            };

            match run_imap_session(&connect_params) {
                Ok(ImapSessionResult { folders, .. }) => {
                    let raw_folders: Vec<(String, String)> = folders
                        .iter()
                        .map(|f| (f.name.clone(), f.attributes.clone()))
                        .collect();
                    return Ok(build_imap_success(
                        candidate.value().to_string(),
                        raw_folders,
                    ));
                }
                Err(ImapClientError::AuthenticationFailed) => {
                    last_error = Some(ImapCheckError::AuthenticationFailed);
                    continue;
                }
                Err(ImapClientError::UntrustedCertificate(info)) => {
                    return Err(ImapCheckError::UntrustedCertificate(Box::new(info)));
                }
                Err(ImapClientError::FolderListFailed(msg)) => {
                    return Err(ImapCheckError::FolderListFailed(msg));
                }
                Err(e) => {
                    return Err(ImapCheckError::ConnectionFailed(format!("{e:?}")));
                }
            }
        }

        Err(last_error.unwrap_or(ImapCheckError::AuthenticationFailed))
    }
}
