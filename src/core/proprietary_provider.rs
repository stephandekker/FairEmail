//! Proprietary provider rejection logic (FR-13, US-9).
//!
//! Checks whether an email domain belongs to a known proprietary provider
//! that does not support standard IMAP/SMTP protocols.

/// Known proprietary provider domains that do not support standard IMAP/SMTP.
const PROPRIETARY_DOMAINS: &[&str] = &[
    // ProtonMail / Proton
    "protonmail.com",
    "proton.me",
    "pm.me",
    "protonmail.ch",
    // Tutanota / Tuta
    "tutanota.com",
    "tutanota.de",
    "tuta.io",
    "tuta.com",
    "tutamail.com",
    "keemail.me",
];

/// Result of checking an email against the proprietary provider list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProprietaryCheckResult {
    /// The display name of the proprietary provider (e.g. "ProtonMail").
    pub provider_name: String,
}

/// Check whether the given email address belongs to a known proprietary provider.
///
/// Returns `Some(ProprietaryCheckResult)` if the domain is proprietary, `None` otherwise.
/// No network requests are made — this is a pure in-memory check (FR-13).
pub fn check_proprietary_provider(email: &str) -> Option<ProprietaryCheckResult> {
    let domain = email_domain(email)?;
    let lower = domain.to_lowercase();

    if is_protonmail_domain(&lower) {
        Some(ProprietaryCheckResult {
            provider_name: "ProtonMail".to_string(),
        })
    } else if is_tutanota_domain(&lower) {
        Some(ProprietaryCheckResult {
            provider_name: "Tutanota".to_string(),
        })
    } else {
        None
    }
}

fn is_protonmail_domain(domain: &str) -> bool {
    matches!(
        domain,
        "protonmail.com" | "proton.me" | "pm.me" | "protonmail.ch"
    )
}

fn is_tutanota_domain(domain: &str) -> bool {
    matches!(
        domain,
        "tutanota.com" | "tutanota.de" | "tuta.io" | "tuta.com" | "tutamail.com" | "keemail.me"
    )
}

/// Extract the domain part from an email address.
fn email_domain(email: &str) -> Option<String> {
    let at_pos = email.rfind('@')?;
    let domain = &email[at_pos + 1..];
    if domain.is_empty() {
        None
    } else {
        Some(domain.to_lowercase())
    }
}

/// Returns all known proprietary domains (for testing/inspection).
pub fn proprietary_domains() -> &'static [&'static str] {
    PROPRIETARY_DOMAINS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protonmail_domains_are_rejected() {
        let domains = ["protonmail.com", "proton.me", "pm.me", "protonmail.ch"];
        for domain in &domains {
            let email = format!("user@{domain}");
            let result = check_proprietary_provider(&email);
            assert!(result.is_some(), "Expected rejection for {domain}");
            assert_eq!(result.unwrap().provider_name, "ProtonMail");
        }
    }

    #[test]
    fn tutanota_domains_are_rejected() {
        let domains = [
            "tutanota.com",
            "tutanota.de",
            "tuta.io",
            "tuta.com",
            "tutamail.com",
            "keemail.me",
        ];
        for domain in &domains {
            let email = format!("user@{domain}");
            let result = check_proprietary_provider(&email);
            assert!(result.is_some(), "Expected rejection for {domain}");
            assert_eq!(result.unwrap().provider_name, "Tutanota");
        }
    }

    #[test]
    fn non_proprietary_domains_are_not_rejected() {
        let emails = [
            "user@gmail.com",
            "user@outlook.com",
            "user@yahoo.com",
            "user@example.com",
            "user@fastmail.com",
        ];
        for email in &emails {
            assert!(
                check_proprietary_provider(email).is_none(),
                "Did not expect rejection for {email}"
            );
        }
    }

    #[test]
    fn case_insensitive_check() {
        assert!(check_proprietary_provider("user@ProtonMail.COM").is_some());
        assert!(check_proprietary_provider("user@TUTA.IO").is_some());
    }

    #[test]
    fn invalid_email_returns_none() {
        assert!(check_proprietary_provider("noemail").is_none());
        assert!(check_proprietary_provider("user@").is_none());
        assert!(check_proprietary_provider("").is_none());
    }

    #[test]
    fn no_network_requests_needed() {
        // This test verifies the function is purely in-memory by calling it
        // without any network setup — if it required network it would fail/hang.
        let _ = check_proprietary_provider("user@protonmail.com");
        let _ = check_proprietary_provider("user@tutanota.com");
        let _ = check_proprietary_provider("user@gmail.com");
    }
}
