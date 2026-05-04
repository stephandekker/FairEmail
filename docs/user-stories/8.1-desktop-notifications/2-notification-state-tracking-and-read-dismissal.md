## Parent Feature

#8.1 Desktop Notifications

## What to build

Implement per-message notification state tracking so the application knows which messages currently have an active notification. When a message is marked as read — whether from within the application, from a notification action, or from a remote client — the corresponding notification is dismissed. Dismissed notifications are recorded so that the same message is not re-notified unless it is marked unread again and detected in a new sync cycle.

This slice establishes the "notification as state, not event" model described in design note N-6, which is foundational for correct lifecycle management in all subsequent notification stories.

Covers epic sections: §7.9 (FR-34, FR-36).

## Acceptance criteria

- [ ] Each message has a tracked "notifying" state that records whether a notification is currently active for it
- [ ] Marking a message as read from within the application causes the corresponding notification to be dismissed (AC-10)
- [ ] Marking a message as read via a notification action causes the notification to be dismissed and the message to be marked read in the mailbox (AC-11)
- [ ] A dismissed notification is not re-displayed for the same message unless it is marked unread again and a new sync cycle detects it
- [ ] The notification state persists across the notification subsystem's internal operations (not yet across restarts — that is story 19)

## Blocked by

- Blocked by `1-notification-categories-and-basic-new-mail`

## User stories addressed

- US-7 (notification dismissed when message marked as read)
