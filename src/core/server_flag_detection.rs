//! Server flag change detection logic.
//!
//! Pure business logic for deciding whether a server-side flag change should
//! be applied to local state. The key rule: if the message has pending local
//! flag changes (`flags_pending_sync`), the server update is skipped to avoid
//! overwriting the user's intent. Conflict resolution is handled by story 5.

use crate::core::sync_event::SyncEvent;

/// Outcome of comparing server flags against local state for one message.
#[derive(Debug, PartialEq, Eq)]
pub enum FlagChangeAction {
    /// Server flags differ and should be applied locally.
    Apply { new_flags: u32 },
    /// No change needed (flags already match).
    NoChange,
    /// Skipped because a local flag operation is pending for this message.
    SkippedPendingSync,
}

/// Decide whether a server-side flag change should be applied to local state.
///
/// - `local_flags`: the flags currently stored in the local database.
/// - `server_flags`: the flags reported by the IMAP server.
/// - `flags_pending_sync`: whether the message has a pending local flag change.
pub fn detect_flag_change(
    local_flags: u32,
    server_flags: u32,
    flags_pending_sync: bool,
) -> FlagChangeAction {
    if server_flags == local_flags {
        return FlagChangeAction::NoChange;
    }
    if flags_pending_sync {
        return FlagChangeAction::SkippedPendingSync;
    }
    FlagChangeAction::Apply {
        new_flags: server_flags,
    }
}

/// Build a `ServerFlagChange` event for a detected flag change.
pub fn make_flag_change_event(account_id: &str, message_id: i64, new_flags: u32) -> SyncEvent {
    SyncEvent::ServerFlagChange {
        account_id: account_id.to_string(),
        message_id,
        new_flags,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::message::{FLAG_ANSWERED, FLAG_DELETED, FLAG_DRAFT, FLAG_FLAGGED, FLAG_SEEN};

    #[test]
    fn no_change_when_flags_match() {
        let action = detect_flag_change(FLAG_SEEN, FLAG_SEEN, false);
        assert_eq!(action, FlagChangeAction::NoChange);
    }

    #[test]
    fn apply_when_flags_differ_and_no_pending() {
        let action = detect_flag_change(0, FLAG_SEEN, false);
        assert_eq!(
            action,
            FlagChangeAction::Apply {
                new_flags: FLAG_SEEN
            }
        );
    }

    #[test]
    fn skip_when_pending_sync() {
        let action = detect_flag_change(0, FLAG_SEEN, true);
        assert_eq!(action, FlagChangeAction::SkippedPendingSync);
    }

    #[test]
    fn skip_when_pending_sync_even_if_flags_match() {
        // Edge case: flags happen to match but pending_sync is set.
        // This returns NoChange because flags match — pending_sync check
        // only matters when flags actually differ.
        let action = detect_flag_change(FLAG_SEEN, FLAG_SEEN, true);
        assert_eq!(action, FlagChangeAction::NoChange);
    }

    #[test]
    fn detects_all_standard_flags() {
        for flag in [
            FLAG_SEEN,
            FLAG_FLAGGED,
            FLAG_ANSWERED,
            FLAG_DELETED,
            FLAG_DRAFT,
        ] {
            let action = detect_flag_change(0, flag, false);
            assert_eq!(action, FlagChangeAction::Apply { new_flags: flag });
        }
    }

    #[test]
    fn detects_flag_removal() {
        let action = detect_flag_change(FLAG_SEEN | FLAG_FLAGGED, FLAG_SEEN, false);
        assert_eq!(
            action,
            FlagChangeAction::Apply {
                new_flags: FLAG_SEEN
            }
        );
    }

    #[test]
    fn detects_multiple_flag_changes() {
        let server = FLAG_SEEN | FLAG_FLAGGED | FLAG_ANSWERED;
        let action = detect_flag_change(FLAG_SEEN, server, false);
        assert_eq!(action, FlagChangeAction::Apply { new_flags: server });
    }

    #[test]
    fn make_event_produces_correct_variant() {
        let event = make_flag_change_event("acct-1", 42, FLAG_SEEN);
        match event {
            SyncEvent::ServerFlagChange {
                account_id,
                message_id,
                new_flags,
            } => {
                assert_eq!(account_id, "acct-1");
                assert_eq!(message_id, 42);
                assert_eq!(new_flags, FLAG_SEEN);
            }
            _ => panic!("expected ServerFlagChange"),
        }
    }
}
