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
}
