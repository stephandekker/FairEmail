use std::collections::HashMap;
use uuid::Uuid;

/// Connection states an account can be in (FR-44, AC-18).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not connected to the server.
    #[default]
    Disconnected,
    /// Actively establishing a connection.
    Connecting,
    /// Connected and operational.
    Connected,
    /// Gracefully shutting down the connection.
    Closing,
    /// Waiting before retrying after a failure.
    BackingOff,
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Connecting => write!(f, "Connecting"),
            Self::Connected => write!(f, "Connected"),
            Self::Closing => write!(f, "Closing"),
            Self::BackingOff => write!(f, "Backing off"),
        }
    }
}

impl ConnectionState {
    /// Icon name suitable for display in the account list (FR-44).
    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Disconnected => "network-offline-symbolic",
            Self::Connecting => "network-transmit-symbolic",
            Self::Connected => "network-idle-symbolic",
            Self::Closing => "network-receive-symbolic",
            Self::BackingOff => "network-error-symbolic",
        }
    }

    /// CSS class for the status indicator.
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Disconnected => "connection-disconnected",
            Self::Connecting => "connection-connecting",
            Self::Connected => "connection-connected",
            Self::Closing => "connection-closing",
            Self::BackingOff => "connection-backing-off",
        }
    }
}

/// A single entry in a per-account connection log (FR-46, US-42).
#[derive(Debug, Clone)]
pub struct ConnectionLogEntry {
    /// Monotonic sequence number within this account's log.
    pub seq: u64,
    /// Timestamp as seconds since UNIX epoch.
    pub timestamp_secs: u64,
    /// Human-readable log message.
    pub message: String,
}

/// Per-account connection information: current state, optional error, and log entries.
#[derive(Debug, Clone)]
pub struct AccountConnectionInfo {
    state: ConnectionState,
    error_detail: Option<String>,
    log: Vec<ConnectionLogEntry>,
    next_seq: u64,
}

impl Default for AccountConnectionInfo {
    fn default() -> Self {
        Self {
            state: ConnectionState::Disconnected,
            error_detail: None,
            log: Vec::new(),
            next_seq: 1,
        }
    }
}

impl AccountConnectionInfo {
    pub fn state(&self) -> ConnectionState {
        self.state
    }

    pub fn error_detail(&self) -> Option<&str> {
        self.error_detail.as_deref()
    }

    pub fn log_entries(&self) -> &[ConnectionLogEntry] {
        &self.log
    }
}

/// Manages per-account connection state and diagnostic logs (FR-44, FR-45, FR-46, NFR-2).
///
/// Each account's state is independent — a failure on one account does not
/// affect any other account (NFR-2, AC-16).
#[derive(Debug, Default)]
pub struct ConnectionStateManager {
    accounts: HashMap<Uuid, AccountConnectionInfo>,
}

/// Maximum number of log entries retained per account.
const MAX_LOG_ENTRIES: usize = 500;

