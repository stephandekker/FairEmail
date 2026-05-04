## Parent Feature

#12.1 Local Contact Store

## What to build

Enable the user to bulk-clear all contacts of one or more types for a given account in a single action (FR-23). This covers both regular contact types (sent-to, received-from) and blocked contacts.

The bulk-clear for blocked contacts specifically addresses US-27 and AC-18: a single action to remove all blocked contacts for a selected account, accessible from the blocked contacts view.

A confirmation step should prevent accidental data loss.

## Acceptance criteria

- [ ] The user can bulk-clear all contacts of a specific type (sent-to, received-from, junk, no-junk) for a given account (FR-23, US-17)
- [ ] The user can bulk-clear all blocked contacts for a specific account in one action (US-27, AC-18)
- [ ] A confirmation prompt prevents accidental bulk deletion
- [ ] After bulk-clear, the contact list and autocomplete reflect the removal immediately

## Blocked by

- Blocked by 4-contact-list-view-and-search (requires the contact list UI)
- Blocked by 9-junk-and-no-junk-contacts (bulk-clear of blocked contacts requires the junk contact type)

## User stories addressed

- US-17 (bulk-clear contacts by type per account)
- US-27 (clear all blocked contacts for an account)
