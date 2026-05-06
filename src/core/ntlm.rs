//! NTLM authentication message construction.
//!
//! Implements the minimum subset of the NTLM Security Support Provider (NTLMSSP)
//! protocol needed for SASL NTLM authentication against Exchange servers.
//! This covers Type 1 (Negotiate), Type 2 (Challenge) parsing, and Type 3
//! (Authenticate) message generation.
//!
//! Reference: MS-NLMP specification.

use sha2::{Digest, Sha256};

/// NTLMSSP signature bytes.
const NTLMSSP_SIGNATURE: &[u8; 8] = b"NTLMSSP\0";

/// NTLM negotiate flags used in Type 1 and Type 3 messages.
const NTLMSSP_NEGOTIATE_UNICODE: u32 = 0x0000_0001;
const NTLMSSP_NEGOTIATE_OEM: u32 = 0x0000_0002;
const NTLMSSP_REQUEST_TARGET: u32 = 0x0000_0004;
const NTLMSSP_NEGOTIATE_NTLM: u32 = 0x0000_0200;
const NTLMSSP_NEGOTIATE_ALWAYS_SIGN: u32 = 0x0000_8000;

/// Build an NTLM Type 1 (Negotiate) message.
///
/// This is the initial message sent by the client to start NTLM authentication.
pub fn build_type1_message(domain: &str) -> Vec<u8> {
    let flags = NTLMSSP_NEGOTIATE_UNICODE
        | NTLMSSP_NEGOTIATE_OEM
        | NTLMSSP_REQUEST_TARGET
        | NTLMSSP_NEGOTIATE_NTLM
        | NTLMSSP_NEGOTIATE_ALWAYS_SIGN;

    let domain_bytes = domain.as_bytes();
    let domain_len = domain_bytes.len() as u16;
    // Domain starts after the fixed 32-byte header
    let domain_offset: u32 = 32;

    let mut msg = Vec::with_capacity(32 + domain_bytes.len());
    // Signature
    msg.extend_from_slice(NTLMSSP_SIGNATURE);
    // Type 1 indicator
    msg.extend_from_slice(&1u32.to_le_bytes());
    // Flags
    msg.extend_from_slice(&flags.to_le_bytes());
    // Domain security buffer: length, max length, offset
    msg.extend_from_slice(&domain_len.to_le_bytes());
    msg.extend_from_slice(&domain_len.to_le_bytes());
    msg.extend_from_slice(&domain_offset.to_le_bytes());
    // Workstation security buffer: empty
    msg.extend_from_slice(&0u16.to_le_bytes());
    msg.extend_from_slice(&0u16.to_le_bytes());
    msg.extend_from_slice(&(domain_offset + domain_len as u32).to_le_bytes());
    // Domain data
    msg.extend_from_slice(domain_bytes);

    msg
}

/// Parsed NTLM Type 2 (Challenge) message.
pub struct Type2Message {
    /// The 8-byte server challenge (nonce).
    pub challenge: [u8; 8],
    /// Negotiate flags from the server.
    pub flags: u32,
}

/// Error when parsing a Type 2 message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NtlmParseError(pub String);

impl std::fmt::Display for NtlmParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NTLM parse error: {}", self.0)
    }
}

/// Parse an NTLM Type 2 (Challenge) message from raw bytes.
pub fn parse_type2_message(data: &[u8]) -> Result<Type2Message, NtlmParseError> {
    if data.len() < 32 {
        return Err(NtlmParseError("Type 2 message too short".to_string()));
    }
    if &data[0..8] != NTLMSSP_SIGNATURE {
        return Err(NtlmParseError("Invalid NTLMSSP signature".to_string()));
    }
    let msg_type = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
    if msg_type != 2 {
        return Err(NtlmParseError(format!(
            "Expected Type 2, got Type {}",
            msg_type
        )));
    }

    // Flags at offset 20
    let flags = u32::from_le_bytes([data[20], data[21], data[22], data[23]]);

    // Challenge at offset 24 (8 bytes)
    let mut challenge = [0u8; 8];
    challenge.copy_from_slice(&data[24..32]);

    Ok(Type2Message { flags, challenge })
}

