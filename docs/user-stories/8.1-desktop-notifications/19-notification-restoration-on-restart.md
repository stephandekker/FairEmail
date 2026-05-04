## Parent Feature

#8.1 Desktop Notifications

## What to build

On application restart, restore notifications for messages that were pending notification at the time of shutdown, so that no unread messages are silently lost. The per-message notification state (from story 2) must be persisted to durable storage and read back on startup to determine which notifications to re-post.

Covers epic sections: §7.9 (FR-37).

## Acceptance criteria

- [ ] After application restart, unread messages that had pending notifications are re-notified (AC-17)
- [ ] Messages that were dismissed before shutdown are NOT re-notified
- [ ] The restoration happens automatically on startup without user intervention
- [ ] Notification state is persisted to durable storage (survives process termination)

## Blocked by

- Blocked by `2-notification-state-tracking-and-read-dismissal`

## User stories addressed

- US-35 (existing unread-mail notifications restored after restart)

## Notes

- OQ-5 (notification restoration on daemon restart) is related but distinct. This story covers application restart. Whether the application should also detect and recover from a notification daemon crash/restart is an open question flagged in the epic.
