# Network Awareness

## Parent Feature
#8.9 Background Daemon (System Tray Icon)

## User Story
As any user, I want the daemon to detect network connectivity changes and respond automatically — connecting when a suitable network is available and pausing when it is not — so that synchronization resumes without my intervention when the network returns.

## Blocked by
- `1-daemon-process-lifecycle`
- `6-tray-icon-status-tooltip`

## Acceptance Criteria
- When a suitable network becomes available, the daemon establishes or re-establishes server connections automatically.
- When the network becomes unsuitable or unavailable, the daemon updates the tray icon/tooltip to reflect the "waiting for connection" state.
- The daemon does not attempt connections while the network is unsuitable.
- When the network returns, monitoring resumes and the tray icon updates accordingly.
- The definition of "suitable network" respects user preferences (e.g. sync on metered networks, sync on unmetered only).

## Mapping to Epic
- US-10
- FR-25, FR-26
- NFR-3 (resilience — tolerate network drops without crashing)
- AC-9

## HITL / AFK
AFK — behavior is well-specified by the epic.

## Notes
- Network suitability preferences (metered vs. unmetered) are likely defined in the account settings (epic 1.1) or connectivity epics (7.x). This story covers the daemon's reaction to network state changes, not the definition of the preferences themselves.
