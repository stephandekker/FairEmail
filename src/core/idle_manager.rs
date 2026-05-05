//! IDLE state management and policy for push delivery.
//!
//! Pure state machine for IMAP IDLE lifecycle per account.
//! UI-free and testable without network connections.

use std::time::Duration;

/// IDLE renewal interval — must be < 29 minutes per RFC 2177.
pub const IDLE_RENEWAL_SECS: u64 = 25 * 60;

/// Default polling interval for servers without IDLE capability.
pub const DEFAULT_POLL_INTERVAL_SECS: u64 = 5 * 60;

/// Backoff durations for reconnection attempts (seconds).
const RECONNECT_BACKOFF_SECS: &[u64] = &[5, 15, 30, 60, 300, 600];

/// State of the IDLE connection for one account's inbox.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdleState {
    /// Not started yet.
    Initial,
    /// IDLE is active on the inbox.
    Idling,
    /// IDLE was interrupted; performing incremental sync.
    Syncing,
    /// Disconnected; waiting to reconnect.
    Disconnected { attempt: u32 },
    /// Polling mode (server lacks IDLE).
    Polling,
    /// Shut down.
    Stopped,
}

/// Events that drive the IDLE state machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdleEvent {
    /// Initial sync completed; ready to start IDLE or polling.
    InitialSyncDone { idle_supported: bool },
    /// Server sent new-mail notification (EXISTS).
    NewMail,
    /// Server sent flag change or expunge.
    FlagChange,
    /// Renewal timer fired (25 min).
    RenewalTimeout,
    /// Poll timer fired.
    PollTimeout,
    /// Network error or disconnect.
    Disconnected,
    /// Reconnect attempt succeeded.
    Reconnected { idle_supported: bool },
    /// Incremental sync completed.
    SyncCompleted,
    /// Shutdown requested.
    Shutdown,
}

/// Action to take after a state transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdleAction {
    /// Enter IDLE on the inbox.
    EnterIdle,
    /// Exit IDLE (DONE) and re-enter.
    RenewIdle,
    /// Trigger an incremental sync of the inbox.
    TriggerSync,
    /// Wait before reconnecting.
    ReconnectAfter(Duration),
    /// Start polling at default interval.
    StartPolling,
    /// Poll now (timer fired).
    PollNow,
    /// No action needed.
    None,
    /// Stop the IDLE manager.
    Stop,
}

/// Advance the state machine given the current state and an event.
pub fn next_state(state: &IdleState, event: &IdleEvent) -> (IdleState, IdleAction) {
    match (state, event) {
        // Initial → decide IDLE or polling.
        (
            IdleState::Initial,
            IdleEvent::InitialSyncDone {
                idle_supported: true,
            },
        ) => (IdleState::Idling, IdleAction::EnterIdle),
        (
            IdleState::Initial,
            IdleEvent::InitialSyncDone {
                idle_supported: false,
            },
        ) => (IdleState::Polling, IdleAction::StartPolling),

        // Idling → new mail or flag change → sync.
        (IdleState::Idling, IdleEvent::NewMail | IdleEvent::FlagChange) => {
            (IdleState::Syncing, IdleAction::TriggerSync)
        }

        // Idling → renewal timeout → renew.
        (IdleState::Idling, IdleEvent::RenewalTimeout) => {
            (IdleState::Idling, IdleAction::RenewIdle)
        }

        // Idling → disconnect → reconnect.
        (IdleState::Idling, IdleEvent::Disconnected) => {
            let delay = reconnect_backoff(1);
            (
                IdleState::Disconnected { attempt: 1 },
                IdleAction::ReconnectAfter(delay),
            )
        }

        // Syncing → done → re-enter IDLE.
        (IdleState::Syncing, IdleEvent::SyncCompleted) => {
            (IdleState::Idling, IdleAction::EnterIdle)
        }

        // Syncing → disconnect → reconnect.
        (IdleState::Syncing, IdleEvent::Disconnected) => {
            let delay = reconnect_backoff(1);
            (
                IdleState::Disconnected { attempt: 1 },
                IdleAction::ReconnectAfter(delay),
            )
        }

        // Disconnected → reconnect succeeded → enter IDLE or poll.
        (
            IdleState::Disconnected { .. },
            IdleEvent::Reconnected {
                idle_supported: true,
            },
        ) => (IdleState::Idling, IdleAction::EnterIdle),
        (
            IdleState::Disconnected { .. },
            IdleEvent::Reconnected {
                idle_supported: false,
            },
        ) => (IdleState::Polling, IdleAction::StartPolling),

        // Disconnected → still failing → retry with increasing backoff.
        (IdleState::Disconnected { attempt }, IdleEvent::Disconnected) => {
            let next = attempt + 1;
            let delay = reconnect_backoff(next);
            (
                IdleState::Disconnected { attempt: next },
                IdleAction::ReconnectAfter(delay),
            )
        }

        // Polling → poll timer → do a sync.
        (IdleState::Polling, IdleEvent::PollTimeout) => (IdleState::Polling, IdleAction::PollNow),

        // Polling → disconnect during poll → reconnect.
        (IdleState::Polling, IdleEvent::Disconnected) => {
            let delay = reconnect_backoff(1);
            (
                IdleState::Disconnected { attempt: 1 },
                IdleAction::ReconnectAfter(delay),
            )
        }

        // Any state → shutdown.
        (_, IdleEvent::Shutdown) => (IdleState::Stopped, IdleAction::Stop),

        // Default: no-op.
        _ => (state.clone(), IdleAction::None),
    }
}

