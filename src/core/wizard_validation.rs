/// Client-side validation for the quick-setup wizard (FR-5, FR-6).
///
/// All logic here is UI-free so it can be unit-tested without a display server.
use thiserror::Error;

/// A field-specific validation error returned when wizard input is invalid.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WizardFieldError {
    #[error("Display name must not be empty")]
    EmptyDisplayName,
    #[error("Email address must not be empty")]
    EmptyEmail,
    #[error("Email address is not valid")]
    InvalidEmail,
    #[error("Password must not be empty")]
    EmptyPassword,
}

/// A non-blocking warning about the password content (FR-6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PasswordWarning {
    LeadingWhitespace,
    TrailingWhitespace,
    NonPrintableCharacters,
}

impl PasswordWarning {
    pub fn message(&self) -> &'static str {
        match self {
            Self::LeadingWhitespace => "Password has leading whitespace",
            Self::TrailingWhitespace => "Password has trailing whitespace",
            Self::NonPrintableCharacters => "Password contains non-printable characters",
        }
    }
}

/// Result of validating wizard fields.
#[derive(Debug, Clone)]
pub struct WizardValidationResult {
    pub errors: Vec<WizardFieldError>,
    pub password_warnings: Vec<PasswordWarning>,
}

impl WizardValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Validate wizard input fields (FR-5a, FR-5b, FR-5c, FR-6).
///
/// Returns all errors and warnings found; callers decide what to display.
pub fn validate_wizard_fields(
    display_name: &str,
    email: &str,
    password: &str,
) -> WizardValidationResult {
    let mut errors = Vec::new();

    // FR-5a: display name is non-empty.
    if display_name.trim().is_empty() {
        errors.push(WizardFieldError::EmptyDisplayName);
    }

    // FR-5b: email is non-empty and matches a standard pattern.
    if email.trim().is_empty() {
        errors.push(WizardFieldError::EmptyEmail);
    } else if !is_valid_email(email.trim()) {
        errors.push(WizardFieldError::InvalidEmail);
    }

    // FR-5c: password is non-empty.
    if password.is_empty() {
        errors.push(WizardFieldError::EmptyPassword);
    }

    // FR-6: password warnings (non-blocking).
    let password_warnings = check_password_warnings(password);

    WizardValidationResult {
        errors,
        password_warnings,
    }
}

/// Check whether a string looks like a standard email address.
/// Intentionally simple: `local@domain.tld` with at least one dot after `@`.
fn is_valid_email(email: &str) -> bool {
    let parts: Vec<&str> = email.splitn(2, '@').collect();
    if parts.len() != 2 {
        return false;
    }
    let local = parts[0];
    let domain = parts[1];

    if local.is_empty() || domain.is_empty() {
        return false;
    }

    // Domain must contain at least one dot and no empty labels.
    let labels: Vec<&str> = domain.split('.').collect();
    if labels.len() < 2 {
        return false;
    }
    labels.iter().all(|l| !l.is_empty())
}

