//! Provider-specific authentication handling.
//!
//! Accommodates known authentication quirks for specific mail providers,
//! keyed by server hostname. Adaptations activate automatically — no user
//! intervention is required (Design Note N-8).
//!
//! Known quirks include:
//! - Provider-specific EHLO/greeting identifiers
//! - Multi-line authentication sequences for POP3 OAuth flows
//! - Provider-mandated SMTP OAuth strategy (XOAUTH2 vs token-as-password via PLAIN)
//! - Custom authentication response handling

use crate::core::auth_mechanism::AuthProtocol;

/// Strategy a provider uses for SMTP OAuth authentication (OQ-3).
///
/// Some providers support `AUTH XOAUTH2` directly on SMTP, while others
/// expect the OAuth access token to be submitted as a password via `AUTH PLAIN`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmtpOAuthStrategy {
    /// Use `AUTH XOAUTH2` with the standard SASL XOAUTH2 token format.
    /// Used by: Gmail, Outlook/Office 365.
    Xoauth2,
    /// Submit the OAuth access token as the password field in `AUTH PLAIN`.
    /// Used by: Yahoo, AOL.
    TokenAsPlainPassword,
}

/// Provider-specific authentication quirks, keyed by hostname.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderQuirks {
    /// The provider identifier (for diagnostics).
    pub provider_id: &'static str,
    /// Custom EHLO identifier the provider expects or prefers in the greeting.
    /// When `Some`, the client should use this value instead of the default
    /// hostname for EHLO/HELO commands to avoid authentication failures.
    pub ehlo_identifier: Option<&'static str>,
    /// Whether POP3 XOAUTH2 authentication requires a multi-line (continuation)
    /// sequence rather than a single-line AUTH command.
    ///
    /// When `true`, the client must send `AUTH XOAUTH2` first, wait for the
    /// server's continuation prompt (`+`), then send the base64 token.
    pub pop3_oauth_multiline: bool,
    /// SMTP OAuth authentication strategy for this provider.
    pub smtp_oauth_strategy: SmtpOAuthStrategy,
    /// Whether the provider requires the `login_hint` parameter in OAuth
    /// authorization requests (containing the user's email address).
    pub oauth_login_hint: bool,
    /// Whether the provider's IMAP server expects SASL-IR (initial response)
    /// to be sent inline with the AUTHENTICATE command rather than waiting
    /// for a continuation.
    pub imap_sasl_ir: bool,
}

impl Default for ProviderQuirks {
    fn default() -> Self {
        Self {
            provider_id: "generic",
            ehlo_identifier: None,
            pop3_oauth_multiline: false,
            smtp_oauth_strategy: SmtpOAuthStrategy::Xoauth2,
            oauth_login_hint: false,
            imap_sasl_ir: false,
        }
    }
}

impl ProviderQuirks {
    /// Returns `true` if the provider uses XOAUTH2 for the given protocol.
    /// When `false` for SMTP, the caller should use the access token as a
    /// PLAIN password instead.
    pub fn uses_xoauth2_for_protocol(&self, protocol: AuthProtocol) -> bool {
        match protocol {
            AuthProtocol::Imap | AuthProtocol::Pop3 => true,
            AuthProtocol::Smtp => self.smtp_oauth_strategy == SmtpOAuthStrategy::Xoauth2,
        }
    }
}

/// Look up provider-specific authentication quirks by server hostname.
///
/// Returns `None` if no provider-specific handling is configured for the
/// given hostname, in which case standard authentication behaviour applies.
///
/// Hostname matching is case-insensitive.
pub fn lookup_quirks(hostname: &str) -> Option<&'static ProviderQuirks> {
    let lower = hostname.to_lowercase();
    PROVIDER_QUIRKS
        .iter()
        .find(|(patterns, _)| patterns.iter().any(|p| lower == *p))
        .map(|(_, quirks)| *quirks)
}

/// Gmail quirks.
static GMAIL_QUIRKS: ProviderQuirks = ProviderQuirks {
    provider_id: "gmail",
    ehlo_identifier: None,
    pop3_oauth_multiline: true,
    smtp_oauth_strategy: SmtpOAuthStrategy::Xoauth2,
    oauth_login_hint: true,
    imap_sasl_ir: true,
};

