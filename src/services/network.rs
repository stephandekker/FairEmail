//! Network availability check for the setup wizard (FR-7).
//!
//! Uses `gio::NetworkMonitor` when the UI feature is enabled;
//! provides a mock for testing without a display server.

#[cfg(feature = "ui")]
pub(crate) fn is_network_available() -> bool {
    use gtk4::gio;
    use gtk4::prelude::NetworkMonitorExt;
    let monitor = gio::NetworkMonitor::default();
    monitor.is_network_available()
}

#[cfg(not(feature = "ui"))]
pub(crate) fn is_network_available() -> bool {
    // Fallback for headless / test builds.
    false
}
