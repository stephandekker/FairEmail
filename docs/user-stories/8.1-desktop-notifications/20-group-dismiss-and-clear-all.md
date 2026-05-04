## Parent Feature

#8.1 Desktop Notifications

## What to build

Support dismissing individual or grouped notifications via standard desktop gestures (click dismiss / swipe away). When a notification is dismissed, the application records the dismissal so it is not re-displayed. Provide a "clear all" action on group summary notifications that dismisses all individual notifications in the group. Allow the user to dismiss all new-mail notifications for a group (account or unified) in one action.

Covers epic sections: §7.9 (FR-34 — dismiss recording, FR-38).

## Acceptance criteria

- [ ] Individual notifications can be dismissed by standard desktop gesture
- [ ] Dismissed notifications are recorded and not re-displayed for the same message
- [ ] A "clear all" action on a group summary dismisses all individual notifications in that group
- [ ] The user can dismiss all new-mail notifications for an account or unified group in one action
- [ ] Dismissal state integrates with the notification state tracking from story 2

## Blocked by

- Blocked by `5-notification-grouping-and-summarization`
- Blocked by `2-notification-state-tracking-and-read-dismissal`

## User stories addressed

- US-33 (dismiss individual or grouped notifications, application records dismissal)
- US-34 (dismiss all new-mail notifications for a group in one action)
