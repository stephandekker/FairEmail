# Local Contact Store Management

## Parent Feature
#4.11 Recipient Autocomplete

## User Story
As any user, I want to view, search, and delete individual entries in the local contact store, and mark contacts as "ignored" so they never appear in autocomplete, so that I can manage my suggestion pool and remove outdated or unwanted entries.

## Blocked by
- `1-local-contact-store-with-sent-mail-learning`

## Acceptance Criteria
- A management screen is accessible from settings that displays all entries in the local contact store.
- The user can search the contact store by name or email address.
- The user can delete individual entries from the store.
- The user can mark an entry as "ignored," which excludes it from all future autocomplete suggestions without deleting the underlying data.
- An ignored contact does not appear in autocomplete suggestions from any data source that uses the local store.
- The management screen is keyboard-navigable and has appropriate screen-reader labels.

## HITL/AFK Classification
**HITL** — the management screen layout, search UX, and ignore/delete interactions should be reviewed by a human.

## Notes
- FR-17 and FR-18 define the view/search/delete and ignore behaviours.
- This story does not cover the system address book — only the application's local contact store.
- The existing Android codebase has an `AdapterContact` for displaying contacts in a list view, which can serve as conceptual reference.
