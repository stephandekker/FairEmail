use serde::{Deserialize, Serialize};

/// Per-account capability cache stored in the `sync_state` table.
/// Populated after a successful IMAP connection test (FR-22, US-18, US-19, US-20).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncState {
    /// Account ID this state belongs to.
    pub account_id: String,
    /// Whether the server advertises IDLE support.
    pub idle_supported: bool,
    /// Whether the server advertises CONDSTORE.
    pub condstore_supported: bool,
    /// Whether the server advertises QRESYNC.
    pub qresync_supported: bool,
    /// Whether the server advertises UTF8=ACCEPT.
    pub utf8_accept: bool,
    /// Advertised maximum message size (APPENDLIMIT), if any.
    pub max_message_size: Option<u64>,
    /// Space-separated list of auth mechanisms advertised.
    pub auth_mechanisms: String,
    /// Raw capability string as reported by the server.
    pub capabilities_raw: String,
    /// When this cache was last updated (UNIX epoch seconds).
    pub updated_at: u64,
}

impl SyncState {
    /// Parse capabilities from a raw capability list (space-separated tokens).
    pub fn from_capabilities(account_id: String, capabilities: &[String]) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let caps_upper: Vec<String> = capabilities.iter().map(|c| c.to_uppercase()).collect();

        let idle_supported = caps_upper.iter().any(|c| c == "IDLE");
        let condstore_supported = caps_upper.iter().any(|c| c == "CONDSTORE");
        let qresync_supported = caps_upper.iter().any(|c| c == "QRESYNC");
        let utf8_accept = caps_upper.iter().any(|c| c == "UTF8=ACCEPT");

        let max_message_size = caps_upper.iter().find_map(|c| {
            c.strip_prefix("APPENDLIMIT=")
                .and_then(|v| v.parse::<u64>().ok())
        });

        let auth_mechanisms: Vec<&str> = caps_upper
            .iter()
            .filter_map(|c| c.strip_prefix("AUTH="))
            .collect();

        Self {
            account_id,
            idle_supported,
            condstore_supported,
            qresync_supported,
            utf8_accept,
            max_message_size,
            auth_mechanisms: auth_mechanisms.join(" "),
            capabilities_raw: capabilities.join(" "),
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_idle_and_condstore() {
        let caps = vec![
            "IMAP4rev1".to_string(),
            "IDLE".to_string(),
            "CONDSTORE".to_string(),
            "NAMESPACE".to_string(),
        ];
        let state = SyncState::from_capabilities("test-id".to_string(), &caps);
        assert!(state.idle_supported);
        assert!(state.condstore_supported);
        assert!(!state.qresync_supported);
        assert!(!state.utf8_accept);
        assert!(state.max_message_size.is_none());
    }

    #[test]
    fn parses_qresync_and_utf8() {
        let caps = vec![
            "IMAP4rev1".to_string(),
            "QRESYNC".to_string(),
            "UTF8=ACCEPT".to_string(),
        ];
        let state = SyncState::from_capabilities("test-id".to_string(), &caps);
        assert!(state.qresync_supported);
        assert!(state.utf8_accept);
    }

    #[test]
    fn parses_appendlimit() {
        let caps = vec!["IMAP4rev1".to_string(), "APPENDLIMIT=52428800".to_string()];
        let state = SyncState::from_capabilities("test-id".to_string(), &caps);
        assert_eq!(state.max_message_size, Some(52428800));
    }

    #[test]
    fn parses_auth_mechanisms() {
        let caps = vec![
            "IMAP4rev1".to_string(),
            "AUTH=PLAIN".to_string(),
            "AUTH=LOGIN".to_string(),
            "AUTH=XOAUTH2".to_string(),
        ];
        let state = SyncState::from_capabilities("test-id".to_string(), &caps);
        assert_eq!(state.auth_mechanisms, "PLAIN LOGIN XOAUTH2");
    }

    #[test]
    fn case_insensitive_parsing() {
        let caps = vec!["idle".to_string(), "condstore".to_string()];
        let state = SyncState::from_capabilities("test-id".to_string(), &caps);
        assert!(state.idle_supported);
        assert!(state.condstore_supported);
    }

    #[test]
    fn empty_capabilities() {
        let caps: Vec<String> = vec![];
        let state = SyncState::from_capabilities("test-id".to_string(), &caps);
        assert!(!state.idle_supported);
        assert!(!state.condstore_supported);
        assert!(!state.qresync_supported);
        assert!(!state.utf8_accept);
        assert!(state.max_message_size.is_none());
        assert!(state.auth_mechanisms.is_empty());
    }
}
