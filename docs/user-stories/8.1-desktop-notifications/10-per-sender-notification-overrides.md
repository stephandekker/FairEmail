## Parent Feature

#8.1 Desktop Notifications

## What to build

Allow the user to assign per-sender notification overrides. A per-sender override takes precedence over the folder and account settings — if a specific sender is configured to always notify, their messages produce notifications regardless of the folder or account default. Per-sender overrides are manageable from the relevant message context (e.g. right-click on a message or sender).

Covers epic sections: §7.7 (FR-28, FR-30 — sender context).

## Acceptance criteria

- [ ] The user can create a per-sender notification override (enable or disable)
- [ ] A per-sender override takes precedence over folder and account settings
- [ ] Per-sender overrides are accessible from the message or sender context
- [ ] A message from a sender with notifications forced on produces a notification even if the folder has notifications disabled
- [ ] A message from a sender with notifications forced off does not produce a notification even if the folder has notifications enabled
- [ ] Per-sender overrides can be deleted/removed

## Blocked by

- Blocked by `9-per-folder-notification-settings`

## User stories addressed

- US-23 (per-sender notification settings overriding folder and account)