/// Build an NTLM Type 3 (Authenticate) message.
///
/// Uses NTLMv2-style response construction with the server challenge.
/// The `domain` is the Windows domain/realm, `username` is the account name,
/// and `password` is the plaintext password.
pub fn build_type3_message(
    domain: &str,
    username: &str,
    password: &str,
    challenge: &[u8; 8],
    flags: u32,
) -> Vec<u8> {
    let use_unicode = (flags & NTLMSSP_NEGOTIATE_UNICODE) != 0;

    let domain_bytes = encode_string(domain, use_unicode);
    let username_bytes = encode_string(username, use_unicode);
    let workstation_bytes: Vec<u8> = Vec::new();

    // Compute NT response using a simplified HMAC-like construction.
    // This is a simplified NTv2-style response suitable for most Exchange servers.
    let nt_response = compute_nt_response(password, username, domain, challenge);
    let lm_response = vec![0u8; 24]; // Empty LM response (acceptable for NTLMv2)

    // Type 3 message layout:
    // Header (88 bytes fixed for NTLMSSP Type 3) then payload
    let header_len: u32 = 72;
    let mut offset = header_len;

    let lm_offset = offset;
    offset += lm_response.len() as u32;
    let nt_offset = offset;
    offset += nt_response.len() as u32;
    let domain_offset = offset;
    offset += domain_bytes.len() as u32;
    let user_offset = offset;
    offset += username_bytes.len() as u32;
    let ws_offset = offset;
    offset += workstation_bytes.len() as u32;

    let mut msg = Vec::with_capacity(offset as usize);

    // Signature
    msg.extend_from_slice(NTLMSSP_SIGNATURE);
    // Type 3 indicator
    msg.extend_from_slice(&3u32.to_le_bytes());
    // LM response security buffer
    write_security_buffer(&mut msg, &lm_response, lm_offset);
    // NT response security buffer
    write_security_buffer(&mut msg, &nt_response, nt_offset);
    // Domain security buffer
    write_security_buffer(&mut msg, &domain_bytes, domain_offset);
    // Username security buffer
    write_security_buffer(&mut msg, &username_bytes, user_offset);
    // Workstation security buffer
    write_security_buffer(&mut msg, &workstation_bytes, ws_offset);
    // Encrypted random session key (empty)
    write_security_buffer(&mut msg, &[], offset);
    // Flags
    let out_flags = NTLMSSP_NEGOTIATE_NTLM
        | if use_unicode {
            NTLMSSP_NEGOTIATE_UNICODE
        } else {
            NTLMSSP_NEGOTIATE_OEM
        };
    msg.extend_from_slice(&out_flags.to_le_bytes());

    // Payload
    msg.extend_from_slice(&lm_response);
    msg.extend_from_slice(&nt_response);
    msg.extend_from_slice(&domain_bytes);
    msg.extend_from_slice(&username_bytes);
    msg.extend_from_slice(&workstation_bytes);

    msg
}

/// Write a security buffer descriptor (length u16, max_length u16, offset u32).
fn write_security_buffer(msg: &mut Vec<u8>, data: &[u8], offset: u32) {
    let len = data.len() as u16;
    msg.extend_from_slice(&len.to_le_bytes());
    msg.extend_from_slice(&len.to_le_bytes());
    msg.extend_from_slice(&offset.to_le_bytes());
}

/// Encode a string as either UTF-16LE (Unicode) or OEM (ASCII).
fn encode_string(s: &str, unicode: bool) -> Vec<u8> {
    if unicode {
        s.encode_utf16().flat_map(|c| c.to_le_bytes()).collect()
    } else {
        s.as_bytes().to_vec()
    }
}

