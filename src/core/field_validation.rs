//! Client-side validation for the manual server configuration form (FR-17 through FR-22).
//!
//! All logic here is UI-free so it can be unit-tested without a display server.

/// A field-specific validation error for the manual configuration form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManualFieldError {
    /// Hostname is required (FR-17).
    EmptyHostname,
    /// Username is required (FR-18, strict default).
    EmptyUsername,
    /// Password/credential is required (FR-19, strict default).
    EmptyPassword,
}

impl ManualFieldError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::EmptyHostname => "Hostname must not be empty",
            Self::EmptyUsername => "Username must not be empty",
            Self::EmptyPassword => "Password must not be empty",
        }
    }
}

/// A non-blocking warning about password content (FR-20).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PasswordWarning {
    /// Password has leading or trailing whitespace.
    LeadingTrailingWhitespace,
    /// Password contains control characters.
    ControlCharacters,
}

impl PasswordWarning {
    pub fn message(&self) -> &'static str {
        match self {
            Self::LeadingTrailingWhitespace => "Password has leading or trailing whitespace",
            Self::ControlCharacters => "Password contains control characters",
        }
    }
}

/// Result of validating the manual configuration form fields.
#[derive(Debug, Clone)]
pub struct ManualValidationResult {
    pub errors: Vec<ManualFieldError>,
    pub password_warnings: Vec<PasswordWarning>,
}

impl ManualValidationResult {
    /// Returns `true` when there are no blocking errors.
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Validate the manual configuration form fields.
///
/// - `hostname` and `username` are trimmed before checking (FR-21).
/// - `password` is checked for emptiness and warned about whitespace/control chars (FR-19, FR-20).
/// - `port` is validated as numeric with at most 5 digits (FR-22).
/// - When `insecure` is true, username and password are not required (FR-18, FR-19).
pub fn validate_manual_fields(
    hostname: &str,
    username: &str,
    password: &str,
    insecure: bool,
) -> ManualValidationResult {
    let mut errors = Vec::new();

    // FR-17: hostname is required after trimming.
    if hostname.trim().is_empty() {
        errors.push(ManualFieldError::EmptyHostname);
    }

    if !insecure {
        // FR-18: username is required after trimming (strict default).
        if username.trim().is_empty() {
            errors.push(ManualFieldError::EmptyUsername);
        }

        // FR-19: password is required (strict default).
        if password.is_empty() {
            errors.push(ManualFieldError::EmptyPassword);
        }
    }

    // FR-20: password warnings (non-blocking).
    let password_warnings = check_password_warnings(password);

    ManualValidationResult {
        errors,
        password_warnings,
    }
}

/// Trim hostname for use (FR-21). Leading/trailing whitespace is removed.
pub fn trim_hostname(hostname: &str) -> String {
    hostname.trim().to_string()
}

/// Trim username for use (FR-21). Leading/trailing whitespace is removed.
pub fn trim_username(username: &str) -> String {
    username.trim().to_string()
}

/// Validate that a port string is numeric and at most 5 digits (FR-22).
/// Returns `None` if valid, or an error message if invalid.
pub fn validate_port_input(input: &str) -> Option<&'static str> {
    if input.is_empty() {
        return None;
    }
    if !input.chars().all(|c| c.is_ascii_digit()) {
        return Some("Port must contain only digits");
    }
    if input.len() > 5 {
        return Some("Port must be at most 5 digits");
    }
    None
}

