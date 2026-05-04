## Parent Feature

#12.1 Local Contact Store

## What to build

From the contact list view (story 4), enable the user to edit a contact's email, name, type, and group assignment, and to delete individual contacts.

Edits must persist across application restart (AC-12). The uniqueness constraint must be respected — changing a contact's email or type must not create a conflict with an existing record (FR-2).

Deletion removes the contact from the store entirely. This is distinct from "ignored" state (which keeps the record but suppresses autocomplete).

## Acceptance criteria

- [ ] The user can edit a contact's name, email, type, and group from the contact list (FR-21)
- [ ] Edits persist across application restart (AC-12)
- [ ] An edit that would violate the uniqueness constraint (account + type + email) is rejected with a clear message
- [ ] The user can delete an individual contact from the contact list (FR-22)
- [ ] A deleted contact no longer appears in the contact list or in autocomplete

## Blocked by

- Blocked by 4-contact-list-view-and-search (requires the contact list UI)

## User stories addressed

- US-15 (edit contact name, email, type, group)
- US-16 (delete individual contacts)
