## Parent Feature

#8.1 Desktop Notifications

## What to build

When a synchronization, connection, or authentication error occurs, emit a desktop notification on the appropriate category — Warning for retryable errors, Error for permanent failures. The notification must include the affected account name, a human-readable error description, and (for retryable errors) the number of remaining retries and whether automatic retry is scheduled. Error and warning notifications are high-urgency and may bypass Do Not Disturb per the category defaults. When the error condition clears (e.g. successful reconnection), the notification is auto-dismissed. The user can manually dismiss an error notification without affecting the retry mechanism.

Covers epic sections: §7.5 (FR-18, FR-19, FR-20, FR-21).

## Acceptance criteria

- [ ] An authentication failure produces a high-urgency notification naming the affected account and describing the error (AC-6)
- [ ] The authentication-failure notification persists until dismissed by the user or until the error clears
- [ ] A transient connection error produces a warning notification showing the retry count (AC-7)
- [ ] When the connection recovers, the warning notification is auto-dismissed (AC-7)
- [ ] Error and warning notifications use their respective notification categories with high urgency
- [ ] The user can dismiss an error/warning notification without affecting automatic retry
- [ ] Error/warning notifications may bypass Do Not Disturb (per category default from story 1)

## Blocked by

- Blocked by `1-notification-categories-and-basic-new-mail`

## User stories addressed

- US-11 (sync/connection error notification with account name and description)
- US-12 (error notification shows retry information)
- US-13 (error notifications are high-urgency, may bypass DND)
- US-14 (error notifications auto-dismissed when problem resolves)
