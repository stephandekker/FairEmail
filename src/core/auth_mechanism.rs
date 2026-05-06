//! Password-based authentication mechanism negotiation.
//!
//! Determines the strongest SASL mechanism that both the application and the
//! mail server support, based on protocol-specific capability advertisements
//! (IMAP CAPABILITY, SMTP EHLO, POP3 CAPA).

use std::fmt;

/// Mail protocol used to determine which mechanisms are applicable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthProtocol {
    Imap,
    Pop3,
    Smtp,
}

impl fmt::Display for AuthProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthProtocol::Imap => write!(f, "IMAP"),
            AuthProtocol::Pop3 => write!(f, "POP3"),
            AuthProtocol::Smtp => write!(f, "SMTP"),
        }
    }
}

/// Authentication mechanisms supported by the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMechanism {
    CramMd5,
    Login,
    Plain,
    Ntlm,
    Xoauth2,
    Apop,
    External,
}

impl fmt::Display for AuthMechanism {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.capability_name())
    }
}

impl AuthMechanism {
    /// The canonical name as it appears in server capability advertisements.
    pub fn capability_name(&self) -> &'static str {
        match self {
            AuthMechanism::CramMd5 => "CRAM-MD5",
            AuthMechanism::Login => "LOGIN",
            AuthMechanism::Plain => "PLAIN",
            AuthMechanism::Ntlm => "NTLM",
            AuthMechanism::Xoauth2 => "XOAUTH2",
            AuthMechanism::Apop => "APOP",
            AuthMechanism::External => "EXTERNAL",
        }
    }

    /// Parse a mechanism name (case-insensitive) into an `AuthMechanism`.
    pub fn from_name(name: &str) -> Option<AuthMechanism> {
        match name.to_uppercase().as_str() {
            "CRAM-MD5" => Some(AuthMechanism::CramMd5),
            "LOGIN" => Some(AuthMechanism::Login),
            "PLAIN" => Some(AuthMechanism::Plain),
            "NTLM" => Some(AuthMechanism::Ntlm),
            "XOAUTH2" => Some(AuthMechanism::Xoauth2),
            "APOP" => Some(AuthMechanism::Apop),
            "EXTERNAL" => Some(AuthMechanism::External),
            _ => None,
        }
    }
}

/// Returns the full set of mechanisms the application supports for a protocol.
pub fn supported_mechanisms(protocol: AuthProtocol) -> &'static [AuthMechanism] {
    match protocol {
        AuthProtocol::Imap => &[
            AuthMechanism::Plain,
            AuthMechanism::Login,
            AuthMechanism::CramMd5,
            AuthMechanism::Ntlm,
            AuthMechanism::Xoauth2,
            AuthMechanism::External,
        ],
        AuthProtocol::Pop3 => &[
            AuthMechanism::Plain,
            AuthMechanism::Login,
            AuthMechanism::CramMd5,
            AuthMechanism::Ntlm,
            AuthMechanism::Xoauth2,
            AuthMechanism::Apop,
            AuthMechanism::External,
        ],
        AuthProtocol::Smtp => &[
            AuthMechanism::Plain,
            AuthMechanism::Login,
            AuthMechanism::CramMd5,
            AuthMechanism::Ntlm,
            AuthMechanism::External,
        ],
    }
}

/// Password-mechanism preference order (highest priority first).
///
/// CRAM-MD5 > LOGIN > PLAIN > NTLM (Design Note N-2: CRAM-MD5 preferred
/// because it never transmits the password).
const PASSWORD_PREFERENCE: &[AuthMechanism] = &[
    AuthMechanism::CramMd5,
    AuthMechanism::Login,
    AuthMechanism::Plain,
    AuthMechanism::Ntlm,
];

