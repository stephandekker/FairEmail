## Parent Feature

#8.1 Desktop Notifications

## What to build

Provide an option to suppress new-mail notifications while the application window is in the foreground, since the user can already see mail arriving in the message list. When enabled, new messages arriving while the window is focused do not produce notifications. Additionally, provide an option to auto-clear all existing new-mail notifications when the application window is brought to the foreground.

Covers epic sections: §7.9 (FR-35), §7.11 (FR-42).

## Acceptance criteria

- [ ] An option to suppress new-mail notifications when the application window is in the foreground is available
- [ ] With suppression enabled, new messages arriving while the window is focused do not produce notifications (AC-12)
- [ ] An option to auto-clear new-mail notifications when the window is brought to the foreground is available
- [ ] With auto-clear enabled, bringing the application window to the foreground clears all new-mail notifications (AC-12)
- [ ] Error/warning notifications are NOT suppressed by the foreground suppression option

## Blocked by

- Blocked by `2-notification-state-tracking-and-read-dismissal`

## User stories addressed

- US-6 (notifications auto-cleared when application brought to foreground)
- US-26 (suppress new-mail notifications while window is in foreground)
