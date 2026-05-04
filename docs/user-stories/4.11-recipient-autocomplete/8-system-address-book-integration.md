# System Address Book Integration

## Parent Feature
#4.11 Recipient Autocomplete

## User Story
As any user who has granted system address-book access, I want autocomplete suggestions to include contacts from the system address book (Evolution Data Server, KAddressBook, or equivalent), so that I can use my existing desktop contacts without re-entering them.

## Blocked by
- `2-basic-autocomplete-from-sent-contacts`
- `7-ranking-modes`

## Acceptance Criteria
- When the user has granted address-book read permission, the autocomplete system queries the system address book for matches in addition to the local store.
- System address-book contacts appear in the suggestion dropdown with display name, email address, and avatar (if available).
- Contacts marked as starred/favorite in the system address book are ranked at the top of suggestions regardless of ranking mode.
- When the same email address exists in both the local store and the system address book, only one entry appears in the dropdown (deduplicated by email address, case-insensitive), preferring the version that has an avatar.
- If address-book permission has not been granted, the system address-book source is silently skipped with no error or prompt.
- If the system address book is unavailable (e.g. Evolution Data Server not running), autocomplete functions normally using only the local store, with no error messages.
- In source-priority ranking mode, system address-book contacts rank above local contacts.
- System address-book data is cached locally so that autocomplete works fully offline.
- The feature handles contact stores with 50,000+ entries without perceptible delay.

## HITL/AFK Classification
**HITL** — the system address-book integration path on Linux (EDS vs. KAddressBook vs. portal-based) needs design-time decisions and manual testing on multiple desktop environments.

## Notes
- OQ-7 in the epic flags that the exact integration path on Linux needs to be determined during design. This story should document the chosen approach.
- N-3 explains the deduplication-prefers-avatars rationale.
- N-5 explains the starred-contacts ranking boost.
- FR-37/FR-38 define the permission-gated behaviour.
