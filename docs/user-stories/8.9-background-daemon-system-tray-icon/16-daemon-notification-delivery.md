# Daemon as Notification Delivery Process

## Parent Feature
#8.9 Background Daemon (System Tray Icon)

## User Story
As any user, I want the daemon to be responsible for delivering desktop notifications for new mail, so that notifications arrive even when the main window is closed, and I want tray icon presence and notification delivery to be independent concerns.

## Blocked by
- `1-daemon-process-lifecycle`
- `2-system-tray-icon-presence`

## Acceptance Criteria
- The daemon is the process that delivers desktop notifications for new mail, regardless of whether the main window is open.
- New-mail notifications continue to be delivered when the main window is closed but the daemon is running.
- The tray icon's presence and the delivery of new-mail notifications are independent concerns: the user can have the tray icon visible without receiving notifications for every folder, and vice versa.
- The tray icon's notification channel (status display) and new-mail notifications (alerting) are separately configurable, maintaining the separation described in N-3.

## Mapping to Epic
- US-19, US-20
- N-3 (separation of monitoring notification from mail notifications)
- AC-2 (partially — notifications continue after window close)

## HITL / AFK
AFK — the behavioral contract is clear. Notification content and configuration are defined in epic 8.1.

## Notes
- This story establishes that the daemon is the delivery mechanism for notifications. It does NOT define notification content, grouping, actions, or per-folder configuration — those belong to the notifications epic (8.1, per NG2).
- N-3 emphasizes strict separation between the tray icon (low-priority status indicator) and mail notifications (high-priority alerts). Implementation should ensure these are independent subsystems.
