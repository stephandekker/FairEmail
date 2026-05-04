# Autostart on Login

## Parent Feature
#8.9 Background Daemon (System Tray Icon)

## User Story
As any user, I want the daemon to start automatically when I log in to my desktop session, so that mail is being synchronized from the moment my session begins without manual intervention.

## Blocked by
- `1-daemon-process-lifecycle`
- `2-system-tray-icon-presence`

## Acceptance Criteria
- After installing and configuring at least one account, the daemon starts automatically on the next login.
- A system tray icon appears in the notification area as part of autostart.
- The daemon reaches its monitoring state (connections initiated to all enabled accounts) within a few seconds of being started, not counting server response time.
- The user can enable or disable autostart via a discoverable application setting.
- With autostart disabled, the daemon does NOT start on login.
- Re-enabling autostart causes the daemon to start on the next login.

## Mapping to Epic
- US-2
- FR-2, FR-7
- NFR-2 (startup speed), NFR-4 (session integration)
- AC-1, AC-15

## HITL / AFK
AFK — standard desktop integration behavior.

## Notes
- The epic does not prescribe the autostart mechanism (NG3). Common approaches include XDG autostart desktop entries or user-level service units. The choice is an implementation decision.
