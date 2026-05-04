# Daemon Process Lifecycle

## Parent Feature
#8.9 Background Daemon (System Tray Icon)

## User Story
As any user, I want the application to run a background daemon process that persists independently of the main application window, so that mail synchronization continues even when the window is closed.

## Blocked by
*(none — this is the foundational slice)*

## Acceptance Criteria
- The application can launch a background daemon process that is separate from (or outlives) the main application window.
- When the main application window is closed, the daemon continues running.
- The daemon remains alive as long as at least one account is enabled for monitoring or there are pending operations.
- The daemon stops when there are no enabled accounts, no pending operations, and no reason to stay alive (subject to the grace period defined in story 9).
- The daemon process can be started and stopped programmatically (other stories will wire this to UI controls).
- The daemon terminates cleanly on explicit quit (no orphan processes, no stale lock files).

## Mapping to Epic
- US-1, US-3 (daemon continues after window close)
- FR-1, FR-3
- NFR-1 (resource efficiency when idle), NFR-4 (session integration)
- AC-2 (partially — tray icon and notifications are separate stories)

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- This story establishes the daemon as a process concept. It does not include the system tray icon (story 2), autostart (story 10), auto-restart (story 11), or any UI to control the daemon. Those are layered on in subsequent stories.
- The epic deliberately does not prescribe the process supervision mechanism (NG3). This story should not lock in whether the daemon is a separate binary, a background thread, a child process, or a service unit — only that it survives window close and terminates cleanly on quit.
