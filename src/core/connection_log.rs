/// Event types for the connection log table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionLogEventType {
    /// TCP connection attempt initiated.
    ConnectAttempt,
    /// TLS handshake result.
    TlsHandshake,
    /// Login/authentication result.
    LoginResult,
    /// Server capability list received.
    CapabilityList,
    /// Folder listing completed.
    ListFolders,
    /// An error occurred.
    Error,
    /// IDLE mode entered on a folder.
    IdleEnter,
    /// IDLE mode exited (renewal, notification, or error).
    IdleExit,
    /// Reconnecting after a disconnect.
    Reconnect,
    /// Authentication mechanism negotiation details.
    AuthNegotiation,
}

impl ConnectionLogEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ConnectAttempt => "connect",
            Self::TlsHandshake => "tls",
            Self::LoginResult => "login",
            Self::CapabilityList => "capability",
            Self::ListFolders => "list_folders",
            Self::Error => "error",
            Self::IdleEnter => "idle_enter",
            Self::IdleExit => "idle_exit",
            Self::Reconnect => "reconnect",
            Self::AuthNegotiation => "auth_negotiation",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "connect" => Some(Self::ConnectAttempt),
            "tls" => Some(Self::TlsHandshake),
            "login" => Some(Self::LoginResult),
            "capability" => Some(Self::CapabilityList),
            "list_folders" => Some(Self::ListFolders),
            "error" => Some(Self::Error),
            "idle_enter" => Some(Self::IdleEnter),
            "idle_exit" => Some(Self::IdleExit),
            "reconnect" => Some(Self::Reconnect),
            "auth_negotiation" => Some(Self::AuthNegotiation),
            _ => None,
        }
    }
}

impl std::fmt::Display for ConnectionLogEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectAttempt => write!(f, "Connect"),
            Self::TlsHandshake => write!(f, "TLS"),
            Self::LoginResult => write!(f, "Login"),
            Self::CapabilityList => write!(f, "Capabilities"),
            Self::ListFolders => write!(f, "List Folders"),
            Self::Error => write!(f, "Error"),
            Self::IdleEnter => write!(f, "IDLE Enter"),
            Self::IdleExit => write!(f, "IDLE Exit"),
            Self::Reconnect => write!(f, "Reconnect"),
            Self::AuthNegotiation => write!(f, "Auth Negotiation"),
        }
    }
}

/// A single row in the connection_log table.
#[derive(Debug, Clone)]
pub struct ConnectionLogRecord {
    pub id: Option<i64>,
    pub account_id: String,
    pub timestamp_secs: u64,
    pub event_type: ConnectionLogEventType,
    pub message: String,
}

impl ConnectionLogRecord {
    pub fn new(account_id: String, event_type: ConnectionLogEventType, message: String) -> Self {
        let timestamp_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            id: None,
            account_id,
            timestamp_secs,
            event_type,
            message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_type_roundtrip() {
        let types = [
            ConnectionLogEventType::ConnectAttempt,
            ConnectionLogEventType::TlsHandshake,
            ConnectionLogEventType::LoginResult,
            ConnectionLogEventType::CapabilityList,
            ConnectionLogEventType::ListFolders,
            ConnectionLogEventType::Error,
            ConnectionLogEventType::IdleEnter,
            ConnectionLogEventType::IdleExit,
            ConnectionLogEventType::Reconnect,
            ConnectionLogEventType::AuthNegotiation,
        ];
        for t in types {
            let s = t.as_str();
            let parsed = ConnectionLogEventType::parse(s).unwrap();
            assert_eq!(parsed, t);
        }
    }

    #[test]
    fn unknown_event_type_returns_none() {
        assert!(ConnectionLogEventType::parse("unknown").is_none());
    }

    #[test]
    fn new_record_has_timestamp() {
        let record = ConnectionLogRecord::new(
            "acct-1".to_string(),
            ConnectionLogEventType::ConnectAttempt,
            "Connecting to imap.example.com:993".to_string(),
        );
        assert!(record.timestamp_secs > 0);
        assert!(record.id.is_none());
        assert_eq!(record.account_id, "acct-1");
    }
}