/// Outlook / Office 365 quirks.
static OUTLOOK_QUIRKS: ProviderQuirks = ProviderQuirks {
    provider_id: "outlook",
    ehlo_identifier: None,
    pop3_oauth_multiline: false,
    smtp_oauth_strategy: SmtpOAuthStrategy::Xoauth2,
    oauth_login_hint: true,
    imap_sasl_ir: true,
};

/// Yahoo quirks.
static YAHOO_QUIRKS: ProviderQuirks = ProviderQuirks {
    provider_id: "yahoo",
    ehlo_identifier: None,
    pop3_oauth_multiline: false,
    smtp_oauth_strategy: SmtpOAuthStrategy::TokenAsPlainPassword,
    oauth_login_hint: false,
    imap_sasl_ir: false,
};

/// AOL quirks (same infrastructure as Yahoo).
static AOL_QUIRKS: ProviderQuirks = ProviderQuirks {
    provider_id: "aol",
    ehlo_identifier: None,
    pop3_oauth_multiline: false,
    smtp_oauth_strategy: SmtpOAuthStrategy::TokenAsPlainPassword,
    oauth_login_hint: false,
    imap_sasl_ir: false,
};

/// Yandex quirks.
static YANDEX_QUIRKS: ProviderQuirks = ProviderQuirks {
    provider_id: "yandex",
    ehlo_identifier: Some("localhost"),
    pop3_oauth_multiline: false,
    smtp_oauth_strategy: SmtpOAuthStrategy::Xoauth2,
    oauth_login_hint: false,
    imap_sasl_ir: false,
};

/// Mail.ru quirks.
static MAILRU_QUIRKS: ProviderQuirks = ProviderQuirks {
    provider_id: "mailru",
    ehlo_identifier: Some("localhost"),
    pop3_oauth_multiline: false,
    smtp_oauth_strategy: SmtpOAuthStrategy::Xoauth2,
    oauth_login_hint: false,
    imap_sasl_ir: false,
};

