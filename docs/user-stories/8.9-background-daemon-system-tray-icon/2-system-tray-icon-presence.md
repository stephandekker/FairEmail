# System Tray Icon Presence

## Parent Feature
#8.9 Background Daemon (System Tray Icon)

## User Story
As any user, I want a system tray icon to appear in my desktop's notification area when the daemon is running, so that I have visual confirmation that background synchronization is active.

## Blocked by
- `1-daemon-process-lifecycle`

## Acceptance Criteria
- When the daemon is running, a system tray icon is displayed in the desktop environment's notification area.
- When the daemon stops, the tray icon either disappears or switches to a visually distinct "inactive" state.
- The tray icon is compatible with freedesktop StatusNotifierItem protocol.
- The tray icon is compatible with the legacy X11 system tray protocol.
- The tray icon is visible and functional on GNOME (via AppIndicator/StatusNotifierItem extension), KDE Plasma, and XFCE.
- The tray icon uses a minimal, non-intrusive visual style — no sound, no animation beyond state changes.
- The tray icon does not compete with new-mail notifications for user attention.

## Mapping to Epic
- US-3, US-7, US-17
- FR-8, FR-9, FR-10, FR-14
- NFR-5 (desktop environment compatibility), NFR-6 (unobtrusive presence)
- AC-1 (partially — tray icon appears)

## HITL / AFK
HITL — the initial icon design and visual states need a design review to ensure they are recognizable and consistent across desktop environments.

## Notes
- OQ-1 (tray icon visual vocabulary) is relevant here. This story needs at minimum a "running/active" state and a "stopped/inactive" state. More granular states (error, waiting, polling) are added in story 6. The exact icon design is a deferred decision per the epic.
- OQ-5 (no tray available) is relevant. The epic leaves open whether the daemon can run headless. This story should handle the case where no notification area is available — at minimum the daemon should not crash. The decision on whether to run headless should be documented.
- OQ-7 (DE interaction conventions) applies to click behavior, which is covered in story 3.
