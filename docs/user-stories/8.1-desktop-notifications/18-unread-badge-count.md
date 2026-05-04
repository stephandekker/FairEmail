## Parent Feature

#8.1 Desktop Notifications

## What to build

Expose an unread message count to the desktop environment's badge/indicator mechanism (system-tray icon badge, launcher count, or equivalent). The user can choose whether the badge count reflects: (a) the number of active new-message notifications, or (b) the total number of unread messages in notify-enabled folders. The badge count updates within a few seconds of a notification being emitted or dismissed.

Covers epic sections: §7.10 (FR-39, FR-40, FR-41).

## Acceptance criteria

- [ ] The system-tray icon and/or launcher entry displays an unread message count badge
- [ ] The user can choose between two badge semantics: active notification count vs. total unread in notify-enabled folders
- [ ] The badge count updates within a few seconds of changes (AC-18)
- [ ] The badge reflects the chosen metric accurately after notifications are emitted or dismissed

## Blocked by

- Blocked by `2-notification-state-tracking-and-read-dismissal`

## User stories addressed

- US-31 (unread badge count on system tray/launcher)
- US-32 (choose badge count semantics)

## Notes

- OQ-4 (badge count mechanism) is directly relevant: Linux desktops have multiple incompatible badge mechanisms (Unity launcher API, KDE StatusNotifierItem, GNOME extensions, freedesktop StatusNotifierItem). This story should define which mechanisms are supported and the fallback behavior (e.g. tooltip-only) on unsupported environments.
- This story depends on the system-tray icon existing (epic 8.9). If the tray icon is not yet available, the badge can target the launcher entry only, with tray integration added when 8.9 lands.