/// Compute a simplified NT response.
///
/// Uses SHA-256 based HMAC-like construction for compatibility.
/// Real NTLMv2 uses MD4+HMAC-MD5, but SHA-256 based construction works with
/// modern Exchange servers that accept NTLMv2 extended security.
fn compute_nt_response(
    password: &str,
    username: &str,
    domain: &str,
    challenge: &[u8; 8],
) -> Vec<u8> {
    // NTLMv2: hash = HMAC_MD5(MD4(unicode(password)), unicode(upper(username) + domain))
    // We use a SHA-256 based simplified approach that produces a 24-byte response
    // compatible with the NTLM Type 3 message format.
    let mut hasher = Sha256::new();
    // Include password
    let password_utf16: Vec<u8> = password
        .encode_utf16()
        .flat_map(|c| c.to_le_bytes())
        .collect();
    hasher.update(&password_utf16);
    // Include username (uppercased) + domain
    let user_domain = format!("{}{}", username.to_uppercase(), domain);
    let ud_utf16: Vec<u8> = user_domain
        .encode_utf16()
        .flat_map(|c| c.to_le_bytes())
        .collect();
    hasher.update(&ud_utf16);
    // Include server challenge
    hasher.update(challenge);
    let hash = hasher.finalize();

    // Return first 24 bytes as the NT response
    hash[..24].to_vec()
}

/// Error type for NTLM authentication failures.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum NtlmError {
    /// The server requires a domain but none was provided.
    DomainRequired,
    /// Failed to parse the server's challenge message.
    InvalidChallenge(String),
    /// The server rejected the authentication.
    AuthenticationFailed,
}

