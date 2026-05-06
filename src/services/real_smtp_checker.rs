//! Real implementation of `SmtpChecker` using the SMTP client.

use crate::core::account::AuthMethod;
use crate::core::imap_check::resolve_username_candidates;
use crate::core::provider::Provider;
use crate::core::smtp_check::{
    SmtpCheckError, SmtpCheckResult, SmtpCheckSuccess, SmtpConnectionParams,
};
use crate::services::smtp_checker::SmtpChecker;
use crate::services::smtp_client::{run_smtp_session, SmtpClientError, SmtpConnectParams};

/// Real SMTP checker that connects to a live server.
pub struct RealSmtpChecker;

impl SmtpChecker for RealSmtpChecker {
    fn check_smtp(
        &self,
        email: &str,
        password: &str,
        provider: &Provider,
        accepted_fingerprint: Option<&str>,
    ) -> SmtpCheckResult {
        let params = SmtpConnectionParams::from_provider(provider);
        let candidates = resolve_username_candidates(email, provider);

        let mut last_error = None;

        for candidate in &candidates {
            let connect_params = SmtpConnectParams {
                host: params.host.clone(),
                port: params.port,
                encryption: params.encryption,
                username: candidate.value().to_string(),
                password: password.to_string(),
                accepted_fingerprint: accepted_fingerprint.map(|s| s.to_string()),
                insecure: accepted_fingerprint.is_some(),
                account_id: String::new(),
                ehlo_hostname: None,
                auth_method: AuthMethod::Plain,
                client_certificate: None,
            };

            match run_smtp_session(&connect_params) {
                Ok(result) => {
                    return Ok(SmtpCheckSuccess {
                        authenticated_username: candidate.value().to_string(),
                        max_message_size: result.max_message_size,
                    });
                }
                Err(SmtpClientError::AuthenticationFailed) => {
                    last_error = Some(SmtpCheckError::AuthenticationFailed);
                    continue;
                }
                Err(SmtpClientError::UntrustedCertificate(info)) => {
                    return Err(SmtpCheckError::UntrustedCertificate(Box::new(info)));
                }
                Err(e) => {
                    return Err(SmtpCheckError::ConnectionFailed(format!("{e:?}")));
                }
            }
        }

        Err(last_error.unwrap_or(SmtpCheckError::AuthenticationFailed))
    }
}
