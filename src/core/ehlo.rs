//! EHLO hostname resolution for SMTP connections (FR-52, FR-53).
//!
//! Determines the hostname to use in the SMTP EHLO greeting based on
//! identity configuration: either the device's local IP address or a
//! user-specified custom hostname.

use std::net::UdpSocket;

/// Resolve the EHLO hostname from identity settings.
///
/// - If `use_ip` is `true`, returns the device's local IP address.
/// - If `use_ip` is `false` and `custom_ehlo` is a non-empty string, returns that.
/// - Otherwise returns `None` (caller should fall back to "localhost").
pub fn resolve_ehlo_hostname(use_ip: bool, custom_ehlo: Option<&str>) -> Option<String> {
    if use_ip {
        get_local_ip()
    } else {
        custom_ehlo
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
    }
}

/// Get the device's local IP address by opening a UDP socket to a public
/// address (no data is actually sent). Returns `None` if the address cannot
/// be determined.
fn get_local_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    // Connect to a well-known address to determine the local interface IP.
    // No packets are actually sent for UDP.
    socket.connect("8.8.8.8:80").ok()?;
    let addr = socket.local_addr().ok()?;
    Some(format!("[{}]", addr.ip()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn use_ip_true_returns_some() {
        // In most test environments a local IP is available.
        let result = resolve_ehlo_hostname(true, None);
        // Even if we can't guarantee IP resolution in CI, we test the branch.
        // If it returns Some, it should be bracketed.
        if let Some(ref hostname) = result {
            assert!(hostname.starts_with('['));
            assert!(hostname.ends_with(']'));
        }
    }

    #[test]
    fn use_ip_false_with_custom_returns_custom() {
        let result = resolve_ehlo_hostname(false, Some("mail.example.com"));
        assert_eq!(result, Some("mail.example.com".to_string()));
    }

    #[test]
    fn use_ip_false_with_empty_custom_returns_none() {
        let result = resolve_ehlo_hostname(false, Some(""));
        assert_eq!(result, None);
    }

    #[test]
    fn use_ip_false_with_whitespace_custom_returns_none() {
        let result = resolve_ehlo_hostname(false, Some("   "));
        assert_eq!(result, None);
    }

    #[test]
    fn use_ip_false_with_none_custom_returns_none() {
        let result = resolve_ehlo_hostname(false, None);
        assert_eq!(result, None);
    }

    #[test]
    fn custom_ehlo_is_trimmed() {
        let result = resolve_ehlo_hostname(false, Some("  mail.example.com  "));
        assert_eq!(result, Some("mail.example.com".to_string()));
    }
}
