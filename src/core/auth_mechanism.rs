//! Password-based authentication mechanism negotiation.
//!
//! Determines the strongest SASL mechanism that both the application and the
//! mail server support, based on protocol-specific capability advertisements
//! (IMAP CAPABILITY, SMTP EHLO, POP3 CAPA).

use crate::core::account::EncryptionMode;
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
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

/// Global toggles controlling which password-based mechanisms the application
/// is allowed to attempt on any connection (FR-25 through FR-29, Design Note N-4).
///
/// All password-based mechanisms except APOP are enabled by default.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MechanismToggles {
    /// Allow AUTH PLAIN.
    #[serde(default = "default_true")]
    pub plain_enabled: bool,
    /// Allow AUTH LOGIN.
    #[serde(default = "default_true")]
    pub login_enabled: bool,
    /// Allow AUTH NTLM.
    #[serde(default = "default_true")]
    pub ntlm_enabled: bool,
    /// Allow AUTH CRAM-MD5 (SASL challenge-response).
    #[serde(default = "default_true")]
    pub cram_md5_enabled: bool,
    /// Allow APOP (POP3 only). Disabled by default (Design Note N-3).
    #[serde(default)]
    pub apop_enabled: bool,
}

fn default_true() -> bool {
    true
}

impl Default for MechanismToggles {
    fn default() -> Self {
        Self {
            plain_enabled: true,
            login_enabled: true,
            ntlm_enabled: true,
            cram_md5_enabled: true,
            apop_enabled: false,
        }
    }
}

impl MechanismToggles {
    /// Returns `true` if the given mechanism is allowed by the global toggles.
    pub fn is_enabled(&self, mechanism: AuthMechanism) -> bool {
        match mechanism {
            AuthMechanism::Plain => self.plain_enabled,
            AuthMechanism::Login => self.login_enabled,
            AuthMechanism::Ntlm => self.ntlm_enabled,
            AuthMechanism::CramMd5 => self.cram_md5_enabled,
            AuthMechanism::Apop => self.apop_enabled,
            // Non-password mechanisms (OAuth, External) are not gated by these toggles.
            AuthMechanism::Xoauth2 | AuthMechanism::External => true,
        }
    }
}

/// Filter a list of mechanisms, removing any disabled by the global toggles.
pub fn filter_by_toggles(
    mechanisms: &[AuthMechanism],
    toggles: &MechanismToggles,
) -> Vec<AuthMechanism> {
    mechanisms
        .iter()
        .copied()
        .filter(|m| toggles.is_enabled(*m))
        .collect()
}

/// Returns `true` for mechanisms that transmit the password in recoverable form
/// (PLAIN and LOGIN). These must not be used over unencrypted connections unless
/// the user has explicitly opted in via `allow_insecure_auth`.
pub fn is_plaintext_mechanism(mechanism: AuthMechanism) -> bool {
    matches!(mechanism, AuthMechanism::Plain | AuthMechanism::Login)
}

