use std::fmt;

use uuid::Uuid;

/// The role a credential plays for an account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CredentialRole {
    ImapPassword,
    SmtpPassword,
    OAuthRefreshToken,
}

impl CredentialRole {
    /// String representation used as the `role` attribute in the keychain.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ImapPassword => "imap-password",
            Self::SmtpPassword => "smtp-password",
            Self::OAuthRefreshToken => "oauth-refresh-token",
        }
    }
}

impl fmt::Display for CredentialRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Errors from the credential store.
#[derive(Debug, thiserror::Error)]
pub enum CredentialError {
    /// The system keychain is locked or the D-Bus session bus is unavailable.
    #[error("Cannot access system keychain: {0}")]
    KeychainUnavailable(String),
    /// The requested credential was not found.
    #[error("Credential not found for account {account_id} role {role}")]
    NotFound {
        account_id: Uuid,
        role: CredentialRole,
    },
    /// An unexpected error from the underlying store.
    #[error("Credential store error: {0}")]
    Other(String),
}

/// A secret value that is redacted in Debug and Display output.
/// Credentials must never appear in logs or error messages.
#[derive(Clone)]
pub struct SecretValue(String);

impl SecretValue {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// Access the raw secret value.
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// Consume and return the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Debug for SecretValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretValue(***)")
    }
}

impl fmt::Display for SecretValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***")
    }
}

/// Trait for reading, writing, and deleting credentials keyed by
/// `(account_uuid, role)`. Implementations must never log or expose
/// credential values in error messages.
pub trait CredentialStore {
    /// Read a credential for the given account and role.
    fn read(&self, account_id: Uuid, role: CredentialRole) -> Result<SecretValue, CredentialError>;

    /// Write (create or update) a credential for the given account and role.
    fn write(
        &self,
        account_id: Uuid,
        role: CredentialRole,
        secret: &SecretValue,
    ) -> Result<(), CredentialError>;

    /// Delete a specific credential.
    fn delete(&self, account_id: Uuid, role: CredentialRole) -> Result<(), CredentialError>;

    /// Delete all credentials for the given account (used on account deletion).
    fn delete_all_for_account(&self, account_id: Uuid) -> Result<(), CredentialError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_value_redacts_in_debug() {
        let secret = SecretValue::new("my-password".into());
        let debug = format!("{:?}", secret);
        assert!(!debug.contains("my-password"));
        assert!(debug.contains("***"));
    }

    #[test]
    fn secret_value_redacts_in_display() {
        let secret = SecretValue::new("my-password".into());
        let display = format!("{}", secret);
        assert!(!display.contains("my-password"));
        assert_eq!(display, "***");
    }

    #[test]
    fn secret_value_expose_returns_raw() {
        let secret = SecretValue::new("hunter2".into());
        assert_eq!(secret.expose(), "hunter2");
    }

    #[test]
    fn credential_role_as_str() {
        assert_eq!(CredentialRole::ImapPassword.as_str(), "imap-password");
        assert_eq!(CredentialRole::SmtpPassword.as_str(), "smtp-password");
        assert_eq!(
            CredentialRole::OAuthRefreshToken.as_str(),
            "oauth-refresh-token"
        );
    }

    #[test]
    fn credential_error_does_not_contain_secret() {
        let err = CredentialError::KeychainUnavailable("D-Bus unavailable".into());
        let msg = format!("{}", err);
        assert!(msg.contains("Cannot access system keychain"));
        assert!(msg.contains("D-Bus unavailable"));
    }

    #[test]
    fn secret_value_never_leaks_via_any_format_trait() {
        let password = "P@ssw0rd!Very$ecret123";
        let secret = SecretValue::new(password.into());

        // Debug
        let debug_output = format!("{:?}", secret);
        assert!(
            !debug_output.contains(password),
            "credential leaked in Debug"
        );

        // Display
        let display_output = format!("{}", secret);
        assert!(
            !display_output.contains(password),
            "credential leaked in Display"
        );

        // Embedded in a larger format string
        let embedded = format!("Error with credential: {}", secret);
        assert!(
            !embedded.contains(password),
            "credential leaked in embedded format"
        );

        // As part of a vec/slice debug
        let vec_debug = format!("{:?}", vec![secret.clone()]);
        assert!(
            !vec_debug.contains(password),
            "credential leaked in vec debug"
        );
    }

    #[test]
    fn not_found_error_does_not_expose_credential_value() {
        let id = Uuid::new_v4();
        let err = CredentialError::NotFound {
            account_id: id,
            role: CredentialRole::ImapPassword,
        };
        let msg = format!("{}", err);
        // Error message should contain the account ID and role (for debugging)
        // but never any credential value.
        assert!(msg.contains(&id.to_string()));
        assert!(msg.contains("imap-password"));
    }
}