/// Compute reconnect backoff for a given attempt number (1-based).
pub fn reconnect_backoff(attempt: u32) -> Duration {
    let idx = (attempt as usize)
        .saturating_sub(1)
        .min(RECONNECT_BACKOFF_SECS.len() - 1);
    Duration::from_secs(RECONNECT_BACKOFF_SECS[idx])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_idle_supported_enters_idle() {
        let (state, action) = next_state(
            &IdleState::Initial,
            &IdleEvent::InitialSyncDone {
                idle_supported: true,
            },
        );
        assert_eq!(state, IdleState::Idling);
        assert_eq!(action, IdleAction::EnterIdle);
    }

    #[test]
    fn initial_no_idle_starts_polling() {
        let (state, action) = next_state(
            &IdleState::Initial,
            &IdleEvent::InitialSyncDone {
                idle_supported: false,
            },
        );
        assert_eq!(state, IdleState::Polling);
        assert_eq!(action, IdleAction::StartPolling);
    }

    #[test]
    fn idling_new_mail_triggers_sync() {
        let (state, action) = next_state(&IdleState::Idling, &IdleEvent::NewMail);
        assert_eq!(state, IdleState::Syncing);
        assert_eq!(action, IdleAction::TriggerSync);
    }

    #[test]
    fn idling_flag_change_triggers_sync() {
        let (state, action) = next_state(&IdleState::Idling, &IdleEvent::FlagChange);
        assert_eq!(state, IdleState::Syncing);
        assert_eq!(action, IdleAction::TriggerSync);
    }

    #[test]
    fn idling_renewal_timeout_renews() {
        let (state, action) = next_state(&IdleState::Idling, &IdleEvent::RenewalTimeout);
        assert_eq!(state, IdleState::Idling);
        assert_eq!(action, IdleAction::RenewIdle);
    }

    #[test]
    fn idling_disconnect_starts_reconnect() {
        let (state, action) = next_state(&IdleState::Idling, &IdleEvent::Disconnected);
        assert_eq!(state, IdleState::Disconnected { attempt: 1 });
        assert_eq!(action, IdleAction::ReconnectAfter(Duration::from_secs(5)));
    }

    #[test]
    fn syncing_complete_re_enters_idle() {
        let (state, action) = next_state(&IdleState::Syncing, &IdleEvent::SyncCompleted);
        assert_eq!(state, IdleState::Idling);
        assert_eq!(action, IdleAction::EnterIdle);
    }

    #[test]
    fn syncing_disconnect_starts_reconnect() {
        let (state, action) = next_state(&IdleState::Syncing, &IdleEvent::Disconnected);
        assert_eq!(state, IdleState::Disconnected { attempt: 1 });
        assert_eq!(action, IdleAction::ReconnectAfter(Duration::from_secs(5)));
    }

    #[test]
    fn disconnected_reconnect_success_enters_idle() {
        let (state, action) = next_state(
            &IdleState::Disconnected { attempt: 2 },
            &IdleEvent::Reconnected {
                idle_supported: true,
            },
        );
        assert_eq!(state, IdleState::Idling);
        assert_eq!(action, IdleAction::EnterIdle);
    }

    #[test]
    fn disconnected_reconnect_no_idle_starts_polling() {
        let (state, action) = next_state(
            &IdleState::Disconnected { attempt: 1 },
            &IdleEvent::Reconnected {
                idle_supported: false,
            },
        );
        assert_eq!(state, IdleState::Polling);
        assert_eq!(action, IdleAction::StartPolling);
    }

    #[test]
    fn disconnected_still_failing_increases_backoff() {
        let (state, action) = next_state(
            &IdleState::Disconnected { attempt: 1 },
            &IdleEvent::Disconnected,
        );
        assert_eq!(state, IdleState::Disconnected { attempt: 2 });
        assert_eq!(action, IdleAction::ReconnectAfter(Duration::from_secs(15)));

        let (state, action) = next_state(&state, &IdleEvent::Disconnected);
        assert_eq!(state, IdleState::Disconnected { attempt: 3 });
        assert_eq!(action, IdleAction::ReconnectAfter(Duration::from_secs(30)));
    }

    #[test]
    fn polling_timeout_polls_now() {
        let (state, action) = next_state(&IdleState::Polling, &IdleEvent::PollTimeout);
        assert_eq!(state, IdleState::Polling);
        assert_eq!(action, IdleAction::PollNow);
    }

    #[test]
    fn polling_disconnect_starts_reconnect() {
        let (state, action) = next_state(&IdleState::Polling, &IdleEvent::Disconnected);
        assert_eq!(state, IdleState::Disconnected { attempt: 1 });
        assert_eq!(action, IdleAction::ReconnectAfter(Duration::from_secs(5)));
    }

    #[test]
    fn any_state_shutdown_stops() {
        for state in &[
            IdleState::Initial,
            IdleState::Idling,
            IdleState::Syncing,
            IdleState::Disconnected { attempt: 3 },
            IdleState::Polling,
        ] {
            let (new_state, action) = next_state(state, &IdleEvent::Shutdown);
            assert_eq!(new_state, IdleState::Stopped);
            assert_eq!(action, IdleAction::Stop);
        }
    }

    #[test]
    fn reconnect_backoff_caps_at_max() {
        assert_eq!(reconnect_backoff(1), Duration::from_secs(5));
        assert_eq!(reconnect_backoff(2), Duration::from_secs(15));
        assert_eq!(reconnect_backoff(3), Duration::from_secs(30));
        assert_eq!(reconnect_backoff(4), Duration::from_secs(60));
        assert_eq!(reconnect_backoff(5), Duration::from_secs(300));
        assert_eq!(reconnect_backoff(6), Duration::from_secs(600));
        assert_eq!(reconnect_backoff(100), Duration::from_secs(600));
    }

    #[test]
    fn idle_renewal_under_29_minutes() {
        // Verify at runtime that the constant respects the RFC 2177 29-minute limit.
        let renewal = IDLE_RENEWAL_SECS;
        let limit = 29 * 60;
        assert!(
            renewal < limit,
            "IDLE_RENEWAL_SECS ({renewal}) must be < {limit}"
        );
    }

    #[test]
    fn full_idle_lifecycle() {
        // Start → IDLE → new mail → sync → back to IDLE → disconnect → reconnect → IDLE
        let (s, a) = next_state(
            &IdleState::Initial,
            &IdleEvent::InitialSyncDone {
                idle_supported: true,
            },
        );
        assert_eq!(s, IdleState::Idling);
        assert_eq!(a, IdleAction::EnterIdle);

        let (s, a) = next_state(&s, &IdleEvent::NewMail);
        assert_eq!(s, IdleState::Syncing);
        assert_eq!(a, IdleAction::TriggerSync);

        let (s, a) = next_state(&s, &IdleEvent::SyncCompleted);
        assert_eq!(s, IdleState::Idling);
        assert_eq!(a, IdleAction::EnterIdle);

        let (s, a) = next_state(&s, &IdleEvent::Disconnected);
        assert_eq!(s, IdleState::Disconnected { attempt: 1 });
        assert_eq!(a, IdleAction::ReconnectAfter(Duration::from_secs(5)));

        let (s, a) = next_state(
            &s,
            &IdleEvent::Reconnected {
                idle_supported: true,
            },
        );
        assert_eq!(s, IdleState::Idling);
        assert_eq!(a, IdleAction::EnterIdle);

        let (s, a) = next_state(&s, &IdleEvent::RenewalTimeout);
        assert_eq!(s, IdleState::Idling);
        assert_eq!(a, IdleAction::RenewIdle);

        let (s, a) = next_state(&s, &IdleEvent::Shutdown);
        assert_eq!(s, IdleState::Stopped);
        assert_eq!(a, IdleAction::Stop);
    }
}
