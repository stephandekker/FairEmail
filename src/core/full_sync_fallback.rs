//! Core logic for deciding when to use the full-sync fallback instead of
//! CONDSTORE incremental sync.
//!
//! Primary drivers: FR-11, FR-12, FR-47, AC-23, N-7.
//!
//! The full-sync fallback compares the complete UID list and flags within
//! the sync window against local state. It is the baseline sync strategy
//! that works on all servers, including those without CONDSTORE/QRESYNC.
//!
//! Key rules:
//! - When the server does not advertise CONDSTORE/QRESYNC, this path is used.
//! - When a sync cycle is triggered within a short interval (default 30s) of
//!   the previous successful sync, the full comparison is forced even if
//!   CONDSTORE is available (FR-12).

/// Default threshold in seconds for rapid re-sync detection.
pub const RAPID_RESYNC_THRESHOLD_SECS: i64 = 30;

/// Decide whether a full UID/flag comparison should be forced instead of
/// relying on CONDSTORE incremental sync.
///
/// Returns `true` when the elapsed time since the last successful sync is
/// less than `threshold_secs`, meaning the caller should bypass CONDSTORE
/// and do a full comparison to avoid relying on cached state.
pub fn should_force_full_sync(last_sync_at: Option<i64>, now: i64, threshold_secs: i64) -> bool {
    match last_sync_at {
        Some(ts) if ts > 0 => {
            let elapsed = now.saturating_sub(ts);
            elapsed < threshold_secs
        }
        _ => false,
    }
}

/// Filter a UID slice to only those within the sync window (>= min_uid).
///
/// When `sync_window_min_uid` is `None`, all UIDs are returned unchanged.
pub fn scope_uids_to_window(uids: &[u32], sync_window_min_uid: Option<u32>) -> Vec<u32> {
    match sync_window_min_uid {
        Some(min) => uids.iter().copied().filter(|uid| *uid >= min).collect(),
        None => uids.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- should_force_full_sync tests ---

    #[test]
    fn force_full_sync_when_within_threshold() {
        let now = 1000;
        let last = Some(985); // 15s ago
        assert!(should_force_full_sync(
            last,
            now,
            RAPID_RESYNC_THRESHOLD_SECS
        ));
    }

    #[test]
    fn no_force_when_outside_threshold() {
        let now = 1000;
        let last = Some(950); // 50s ago
        assert!(!should_force_full_sync(
            last,
            now,
            RAPID_RESYNC_THRESHOLD_SECS
        ));
    }

    #[test]
    fn no_force_when_exactly_at_threshold() {
        let now = 1000;
        let last = Some(970); // exactly 30s ago
        assert!(!should_force_full_sync(
            last,
            now,
            RAPID_RESYNC_THRESHOLD_SECS
        ));
    }

    #[test]
    fn no_force_when_no_previous_sync() {
        assert!(!should_force_full_sync(
            None,
            1000,
            RAPID_RESYNC_THRESHOLD_SECS
        ));
    }

    #[test]
    fn no_force_when_last_sync_is_zero() {
        assert!(!should_force_full_sync(
            Some(0),
            1000,
            RAPID_RESYNC_THRESHOLD_SECS
        ));
    }

    #[test]
    fn force_when_one_second_apart() {
        assert!(should_force_full_sync(
            Some(999),
            1000,
            RAPID_RESYNC_THRESHOLD_SECS
        ));
    }

    #[test]
    fn custom_threshold() {
        // 10s threshold
        assert!(should_force_full_sync(Some(995), 1000, 10));
        assert!(!should_force_full_sync(Some(989), 1000, 10));
    }

    // --- scope_uids_to_window tests ---

    #[test]
    fn scope_none_returns_all() {
        let uids = vec![1, 2, 3, 4, 5];
        assert_eq!(scope_uids_to_window(&uids, None), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn scope_filters_below_min() {
        let uids = vec![1, 2, 3, 4, 5];
        assert_eq!(scope_uids_to_window(&uids, Some(3)), vec![3, 4, 5]);
    }

    #[test]
    fn scope_all_below_min() {
        let uids = vec![1, 2, 3];
        assert_eq!(scope_uids_to_window(&uids, Some(10)), Vec::<u32>::new());
    }

    #[test]
    fn scope_empty_input() {
        assert_eq!(scope_uids_to_window(&[], Some(5)), Vec::<u32>::new());
    }

    #[test]
    fn scope_min_uid_of_one() {
        let uids = vec![1, 2, 3];
        assert_eq!(scope_uids_to_window(&uids, Some(1)), vec![1, 2, 3]);
    }
}
