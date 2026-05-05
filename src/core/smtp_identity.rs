//! SMTP identity configuration model and validation (FR-45 through FR-49).
//!
//! An SMTP identity represents an outgoing mail configuration: the SMTP server
//! connection details, the sender's email address and display name, and the
//! associated inbound account whose credentials serve as defaults.

use super::account::EncryptionMode;
use super::field_validation::{validate_manual_fields, PasswordWarning};

/// Parameters for creating or updating an SMTP identity.
#[derive(Debug, Clone)]
pub struct SmtpIdentityParams {
    /// The inbound account this identity is associated with.
    pub account_id: String,
    /// Sender email address (appears in From: header).
    pub email_address: String,
    /// Sender display name (appears in From: header).
    pub display_name: String,
    /// SMTP server hostname.
    pub smtp_host: String,
    /// SMTP server port.
    pub smtp_port: u16,
    /// Connection encryption mode.
    pub smtp_encryption: EncryptionMode,
    /// SMTP username (defaults to inbound account's username).
    pub smtp_username: String,
    /// Path to a client certificate for mutual TLS (optional).
    pub client_certificate: Option<String>,
    /// Authentication realm override (optional).
    pub smtp_realm: Option<String>,
    /// Require DANE (TLSA) verification for TLS connections.
    pub dane: bool,
    /// Require DNSSEC validation for DNS lookups.
    pub dnssec: bool,
}

/// Result of validating SMTP identity fields.
#[derive(Debug, Clone)]
pub struct SmtpIdentityValidationResult {
    pub errors: Vec<SmtpIdentityFieldError>,
    pub password_warnings: Vec<PasswordWarning>,
}

impl SmtpIdentityValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Field-specific validation errors for the SMTP identity form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmtpIdentityFieldError {
    EmptyHostname,
    EmptyUsername,
    EmptyPassword,
    EmptyEmailAddress,
}

impl SmtpIdentityFieldError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::EmptyHostname => "SMTP hostname must not be empty",
            Self::EmptyUsername => "SMTP username must not be empty",
            Self::EmptyPassword => "SMTP password must not be empty",
            Self::EmptyEmailAddress => "Email address must not be empty",
        }
    }
}

/// Validate the SMTP identity form fields.
///
/// Reuses the shared manual-field validation for hostname/username/password,
/// and additionally requires a non-empty email address.
pub fn validate_smtp_identity(
    hostname: &str,
    username: &str,
    password: &str,
    email_address: &str,
    has_client_certificate: bool,
) -> SmtpIdentityValidationResult {
    let mut errors = Vec::new();

    // Reuse shared validation (hostname, username, password).
    // When a client certificate is provided, password is not required.
    let shared = validate_manual_fields(hostname, username, password, has_client_certificate);
    for e in &shared.errors {
        match e {
            super::field_validation::ManualFieldError::EmptyHostname => {
                errors.push(SmtpIdentityFieldError::EmptyHostname);
            }
            super::field_validation::ManualFieldError::EmptyUsername => {
                errors.push(SmtpIdentityFieldError::EmptyUsername);
            }
            super::field_validation::ManualFieldError::EmptyPassword => {
                errors.push(SmtpIdentityFieldError::EmptyPassword);
            }
        }
    }

    // Email address is always required.
    if email_address.trim().is_empty() {
        errors.push(SmtpIdentityFieldError::EmptyEmailAddress);
    }

    SmtpIdentityValidationResult {
        errors,
        password_warnings: shared.password_warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_identity_passes() {
        let result = validate_smtp_identity(
            "smtp.example.com",
            "user@example.com",
            "secret",
            "user@example.com",
            false,
        );
        assert!(result.is_valid());
    }

    #[test]
    fn empty_hostname_fails() {
        let result = validate_smtp_identity("", "user", "pass", "user@example.com", false);
        assert!(result
            .errors
            .contains(&SmtpIdentityFieldError::EmptyHostname));
    }

    #[test]
    fn empty_username_fails() {
        let result =
            validate_smtp_identity("smtp.example.com", "", "pass", "user@example.com", false);
        assert!(result
            .errors
            .contains(&SmtpIdentityFieldError::EmptyUsername));
    }

    #[test]
    fn empty_password_fails() {
        let result =
            validate_smtp_identity("smtp.example.com", "user", "", "user@example.com", false);
        assert!(result
            .errors
            .contains(&SmtpIdentityFieldError::EmptyPassword));
    }

    #[test]
    fn empty_email_fails() {
        let result = validate_smtp_identity("smtp.example.com", "user", "pass", "", false);
        assert!(result
            .errors
            .contains(&SmtpIdentityFieldError::EmptyEmailAddress));
    }

    #[test]
    fn client_certificate_relaxes_password_requirement() {
        let result =
            validate_smtp_identity("smtp.example.com", "user", "", "user@example.com", true);
        assert!(!result
            .errors
            .contains(&SmtpIdentityFieldError::EmptyPassword));
        assert!(result.is_valid());
    }

    #[test]
    fn password_warnings_propagated() {
        let result = validate_smtp_identity(
            "smtp.example.com",
            "user",
            " secret ",
            "user@example.com",
            false,
        );
        assert!(result.is_valid());
        assert!(!result.password_warnings.is_empty());
    }

    #[test]
    fn all_empty_produces_all_errors() {
        let result = validate_smtp_identity("", "", "", "", false);
        assert_eq!(result.errors.len(), 4);
    }

    #[test]
    fn whitespace_only_email_fails() {
        let result = validate_smtp_identity("smtp.example.com", "user", "pass", "   ", false);
        assert!(result
            .errors
            .contains(&SmtpIdentityFieldError::EmptyEmailAddress));
    }
}
