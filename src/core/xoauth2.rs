//! XOAUTH2 SASL token builder.
//!
//! Builds the base64-encoded token used by the `AUTHENTICATE XOAUTH2` command
//! for IMAP and `AUTH XOAUTH2` for SMTP, per the XOAUTH2 protocol specification.
//!
//! Token format (before base64 encoding):
//! ```text
//! "user=" {username} "\x01" "auth=Bearer " {access_token} "\x01\x01"
//! ```

use base64::{engine::general_purpose::STANDARD, Engine as _};

/// Build the XOAUTH2 SASL token string (base64-encoded).
///
/// The raw token format is:
/// `user={username}\x01auth=Bearer {access_token}\x01\x01`
///
/// This is then base64-encoded for transmission over IMAP/SMTP.
pub(crate) fn build_xoauth2_token(username: &str, access_token: &str) -> String {
    let raw = format!("user={username}\x01auth=Bearer {access_token}\x01\x01");
    STANDARD.encode(raw.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::STANDARD;

    #[test]
    fn token_decodes_to_expected_format() {
        let token = build_xoauth2_token("user@example.com", "ya29.access-token");
        let decoded = STANDARD.decode(&token).unwrap();
        let decoded_str = String::from_utf8(decoded).unwrap();
        assert_eq!(
            decoded_str,
            "user=user@example.com\x01auth=Bearer ya29.access-token\x01\x01"
        );
    }

    #[test]
    fn token_is_valid_base64() {
        let token = build_xoauth2_token("test@gmail.com", "token123");
        assert!(STANDARD.decode(&token).is_ok());
    }

    #[test]
    fn token_contains_bearer_prefix() {
        let token = build_xoauth2_token("u@x.com", "tok");
        let decoded = String::from_utf8(STANDARD.decode(&token).unwrap()).unwrap();
        assert!(decoded.contains("auth=Bearer tok"));
    }

    #[test]
    fn token_terminates_with_double_ctrl_a() {
        let token = build_xoauth2_token("u@x.com", "tok");
        let decoded = STANDARD.decode(&token).unwrap();
        assert_eq!(decoded[decoded.len() - 2], 0x01);
        assert_eq!(decoded[decoded.len() - 1], 0x01);
    }

    #[test]
    fn empty_username_still_produces_valid_token() {
        let token = build_xoauth2_token("", "tok");
        let decoded = String::from_utf8(STANDARD.decode(&token).unwrap()).unwrap();
        assert!(decoded.starts_with("user=\x01"));
    }

    #[test]
    fn special_characters_in_token_are_preserved() {
        let token = build_xoauth2_token("user@example.com", "ya29.a/b+c=d");
        let decoded = String::from_utf8(STANDARD.decode(&token).unwrap()).unwrap();
        assert!(decoded.contains("ya29.a/b+c=d"));
    }
}