/// Produce non-blocking warnings about password content (FR-6).
fn check_password_warnings(password: &str) -> Vec<PasswordWarning> {
    let mut warnings = Vec::new();

    if !password.is_empty() {
        if password.starts_with(char::is_whitespace) {
            warnings.push(PasswordWarning::LeadingWhitespace);
        }
        if password.ends_with(char::is_whitespace) {
            warnings.push(PasswordWarning::TrailingWhitespace);
        }
        if password.chars().any(|c| c.is_control()) {
            warnings.push(PasswordWarning::NonPrintableCharacters);
        }
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Display name validation --

    #[test]
    fn empty_display_name_produces_error() {
        let result = validate_wizard_fields("", "user@example.com", "secret");
        assert!(result.errors.contains(&WizardFieldError::EmptyDisplayName));
    }

    #[test]
    fn whitespace_only_display_name_produces_error() {
        let result = validate_wizard_fields("   ", "user@example.com", "secret");
        assert!(result.errors.contains(&WizardFieldError::EmptyDisplayName));
    }

    #[test]
    fn valid_display_name_no_error() {
        let result = validate_wizard_fields("Alice", "user@example.com", "secret");
        assert!(!result.errors.contains(&WizardFieldError::EmptyDisplayName));
    }

    // -- Email validation --

    #[test]
    fn empty_email_produces_error() {
        let result = validate_wizard_fields("Alice", "", "secret");
        assert!(result.errors.contains(&WizardFieldError::EmptyEmail));
    }

    #[test]
    fn invalid_email_no_at_sign() {
        let result = validate_wizard_fields("Alice", "userexample.com", "secret");
        assert!(result.errors.contains(&WizardFieldError::InvalidEmail));
    }

    #[test]
    fn invalid_email_no_domain_dot() {
        let result = validate_wizard_fields("Alice", "user@example", "secret");
        assert!(result.errors.contains(&WizardFieldError::InvalidEmail));
    }

    #[test]
    fn valid_email_no_error() {
        let result = validate_wizard_fields("Alice", "user@example.com", "secret");
        assert!(!result.errors.iter().any(|e| matches!(
            e,
            WizardFieldError::EmptyEmail | WizardFieldError::InvalidEmail
        )));
    }

    // -- Password validation --

    #[test]
    fn empty_password_produces_error() {
        let result = validate_wizard_fields("Alice", "user@example.com", "");
        assert!(result.errors.contains(&WizardFieldError::EmptyPassword));
    }

    #[test]
    fn non_empty_password_no_error() {
        let result = validate_wizard_fields("Alice", "user@example.com", "secret");
        assert!(!result.errors.contains(&WizardFieldError::EmptyPassword));
    }

    // -- Password warnings (FR-6) --

    #[test]
    fn leading_whitespace_warning() {
        let result = validate_wizard_fields("Alice", "user@example.com", " secret");
        assert!(result
            .password_warnings
            .contains(&PasswordWarning::LeadingWhitespace));
    }

    #[test]
    fn trailing_whitespace_warning() {
        let result = validate_wizard_fields("Alice", "user@example.com", "secret ");
        assert!(result
            .password_warnings
            .contains(&PasswordWarning::TrailingWhitespace));
    }

    #[test]
    fn non_printable_character_warning() {
        let result = validate_wizard_fields("Alice", "user@example.com", "sec\x01ret");
        assert!(result
            .password_warnings
            .contains(&PasswordWarning::NonPrintableCharacters));
    }

    #[test]
    fn clean_password_no_warnings() {
        let result = validate_wizard_fields("Alice", "user@example.com", "secret");
        assert!(result.password_warnings.is_empty());
    }

    #[test]
    fn warnings_are_non_blocking() {
        // Even with warnings, the result should be valid (no errors).
        let result = validate_wizard_fields("Alice", "user@example.com", " secret\t");
        assert!(result.is_valid());
        assert!(!result.password_warnings.is_empty());
    }

    // -- Combined validation --

    #[test]
    fn all_empty_produces_all_errors() {
        let result = validate_wizard_fields("", "", "");
        assert_eq!(result.errors.len(), 3);
        assert!(result.errors.contains(&WizardFieldError::EmptyDisplayName));
        assert!(result.errors.contains(&WizardFieldError::EmptyEmail));
        assert!(result.errors.contains(&WizardFieldError::EmptyPassword));
    }

    #[test]
    fn all_valid_no_errors() {
        let result = validate_wizard_fields("Alice", "user@example.com", "secret");
        assert!(result.is_valid());
        assert!(result.password_warnings.is_empty());
    }

    // -- Email edge cases --

    #[test]
    fn email_with_subdomain_is_valid() {
        let result = validate_wizard_fields("Alice", "user@mail.example.com", "secret");
        assert!(result.is_valid());
    }

    #[test]
    fn email_with_empty_local_part_is_invalid() {
        let result = validate_wizard_fields("Alice", "@example.com", "secret");
        assert!(result.errors.contains(&WizardFieldError::InvalidEmail));
    }

    #[test]
    fn email_with_trailing_dot_is_invalid() {
        let result = validate_wizard_fields("Alice", "user@example.", "secret");
        assert!(result.errors.contains(&WizardFieldError::InvalidEmail));
    }
}
