//! Display-ready summaries of pending operations for the queue view (AC-16)
//! and message-level pending status detection (AC-12).

use std::collections::HashSet;

use crate::core::pending_operation::{
    CopyMessagePayload, DeleteMessagePayload, FolderCreatePayload, FolderDeletePayload,
    FolderRenamePayload, MoveMessagePayload, OperationKind, OperationState, PendingOperation,
    StoreFlagsPayload,
};

/// A display-ready summary of a single pending operation (AC-16).
#[derive(Debug, Clone)]
pub struct PendingOperationSummary {
    pub id: i64,
    pub account_id: String,
    pub operation_type: String,
    pub target: String,
    pub status: String,
    pub error: Option<String>,
    pub created_at: i64,
}

/// Build a display-ready summary from a raw `PendingOperation`.
pub fn summarize_operation(op: &PendingOperation) -> PendingOperationSummary {
    let operation_type = friendly_kind_label(&op.kind);
    let target = extract_target(&op.kind, &op.payload);
    let status = friendly_state_label(&op.state);

    PendingOperationSummary {
        id: op.id,
        account_id: op.account_id.clone(),
        operation_type,
        target,
        status,
        error: op.last_error.clone(),
        created_at: op.created_at,
    }
}

/// Collect the set of message IDs that have at least one pending or in-flight
/// operation. Used by the message list UI to visually distinguish unconfirmed
/// messages (AC-12).
pub fn message_ids_with_pending_ops(ops: &[PendingOperation]) -> HashSet<i64> {
    let mut ids = HashSet::new();
    for op in ops {
        if op.state == OperationState::Pending || op.state == OperationState::InFlight {
            if let Some(mid) = extract_message_id(&op.kind, &op.payload) {
                ids.insert(mid);
            }
        }
    }
    ids
}

/// Human-readable label for the operation kind.
fn friendly_kind_label(kind: &OperationKind) -> String {
    match kind {
        OperationKind::StoreFlags => "Update flags".to_string(),
        OperationKind::MoveMessage => "Move message".to_string(),
        OperationKind::CopyMessage => "Copy message".to_string(),
        OperationKind::DeleteMessage => "Delete message".to_string(),
        OperationKind::Send => "Send message".to_string(),
        OperationKind::FolderCreate => "Create folder".to_string(),
        OperationKind::FolderRename => "Rename folder".to_string(),
        OperationKind::FolderDelete => "Delete folder".to_string(),
    }
}

/// Human-readable label for the operation state.
fn friendly_state_label(state: &OperationState) -> String {
    match state {
        OperationState::Pending => "Queued".to_string(),
        OperationState::InFlight => "In progress".to_string(),
        OperationState::Failed => "Failed".to_string(),
    }
}

/// Extract a human-readable target description from the JSON payload.
fn extract_target(kind: &OperationKind, payload: &str) -> String {
    match kind {
        OperationKind::StoreFlags => {
            if let Ok(p) = serde_json::from_str::<StoreFlagsPayload>(payload) {
                return format!("Message {} in {}", p.message_id, p.folder_name);
            }
        }
        OperationKind::MoveMessage => {
            if let Ok(p) = serde_json::from_str::<MoveMessagePayload>(payload) {
                return format!("Message {} → {}", p.message_id, p.destination_folder);
            }
        }
        OperationKind::CopyMessage => {
            if let Ok(p) = serde_json::from_str::<CopyMessagePayload>(payload) {
                return format!("Message {} → {}", p.message_id, p.destination_folder);
            }
        }
        OperationKind::DeleteMessage => {
            if let Ok(p) = serde_json::from_str::<DeleteMessagePayload>(payload) {
                return format!("Message {} in {}", p.message_id, p.folder_name);
            }
        }
        OperationKind::Send => {
            return "Outgoing message".to_string();
        }
        OperationKind::FolderCreate => {
            if let Ok(p) = serde_json::from_str::<FolderCreatePayload>(payload) {
                return format!("Folder \"{}\"", p.folder_name);
            }
        }
        OperationKind::FolderRename => {
            if let Ok(p) = serde_json::from_str::<FolderRenamePayload>(payload) {
                return format!("\"{}\" → \"{}\"", p.old_name, p.new_name);
            }
        }
        OperationKind::FolderDelete => {
            if let Ok(p) = serde_json::from_str::<FolderDeletePayload>(payload) {
                return format!("Folder \"{}\"", p.folder_name);
            }
        }
    }
    "Unknown target".to_string()
}