impl ConnectionStateManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the connection info for an account, creating a default entry if absent.
    pub fn get(&self, account_id: Uuid) -> Option<&AccountConnectionInfo> {
        self.accounts.get(&account_id)
    }

    /// Get the connection state for an account, defaulting to Disconnected.
    pub fn state(&self, account_id: Uuid) -> ConnectionState {
        self.accounts
            .get(&account_id)
            .map_or(ConnectionState::Disconnected, |info| info.state)
    }

    /// Get the error detail for an account.
    pub fn error_detail(&self, account_id: Uuid) -> Option<&str> {
        self.accounts
            .get(&account_id)
            .and_then(|info| info.error_detail.as_deref())
    }

    /// Get log entries for an account.
    pub fn log_entries(&self, account_id: Uuid) -> &[ConnectionLogEntry] {
        self.accounts
            .get(&account_id)
            .map_or(&[], |info| info.log.as_slice())
    }

    /// Transition an account to a new connection state (AC-18).
    /// Appends a log entry describing the transition.
    pub fn set_state(&mut self, account_id: Uuid, new_state: ConnectionState) {
        let info = self.accounts.entry(account_id).or_default();
        let old_state = info.state;
        info.state = new_state;

        if new_state != ConnectionState::BackingOff {
            info.error_detail = None;
        }

        let message = format!("State changed: {old_state} → {new_state}");
        self.append_log(account_id, message);
    }

    /// Set an error on an account and transition to BackingOff (FR-45).
    pub fn set_error(&mut self, account_id: Uuid, error: String) {
        let info = self.accounts.entry(account_id).or_default();
        info.state = ConnectionState::BackingOff;
        let msg = format!("Error: {error}");
        info.error_detail = Some(error);
        self.append_log(account_id, msg);
    }

    /// Clear the error detail for an account.
    pub fn clear_error(&mut self, account_id: Uuid) {
        if let Some(info) = self.accounts.get_mut(&account_id) {
            info.error_detail = None;
        }
    }

    /// Append a message to the account's connection log (FR-46, US-42).
    pub fn append_log(&mut self, account_id: Uuid, message: String) {
        let info = self.accounts.entry(account_id).or_default();
        let seq = info.next_seq;
        info.next_seq += 1;

        let timestamp_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        info.log.push(ConnectionLogEntry {
            seq,
            timestamp_secs,
            message,
        });

        // Trim old entries.
        if info.log.len() > MAX_LOG_ENTRIES {
            let excess = info.log.len() - MAX_LOG_ENTRIES;
            info.log.drain(..excess);
        }
    }

    /// Remove all state for an account (e.g. when it is deleted).
    pub fn remove(&mut self, account_id: Uuid) {
        self.accounts.remove(&account_id);
    }

    /// Initialise state for an account if it doesn't already have an entry.
    pub fn ensure_account(&mut self, account_id: Uuid) {
        self.accounts.entry(account_id).or_default();
    }
}

/// Format a UNIX timestamp as a human-readable local time string.
pub fn format_log_timestamp(timestamp_secs: u64) -> String {
    // Simple UTC formatting without external crate dependency.
    let secs = timestamp_secs;
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Days since epoch → date (simplified: only works for dates after 1970).
    let (year, month, day) = days_to_date(days);
    format!("{year:04}-{month:02}-{day:02} {hours:02}:{minutes:02}:{seconds:02} UTC")
}

