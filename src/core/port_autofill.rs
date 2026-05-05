//! Port auto-fill logic for the manual server configuration form (FR-6, FR-7).
//!
//! When the user changes the encryption mode, the port field should auto-fill with
//! the conventional default for the selected protocol+encryption combination — but
//! only if the current port value is empty or matches a known default.

use super::account::{EncryptionMode, Protocol};

/// All known default ports across protocol/encryption combinations (inbound + SMTP).
const KNOWN_DEFAULT_PORTS: &[u16] = &[993, 143, 995, 110, 465, 587, 25];

/// Return the conventional default port for a given protocol and encryption mode.
pub fn default_port(protocol: Protocol, encryption: EncryptionMode) -> u16 {
    match (protocol, encryption) {
        (Protocol::Imap, EncryptionMode::SslTls) => 993,
        (Protocol::Imap, EncryptionMode::StartTls | EncryptionMode::None) => 143,
        (Protocol::Pop3, EncryptionMode::SslTls) => 995,
        (Protocol::Pop3, EncryptionMode::StartTls | EncryptionMode::None) => 110,
    }
}

/// Return the conventional default SMTP port for the given encryption mode.
///
/// SSL/TLS → 465, STARTTLS → 587, None → 25.
pub fn smtp_default_port(encryption: EncryptionMode) -> u16 {
    match encryption {
        EncryptionMode::SslTls => 465,
        EncryptionMode::StartTls => 587,
        EncryptionMode::None => 25,
    }
}

/// Determine whether the current port value should be replaced by the new default.
///
/// Returns `true` (meaning: auto-fill is allowed) when:
/// - `current_port` is `None` (field is empty / unset), OR
/// - `current_port` matches one of the known default ports.
///
/// A user-entered non-default port (e.g. 1993) is never overwritten.
pub fn should_autofill(current_port: Option<u16>) -> bool {
    match current_port {
        None => true,
        Some(port) => KNOWN_DEFAULT_PORTS.contains(&port),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- default_port tests --

    #[test]
    fn imap_ssl_tls_returns_993() {
        assert_eq!(default_port(Protocol::Imap, EncryptionMode::SslTls), 993);
    }

    #[test]
    fn imap_starttls_returns_143() {
        assert_eq!(default_port(Protocol::Imap, EncryptionMode::StartTls), 143);
    }

    #[test]
    fn imap_none_returns_143() {
        assert_eq!(default_port(Protocol::Imap, EncryptionMode::None), 143);
    }

    #[test]
    fn pop3_ssl_tls_returns_995() {
        assert_eq!(default_port(Protocol::Pop3, EncryptionMode::SslTls), 995);
    }

    #[test]
    fn pop3_starttls_returns_110() {
        assert_eq!(default_port(Protocol::Pop3, EncryptionMode::StartTls), 110);
    }

    #[test]
    fn pop3_none_returns_110() {
        assert_eq!(default_port(Protocol::Pop3, EncryptionMode::None), 110);
    }

    // -- should_autofill tests --

    #[test]
    fn empty_port_allows_autofill() {
        assert!(should_autofill(None));
    }

    #[test]
    fn known_default_993_allows_autofill() {
        assert!(should_autofill(Some(993)));
    }

    #[test]
    fn known_default_143_allows_autofill() {
        assert!(should_autofill(Some(143)));
    }

    #[test]
    fn known_default_995_allows_autofill() {
        assert!(should_autofill(Some(995)));
    }

    #[test]
    fn known_default_110_allows_autofill() {
        assert!(should_autofill(Some(110)));
    }

    #[test]
    fn custom_port_does_not_allow_autofill() {
        assert!(!should_autofill(Some(1993)));
    }

    #[test]
    fn another_custom_port_does_not_allow_autofill() {
        assert!(!should_autofill(Some(8993)));
    }

    #[test]
    fn port_zero_does_not_allow_autofill() {
        // 0 is not a known default, so it's treated as user-entered
        assert!(!should_autofill(Some(0)));
    }

    // -- smtp_default_port tests --

    #[test]
    fn smtp_ssl_tls_returns_465() {
        assert_eq!(smtp_default_port(EncryptionMode::SslTls), 465);
    }

    #[test]
    fn smtp_starttls_returns_587() {
        assert_eq!(smtp_default_port(EncryptionMode::StartTls), 587);
    }

    #[test]
    fn smtp_none_returns_25() {
        assert_eq!(smtp_default_port(EncryptionMode::None), 25);
    }

    // -- SMTP ports in should_autofill --

    #[test]
    fn known_default_465_allows_autofill() {
        assert!(should_autofill(Some(465)));
    }

    #[test]
    fn known_default_587_allows_autofill() {
        assert!(should_autofill(Some(587)));
    }

    #[test]
    fn known_default_25_allows_autofill() {
        assert!(should_autofill(Some(25)));
    }
}
