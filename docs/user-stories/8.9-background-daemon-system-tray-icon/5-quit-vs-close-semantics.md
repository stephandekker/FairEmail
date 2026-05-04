# Quit vs Close Semantics

## Parent Feature
#8.9 Background Daemon (System Tray Icon)

## User Story
As any user, I want a clear distinction between closing the window and quitting the application, so that I do not accidentally stop mail synchronization by closing the window.

## Blocked by
- `1-daemon-process-lifecycle`
- `4-tray-icon-context-menu`

## Acceptance Criteria
- Closing the main application window (via the window manager close button, Alt+F4, or equivalent) does NOT terminate the daemon. The tray icon remains visible and synchronization continues.
- An explicit "Quit" action (from the application menu, the tray icon context menu, or a keyboard shortcut) terminates both the main window and the daemon.
- On quit: the tray icon is removed, all server connections are closed, and no orphan processes remain.
- The distinction between close and quit is discoverable — e.g. via the tray icon remaining visible after window close, or through menu labeling.

## Mapping to Epic
- US-21, US-22
- FR-35, FR-36
- AC-2 (close leaves daemon running), AC-5 (quit terminates everything)

## HITL / AFK
AFK — the behavior is clearly specified by the epic. No ambiguity.

## Notes
- OQ-6 (graceful shutdown on session end) is tangentially relevant. This story covers user-initiated quit. Session-end behavior (logout/shutdown) should follow the same quit path but may need a grace period for in-progress operations — that aspect is covered in story 11's restart/shutdown handling.