/// Remove plaintext password mechanisms (PLAIN, LOGIN) from `mechanisms` when the
/// connection is unencrypted and the user has not opted in to insecure auth (FR-30).
///
/// Returns `Ok(filtered)` on success.  Returns `Err(message)` when *all*
/// password-capable mechanisms were removed, meaning authentication cannot proceed
/// without exposing the password on the wire.
pub fn filter_insecure_mechanisms(
    mechanisms: &[AuthMechanism],
    encryption: EncryptionMode,
    allow_insecure_auth: bool,
) -> Result<Vec<AuthMechanism>, String> {
    if encryption != EncryptionMode::None || allow_insecure_auth {
        // Connection is encrypted, or user opted in — no filtering needed.
        return Ok(mechanisms.to_vec());
    }

    let filtered: Vec<AuthMechanism> = mechanisms
        .iter()
        .copied()
        .filter(|m| !is_plaintext_mechanism(*m))
        .collect();

    // If filtering removed every mechanism the caller had, return an error so
    // the connection layer can surface a clear message instead of silently
    // failing or falling through to an empty-mechanism path.
    if filtered.is_empty() && !mechanisms.is_empty() {
        return Err(
            "Refusing to authenticate: PLAIN/LOGIN not permitted over an unencrypted connection. \
             Enable \"Allow insecure authentication\" in account settings to override."
                .to_string(),
        );
    }

    Ok(filtered)
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

/// Extract the APOP timestamp from a POP3 server greeting.
///
/// Per RFC 1939 §7 the greeting may contain a timestamp of the form
/// `<process-id.clock@hostname>`.  This function returns the full
/// angle-bracket-delimited token if present.
///
/// # Examples
/// ```text
/// +OK POP3 server ready <1896.697170952@dbc.mtview.ca.us>
/// ```
/// returns `Some("<1896.697170952@dbc.mtview.ca.us>")`.
pub fn parse_pop3_greeting_timestamp(greeting: &str) -> Option<&str> {
    let start = greeting.find('<')?;
    let end = greeting[start..].find('>')? + start + 1;
    let candidate = &greeting[start..end];
    // RFC 1939: timestamp must contain an '@' inside the angle brackets.
    if candidate.contains('@') {
        Some(candidate)
    } else {
        None
    }
}

/// Compute the APOP digest for a given timestamp and password.
///
/// The digest is `MD5(timestamp || password)` rendered as a lowercase
/// hex string (RFC 1939 §7).
pub fn compute_apop_digest(timestamp: &str, password: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(timestamp.as_bytes());
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    // Format as lowercase hex
    result.iter().map(|b| format!("{b:02x}")).collect()
}

/// Determine whether APOP should be used for the current POP3 connection.
///
/// Returns `Some((username, digest))` when all conditions are met:
/// 1. `apop_enabled` is `true` in the account's POP3 settings.
/// 2. The server greeting contains a valid RFC 1939 timestamp.
///
/// When `None` is returned the caller should fall back to the standard
/// password-based mechanism negotiation.
pub fn should_use_apop(
    apop_enabled: bool,
    greeting: &str,
    username: &str,
    password: &str,
) -> Option<(String, String)> {
    if !apop_enabled {
        return None;
    }
    let timestamp = parse_pop3_greeting_timestamp(greeting)?;
    let digest = compute_apop_digest(timestamp, password);
    Some((username.to_string(), digest))
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

    // --- APOP timestamp parsing ---

    #[test]
    fn parse_greeting_extracts_timestamp() {
        let greeting = "+OK POP3 server ready <1896.697170952@dbc.mtview.ca.us>";
        assert_eq!(
            parse_pop3_greeting_timestamp(greeting),
            Some("<1896.697170952@dbc.mtview.ca.us>")
        );
    }

    #[test]
    fn parse_greeting_no_timestamp() {
        let greeting = "+OK POP3 server ready";
        assert_eq!(parse_pop3_greeting_timestamp(greeting), None);
    }

    #[test]
    fn parse_greeting_angle_brackets_without_at() {
        let greeting = "+OK ready <no-at-sign>";
        assert_eq!(parse_pop3_greeting_timestamp(greeting), None);
    }

    #[test]
    fn parse_greeting_timestamp_at_end() {
        let greeting = "+OK <42.12345@mail.example.com>";
        assert_eq!(
            parse_pop3_greeting_timestamp(greeting),
            Some("<42.12345@mail.example.com>")
        );
    }

    // --- APOP digest computation ---

    #[test]
    fn compute_apop_digest_rfc_example() {
        // RFC 1939 §7 example timestamp with a known password.
        // Verified: echo -n '<1896.697170952@dbc.mtview.ca.us>tanstraafl' | md5sum
        let digest = compute_apop_digest("<1896.697170952@dbc.mtview.ca.us>", "tanstraafl");
        assert_eq!(digest, "e4e56d68fc0ee4afd97e43990456172a");
    }

    #[test]
    fn compute_apop_digest_empty_password() {
        // Just ensure it doesn't panic; the digest is deterministic.
        let d1 = compute_apop_digest("<ts@host>", "");
        let d2 = compute_apop_digest("<ts@host>", "");
        assert_eq!(d1, d2);
        assert_eq!(d1.len(), 32); // 128-bit MD5 = 32 hex chars
    }

    // --- should_use_apop ---

    #[test]
    fn should_use_apop_enabled_with_timestamp() {
        let greeting = "+OK POP3 ready <123.456@example.com>";
        let result = should_use_apop(true, greeting, "user", "pass");
        assert!(result.is_some());
        let (username, digest) = result.unwrap();
        assert_eq!(username, "user");
        assert_eq!(digest, compute_apop_digest("<123.456@example.com>", "pass"));
    }

    #[test]
    fn should_use_apop_disabled() {
        let greeting = "+OK POP3 ready <123.456@example.com>";
        assert_eq!(should_use_apop(false, greeting, "user", "pass"), None);
    }

    #[test]
    fn should_use_apop_enabled_no_timestamp() {
        let greeting = "+OK POP3 ready";
        assert_eq!(should_use_apop(true, greeting, "user", "pass"), None);
    }

    #[test]
    fn should_use_apop_disabled_no_timestamp() {
        let greeting = "+OK POP3 ready";
        assert_eq!(should_use_apop(false, greeting, "user", "pass"), None);
    }

    // --- MechanismToggles ---

    #[test]
    fn default_toggles_enable_all_password_mechanisms_except_apop() {
        let toggles = MechanismToggles::default();
        assert!(toggles.plain_enabled);
        assert!(toggles.login_enabled);
        assert!(toggles.ntlm_enabled);
        assert!(toggles.cram_md5_enabled);
        assert!(!toggles.apop_enabled);
    }

    #[test]
    fn is_enabled_matches_toggle_state() {
        let all_on = MechanismToggles::default();
        assert!(all_on.is_enabled(AuthMechanism::Plain));
        assert!(all_on.is_enabled(AuthMechanism::CramMd5));

        let plain_off = MechanismToggles {
            plain_enabled: false,
            ..Default::default()
        };
        assert!(!plain_off.is_enabled(AuthMechanism::Plain));
        assert!(plain_off.is_enabled(AuthMechanism::CramMd5));

        // Non-password mechanisms are always enabled.
        assert!(plain_off.is_enabled(AuthMechanism::Xoauth2));
        assert!(plain_off.is_enabled(AuthMechanism::External));
    }

    #[test]
    fn is_enabled_apop_default_off() {
        let toggles = MechanismToggles::default();
        assert!(!toggles.is_enabled(AuthMechanism::Apop));
    }

    // --- filter_by_toggles ---

    #[test]
    fn filter_removes_disabled_mechanisms() {
        let toggles = MechanismToggles {
            cram_md5_enabled: false,
            ..Default::default()
        };

        let server = vec![
            AuthMechanism::CramMd5,
            AuthMechanism::Login,
            AuthMechanism::Plain,
        ];
        let filtered = filter_by_toggles(&server, &toggles);
        assert_eq!(filtered, vec![AuthMechanism::Login, AuthMechanism::Plain]);
    }

    #[test]
    fn filter_keeps_non_password_mechanisms_regardless() {
        let toggles = MechanismToggles {
            plain_enabled: false,
            ..Default::default()
        };

        let server = vec![AuthMechanism::Xoauth2, AuthMechanism::Plain];
        let filtered = filter_by_toggles(&server, &toggles);
        assert_eq!(filtered, vec![AuthMechanism::Xoauth2]);
    }

    #[test]
    fn filter_with_all_defaults_keeps_password_mechanisms() {
        let toggles = MechanismToggles::default();
        let server = vec![
            AuthMechanism::CramMd5,
            AuthMechanism::Login,
            AuthMechanism::Plain,
            AuthMechanism::Ntlm,
        ];
        let filtered = filter_by_toggles(&server, &toggles);
        assert_eq!(filtered, server);
    }

    #[test]
    fn disabling_cram_md5_causes_fallback_to_login_or_plain() {
        let toggles = MechanismToggles {
            cram_md5_enabled: false,
            ..Default::default()
        };

        let server = vec![
            AuthMechanism::CramMd5,
            AuthMechanism::Login,
            AuthMechanism::Plain,
        ];
        let filtered = filter_by_toggles(&server, &toggles);
        let negotiated = negotiate_password_mechanism(AuthProtocol::Imap, &filtered);
        assert_eq!(negotiated, Some(AuthMechanism::Login));
    }

    #[test]
    fn all_mechanisms_disabled_returns_none_from_negotiation() {
        let toggles = MechanismToggles {
            plain_enabled: false,
            login_enabled: false,
            cram_md5_enabled: false,
            ntlm_enabled: false,
            ..Default::default()
        };

        let server = vec![
            AuthMechanism::CramMd5,
            AuthMechanism::Login,
            AuthMechanism::Plain,
            AuthMechanism::Ntlm,
        ];
        let filtered = filter_by_toggles(&server, &toggles);
        let negotiated = negotiate_password_mechanism(AuthProtocol::Imap, &filtered);
        assert_eq!(negotiated, None);
    }

    #[test]
    fn toggles_serde_roundtrip() {
        let toggles = MechanismToggles {
            cram_md5_enabled: false,
            apop_enabled: true,
            ..Default::default()
        };
        let json = serde_json::to_string(&toggles).unwrap();
        let restored: MechanismToggles = serde_json::from_str(&json).unwrap();
        assert!(!restored.cram_md5_enabled);
        assert!(restored.apop_enabled);
        assert!(restored.plain_enabled);
    }

    #[test]
    fn toggles_deserialize_empty_json_gives_defaults() {
        let restored: MechanismToggles = serde_json::from_str("{}").unwrap();
        assert!(restored.plain_enabled);
        assert!(restored.login_enabled);
        assert!(restored.ntlm_enabled);
        assert!(restored.cram_md5_enabled);
        assert!(!restored.apop_enabled);
    }

    // --- is_plaintext_mechanism ---

    #[test]
    fn plaintext_mechanisms_identified() {
        assert!(is_plaintext_mechanism(AuthMechanism::Plain));
        assert!(is_plaintext_mechanism(AuthMechanism::Login));
        assert!(!is_plaintext_mechanism(AuthMechanism::CramMd5));
        assert!(!is_plaintext_mechanism(AuthMechanism::Ntlm));
        assert!(!is_plaintext_mechanism(AuthMechanism::Xoauth2));
        assert!(!is_plaintext_mechanism(AuthMechanism::External));
        assert!(!is_plaintext_mechanism(AuthMechanism::Apop));
    }

    // --- filter_insecure_mechanisms ---

    #[test]
    fn filter_insecure_allows_all_when_encrypted() {
        let mechs = vec![AuthMechanism::Plain, AuthMechanism::Login];
        let result = filter_insecure_mechanisms(&mechs, EncryptionMode::SslTls, false);
        assert_eq!(result.unwrap(), mechs);
    }

    #[test]
    fn filter_insecure_allows_all_with_starttls() {
        let mechs = vec![AuthMechanism::Plain, AuthMechanism::Login];
        let result = filter_insecure_mechanisms(&mechs, EncryptionMode::StartTls, false);
        assert_eq!(result.unwrap(), mechs);
    }

    #[test]
    fn filter_insecure_allows_all_when_opted_in() {
        let mechs = vec![AuthMechanism::Plain, AuthMechanism::Login];
        let result = filter_insecure_mechanisms(&mechs, EncryptionMode::None, true);
        assert_eq!(result.unwrap(), mechs);
    }

    #[test]
    fn filter_insecure_removes_plain_login_over_none() {
        let mechs = vec![
            AuthMechanism::CramMd5,
            AuthMechanism::Plain,
            AuthMechanism::Login,
        ];
        let result = filter_insecure_mechanisms(&mechs, EncryptionMode::None, false).unwrap();
        assert_eq!(result, vec![AuthMechanism::CramMd5]);
    }

    #[test]
    fn filter_insecure_errors_when_only_plaintext_over_none() {
        let mechs = vec![AuthMechanism::Plain, AuthMechanism::Login];
        let result = filter_insecure_mechanisms(&mechs, EncryptionMode::None, false);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("PLAIN/LOGIN not permitted"));
    }

    #[test]
    fn filter_insecure_empty_input_returns_empty() {
        let result = filter_insecure_mechanisms(&[], EncryptionMode::None, false).unwrap();
        assert!(result.is_empty());
    }
}
