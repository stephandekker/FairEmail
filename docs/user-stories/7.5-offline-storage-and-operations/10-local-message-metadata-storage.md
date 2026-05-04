# Local Message Metadata Storage for Offline List View

## Parent Feature
#7.5 Offline Storage and Operations

## User Story
As any user, I want message previews and metadata (sender, subject, date, flags) to always be available locally for messages within my configured sync window, so that the message list is always usable even when offline.

## Acceptance Criteria
- Message metadata (sender, recipients, subject, date, size, flags, preview text) is stored locally for all messages within the configured sync window.
- The message list (with previews, sender, subject, date, flags) is fully usable offline for synced messages.
- Metadata persists across application restarts.
- Local metadata reflects the user's most recent actions (optimistic state) when operations are pending.

## Complexity
Medium

## Blocked by
(none — independent of the operation queue stories)

## HITL/AFK
AFK

## Notes
- This story is about *metadata* only — message bodies and attachments are separate stories. The metadata must be sufficient to render a fully functional message list view.
- NFR-8 (storage efficiency) applies: local storage must respect the user's configured retention settings and not grow unbounded.
- This story overlaps with sync concerns (feature 2.1 two-way IMAP synchronization). The boundary is: sync *fetches* the metadata; this story ensures it is *stored durably and served offline*.
