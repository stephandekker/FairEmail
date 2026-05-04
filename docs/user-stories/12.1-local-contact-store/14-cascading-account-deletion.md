## Parent Feature

#12.1 Local Contact Store

## What to build

When an account is deleted from the application, all contacts associated with that account must be deleted from the local contact store (FR-36). This covers all contact types: sent-to, received-from, junk, and no-junk.

This is a data integrity safeguard — orphaned contacts for a deleted account should never remain in the store.

## Acceptance criteria

- [ ] Deleting an account removes all associated contacts from the store (AC-13)
- [ ] All contact types (sent-to, received-from, junk, no-junk) for the account are removed
- [ ] The deletion is atomic — no partial removal if the operation is interrupted
- [ ] After account deletion, autocomplete no longer returns contacts from the deleted account

## Blocked by

- Blocked by 1-contact-record-and-learn-on-send (requires the contact store)

## User stories addressed

- (FR-36 — cascading deletion is a functional requirement, not a user story in the epic, but it is validated by AC-13)

## Notes

This is a small but critical data integrity slice. It may be natural to implement it alongside the account deletion flow in the application. If account deletion is not yet implemented in the desktop app, this story should be deferred until account management exists — but the contact store schema should be designed to support efficient deletion by account from the start (story 1).
