## Parent Feature

#8.1 Desktop Notifications

## What to build

When a privacy mode is active (e.g. the application requires authentication to unlock), redact notification content so that only a count or generic "new mail" label is shown. No sender name, subject, or preview text is visible in the notification. This is a hard constraint — if the application requires authentication to view content, notifications must not leak that content (design note N-5).

Covers epic sections: §7.8 (FR-31).

## Acceptance criteria

- [ ] With privacy mode active, a new-mail notification shows only a count or generic label (AC-14)
- [ ] No sender name, subject, or preview text is visible in notifications when privacy mode is active
- [ ] Privacy mode redaction applies regardless of the user's content display preferences
- [ ] When privacy mode is deactivated, notifications revert to showing configured content fields

## Blocked by

- Blocked by `1-notification-categories-and-basic-new-mail`

## User stories addressed

- US-10 (notification content respects privacy settings)
