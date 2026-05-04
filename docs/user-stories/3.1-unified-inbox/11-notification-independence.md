# Notification Independence from Membership

## Parent Feature

#3.1 Unified Inbox

## What to build

Ensure that unified-inbox membership and folder notification settings are fully independent (FR-26, FR-27). Toggling one must never modify the other. Notifications are governed solely by the per-folder notification setting and global mute/quiet-hours — not by unified-inbox membership. All eight combinations of {synced, notifying, unified} are meaningful and must be supported (see Design Note N-2 in the epic).

## Acceptance criteria

- [ ] Toggling unified-inbox membership ON for a folder does not enable notifications for that folder (AC-5).
- [ ] Toggling unified-inbox membership OFF for a folder does not disable notifications for that folder (AC-5).
- [ ] Toggling notifications ON for a folder does not change its unified-inbox membership.
- [ ] Toggling notifications OFF for a folder does not change its unified-inbox membership.
- [ ] A folder can be: unified + notifying, unified + silent, not-unified + notifying, not-unified + silent — all four states work correctly.

## Blocked by

- Blocked by `1-folder-membership-state`
- Blocked by `5-toggle-membership-context-action`

## User stories addressed

- US-20 (notification settings independent of membership)
- US-21 (membership toggle has no effect on notifications)