/// Negotiate the strongest password-based authentication mechanism.
///
/// Intersects `server_mechanisms` (parsed from capability advertisements) with
/// the application's supported set for `protocol`, then returns the
/// highest-priority mechanism according to the preference order
/// CRAM-MD5 > LOGIN > PLAIN > NTLM.
///
/// Returns `None` if no common password mechanism is available.
pub fn negotiate_password_mechanism(
    protocol: AuthProtocol,
    server_mechanisms: &[AuthMechanism],
) -> Option<AuthMechanism> {
    let supported = supported_mechanisms(protocol);

    PASSWORD_PREFERENCE
        .iter()
        .find(|&&preferred| {
            supported.contains(&preferred) && server_mechanisms.contains(&preferred)
        })
        .copied()
}

/// Extract advertised AUTH mechanisms from an IMAP CAPABILITY response.
///
/// IMAP advertises SASL mechanisms as `AUTH=PLAIN`, `AUTH=CRAM-MD5`, etc.
/// in the CAPABILITY response line.
pub fn parse_imap_capabilities(capabilities: &[String]) -> Vec<AuthMechanism> {
    capabilities
        .iter()
        .filter_map(|cap| {
            cap.to_uppercase()
                .strip_prefix("AUTH=")
                .and_then(AuthMechanism::from_name)
        })
        .collect()
}

/// Extract advertised AUTH mechanisms from an SMTP EHLO response.
///
/// SMTP advertises mechanisms on the `250-AUTH` or `250 AUTH` line, e.g.:
/// `250-AUTH LOGIN PLAIN CRAM-MD5`
pub fn parse_smtp_ehlo(ehlo_response: &str) -> Vec<AuthMechanism> {
    let mut mechanisms = Vec::new();
    for line in ehlo_response.lines() {
        let upper = line.to_uppercase();
        // Match "250-AUTH ..." or "250 AUTH ..."
        let auth_part = upper
            .strip_prefix("250-AUTH ")
            .or_else(|| upper.strip_prefix("250 AUTH "));
        if let Some(mechs_str) = auth_part {
            for token in mechs_str.split_whitespace() {
                if let Some(mech) = AuthMechanism::from_name(token) {
                    mechanisms.push(mech);
                }
            }
        }
    }
    mechanisms
}

/// Extract advertised AUTH mechanisms from a POP3 CAPA response.
///
/// POP3 lists SASL mechanisms on lines starting with `SASL`, e.g.:
/// ```text
/// SASL PLAIN LOGIN CRAM-MD5
/// ```
pub fn parse_pop3_capa(capa_response: &str) -> Vec<AuthMechanism> {
    let mut mechanisms = Vec::new();
    for line in capa_response.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed
            .to_uppercase()
            .strip_prefix("SASL ")
            .map(|s| s.to_string())
        {
            for token in rest.split_whitespace() {
                if let Some(mech) = AuthMechanism::from_name(token) {
                    mechanisms.push(mech);
                }
            }
        }
    }
    mechanisms
}

