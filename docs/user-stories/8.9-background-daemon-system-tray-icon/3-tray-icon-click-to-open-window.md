# Tray Icon Click to Open Window

## Parent Feature
#8.9 Background Daemon (System Tray Icon)

## User Story
As any user, I want to click (or activate) the system tray icon to open or bring to focus the main application window, so that I can quickly access my mail from the tray.

## Blocked by
- `2-system-tray-icon-presence`

## Acceptance Criteria
- Clicking (activating) the tray icon when the main window is not open launches or shows the main application window.
- Clicking the tray icon when the main window is open but not focused brings it to focus.
- When the main window is already focused, clicking the tray icon may minimize or hide the window, following the host desktop environment's conventions.
- The main window opens to the user's default view (typically the unified inbox), consistent with N-8 in the epic.

## Mapping to Epic
- US-12
- FR-15, FR-17
- AC-3

## HITL / AFK
AFK — standard desktop integration behavior, no design decision needed beyond following DE conventions.

## Notes
- OQ-7 (click conventions across DEs) is relevant. Single-click vs. double-click to activate varies. This story should follow whatever convention the tray protocol specifies for the "Activate" action, rather than inventing custom behavior.