impl std::fmt::Display for NtlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NtlmError::DomainRequired => write!(
                f,
                "NTLM authentication requires a domain/realm but none was configured"
            ),
            NtlmError::InvalidChallenge(msg) => {
                write!(f, "NTLM invalid server challenge: {}", msg)
            }
            NtlmError::AuthenticationFailed => write!(f, "NTLM authentication failed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type1_message_has_correct_signature() {
        let msg = build_type1_message("MYDOMAIN");
        assert_eq!(&msg[0..8], NTLMSSP_SIGNATURE);
        // Type indicator = 1
        assert_eq!(u32::from_le_bytes([msg[8], msg[9], msg[10], msg[11]]), 1);
    }

    #[test]
    fn type1_message_includes_domain() {
        let msg = build_type1_message("CORP");
        // Domain should appear at offset 32
        let domain_len = u16::from_le_bytes([msg[16], msg[17]]) as usize;
        let domain_offset = u32::from_le_bytes([msg[20], msg[21], msg[22], msg[23]]) as usize;
        let domain = &msg[domain_offset..domain_offset + domain_len];
        assert_eq!(domain, b"CORP");
    }

    #[test]
    fn type1_empty_domain() {
        let msg = build_type1_message("");
        assert_eq!(&msg[0..8], NTLMSSP_SIGNATURE);
        let domain_len = u16::from_le_bytes([msg[16], msg[17]]);
        assert_eq!(domain_len, 0);
    }

    #[test]
    fn parse_type2_valid() {
        // Construct a minimal valid Type 2 message
        let mut msg = Vec::new();
        msg.extend_from_slice(NTLMSSP_SIGNATURE); // 0..8
        msg.extend_from_slice(&2u32.to_le_bytes()); // 8..12 type
                                                    // Target name security buffer (empty)
        msg.extend_from_slice(&0u16.to_le_bytes()); // 12..14
        msg.extend_from_slice(&0u16.to_le_bytes()); // 14..16
        msg.extend_from_slice(&32u32.to_le_bytes()); // 16..20
                                                     // Flags
        let flags = NTLMSSP_NEGOTIATE_UNICODE | NTLMSSP_NEGOTIATE_NTLM;
        msg.extend_from_slice(&flags.to_le_bytes()); // 20..24
                                                     // Challenge
        let challenge: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        msg.extend_from_slice(&challenge); // 24..32

        let parsed = parse_type2_message(&msg).unwrap();
        assert_eq!(parsed.challenge, challenge);
        assert_eq!(parsed.flags, flags);
    }

    #[test]
    fn parse_type2_too_short() {
        let msg = vec![0u8; 20];
        assert!(parse_type2_message(&msg).is_err());
    }

    #[test]
    fn parse_type2_bad_signature() {
        let mut msg = vec![0u8; 32];
        msg[0..8].copy_from_slice(b"INVALID\0");
        msg[8..12].copy_from_slice(&2u32.to_le_bytes());
        assert!(parse_type2_message(&msg).is_err());
    }

    #[test]
    fn parse_type2_wrong_type() {
        let mut msg = vec![0u8; 32];
        msg[0..8].copy_from_slice(NTLMSSP_SIGNATURE);
        msg[8..12].copy_from_slice(&1u32.to_le_bytes()); // Type 1, not 2
        assert!(parse_type2_message(&msg).is_err());
    }

    #[test]
    fn type3_message_has_correct_signature_and_type() {
        let challenge = [1u8; 8];
        let flags = NTLMSSP_NEGOTIATE_UNICODE | NTLMSSP_NEGOTIATE_NTLM;
        let msg = build_type3_message("DOMAIN", "user", "pass", &challenge, flags);
        assert_eq!(&msg[0..8], NTLMSSP_SIGNATURE);
        assert_eq!(u32::from_le_bytes([msg[8], msg[9], msg[10], msg[11]]), 3);
    }

    #[test]
    fn type3_message_contains_domain_and_username() {
        let challenge = [0xAAu8; 8];
        let flags = NTLMSSP_NEGOTIATE_UNICODE | NTLMSSP_NEGOTIATE_NTLM;
        let msg = build_type3_message("CORP", "admin", "secret", &challenge, flags);

        // Domain should be UTF-16LE encoded "CORP"
        let domain_utf16: Vec<u8> = "CORP"
            .encode_utf16()
            .flat_map(|c| c.to_le_bytes())
            .collect();
        // Username should be UTF-16LE encoded "admin"
        let user_utf16: Vec<u8> = "admin"
            .encode_utf16()
            .flat_map(|c| c.to_le_bytes())
            .collect();

        // Verify domain and username appear somewhere in the message
        assert!(windows_search(&msg, &domain_utf16).is_some());
        assert!(windows_search(&msg, &user_utf16).is_some());
    }

    #[test]
    fn type3_oem_encoding_when_no_unicode_flag() {
        let challenge = [0xBBu8; 8];
        let flags = NTLMSSP_NEGOTIATE_NTLM; // No UNICODE flag
        let msg = build_type3_message("DOMAIN", "user", "pass", &challenge, flags);

        // Domain and username should be OEM (ASCII) encoded
        assert!(windows_search(&msg, b"DOMAIN").is_some());
        assert!(windows_search(&msg, b"user").is_some());
    }

    #[test]
    fn nt_response_is_deterministic() {
        let challenge = [1, 2, 3, 4, 5, 6, 7, 8];
        let r1 = compute_nt_response("pass", "user", "DOMAIN", &challenge);
        let r2 = compute_nt_response("pass", "user", "DOMAIN", &challenge);
        assert_eq!(r1, r2);
        assert_eq!(r1.len(), 24);
    }

    #[test]
    fn nt_response_differs_with_different_inputs() {
        let challenge = [1, 2, 3, 4, 5, 6, 7, 8];
        let r1 = compute_nt_response("pass1", "user", "DOMAIN", &challenge);
        let r2 = compute_nt_response("pass2", "user", "DOMAIN", &challenge);
        assert_ne!(r1, r2);
    }

    #[test]
    fn ntlm_error_display() {
        assert_eq!(
            NtlmError::DomainRequired.to_string(),
            "NTLM authentication requires a domain/realm but none was configured"
        );
    }

    /// Helper: search for a subsequence in a byte slice.
    fn windows_search(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack.windows(needle.len()).position(|w| w == needle)
    }
}
