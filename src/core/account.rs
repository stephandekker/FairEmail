use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Supported mail protocols.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protocol {
    Imap,
}

/// Connection encryption mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionMode {
    None,
    SslTls,
    StartTls,
}

/// Authentication method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthMethod {
    Plain,
    Login,
    OAuth2,
}

/// Parameters for creating a new account (avoids too-many-arguments).
pub struct NewAccountParams {
    pub display_name: String,
    pub protocol: Protocol,
    pub host: String,
    pub port: u16,
    pub encryption: EncryptionMode,
    pub auth_method: AuthMethod,
    pub username: String,
    pub credential: String,
}

/// A mail account with connection settings and a stable unique identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    id: Uuid,
    display_name: String,
    protocol: Protocol,
    host: String,
    port: u16,
    encryption: EncryptionMode,
    auth_method: AuthMethod,
    username: String,
    /// Password or OAuth token, depending on `auth_method`.
    credential: String,
}

/// Errors that can occur when building an account.
#[derive(Debug, Clone, thiserror::Error)]
pub enum AccountValidationError {
    #[error("display name must not be empty")]
    EmptyDisplayName,
    #[error("host must not be empty")]
    EmptyHost,
    #[error("username must not be empty")]
    EmptyUsername,
    #[error("credential must not be empty")]
    EmptyCredential,
}

impl Account {
    /// Create a new account after validating required fields.
    /// Assigns a new UUID automatically (FR-2: globally-unique, stable identifier).
    pub fn new(params: NewAccountParams) -> Result<Self, AccountValidationError> {
        if params.display_name.trim().is_empty() {
            return Err(AccountValidationError::EmptyDisplayName);
        }
        if params.host.trim().is_empty() {
            return Err(AccountValidationError::EmptyHost);
        }
        if params.username.trim().is_empty() {
            return Err(AccountValidationError::EmptyUsername);
        }
        if params.credential.trim().is_empty() {
            return Err(AccountValidationError::EmptyCredential);
        }

        Ok(Self {
            id: Uuid::new_v4(),
            display_name: params.display_name,
            protocol: params.protocol,
            host: params.host,
            port: params.port,
            encryption: params.encryption,
            auth_method: params.auth_method,
            username: params.username,
            credential: params.credential,
        })
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn protocol(&self) -> Protocol {
        self.protocol
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn encryption(&self) -> EncryptionMode {
        self.encryption
    }

    pub fn auth_method(&self) -> AuthMethod {
        self.auth_method
    }

    pub fn username(&self) -> &str {
        &self.username
    }
}

impl std::fmt::Display for EncryptionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::SslTls => write!(f, "SSL/TLS"),
            Self::StartTls => write!(f, "STARTTLS"),
        }
    }
}

impl std::fmt::Display for AuthMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plain => write!(f, "PLAIN"),
            Self::Login => write!(f, "LOGIN"),
            Self::OAuth2 => write!(f, "OAuth2"),
        }
    }
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Imap => write!(f, "IMAP"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_params() -> NewAccountParams {
        NewAccountParams {
            display_name: "Work Email".into(),
            protocol: Protocol::Imap,
            host: "imap.example.com".into(),
            port: 993,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "secret".into(),
        }
    }

    fn valid_account() -> Account {
        Account::new(valid_params()).unwrap()
    }

    #[test]
    fn new_account_has_unique_id() {
        let a = valid_account();
        let b = valid_account();
        assert_ne!(a.id(), b.id());
    }

    #[test]
    fn new_account_stores_all_fields() {
        let a = valid_account();
        assert_eq!(a.display_name(), "Work Email");
        assert_eq!(a.protocol(), Protocol::Imap);
        assert_eq!(a.host(), "imap.example.com");
        assert_eq!(a.port(), 993);
        assert_eq!(a.encryption(), EncryptionMode::SslTls);
        assert_eq!(a.auth_method(), AuthMethod::Plain);
        assert_eq!(a.username(), "user@example.com");
    }

    #[test]
    fn empty_display_name_rejected() {
        let mut p = valid_params();
        p.display_name = "  ".into();
        assert!(matches!(
            Account::new(p),
            Err(AccountValidationError::EmptyDisplayName)
        ));
    }

    #[test]
    fn empty_host_rejected() {
        let mut p = valid_params();
        p.host = "".into();
        assert!(matches!(
            Account::new(p),
            Err(AccountValidationError::EmptyHost)
        ));
    }

    #[test]
    fn empty_username_rejected() {
        let mut p = valid_params();
        p.username = "".into();
        assert!(matches!(
            Account::new(p),
            Err(AccountValidationError::EmptyUsername)
        ));
    }

    #[test]
    fn empty_credential_rejected() {
        let mut p = valid_params();
        p.credential = " ".into();
        assert!(matches!(
            Account::new(p),
            Err(AccountValidationError::EmptyCredential)
        ));
    }

    #[test]
    fn account_serialization_roundtrip() {
        let a = valid_account();
        let json = serde_json::to_string(&a).unwrap();
        let b: Account = serde_json::from_str(&json).unwrap();
        assert_eq!(a.id(), b.id());
        assert_eq!(a.display_name(), b.display_name());
        assert_eq!(a.host(), b.host());
    }
}