/// Format a diagnostic log message for mechanism negotiation.
pub fn log_negotiation(protocol: AuthProtocol, result: Option<AuthMechanism>) -> String {
    match result {
        Some(mech) => format!(
            "{} auth negotiation: selected {}",
            protocol,
            mech.capability_name()
        ),
        None => format!(
            "{} auth negotiation: no common password mechanism found",
            protocol
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- supported_mechanisms ---

    #[test]
    fn imap_supported_mechanisms() {
        let mechs = supported_mechanisms(AuthProtocol::Imap);
        assert!(mechs.contains(&AuthMechanism::Plain));
        assert!(mechs.contains(&AuthMechanism::Login));
        assert!(mechs.contains(&AuthMechanism::CramMd5));
        assert!(mechs.contains(&AuthMechanism::Ntlm));
        assert!(mechs.contains(&AuthMechanism::Xoauth2));
        assert!(mechs.contains(&AuthMechanism::External));
        assert!(!mechs.contains(&AuthMechanism::Apop));
    }

    #[test]
    fn pop3_supported_mechanisms() {
        let mechs = supported_mechanisms(AuthProtocol::Pop3);
        assert!(mechs.contains(&AuthMechanism::Plain));
        assert!(mechs.contains(&AuthMechanism::Login));
        assert!(mechs.contains(&AuthMechanism::CramMd5));
        assert!(mechs.contains(&AuthMechanism::Ntlm));
        assert!(mechs.contains(&AuthMechanism::Xoauth2));
        assert!(mechs.contains(&AuthMechanism::Apop));
        assert!(mechs.contains(&AuthMechanism::External));
    }

    #[test]
    fn smtp_supported_mechanisms() {
        let mechs = supported_mechanisms(AuthProtocol::Smtp);
        assert!(mechs.contains(&AuthMechanism::Plain));
        assert!(mechs.contains(&AuthMechanism::Login));
        assert!(mechs.contains(&AuthMechanism::CramMd5));
        assert!(mechs.contains(&AuthMechanism::Ntlm));
        assert!(mechs.contains(&AuthMechanism::External));
        assert!(!mechs.contains(&AuthMechanism::Xoauth2));
        assert!(!mechs.contains(&AuthMechanism::Apop));
    }

    // --- negotiate_password_mechanism ---

    #[test]
    fn negotiate_prefers_cram_md5_over_login_and_plain() {
        let server = vec![
            AuthMechanism::Plain,
            AuthMechanism::Login,
            AuthMechanism::CramMd5,
        ];
        let result = negotiate_password_mechanism(AuthProtocol::Imap, &server);
        assert_eq!(result, Some(AuthMechanism::CramMd5));
    }

    #[test]
    fn negotiate_falls_back_to_login_when_no_cram() {
        let server = vec![AuthMechanism::Plain, AuthMechanism::Login];
        let result = negotiate_password_mechanism(AuthProtocol::Imap, &server);
        assert_eq!(result, Some(AuthMechanism::Login));
    }

    #[test]
    fn negotiate_falls_back_to_plain() {
        let server = vec![AuthMechanism::Plain];
        let result = negotiate_password_mechanism(AuthProtocol::Smtp, &server);
        assert_eq!(result, Some(AuthMechanism::Plain));
    }

    #[test]
    fn negotiate_ntlm_last_resort() {
        let server = vec![AuthMechanism::Ntlm];
        let result = negotiate_password_mechanism(AuthProtocol::Imap, &server);
        assert_eq!(result, Some(AuthMechanism::Ntlm));
    }

    #[test]
    fn negotiate_none_when_no_overlap() {
        let server = vec![AuthMechanism::Xoauth2, AuthMechanism::External];
        let result = negotiate_password_mechanism(AuthProtocol::Imap, &server);
        assert_eq!(result, None);
    }

    #[test]
    fn negotiate_respects_protocol_support() {
        // APOP is only supported for POP3, not IMAP
        let server = vec![AuthMechanism::Apop];
        assert_eq!(
            negotiate_password_mechanism(AuthProtocol::Imap, &server),
            None
        );
        // APOP is not a password-preference mechanism anyway
        assert_eq!(
            negotiate_password_mechanism(AuthProtocol::Pop3, &server),
            None
        );
    }

    #[test]
    fn negotiate_empty_server_list() {
        let result = negotiate_password_mechanism(AuthProtocol::Imap, &[]);
        assert_eq!(result, None);
    }

    #[test]
    fn negotiate_preference_order_cram_login_plain_ntlm() {
        // All four password mechanisms available; should pick CRAM-MD5
        let server = vec![
            AuthMechanism::Ntlm,
            AuthMechanism::Plain,
            AuthMechanism::Login,
            AuthMechanism::CramMd5,
        ];
        assert_eq!(
            negotiate_password_mechanism(AuthProtocol::Imap, &server),
            Some(AuthMechanism::CramMd5)
        );
    }

    // --- parse_imap_capabilities ---

    #[test]
    fn parse_imap_auth_capabilities() {
        let caps = vec![
            "IMAP4rev1".to_string(),
            "AUTH=PLAIN".to_string(),
            "AUTH=LOGIN".to_string(),
            "AUTH=CRAM-MD5".to_string(),
            "IDLE".to_string(),
        ];
        let mechs = parse_imap_capabilities(&caps);
        assert_eq!(mechs.len(), 3);
        assert!(mechs.contains(&AuthMechanism::Plain));
        assert!(mechs.contains(&AuthMechanism::Login));
        assert!(mechs.contains(&AuthMechanism::CramMd5));
    }

    #[test]
    fn parse_imap_no_auth_capabilities() {
        let caps = vec!["IMAP4rev1".to_string(), "IDLE".to_string()];
        let mechs = parse_imap_capabilities(&caps);
        assert!(mechs.is_empty());
    }

    #[test]
    fn parse_imap_case_insensitive() {
        let caps = vec!["auth=plain".to_string(), "Auth=Cram-MD5".to_string()];
        let mechs = parse_imap_capabilities(&caps);
        assert_eq!(mechs.len(), 2);
        assert!(mechs.contains(&AuthMechanism::Plain));
        assert!(mechs.contains(&AuthMechanism::CramMd5));
    }

    // --- parse_smtp_ehlo ---

    #[test]
    fn parse_smtp_ehlo_auth_line() {
        let ehlo = "250-smtp.example.com\r\n250-AUTH LOGIN PLAIN CRAM-MD5\r\n250 OK\r\n";
        let mechs = parse_smtp_ehlo(ehlo);
        assert_eq!(mechs.len(), 3);
        assert!(mechs.contains(&AuthMechanism::Login));
        assert!(mechs.contains(&AuthMechanism::Plain));
        assert!(mechs.contains(&AuthMechanism::CramMd5));
    }

    #[test]
    fn parse_smtp_ehlo_no_auth() {
        let ehlo = "250-smtp.example.com\r\n250-SIZE 26214400\r\n250 OK\r\n";
        let mechs = parse_smtp_ehlo(ehlo);
        assert!(mechs.is_empty());
    }

    #[test]
    fn parse_smtp_ehlo_final_line_auth() {
        let ehlo = "250 AUTH PLAIN LOGIN\r\n";
        let mechs = parse_smtp_ehlo(ehlo);
        assert_eq!(mechs.len(), 2);
        assert!(mechs.contains(&AuthMechanism::Plain));
        assert!(mechs.contains(&AuthMechanism::Login));
    }

    // --- parse_pop3_capa ---

    #[test]
    fn parse_pop3_sasl_line() {
        let capa = "CAPA\r\nSASL PLAIN LOGIN CRAM-MD5\r\nUIDL\r\n.\r\n";
        let mechs = parse_pop3_capa(capa);
        assert_eq!(mechs.len(), 3);
        assert!(mechs.contains(&AuthMechanism::Plain));
        assert!(mechs.contains(&AuthMechanism::Login));
        assert!(mechs.contains(&AuthMechanism::CramMd5));
    }

    #[test]
    fn parse_pop3_no_sasl() {
        let capa = "CAPA\r\nUIDL\r\nTOP\r\n.\r\n";
        let mechs = parse_pop3_capa(capa);
        assert!(mechs.is_empty());
    }

    // --- from_name ---

    #[test]
    fn from_name_known_mechanisms() {
        assert_eq!(
            AuthMechanism::from_name("CRAM-MD5"),
            Some(AuthMechanism::CramMd5)
        );
        assert_eq!(
            AuthMechanism::from_name("login"),
            Some(AuthMechanism::Login)
        );
        assert_eq!(
            AuthMechanism::from_name("Plain"),
            Some(AuthMechanism::Plain)
        );
        assert_eq!(AuthMechanism::from_name("NTLM"), Some(AuthMechanism::Ntlm));
        assert_eq!(
            AuthMechanism::from_name("XOAUTH2"),
            Some(AuthMechanism::Xoauth2)
        );
        assert_eq!(AuthMechanism::from_name("APOP"), Some(AuthMechanism::Apop));
        assert_eq!(
            AuthMechanism::from_name("EXTERNAL"),
            Some(AuthMechanism::External)
        );
    }

    #[test]
    fn from_name_unknown() {
        assert_eq!(AuthMechanism::from_name("GSSAPI"), None);
        assert_eq!(AuthMechanism::from_name(""), None);
    }

    // --- display / capability_name ---

    #[test]
    fn display_matches_capability_name() {
        assert_eq!(format!("{}", AuthMechanism::CramMd5), "CRAM-MD5");
        assert_eq!(format!("{}", AuthMechanism::Login), "LOGIN");
        assert_eq!(format!("{}", AuthMechanism::Plain), "PLAIN");
    }

    // --- log_negotiation ---

    #[test]
    fn log_negotiation_selected() {
        let msg = log_negotiation(AuthProtocol::Imap, Some(AuthMechanism::CramMd5));
        assert_eq!(msg, "IMAP auth negotiation: selected CRAM-MD5");
    }

    #[test]
    fn log_negotiation_none() {
        let msg = log_negotiation(AuthProtocol::Smtp, None);
        assert_eq!(
            msg,
            "SMTP auth negotiation: no common password mechanism found"
        );
    }

    // --- end-to-end: parse + negotiate ---

    #[test]
    fn end_to_end_imap_cram_md5_wins() {
        let caps = vec![
            "IMAP4rev1".to_string(),
            "AUTH=CRAM-MD5".to_string(),
            "AUTH=LOGIN".to_string(),
            "AUTH=PLAIN".to_string(),
        ];
        let server = parse_imap_capabilities(&caps);
        let result = negotiate_password_mechanism(AuthProtocol::Imap, &server);
        assert_eq!(result, Some(AuthMechanism::CramMd5));
    }

    #[test]
    fn end_to_end_smtp_login_when_no_cram() {
        let ehlo = "250-smtp.example.com\r\n250-AUTH LOGIN PLAIN\r\n250 OK\r\n";
        let server = parse_smtp_ehlo(ehlo);
        let result = negotiate_password_mechanism(AuthProtocol::Smtp, &server);
        assert_eq!(result, Some(AuthMechanism::Login));
    }

    #[test]
    fn end_to_end_pop3_plain_only() {
        let capa = "SASL PLAIN\r\n.\r\n";
        let server = parse_pop3_capa(capa);
        let result = negotiate_password_mechanism(AuthProtocol::Pop3, &server);
        assert_eq!(result, Some(AuthMechanism::Plain));
    }

    // --- EXTERNAL mechanism tests ---

    #[test]
    fn external_not_selected_by_password_negotiation() {
        // EXTERNAL is not a password mechanism — negotiate_password_mechanism must never pick it.
        let server = vec![AuthMechanism::External, AuthMechanism::Plain];
        let result = negotiate_password_mechanism(AuthProtocol::Imap, &server);
        assert_eq!(result, Some(AuthMechanism::Plain));
    }

    #[test]
    fn external_parsed_from_imap_capabilities() {
        let caps = vec![
            "AUTH=EXTERNAL".to_string(),
            "AUTH=PLAIN".to_string(),
            "IMAP4rev1".to_string(),
        ];
        let mechs = parse_imap_capabilities(&caps);
        assert!(mechs.contains(&AuthMechanism::External));
        assert!(mechs.contains(&AuthMechanism::Plain));
    }

    #[test]
    fn external_parsed_from_smtp_ehlo() {
        let ehlo = "250-smtp.example.com\r\n250-AUTH EXTERNAL PLAIN LOGIN\r\n250 OK\r\n";
        let mechs = parse_smtp_ehlo(ehlo);
        assert!(mechs.contains(&AuthMechanism::External));
        assert!(mechs.contains(&AuthMechanism::Plain));
        assert!(mechs.contains(&AuthMechanism::Login));
    }

    #[test]
    fn external_supported_for_all_protocols() {
        assert!(supported_mechanisms(AuthProtocol::Imap).contains(&AuthMechanism::External));
        assert!(supported_mechanisms(AuthProtocol::Pop3).contains(&AuthMechanism::External));
        assert!(supported_mechanisms(AuthProtocol::Smtp).contains(&AuthMechanism::External));
    }

    #[test]
    fn external_display_and_capability_name() {
        assert_eq!(AuthMechanism::External.capability_name(), "EXTERNAL");
        assert_eq!(format!("{}", AuthMechanism::External), "EXTERNAL");
    }
}