/// Produce non-blocking warnings about password content (FR-20).
fn check_password_warnings(password: &str) -> Vec<PasswordWarning> {
    let mut warnings = Vec::new();

    if !password.is_empty() {
        if password.starts_with(char::is_whitespace) || password.ends_with(char::is_whitespace) {
            warnings.push(PasswordWarning::LeadingTrailingWhitespace);
        }
        if password.chars().any(|c| c.is_control()) {
            warnings.push(PasswordWarning::ControlCharacters);
        }
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Hostname validation (FR-17) --

    #[test]
    fn empty_hostname_produces_error() {
        let result = validate_manual_fields("", "user", "pass", false);
        assert!(result.errors.contains(&ManualFieldError::EmptyHostname));
    }

    #[test]
    fn whitespace_only_hostname_produces_error() {
        let result = validate_manual_fields("   ", "user", "pass", false);
        assert!(result.errors.contains(&ManualFieldError::EmptyHostname));
    }

    #[test]
    fn valid_hostname_no_error() {
        let result = validate_manual_fields("imap.example.com", "user", "pass", false);
        assert!(!result.errors.contains(&ManualFieldError::EmptyHostname));
    }

    // -- Username validation (FR-18) --

    #[test]
    fn empty_username_produces_error() {
        let result = validate_manual_fields("host.com", "", "pass", false);
        assert!(result.errors.contains(&ManualFieldError::EmptyUsername));
    }

    #[test]
    fn whitespace_only_username_produces_error() {
        let result = validate_manual_fields("host.com", "  \t  ", "pass", false);
        assert!(result.errors.contains(&ManualFieldError::EmptyUsername));
    }

    #[test]
    fn valid_username_no_error() {
        let result = validate_manual_fields("host.com", "alice", "pass", false);
        assert!(!result.errors.contains(&ManualFieldError::EmptyUsername));
    }

    // -- Password validation (FR-19) --

    #[test]
    fn empty_password_produces_error() {
        let result = validate_manual_fields("host.com", "user", "", false);
        assert!(result.errors.contains(&ManualFieldError::EmptyPassword));
    }

    #[test]
    fn non_empty_password_no_error() {
        let result = validate_manual_fields("host.com", "user", "secret", false);
        assert!(!result.errors.contains(&ManualFieldError::EmptyPassword));
    }

    // -- Password warnings (FR-20) --

    #[test]
    fn password_leading_whitespace_warning() {
        let result = validate_manual_fields("host.com", "user", " secret", false);
        assert!(result
            .password_warnings
            .contains(&PasswordWarning::LeadingTrailingWhitespace));
    }

    #[test]
    fn password_trailing_whitespace_warning() {
        let result = validate_manual_fields("host.com", "user", "secret ", false);
        assert!(result
            .password_warnings
            .contains(&PasswordWarning::LeadingTrailingWhitespace));
    }

    #[test]
    fn password_control_character_warning() {
        let result = validate_manual_fields("host.com", "user", "sec\x01ret", false);
        assert!(result
            .password_warnings
            .contains(&PasswordWarning::ControlCharacters));
    }

    #[test]
    fn clean_password_no_warnings() {
        let result = validate_manual_fields("host.com", "user", "secret", false);
        assert!(result.password_warnings.is_empty());
    }

    #[test]
    fn password_warnings_are_non_blocking() {
        let result = validate_manual_fields("host.com", "user", " secret\t", false);
        assert!(result.is_valid());
        assert!(!result.password_warnings.is_empty());
    }

    // -- Trimming (FR-21) --

    #[test]
    fn trim_hostname_removes_whitespace() {
        assert_eq!(trim_hostname("  imap.example.com  "), "imap.example.com");
    }

    #[test]
    fn trim_username_removes_whitespace() {
        assert_eq!(trim_username("  alice@example.com  "), "alice@example.com");
    }

    #[test]
    fn hostname_with_leading_whitespace_passes_validation_after_trim() {
        // The field has whitespace but the actual value is non-empty
        let result = validate_manual_fields("  imap.example.com  ", "user", "pass", false);
        assert!(result.is_valid());
    }

    // -- Port validation (FR-22) --

    #[test]
    fn valid_port_number() {
        assert_eq!(validate_port_input("993"), None);
    }

    #[test]
    fn port_max_five_digits() {
        assert_eq!(validate_port_input("65535"), None);
    }

    #[test]
    fn port_too_many_digits() {
        assert!(validate_port_input("123456").is_some());
    }

    #[test]
    fn port_non_numeric() {
        assert!(validate_port_input("abc").is_some());
    }

    #[test]
    fn port_mixed_chars() {
        assert!(validate_port_input("99a").is_some());
    }

    #[test]
    fn empty_port_is_valid() {
        assert_eq!(validate_port_input(""), None);
    }

    // -- Combined --

    #[test]
    fn all_empty_produces_all_errors() {
        let result = validate_manual_fields("", "", "", false);
        assert_eq!(result.errors.len(), 3);
    }

    #[test]
    fn all_valid_no_errors() {
        let result = validate_manual_fields("host.com", "user", "pass", false);
        assert!(result.is_valid());
        assert!(result.password_warnings.is_empty());
    }

    // -- Insecure mode relaxation (FR-18, FR-19) --

    #[test]
    fn insecure_allows_empty_username() {
        let result = validate_manual_fields("host.com", "", "pass", true);
        assert!(!result.errors.contains(&ManualFieldError::EmptyUsername));
    }

    #[test]
    fn insecure_allows_empty_password() {
        let result = validate_manual_fields("host.com", "user", "", true);
        assert!(!result.errors.contains(&ManualFieldError::EmptyPassword));
    }

    #[test]
    fn insecure_still_requires_hostname() {
        let result = validate_manual_fields("", "", "", true);
        assert!(result.errors.contains(&ManualFieldError::EmptyHostname));
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn insecure_with_all_fields_valid() {
        let result = validate_manual_fields("host.com", "user", "pass", true);
        assert!(result.is_valid());
    }
}
