//! UI-free connectivity state tracker for offline queue replay.
//!
//! Tracks online/offline transitions and determines when the operation
//! queue should be replayed (i.e. on an offline-to-online transition).

/// Connectivity state observed by the monitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectivityState {
    Online,
    Offline,
}

/// Tracks connectivity transitions and signals when replay is needed.
#[derive(Debug)]
pub struct ConnectivityTracker {
    previous: ConnectivityState,
}

impl ConnectivityTracker {
    /// Create a new tracker with the given initial state.
    pub fn new(initial: ConnectivityState) -> Self {
        Self { previous: initial }
    }

    /// Record a connectivity change. Returns `true` if this is an
    /// offline-to-online transition (i.e. replay should be triggered).
    pub fn update(&mut self, current: ConnectivityState) -> bool {
        let should_replay =
            self.previous == ConnectivityState::Offline && current == ConnectivityState::Online;
        self.previous = current;
        should_replay
    }

    /// Current known state.
    #[cfg(test)]
    pub fn state(&self) -> ConnectivityState {
        self.previous
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn online_to_online_no_replay() {
        let mut tracker = ConnectivityTracker::new(ConnectivityState::Online);
        assert!(!tracker.update(ConnectivityState::Online));
    }

    #[test]
    fn online_to_offline_no_replay() {
        let mut tracker = ConnectivityTracker::new(ConnectivityState::Online);
        assert!(!tracker.update(ConnectivityState::Offline));
    }

    #[test]
    fn offline_to_online_triggers_replay() {
        let mut tracker = ConnectivityTracker::new(ConnectivityState::Offline);
        assert!(tracker.update(ConnectivityState::Online));
    }

    #[test]
    fn offline_to_offline_no_replay() {
        let mut tracker = ConnectivityTracker::new(ConnectivityState::Offline);
        assert!(!tracker.update(ConnectivityState::Offline));
    }

    #[test]
    fn repeated_transitions_trigger_replay_each_time() {
        let mut tracker = ConnectivityTracker::new(ConnectivityState::Online);
        // Go offline
        assert!(!tracker.update(ConnectivityState::Offline));
        // Come back online — replay
        assert!(tracker.update(ConnectivityState::Online));
        // Stay online — no replay
        assert!(!tracker.update(ConnectivityState::Online));
        // Go offline again
        assert!(!tracker.update(ConnectivityState::Offline));
        // Come back online again — replay
        assert!(tracker.update(ConnectivityState::Online));
    }

    #[test]
    fn state_returns_current() {
        let mut tracker = ConnectivityTracker::new(ConnectivityState::Online);
        assert_eq!(tracker.state(), ConnectivityState::Online);
        tracker.update(ConnectivityState::Offline);
        assert_eq!(tracker.state(), ConnectivityState::Offline);
    }

    #[test]
    fn starting_offline_then_online_triggers_replay() {
        let mut tracker = ConnectivityTracker::new(ConnectivityState::Offline);
        // First transition to online should trigger replay
        assert!(tracker.update(ConnectivityState::Online));
        assert_eq!(tracker.state(), ConnectivityState::Online);
    }
}
