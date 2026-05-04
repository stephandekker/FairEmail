## Parent Feature

#8.1 Desktop Notifications

## What to build

Provide an option to restrict new-mail notifications to known contacts only — senders whose address exists in the application's local contact store or the system address book. When enabled, messages from unknown senders do not produce notifications. Messages from known senders still notify per the normal configuration.

Covers epic sections: §7.11 (FR-43).

## Acceptance criteria

- [ ] An option to restrict new-mail notifications to known contacts only is available
- [ ] With the option enabled, a message from an address not in the contact store does not produce a notification (AC-13)
- [ ] With the option enabled, a message from a known contact address does produce a notification (AC-13)
- [ ] The known-contacts filter interacts correctly with the precedence chain (a per-sender override for an unknown sender should still be able to force notification)

## Blocked by

- Blocked by `1-notification-categories-and-basic-new-mail`

## User stories addressed

- US-9 (notifications only from known contacts)
