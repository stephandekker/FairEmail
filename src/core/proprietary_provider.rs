//! Proprietary provider rejection logic (FR-36, FR-37, FR-38, AC-12).
//!
//! Checks whether an email domain belongs to a known proprietary provider
//! that does not support standard IMAP/SMTP protocols.
//!
//! The proprietary provider list is data-driven: adding or removing a provider
//! requires only a data change to [`PROPRIETARY_PROVIDERS`], not a code change.

/// A known proprietary email provider that does not support standard protocols.
struct ProprietaryProvider {
    /// Display name shown to the user (e.g. "ProtonMail").
    name: &'static str,
    /// Domains associated with this provider.
    domains: &'static [&'static str],
}

/// Known proprietary providers that do not support standard IMAP/SMTP.
///
/// To add a new provider, append an entry here — no other code changes are needed.
const PROPRIETARY_PROVIDERS: &[ProprietaryProvider] = &[
    ProprietaryProvider {
        name: "ProtonMail",
        domains: &["protonmail.com", "proton.me", "pm.me", "protonmail.ch"],
    },
    ProprietaryProvider {
        name: "Tutanota",
        domains: &[
            "tutanota.com",
            "tutanota.de",
            "tuta.io",
            "tuta.com",
            "tutamail.com",
            "keemail.me",
        ],
    },
    ProprietaryProvider {
        name: "Skiff",
        domains: &["skiff.com"],
    },
    ProprietaryProvider {
        name: "Ctemplar",
        domains: &["ctemplar.com"],
    },
    ProprietaryProvider {
        name: "Criptext",
        domains: &["criptext.com"],
    },
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
/// No network requests are made — this is a pure in-memory check (FR-38).
pub fn check_proprietary_provider(email: &str) -> Option<ProprietaryCheckResult> {
    let domain = email_domain(email)?;

    for provider in PROPRIETARY_PROVIDERS {
        if provider.domains.iter().any(|d| *d == domain) {
            return Some(ProprietaryCheckResult {
                provider_name: provider.name.to_string(),
            });
        }
    }

    None
}

/// Build a user-facing rejection message explaining why a proprietary provider
/// is not supported (FR-37). The message is non-technical and non-blaming.
pub fn rejection_message(provider_name: &str) -> String {
    gettextrs::gettext(
        "%s uses its own private system for sending and receiving email. \
         It does not support the standard protocols that this application needs to connect. \
         Please use a different email address, or access %s through its own app or website.",
    )
    .replace("%s", provider_name)
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
pub fn proprietary_domains() -> Vec<&'static str> {
    PROPRIETARY_PROVIDERS
        .iter()
        .flat_map(|p| p.domains.iter().copied())
        .collect()
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
    fn skiff_domains_are_rejected() {
        let result = check_proprietary_provider("user@skiff.com");
        assert!(result.is_some());
        assert_eq!(result.unwrap().provider_name, "Skiff");
    }

    #[test]
    fn ctemplar_domains_are_rejected() {
        let result = check_proprietary_provider("user@ctemplar.com");
        assert!(result.is_some());
        assert_eq!(result.unwrap().provider_name, "Ctemplar");
    }

    #[test]
    fn criptext_domains_are_rejected() {
        let result = check_proprietary_provider("user@criptext.com");
        assert!(result.is_some());
        assert_eq!(result.unwrap().provider_name, "Criptext");
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

    #[test]
    fn rejection_message_includes_provider_name() {
        let msg = rejection_message("ProtonMail");
        assert!(msg.contains("ProtonMail"));
        assert!(!msg.contains("%s"));
    }

    #[test]
    fn proprietary_domains_returns_all() {
        let all = proprietary_domains();
        assert!(all.contains(&"protonmail.com"));
        assert!(all.contains(&"tutanota.com"));
        assert!(all.contains(&"skiff.com"));
        assert!(all.contains(&"ctemplar.com"));
        assert!(all.contains(&"criptext.com"));
    }

    #[test]
    fn data_driven_addition_works() {
        // Verify that all providers in the data list are actually checked.
        for provider in PROPRIETARY_PROVIDERS {
            for domain in provider.domains {
                let email = format!("test@{domain}");
                let result = check_proprietary_provider(&email);
                assert!(
                    result.is_some(),
                    "Provider {} domain {} not detected",
                    provider.name,
                    domain
                );
                assert_eq!(result.unwrap().provider_name, provider.name);
            }
        }
    }
}
