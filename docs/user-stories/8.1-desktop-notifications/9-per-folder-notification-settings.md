## Parent Feature

#8.1 Desktop Notifications

## What to build

Add a per-folder notification enable/disable toggle that overrides the account-level default. The toggle is accessible from the folder's context. A folder's notification setting is independent of its Unified Inbox membership and its synchronization setting — changing one never modifies the others. A folder that is synchronized but has notifications disabled never produces a new-mail notification. A user can explicitly enable notifications for a non-Inbox folder.

Covers epic sections: §7.7 (FR-27, FR-30 — folder context), §7.12 (FR-45, FR-46).

## Acceptance criteria

- [ ] Each folder has an independent notification enable/disable toggle
- [ ] The per-folder toggle overrides the account-level setting
- [ ] The per-folder toggle is accessible from the folder's context menu or settings
- [ ] A message arriving in a non-Inbox folder that the user has explicitly enabled notifications for produces a notification (AC-3)
- [ ] Toggling a folder's Unified Inbox membership does not change its notification setting, and vice versa (AC-16)
- [ ] A synchronized folder with notifications disabled never produces a new-mail notification
- [ ] A folder's notification and synchronization settings are independently configurable

## Blocked by

- Blocked by `8-per-account-notification-settings`

## User stories addressed

- US-8 (per-folder notification setting respected)
- US-22 (per-folder notification settings overriding account defaults)
