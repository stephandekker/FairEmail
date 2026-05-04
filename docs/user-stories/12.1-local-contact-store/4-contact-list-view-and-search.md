## Parent Feature

#12.1 Local Contact Store

## What to build

Provide a contact list view that shows all local contacts for a given account. The list displays columns for: avatar/photo placeholder, name, email address, type indicator (sent-to / received-from / junk / no-junk), times contacted, last contacted, and favorite/ignored state (FR-19).

The contact list must be searchable by name and email address, using a case-insensitive substring match (FR-20).

The list must be fully navigable via keyboard and compatible with screen readers (NFR-7).

## Acceptance criteria

- [ ] A contact list view exists and is accessible from the application's navigation
- [ ] The list shows all local contacts for a selected account with the columns specified in FR-19
- [ ] Searching by name or email filters the list in real time (FR-20)
- [ ] The list is fully keyboard-navigable (NFR-7)
- [ ] The list handles an empty store gracefully (no errors, informative empty state)

## Blocked by

- Blocked by 1-contact-record-and-learn-on-send (requires contacts to exist in the store)

## User stories addressed

- US-13 (view all local contacts with metadata)
- US-14 (search contacts by name or email)