/// Convert days since UNIX epoch to (year, month, day).
fn days_to_date(days: u64) -> (u64, u64, u64) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_is_disconnected() {
        let mgr = ConnectionStateManager::new();
        let id = Uuid::new_v4();
        assert_eq!(mgr.state(id), ConnectionState::Disconnected);
    }

    #[test]
    fn set_state_transitions() {
        let mut mgr = ConnectionStateManager::new();
        let id = Uuid::new_v4();

        mgr.set_state(id, ConnectionState::Connecting);
        assert_eq!(mgr.state(id), ConnectionState::Connecting);

        mgr.set_state(id, ConnectionState::Connected);
        assert_eq!(mgr.state(id), ConnectionState::Connected);

        mgr.set_state(id, ConnectionState::Closing);
        assert_eq!(mgr.state(id), ConnectionState::Closing);

        mgr.set_state(id, ConnectionState::Disconnected);
        assert_eq!(mgr.state(id), ConnectionState::Disconnected);
    }

    #[test]
    fn set_error_transitions_to_backing_off() {
        let mut mgr = ConnectionStateManager::new();
        let id = Uuid::new_v4();

        mgr.set_state(id, ConnectionState::Connecting);
        mgr.set_error(id, "Connection refused".into());

        assert_eq!(mgr.state(id), ConnectionState::BackingOff);
        assert_eq!(mgr.error_detail(id), Some("Connection refused"));
    }

    #[test]
    fn state_change_clears_error_unless_backing_off() {
        let mut mgr = ConnectionStateManager::new();
        let id = Uuid::new_v4();

        mgr.set_error(id, "Timeout".into());
        assert_eq!(mgr.error_detail(id), Some("Timeout"));

        mgr.set_state(id, ConnectionState::Connecting);
        assert!(mgr.error_detail(id).is_none());
    }

    #[test]
    fn log_entries_are_appended() {
        let mut mgr = ConnectionStateManager::new();
        let id = Uuid::new_v4();

        mgr.set_state(id, ConnectionState::Connecting);
        mgr.set_state(id, ConnectionState::Connected);

        let entries = mgr.log_entries(id);
        assert_eq!(entries.len(), 2);
        assert!(entries[0].message.contains("Connecting"));
        assert!(entries[1].message.contains("Connected"));
        assert_eq!(entries[0].seq, 1);
        assert_eq!(entries[1].seq, 2);
    }

    #[test]
    fn log_trimmed_to_max_entries() {
        let mut mgr = ConnectionStateManager::new();
        let id = Uuid::new_v4();

        for i in 0..600 {
            mgr.append_log(id, format!("Entry {i}"));
        }

        let entries = mgr.log_entries(id);
        assert_eq!(entries.len(), 500);
        // Oldest entries should have been trimmed.
        assert!(entries[0].message.contains("Entry 100"));
    }

    #[test]
    fn accounts_are_independent_nfr2() {
        let mut mgr = ConnectionStateManager::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        mgr.set_state(id1, ConnectionState::Connected);
        mgr.set_error(id2, "Auth failed".into());

        // Account 1 should still be connected despite account 2's error.
        assert_eq!(mgr.state(id1), ConnectionState::Connected);
        assert!(mgr.error_detail(id1).is_none());

        // Account 2 should be in error state.
        assert_eq!(mgr.state(id2), ConnectionState::BackingOff);
        assert_eq!(mgr.error_detail(id2), Some("Auth failed"));
    }

    #[test]
    fn remove_clears_account_state() {
        let mut mgr = ConnectionStateManager::new();
        let id = Uuid::new_v4();

        mgr.set_state(id, ConnectionState::Connected);
        mgr.append_log(id, "test".into());
        mgr.remove(id);

        assert_eq!(mgr.state(id), ConnectionState::Disconnected);
        assert!(mgr.log_entries(id).is_empty());
    }

    #[test]
    fn clear_error() {
        let mut mgr = ConnectionStateManager::new();
        let id = Uuid::new_v4();

        mgr.set_error(id, "Something failed".into());
        assert!(mgr.error_detail(id).is_some());

        mgr.clear_error(id);
        assert!(mgr.error_detail(id).is_none());
    }

    #[test]
    fn connection_state_display() {
        assert_eq!(ConnectionState::Disconnected.to_string(), "Disconnected");
        assert_eq!(ConnectionState::Connecting.to_string(), "Connecting");
        assert_eq!(ConnectionState::Connected.to_string(), "Connected");
        assert_eq!(ConnectionState::Closing.to_string(), "Closing");
        assert_eq!(ConnectionState::BackingOff.to_string(), "Backing off");
    }

    #[test]
    fn connection_state_icon_names() {
        assert_eq!(
            ConnectionState::Disconnected.icon_name(),
            "network-offline-symbolic"
        );
        assert_eq!(
            ConnectionState::Connected.icon_name(),
            "network-idle-symbolic"
        );
        assert_eq!(
            ConnectionState::BackingOff.icon_name(),
            "network-error-symbolic"
        );
    }

    #[test]
    fn format_timestamp_basic() {
        // 2024-01-01 00:00:00 UTC = 1704067200
        let s = format_log_timestamp(1704067200);
        assert!(s.contains("2024"));
        assert!(s.contains("UTC"));
    }

    #[test]
    fn ensure_account_is_idempotent() {
        let mut mgr = ConnectionStateManager::new();
        let id = Uuid::new_v4();

        mgr.set_state(id, ConnectionState::Connected);
        mgr.ensure_account(id);
        // Should not have reset the state.
        assert_eq!(mgr.state(id), ConnectionState::Connected);
    }
}
