# Global Synchronization Toggle

## Parent Feature
#8.9 Background Daemon (System Tray Icon)

## User Story
As any user, I want a single global toggle that enables or disables all background synchronization, so that I have one clear control to turn off all background activity.

## Blocked by
- `1-daemon-process-lifecycle`
- `2-system-tray-icon-presence`

## Acceptance Criteria
- The application provides a single, global synchronization toggle accessible from the main application settings.
- The toggle defaults to **enabled**, so new users receive mail without additional configuration.
- When the toggle is disabled:
  - The daemon closes all server connections.
  - The daemon stops monitoring all accounts.
  - The tray icon reflects the inactive state (disappears or shows inactive icon).
  - No new-mail notifications are delivered.
  - Pending operations remain queued but are not executed.
- When the toggle is re-enabled:
  - The daemon resumes monitoring all previously enabled accounts.
  - Queued pending operations begin executing.
  - The tray icon returns to its "monitoring" state.
- The toggle may optionally also be accessible from the tray icon's context menu (design decision).

## Mapping to Epic
- US-4, US-5, US-14, US-15
- FR-18, FR-19, FR-20, FR-21
- AC-6

## HITL / AFK
AFK — behavior is clearly specified in the epic.

## Notes
- Whether the global sync toggle appears in the tray context menu is left to the design review (OQ-3). The mandatory location is the application settings.
