## Parent Feature

#8.1 Desktop Notifications

## What to build

Define the application's fixed set of notification categories (Service/monitoring, New mail, Send status, Warning, Error, Alerts) with independent default urgency levels, and emit a basic desktop notification when a new message arrives in a folder with notifications enabled. By default only the Inbox has notifications enabled. Each notification must include at minimum the sender's display name, and optionally (per configuration) the subject, a plain-text body preview, account name, folder name, and received timestamp. The account and folder should be visually distinguishable so the user can tell work mail from personal mail at a glance. Warning, Error, and Alert categories bypass Do Not Disturb by default.

This is the foundational tracer-bullet slice: it proves that the application can define notification categories, detect a new message during sync, and emit a desktop notification that appears on the user's desktop — end to end.

Covers epic sections: §7.1 (FR-1, FR-2, FR-3), §7.2 (FR-4, FR-5, FR-6, FR-8).

## Acceptance criteria

- [ ] The application defines six notification categories (Service/monitoring, New mail, Send status, Warning, Error, Alerts) each with an independent default urgency
- [ ] Warning, Error, and Alert categories are configured to bypass Do Not Disturb by default
- [ ] When a new (previously unseen) message arrives in the Inbox (notifications enabled by default), a desktop notification is emitted
- [ ] The notification displays the sender's display name and received timestamp at minimum
- [ ] Subject, body preview, account name, and folder name are shown in the notification when their respective display options are enabled
- [ ] Notifications distinguish which account and folder the message belongs to (e.g. subtitle or label)
- [ ] Sent, Drafts, Trash, Spam, and Archive folders do NOT produce notifications by default
- [ ] After adding a new account and receiving a message in the Inbox with default settings, a desktop notification appears (AC-1)
- [ ] A message arriving in a folder with notifications disabled does not produce a notification (AC-2)

## Blocked by

None — can start immediately.

## User stories addressed

- US-1 (newcomer sees notification automatically after adding first account)
- US-2 (new-mail notification with sender, subject, preview)
- US-3 (notification identifies account and folder)
- US-8 (per-folder notification setting respected — default behavior)
- US-29 (notification shows received timestamp)

## Notes

- This slice does NOT include notification state tracking (story 2), grouping (story 5), sound (story 6), or configuration UI (story 7). It emits notifications but does not yet manage their lifecycle.
- The exact desktop notification transport (libnotify, D-Bus direct, etc.) is deliberately unspecified per NG6 / the epic's design-level abstraction. This story delivers the user-visible behavior regardless of transport.
- OQ-1 (notification daemon capability variance) is relevant here: this story should document how the application degrades when grouping or images are unsupported, even if full grouping support comes in a later story.