/// Extract the message_id from a payload, if the operation kind targets a message.
fn extract_message_id(kind: &OperationKind, payload: &str) -> Option<i64> {
    match kind {
        OperationKind::StoreFlags => serde_json::from_str::<StoreFlagsPayload>(payload)
            .ok()
            .map(|p| p.message_id),
        OperationKind::MoveMessage => serde_json::from_str::<MoveMessagePayload>(payload)
            .ok()
            .map(|p| p.message_id),
        OperationKind::CopyMessage => serde_json::from_str::<CopyMessagePayload>(payload)
            .ok()
            .map(|p| p.message_id),
        OperationKind::DeleteMessage => serde_json::from_str::<DeleteMessagePayload>(payload)
            .ok()
            .map(|p| p.message_id),
        OperationKind::Send
        | OperationKind::FolderCreate
        | OperationKind::FolderRename
        | OperationKind::FolderDelete => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_op(kind: OperationKind, payload: &str, state: OperationState) -> PendingOperation {
        PendingOperation {
            id: 1,
            account_id: "acct-1".to_string(),
            kind,
            payload: payload.to_string(),
            state,
            retry_count: 0,
            last_error: None,
            created_at: 1000,
            next_retry_at: None,
        }
    }

    #[test]
    fn summarize_store_flags_operation() {
        let payload = r#"{"message_id":42,"uid":100,"folder_name":"INBOX","new_flags":1}"#;
        let op = make_op(OperationKind::StoreFlags, payload, OperationState::Pending);
        let summary = summarize_operation(&op);
        assert_eq!(summary.operation_type, "Update flags");
        assert_eq!(summary.target, "Message 42 in INBOX");
        assert_eq!(summary.status, "Queued");
        assert!(summary.error.is_none());
    }

    #[test]
    fn summarize_move_message_operation() {
        let payload =
            r#"{"message_id":10,"uid":200,"source_folder":"INBOX","destination_folder":"Archive"}"#;
        let op = make_op(
            OperationKind::MoveMessage,
            payload,
            OperationState::InFlight,
        );
        let summary = summarize_operation(&op);
        assert_eq!(summary.operation_type, "Move message");
        assert!(summary.target.contains("Archive"));
        assert_eq!(summary.status, "In progress");
    }

    #[test]
    fn summarize_failed_operation_includes_error() {
        let payload = r#"{"message_id":5,"uid":300,"folder_name":"Trash"}"#;
        let mut op = make_op(
            OperationKind::DeleteMessage,
            payload,
            OperationState::Failed,
        );
        op.last_error = Some("Permission denied".to_string());
        let summary = summarize_operation(&op);
        assert_eq!(summary.status, "Failed");
        assert_eq!(summary.error.as_deref(), Some("Permission denied"));
    }

    #[test]
    fn summarize_send_operation() {
        let payload = r#"{"identity_id":1,"content_hash":"abc","inline_rfc822_b64":null}"#;
        let op = make_op(OperationKind::Send, payload, OperationState::Pending);
        let summary = summarize_operation(&op);
        assert_eq!(summary.operation_type, "Send message");
        assert_eq!(summary.target, "Outgoing message");
    }

    #[test]
    fn summarize_folder_create() {
        let payload = r#"{"folder_id":1,"folder_name":"Projects"}"#;
        let op = make_op(
            OperationKind::FolderCreate,
            payload,
            OperationState::Pending,
        );
        let summary = summarize_operation(&op);
        assert_eq!(summary.operation_type, "Create folder");
        assert!(summary.target.contains("Projects"));
    }

    #[test]
    fn summarize_folder_rename() {
        let payload = r#"{"folder_id":1,"old_name":"Old","new_name":"New"}"#;
        let op = make_op(
            OperationKind::FolderRename,
            payload,
            OperationState::Pending,
        );
        let summary = summarize_operation(&op);
        assert_eq!(summary.operation_type, "Rename folder");
        assert!(summary.target.contains("Old"));
        assert!(summary.target.contains("New"));
    }

    #[test]
    fn summarize_folder_delete() {
        let payload = r#"{"folder_id":1,"folder_name":"OldFolder"}"#;
        let op = make_op(
            OperationKind::FolderDelete,
            payload,
            OperationState::Pending,
        );
        let summary = summarize_operation(&op);
        assert_eq!(summary.operation_type, "Delete folder");
        assert!(summary.target.contains("OldFolder"));
    }

    #[test]
    fn summarize_copy_message() {
        let payload =
            r#"{"message_id":7,"uid":50,"source_folder":"INBOX","destination_folder":"Backup"}"#;
        let op = make_op(OperationKind::CopyMessage, payload, OperationState::Pending);
        let summary = summarize_operation(&op);
        assert_eq!(summary.operation_type, "Copy message");
        assert!(summary.target.contains("Backup"));
    }

    #[test]
    fn message_ids_with_pending_ops_collects_pending_and_inflight() {
        let ops = vec![
            make_op(
                OperationKind::StoreFlags,
                r#"{"message_id":1,"uid":10,"folder_name":"INBOX","new_flags":1}"#,
                OperationState::Pending,
            ),
            make_op(
                OperationKind::MoveMessage,
                r#"{"message_id":2,"uid":20,"source_folder":"INBOX","destination_folder":"Archive"}"#,
                OperationState::InFlight,
            ),
            make_op(
                OperationKind::DeleteMessage,
                r#"{"message_id":3,"uid":30,"folder_name":"Trash"}"#,
                OperationState::Failed,
            ),
        ];
        let ids = message_ids_with_pending_ops(&ops);
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
        assert!(
            !ids.contains(&3),
            "failed ops should not mark messages as pending"
        );
    }

    #[test]
    fn message_ids_excludes_folder_operations() {
        let ops = vec![make_op(
            OperationKind::FolderCreate,
            r#"{"folder_id":1,"folder_name":"New"}"#,
            OperationState::Pending,
        )];
        let ids = message_ids_with_pending_ops(&ops);
        assert!(ids.is_empty());
    }

    #[test]
    fn malformed_payload_produces_fallback() {
        let op = make_op(
            OperationKind::StoreFlags,
            "not-json",
            OperationState::Pending,
        );
        let summary = summarize_operation(&op);
        assert_eq!(summary.target, "Unknown target");
    }
}
