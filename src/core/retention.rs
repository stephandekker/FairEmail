//! Sync window and local retention configuration (US-25, FR-41–FR-44, AC-24).
//!
//! Provides per-folder defaults, validation, and cutoff-date computation
//! used by the sync engine to limit flag-checking and local cleanup.

use crate::core::account::FolderRole;

/// Default sync window in days (how far back to check for flag changes).
pub const DEFAULT_SYNC_WINDOW_DAYS: u32 = 7;

/// Default keep window in days (how long messages are kept locally).
pub const DEFAULT_KEEP_WINDOW_DAYS: u32 = 30;

/// Drafts get a longer keep window by default (1 year).
pub const DEFAULT_KEEP_WINDOW_DAYS_DRAFTS: u32 = 365;

/// Per-folder retention configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetentionConfig {
    /// How far back (in days) to synchronize flags on routine sync cycles.
    pub sync_window_days: u32,
    /// How long (in days) to keep messages locally. Must be >= `sync_window_days`.
    pub keep_window_days: u32,
}

impl RetentionConfig {
    /// Create a new config, enforcing keep_window >= sync_window.
    pub fn new(sync_window_days: u32, keep_window_days: u32) -> Self {
        Self {
            sync_window_days,
            keep_window_days: keep_window_days.max(sync_window_days),
        }
    }

    /// Default retention config for a folder with the given role.
    pub fn default_for_role(role: Option<&FolderRole>) -> Self {
        let keep = match role {
            Some(FolderRole::Drafts) => DEFAULT_KEEP_WINDOW_DAYS_DRAFTS,
            _ => DEFAULT_KEEP_WINDOW_DAYS,
        };
        Self {
            sync_window_days: DEFAULT_SYNC_WINDOW_DAYS,
            keep_window_days: keep,
        }
    }

    /// Compute the Unix timestamp cutoff for the sync window.
    /// Messages with `date_received` before this timestamp are outside the sync
    /// window and should NOT be checked for flag changes (AC-24).
    pub fn sync_cutoff_timestamp(&self, now_epoch_secs: i64) -> i64 {
        now_epoch_secs - (self.sync_window_days as i64 * 86_400)
    }

    /// Compute the Unix timestamp cutoff for the keep window.
    /// Messages with `date_received` before this timestamp are outside the keep
    /// window and should be removed from local storage (FR-44).
    pub fn keep_cutoff_timestamp(&self, now_epoch_secs: i64) -> i64 {
        now_epoch_secs - (self.keep_window_days as i64 * 86_400)
    }
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            sync_window_days: DEFAULT_SYNC_WINDOW_DAYS,
            keep_window_days: DEFAULT_KEEP_WINDOW_DAYS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_enforces_keep_gte_sync() {
        let cfg = RetentionConfig::new(30, 7);
        assert_eq!(cfg.sync_window_days, 30);
        assert_eq!(cfg.keep_window_days, 30);
    }

    #[test]
    fn new_preserves_valid_values() {
        let cfg = RetentionConfig::new(7, 30);
        assert_eq!(cfg.sync_window_days, 7);
        assert_eq!(cfg.keep_window_days, 30);
    }

    #[test]
    fn default_for_inbox() {
        let cfg = RetentionConfig::default_for_role(None);
        assert_eq!(cfg.sync_window_days, DEFAULT_SYNC_WINDOW_DAYS);
        assert_eq!(cfg.keep_window_days, DEFAULT_KEEP_WINDOW_DAYS);
    }

    #[test]
    fn default_for_drafts_has_longer_keep() {
        let cfg = RetentionConfig::default_for_role(Some(&FolderRole::Drafts));
        assert_eq!(cfg.keep_window_days, DEFAULT_KEEP_WINDOW_DAYS_DRAFTS);
    }

    #[test]
    fn default_for_sent() {
        let cfg = RetentionConfig::default_for_role(Some(&FolderRole::Sent));
        assert_eq!(cfg.keep_window_days, DEFAULT_KEEP_WINDOW_DAYS);
    }

    #[test]
    fn sync_cutoff_is_correct() {
        let cfg = RetentionConfig::new(7, 30);
        let now = 1_000_000;
        assert_eq!(cfg.sync_cutoff_timestamp(now), now - 7 * 86_400);
    }

    #[test]
    fn keep_cutoff_is_correct() {
        let cfg = RetentionConfig::new(7, 30);
        let now = 1_000_000;
        assert_eq!(cfg.keep_cutoff_timestamp(now), now - 30 * 86_400);
    }

    #[test]
    fn default_trait_matches_constants() {
        let cfg = RetentionConfig::default();
        assert_eq!(cfg.sync_window_days, DEFAULT_SYNC_WINDOW_DAYS);
        assert_eq!(cfg.keep_window_days, DEFAULT_KEEP_WINDOW_DAYS);
    }
}
