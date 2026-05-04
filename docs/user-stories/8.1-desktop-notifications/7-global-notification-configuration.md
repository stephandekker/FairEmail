## Parent Feature

#8.1 Desktop Notifications

## What to build

Provide a global notification settings UI where the user can: enable/disable all notifications, choose whether to show subject, preview text, and account/folder subtitles in notifications, and configure each notification category's urgency, sound, and DND-bypass setting independently. This is the single place for controlling the overall notification experience before per-account/per-folder overrides are applied.

Covers epic sections: §7.1 (FR-2), §7.8 (FR-32), and the global layer of §7.7.

## Acceptance criteria

- [ ] A global notification settings screen is accessible from the application's settings
- [ ] The user can enable/disable all notifications globally
- [ ] The user can independently enable/disable each content field: sender name, subject, body preview, account/folder subtitle
- [ ] Each notification category (New mail, Send status, Warning, Error, Alerts, Service) can be independently configured for: sound, urgency, and DND bypass
- [ ] Changes to global settings take effect immediately for subsequent notifications

## Blocked by

- Blocked by `1-notification-categories-and-basic-new-mail`

## User stories addressed

- US-19 (global notification configuration)
- US-20 (distinct notification categories with independent settings)
- US-27 (control what content appears in notifications)