/// Hostname-to-quirks mapping table.
///
/// Each entry is a tuple of (hostname patterns, quirks reference).
/// Lookup is O(n) but the table is small and lookups are infrequent
/// (once per connection establishment).
static PROVIDER_QUIRKS: &[(&[&str], &ProviderQuirks)] = &[
    // Gmail
    (
        &[
            "imap.gmail.com",
            "smtp.gmail.com",
            "pop.gmail.com",
            "imap.googlemail.com",
            "smtp.googlemail.com",
        ],
        &GMAIL_QUIRKS,
    ),
    // Outlook / Office 365
    (
        &[
            "outlook.office365.com",
            "smtp.office365.com",
            "imap-mail.outlook.com",
            "smtp-mail.outlook.com",
            "pop-mail.outlook.com",
            "pop3.live.com",
            "imap.outlook.com",
            "smtp.outlook.com",
        ],
        &OUTLOOK_QUIRKS,
    ),
    // Yahoo
    (
        &[
            "imap.mail.yahoo.com",
            "smtp.mail.yahoo.com",
            "pop.mail.yahoo.com",
            "imap.mail.yahoo.co.jp",
            "smtp.mail.yahoo.co.jp",
        ],
        &YAHOO_QUIRKS,
    ),
    // AOL
    (
        &["imap.aol.com", "smtp.aol.com", "pop.aol.com"],
        &AOL_QUIRKS,
    ),
    // Yandex
    (
        &[
            "imap.yandex.com",
            "smtp.yandex.com",
            "pop.yandex.com",
            "imap.yandex.ru",
            "smtp.yandex.ru",
            "pop.yandex.ru",
        ],
        &YANDEX_QUIRKS,
    ),
    // Mail.ru
    (
        &["imap.mail.ru", "smtp.mail.ru", "pop.mail.ru"],
        &MAILRU_QUIRKS,
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    // --- lookup_quirks ---

    #[test]
    fn lookup_gmail_imap() {
        let quirks = lookup_quirks("imap.gmail.com").unwrap();
        assert_eq!(quirks.provider_id, "gmail");
    }

    #[test]
    fn lookup_gmail_smtp() {
        let quirks = lookup_quirks("smtp.gmail.com").unwrap();
        assert_eq!(quirks.provider_id, "gmail");
    }

    #[test]
    fn lookup_gmail_pop() {
        let quirks = lookup_quirks("pop.gmail.com").unwrap();
        assert_eq!(quirks.provider_id, "gmail");
    }

    #[test]
    fn lookup_case_insensitive() {
        let quirks = lookup_quirks("IMAP.GMAIL.COM").unwrap();
        assert_eq!(quirks.provider_id, "gmail");
    }

    #[test]
    fn lookup_outlook() {
        let quirks = lookup_quirks("outlook.office365.com").unwrap();
        assert_eq!(quirks.provider_id, "outlook");
    }

    #[test]
    fn lookup_outlook_smtp() {
        let quirks = lookup_quirks("smtp-mail.outlook.com").unwrap();
        assert_eq!(quirks.provider_id, "outlook");
    }

    #[test]
    fn lookup_yahoo() {
        let quirks = lookup_quirks("imap.mail.yahoo.com").unwrap();
        assert_eq!(quirks.provider_id, "yahoo");
    }

    #[test]
    fn lookup_aol() {
        let quirks = lookup_quirks("imap.aol.com").unwrap();
        assert_eq!(quirks.provider_id, "aol");
    }

    #[test]
    fn lookup_yandex() {
        let quirks = lookup_quirks("imap.yandex.com").unwrap();
        assert_eq!(quirks.provider_id, "yandex");
    }

    #[test]
    fn lookup_mailru() {
        let quirks = lookup_quirks("imap.mail.ru").unwrap();
        assert_eq!(quirks.provider_id, "mailru");
    }

    #[test]
    fn lookup_unknown_returns_none() {
        assert!(lookup_quirks("imap.example.com").is_none());
    }

    #[test]
    fn lookup_empty_hostname_returns_none() {
        assert!(lookup_quirks("").is_none());
    }

    // --- Gmail quirks ---

    #[test]
    fn gmail_pop3_oauth_multiline() {
        let quirks = lookup_quirks("pop.gmail.com").unwrap();
        assert!(quirks.pop3_oauth_multiline);
    }

    #[test]
    fn gmail_smtp_uses_xoauth2() {
        let quirks = lookup_quirks("smtp.gmail.com").unwrap();
        assert_eq!(quirks.smtp_oauth_strategy, SmtpOAuthStrategy::Xoauth2);
    }

    #[test]
    fn gmail_imap_sasl_ir() {
        let quirks = lookup_quirks("imap.gmail.com").unwrap();
        assert!(quirks.imap_sasl_ir);
    }

    #[test]
    fn gmail_oauth_login_hint() {
        let quirks = lookup_quirks("imap.gmail.com").unwrap();
        assert!(quirks.oauth_login_hint);
    }

    #[test]
    fn gmail_no_custom_ehlo() {
        let quirks = lookup_quirks("smtp.gmail.com").unwrap();
        assert!(quirks.ehlo_identifier.is_none());
    }

    // --- Yahoo quirks ---

    #[test]
    fn yahoo_smtp_uses_token_as_plain_password() {
        let quirks = lookup_quirks("smtp.mail.yahoo.com").unwrap();
        assert_eq!(
            quirks.smtp_oauth_strategy,
            SmtpOAuthStrategy::TokenAsPlainPassword
        );
    }

    #[test]
    fn yahoo_no_pop3_multiline() {
        let quirks = lookup_quirks("pop.mail.yahoo.com").unwrap();
        assert!(!quirks.pop3_oauth_multiline);
    }

    // --- AOL quirks ---

    #[test]
    fn aol_smtp_uses_token_as_plain_password() {
        let quirks = lookup_quirks("smtp.aol.com").unwrap();
        assert_eq!(
            quirks.smtp_oauth_strategy,
            SmtpOAuthStrategy::TokenAsPlainPassword
        );
    }

    // --- Outlook quirks ---

    #[test]
    fn outlook_smtp_uses_xoauth2() {
        let quirks = lookup_quirks("smtp.office365.com").unwrap();
        assert_eq!(quirks.smtp_oauth_strategy, SmtpOAuthStrategy::Xoauth2);
    }

    #[test]
    fn outlook_imap_sasl_ir() {
        let quirks = lookup_quirks("outlook.office365.com").unwrap();
        assert!(quirks.imap_sasl_ir);
    }

    #[test]
    fn outlook_oauth_login_hint() {
        let quirks = lookup_quirks("imap-mail.outlook.com").unwrap();
        assert!(quirks.oauth_login_hint);
    }

    // --- Yandex quirks ---

    #[test]
    fn yandex_custom_ehlo_identifier() {
        let quirks = lookup_quirks("smtp.yandex.com").unwrap();
        assert_eq!(quirks.ehlo_identifier, Some("localhost"));
    }

    #[test]
    fn yandex_smtp_uses_xoauth2() {
        let quirks = lookup_quirks("smtp.yandex.com").unwrap();
        assert_eq!(quirks.smtp_oauth_strategy, SmtpOAuthStrategy::Xoauth2);
    }

    // --- Mail.ru quirks ---

    #[test]
    fn mailru_custom_ehlo_identifier() {
        let quirks = lookup_quirks("smtp.mail.ru").unwrap();
        assert_eq!(quirks.ehlo_identifier, Some("localhost"));
    }

    // --- uses_xoauth2_for_protocol ---

    #[test]
    fn xoauth2_always_for_imap() {
        let gmail = lookup_quirks("imap.gmail.com").unwrap();
        assert!(gmail.uses_xoauth2_for_protocol(AuthProtocol::Imap));

        let yahoo = lookup_quirks("imap.mail.yahoo.com").unwrap();
        assert!(yahoo.uses_xoauth2_for_protocol(AuthProtocol::Imap));
    }

    #[test]
    fn xoauth2_always_for_pop3() {
        let gmail = lookup_quirks("pop.gmail.com").unwrap();
        assert!(gmail.uses_xoauth2_for_protocol(AuthProtocol::Pop3));

        let yahoo = lookup_quirks("pop.mail.yahoo.com").unwrap();
        assert!(yahoo.uses_xoauth2_for_protocol(AuthProtocol::Pop3));
    }

    #[test]
    fn xoauth2_for_smtp_depends_on_provider() {
        // Gmail uses XOAUTH2 for SMTP
        let gmail = lookup_quirks("smtp.gmail.com").unwrap();
        assert!(gmail.uses_xoauth2_for_protocol(AuthProtocol::Smtp));

        // Yahoo uses token-as-password for SMTP
        let yahoo = lookup_quirks("smtp.mail.yahoo.com").unwrap();
        assert!(!yahoo.uses_xoauth2_for_protocol(AuthProtocol::Smtp));
    }

    // --- Isolation: quirks do not affect other providers ---

    #[test]
    fn quirks_isolated_per_provider() {
        // Gmail has multiline POP3 OAuth
        let gmail = lookup_quirks("pop.gmail.com").unwrap();
        assert!(gmail.pop3_oauth_multiline);

        // Outlook does NOT have multiline POP3 OAuth
        let outlook = lookup_quirks("pop-mail.outlook.com").unwrap();
        assert!(!outlook.pop3_oauth_multiline);

        // Unknown providers get no quirks at all
        assert!(lookup_quirks("pop.example.com").is_none());
    }

    #[test]
    fn default_quirks_are_neutral() {
        let default = ProviderQuirks::default();
        assert_eq!(default.provider_id, "generic");
        assert!(default.ehlo_identifier.is_none());
        assert!(!default.pop3_oauth_multiline);
        assert_eq!(default.smtp_oauth_strategy, SmtpOAuthStrategy::Xoauth2);
        assert!(!default.oauth_login_hint);
        assert!(!default.imap_sasl_ir);
    }

    // --- All registered hostnames resolve ---

    #[test]
    fn all_registered_hostnames_resolve() {
        for (patterns, expected_quirks) in PROVIDER_QUIRKS.iter() {
            for hostname in patterns.iter() {
                let result = lookup_quirks(hostname);
                assert!(
                    result.is_some(),
                    "Hostname {hostname} did not resolve to quirks"
                );
                assert_eq!(
                    result.unwrap().provider_id,
                    expected_quirks.provider_id,
                    "Hostname {hostname} resolved to wrong provider"
                );
            }
        }
    }

    // --- No duplicate hostnames across providers ---

    #[test]
    fn no_duplicate_hostnames() {
        let mut seen = std::collections::HashSet::new();
        for (patterns, quirks) in PROVIDER_QUIRKS.iter() {
            for hostname in patterns.iter() {
                assert!(
                    seen.insert(*hostname),
                    "Duplicate hostname {hostname} in provider {}",
                    quirks.provider_id
                );
            }
        }
    }
}
