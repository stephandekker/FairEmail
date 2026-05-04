# Account Connection Status Display

## Parent Feature
#7.1 IMAP IDLE (Push)

## User Story
As any user, I want the application to display the current push/poll status for each account (e.g. "Push active", "Polling every 15 min", "Disconnected — retrying"), so that I can verify at a glance whether real-time delivery is working.

## Blocked by
- `2-single-folder-idle-session`
- `3-poll-mode-fallback`
- `7-connection-failure-recovery`

## Acceptance Criteria
- The application maintains a per-account connection status indicating the current state of push: connected and idle, connecting, polling, disconnected (with reason), or optimized-to-poll (FR-40).
- Account connection status is visible in the user interface at all times (AC-11).
- Status updates within 10 seconds of a state change (AC-11).
- The status is understandable without protocol-level knowledge (NFR-6) — e.g. "Push active" rather than "IDLE established on INBOX".
- When push is degraded (e.g. auto-optimized to poll, or repeated reconnection failures), the status reflects this clearly (US-23).

## Mapping to Epic
- US-23
- FR-40
- NFR-6
- AC-11

## HITL / AFK
HITL — the status display UI/UX should be reviewed for clarity and discoverability.

## Notes
- This story covers the status model and its display. The diagnostic logging and detailed metrics are in story 15.
- The exact placement of the status indicator (account list, status bar, settings) is a UI design decision not prescribed by the epic.
