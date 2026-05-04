# Poll Mode Support

## Parent Feature
#8.9 Background Daemon (System Tray Icon)

## User Story
As a resource-conscious user, I want to switch from continuous push (IMAP IDLE) to periodic polling at a configurable interval, so that the daemon can release persistent connections between polls and reduce resource usage.

## Blocked by
- `1-daemon-process-lifecycle`
- `2-system-tray-icon-presence`

## Acceptance Criteria
- The user can configure the daemon to use periodic polling instead of continuous push connections.
- The polling interval is configurable.
- When in poll mode, the daemon wakes at the configured interval, synchronizes, and releases server connections until the next poll.
- The tray icon remains visible in poll mode.
- The tray icon reflects the daemon's current phase: idle between polls vs. actively syncing during a poll.
- Switching from continuous to poll mode (or vice versa) takes effect without requiring an application restart.

## Mapping to Epic
- US-16, US-17
- FR-30, FR-31, FR-32
- AC-11

## HITL / AFK
AFK — behavior is well-specified by the epic.

## Notes
- The sync mode setting (continuous vs. poll) and interval configuration likely overlap with account-level settings defined in epic 1.1 or connectivity epics (7.x). This story covers the daemon's behavior in poll mode and the tray icon's reflection of poll state, not the settings UI for configuring the interval.
