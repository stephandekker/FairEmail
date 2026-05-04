use crate::core::account::{AuthMethod, EncryptionMode, Protocol};

/// Describes a server to test connectivity against.
#[derive(Debug, Clone)]
pub struct ServerConnectionParams {
    pub host: String,
    pub port: u16,
    pub encryption: EncryptionMode,
    pub auth_method: AuthMethod,
    pub username: String,
    pub credential: String,
}

/// A request to test connection to one or more servers.
#[derive(Debug, Clone)]
pub struct ConnectionTestRequest {
    /// Incoming mail server (IMAP or POP3).
    pub incoming: ServerConnectionParams,
    /// Protocol for the incoming server.
    pub incoming_protocol: Protocol,
    /// Optional outgoing SMTP server to test.
    pub outgoing: Option<ServerConnectionParams>,
}

/// Outcome of testing a single server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerTestOutcome {
    Success,
    Failure(String),
}

/// Combined result of testing incoming (and optionally outgoing) servers.
#[derive(Debug, Clone)]
pub struct ConnectionTestResult {
    pub incoming: ServerTestOutcome,
    pub outgoing: Option<ServerTestOutcome>,
}

impl ConnectionTestResult {
    /// Returns true if all tested servers connected successfully.
    pub fn is_success(&self) -> bool {
        self.incoming == ServerTestOutcome::Success
            && self
                .outgoing
                .as_ref()
                .is_none_or(|o| *o == ServerTestOutcome::Success)
    }

    /// Returns a user-friendly summary message.
    pub fn summary(&self) -> String {
        if self.is_success() {
            return "Connection successful".to_string();
        }

        let mut messages = Vec::new();
        if let ServerTestOutcome::Failure(ref msg) = self.incoming {
            messages.push(format!("Incoming: {msg}"));
        }
        if let Some(ServerTestOutcome::Failure(ref msg)) = self.outgoing {
            messages.push(format!("Outgoing: {msg}"));
        }
        messages.join("; ")
    }
}

/// Errors that prevent the connection test from even being attempted.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ConnectionTestError {
    #[error("host must not be empty")]
    EmptyHost,
    #[error("username must not be empty")]
    EmptyUsername,
    #[error("credential must not be empty")]
    EmptyCredential,
    #[error("device is offline")]
    Offline,
}

impl ConnectionTestRequest {
    /// Validate that the request has enough information to attempt a connection.
    pub fn validate(&self) -> Result<(), ConnectionTestError> {
        Self::validate_server(&self.incoming)?;
        if let Some(ref outgoing) = self.outgoing {
            Self::validate_server(outgoing)?;
        }
        Ok(())
    }

    fn validate_server(params: &ServerConnectionParams) -> Result<(), ConnectionTestError> {
        if params.host.trim().is_empty() {
            return Err(ConnectionTestError::EmptyHost);
        }
        if params.username.trim().is_empty() {
            return Err(ConnectionTestError::EmptyUsername);
        }
        if params.credential.trim().is_empty() {
            return Err(ConnectionTestError::EmptyCredential);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_params() -> ServerConnectionParams {
        ServerConnectionParams {
            host: "mail.example.com".into(),
            port: 993,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "password".into(),
        }
    }

    #[test]
    fn validate_passes_for_valid_incoming_only() {
        let req = ConnectionTestRequest {
            incoming: sample_params(),
            incoming_protocol: Protocol::Imap,
            outgoing: None,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn validate_passes_for_valid_incoming_and_outgoing() {
        let mut smtp = sample_params();
        smtp.host = "smtp.example.com".into();
        smtp.port = 587;
        let req = ConnectionTestRequest {
            incoming: sample_params(),
            incoming_protocol: Protocol::Pop3,
            outgoing: Some(smtp),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn validate_fails_empty_incoming_host() {
        let mut params = sample_params();
        params.host = "  ".into();
        let req = ConnectionTestRequest {
            incoming: params,
            incoming_protocol: Protocol::Imap,
            outgoing: None,
        };
        assert!(matches!(
            req.validate(),
            Err(ConnectionTestError::EmptyHost)
        ));
    }

    #[test]
    fn validate_fails_empty_outgoing_username() {
        let mut smtp = sample_params();
        smtp.username = "".into();
        let req = ConnectionTestRequest {
            incoming: sample_params(),
            incoming_protocol: Protocol::Imap,
            outgoing: Some(smtp),
        };
        assert!(matches!(
            req.validate(),
            Err(ConnectionTestError::EmptyUsername)
        ));
    }

    #[test]
    fn result_is_success_when_all_pass() {
        let result = ConnectionTestResult {
            incoming: ServerTestOutcome::Success,
            outgoing: Some(ServerTestOutcome::Success),
        };
        assert!(result.is_success());
        assert_eq!(result.summary(), "Connection successful");
    }

    #[test]
    fn result_is_failure_when_incoming_fails() {
        let result = ConnectionTestResult {
            incoming: ServerTestOutcome::Failure("authentication failed".into()),
            outgoing: None,
        };
        assert!(!result.is_success());
        assert!(result.summary().contains("authentication failed"));
    }

    #[test]
    fn result_is_failure_when_outgoing_fails() {
        let result = ConnectionTestResult {
            incoming: ServerTestOutcome::Success,
            outgoing: Some(ServerTestOutcome::Failure("unreachable host".into())),
        };
        assert!(!result.is_success());
        assert!(result.summary().contains("unreachable host"));
    }

    #[test]
    fn result_shows_both_failures() {
        let result = ConnectionTestResult {
            incoming: ServerTestOutcome::Failure("timeout".into()),
            outgoing: Some(ServerTestOutcome::Failure("certificate error".into())),
        };
        let summary = result.summary();
        assert!(summary.contains("Incoming: timeout"));
        assert!(summary.contains("Outgoing: certificate error"));
    }
}
