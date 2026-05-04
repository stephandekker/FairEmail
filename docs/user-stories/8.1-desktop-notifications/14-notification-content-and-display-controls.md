## Parent Feature

#8.1 Desktop Notifications

## What to build

Provide user controls to independently enable or disable each content field in notifications: sender name (always shown by default), subject, body preview, and account/folder subtitle. Additionally, provide an option to transliterate or restrict notification text to ASCII for compatibility with notification daemons or external displays that have limited character support.

Covers epic sections: §7.8 (FR-32, FR-33).

## Acceptance criteria

- [ ] The user can independently enable/disable: sender name, subject, body preview, account/folder subtitle in notifications
- [ ] Disabling a content field removes it from all subsequent notifications
- [ ] An option to transliterate/restrict notification text to ASCII is available
- [ ] The transliteration option, when enabled, converts non-Latin characters in notification text to ASCII equivalents or strips them

## Blocked by

- Blocked by `7-global-notification-configuration`

## User stories addressed

- US-27 (control what content appears in notifications)
- US-30 (transliterate non-Latin scripts in notification text)
