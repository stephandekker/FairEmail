# Sort and Filter in Unified Inbox

## Parent Feature

#3.1 Unified Inbox

## What to build

Expose all sort orders available in a single-folder view (date, sender, subject, size, attachment count, importance, etc., ascending/descending) in the Unified Inbox (FR-30). Expose all list filters available in a folder view (read/unread, flagged/unflagged, has-attachment, snoozed, deleted, sent, language, duplicates, etc.) in the Unified Inbox (FR-32). The Unified Inbox's chosen sort order and filter set must be persisted independently of any individual folder's sort/filter (FR-31).

## Acceptance criteria

- [ ] All sort orders available in a folder view are available in the Unified Inbox.
- [ ] All list filters available in a folder view are available in the Unified Inbox (US-23).
- [ ] The Unified Inbox's sort order persists across application restart, independently of any folder's sort (AC-13).
- [ ] The Unified Inbox's filter set persists across restart, independently of any folder's filter (AC-13, US-24).
- [ ] Changing sort/filter in the Unified Inbox does not affect any individual folder's sort/filter, and vice versa.

## Blocked by

- Blocked by `4-basic-unified-message-list`

## User stories addressed

- US-23 (same filters as folder view)
- US-24 (filter set remembered separately)
- US-25 (sort options with independent persistence)
