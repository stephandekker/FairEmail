//! Pending operation domain model for offline-first sync.

use serde::{Deserialize, Serialize};

/// The kind of pending operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationKind {
    /// STORE flags (mark read/unread).
    StoreFlags,
    /// Send a message via SMTP.
    Send,
    /// Create a folder on the IMAP server.
    FolderCreate,
    /// Rename a folder on the IMAP server.
    FolderRename,
    /// Delete a folder on the IMAP server.
    FolderDelete,
}

impl OperationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            OperationKind::StoreFlags => "store_flags",
            OperationKind::Send => "send",
            OperationKind::FolderCreate => "folder-create",
            OperationKind::FolderRename => "folder-rename",
            OperationKind::FolderDelete => "folder-delete",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "store_flags" => Some(OperationKind::StoreFlags),
            "send" => Some(OperationKind::Send),
            "folder-create" => Some(OperationKind::FolderCreate),
            "folder-rename" => Some(OperationKind::FolderRename),
            "folder-delete" => Some(OperationKind::FolderDelete),
            _ => None,
        }
    }
}

/// State of a pending operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationState {
    Pending,
    InFlight,
    Failed,
}

impl OperationState {
    pub fn as_str(&self) -> &'static str {
        match self {
            OperationState::Pending => "pending",
            OperationState::InFlight => "in_flight",
            OperationState::Failed => "failed",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(OperationState::Pending),
            "in_flight" => Some(OperationState::InFlight),
            "failed" => Some(OperationState::Failed),
            _ => None,
        }
    }
}

/// Payload for a STORE flags operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreFlagsPayload {
    pub message_id: i64,
    pub uid: u32,
    pub folder_name: String,
    pub new_flags: u32,
}

/// Payload for a send-message operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendPayload {
    /// The identity to send from.
    pub identity_id: i64,
    /// Content-store hash of the composed message bytes, if already stored.
    pub content_hash: Option<String>,
    /// Inline RFC 5322 message bytes (base64-encoded), used when the draft
    /// was not previously persisted in the content store.
    pub inline_rfc822_b64: Option<String>,
}

/// Payload for a folder-create operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderCreatePayload {
    pub folder_id: i64,
    pub folder_name: String,
}

/// Payload for a folder-rename operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderRenamePayload {
    pub folder_id: i64,
    pub old_name: String,
    pub new_name: String,
}

/// Payload for a folder-delete operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderDeletePayload {
    pub folder_id: i64,
    pub folder_name: String,
}

/// A row from the `pending_operations` table.
#[derive(Debug, Clone)]
pub struct PendingOperation {
    pub id: i64,
    pub account_id: String,
    pub kind: OperationKind,
    pub payload: String,
    pub state: OperationState,
    pub retry_count: i32,
    pub last_error: Option<String>,
    pub created_at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_kind_roundtrips() {
        let kind = OperationKind::StoreFlags;
        assert_eq!(OperationKind::parse(kind.as_str()), Some(kind));
    }

    #[test]
    fn operation_state_roundtrips() {
        for state in [
            OperationState::Pending,
            OperationState::InFlight,
            OperationState::Failed,
        ] {
            assert_eq!(OperationState::parse(state.as_str()), Some(state.clone()));
        }
    }

    #[test]
    fn store_flags_payload_serializes() {
        let payload = StoreFlagsPayload {
            message_id: 42,
            uid: 100,
            folder_name: "INBOX".to_string(),
            new_flags: 1,
        };
        let json = serde_json::to_string(&payload).unwrap();
        let parsed: StoreFlagsPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.message_id, 42);
        assert_eq!(parsed.uid, 100);
        assert_eq!(parsed.new_flags, 1);
    }
}
