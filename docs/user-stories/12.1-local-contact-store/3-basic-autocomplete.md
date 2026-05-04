## Parent Feature

#12.1 Local Contact Store

## What to build

When the user starts typing in the To, Cc, or Bcc field of a new message, the application queries the local contact store and suggests matching contacts. This is the first end-to-end integration of the contact store into the compose workflow.

Matching uses a case-insensitive substring match against both the email address and the display name (FR-13). Results are sorted by: (1) times-contacted descending, (2) last-contacted descending (a simplified initial ranking; story 6 adds favorite-first and avatar ordering).

Autocomplete must return results within 200 ms on a store containing 50,000 contacts (NFR-1, AC-19). If the store is empty, autocomplete returns no suggestions rather than failing (NFR-6).

## Acceptance criteria

- [ ] Typing the first few characters of a known contact's name or email in the compose To/Cc/Bcc field produces suggestions (AC-4)
- [ ] Matching is case-insensitive and works on both name and email address (FR-13)
- [ ] Results are sorted by times-contacted descending, then last-contacted descending
- [ ] Autocomplete returns results within 200 ms on a store of 50,000 contacts (AC-19)
- [ ] An empty contact store produces no suggestions without errors (NFR-6)

## Blocked by

- Blocked by 1-contact-record-and-learn-on-send (requires contacts to exist in the store)

## User stories addressed

- US-7 (autocomplete from local store, match on name and email)

## Notes

The full autocomplete ranking (favorites first, avatar-holders second) is deferred to story 6. This story delivers the basic end-to-end path: type → query → suggest. Story 7 adds source filtering, account scoping, and deduplication.
