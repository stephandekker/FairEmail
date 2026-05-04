/// URL for the application's own privacy policy (FR-37).
pub(crate) const APP_PRIVACY_POLICY_URL: &str =
    "https://github.com/nicfab/FairEmail/blob/master/PRIVACY.md";

/// URL for the third-party autoconfig service privacy policy (FR-37).
/// The wizard uses Mozilla's Thunderbird ISPDB for provider auto-detection.
pub(crate) const AUTOCONFIG_PRIVACY_POLICY_URL: &str = "https://www.mozilla.org/privacy/";

/// Security guarantee shown to the user (FR-38).
/// The wizard never transmits the user's password to any third-party service.
pub(crate) fn password_security_notice() -> &'static str {
    "Your password is never sent to any third-party service. It is only used to connect to your own mail server."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_privacy_url_is_not_empty() {
        assert!(!APP_PRIVACY_POLICY_URL.is_empty());
    }

    #[test]
    fn autoconfig_privacy_url_is_not_empty() {
        assert!(!AUTOCONFIG_PRIVACY_POLICY_URL.is_empty());
    }

    #[test]
    fn password_security_notice_is_not_empty() {
        assert!(!password_security_notice().is_empty());
    }

    #[test]
    fn urls_start_with_https() {
        assert!(APP_PRIVACY_POLICY_URL.starts_with("https://"));
        assert!(AUTOCONFIG_PRIVACY_POLICY_URL.starts_with("https://"));
    }
}
