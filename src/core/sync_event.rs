//! Typed events emitted by the sync engine over a broadcast channel.

/// Events emitted by the sync engine.
#[derive(Debug, Clone)]
pub enum SyncEvent {
    /// A message's flags changed on the server successfully.
    MessageFlagsChanged {
        account_id: String,
        message_id: i64,
        new_flags: u32,
    },
    /// A pending operation failed permanently.
    OperationFailed {
        account_id: String,
        operation_id: i64,
        error: String,
    },
    /// A server-side flag change was detected during incremental sync.
    ServerFlagChange {
        account_id: String,
        message_id: i64,
        new_flags: u32,
    },
    /// New mail was received via IDLE push notification.
    NewMailReceived {
        account_id: String,
        folder_name: String,
        bodies_fetched: usize,
    },
    /// A message was successfully sent via SMTP.
    MessageSent {
        account_id: String,
        operation_id: i64,
    },
    /// A message was successfully moved between folders on the server.
    MessageMoved {
        account_id: String,
        message_id: i64,
        source_folder: String,
        destination_folder: String,
        new_uid: Option<u32>,
    },
    /// A message was successfully copied to another folder on the server.
    MessageCopied {
        account_id: String,
        message_id: i64,
        source_folder: String,
        destination_folder: String,
        new_uid: Option<u32>,
    },
    /// The folder list for an account has changed (create, rename, or delete).
    FolderListChanged { account_id: String },
    /// A message was permanently deleted (expunged) from the server.
    MessageExpunged {
        account_id: String,
        message_id: i64,
        folder_name: String,
    },
    /// Messages were removed from the server (detected during sync).
    MessagesRemoved {
        account_id: String,
        folder_name: String,
        count: usize,
    },
    /// An operation was cancelled because the target message no longer exists on the server.
    OperationVanished {
        account_id: String,
        operation_id: i64,
        message_id: i64,
        error: String,
    },
}
