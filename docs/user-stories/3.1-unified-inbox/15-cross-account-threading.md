# Cross-Account Conversation Threading

## Parent Feature

#3.1 Unified Inbox

## What to build

When two or more messages in the Unified Inbox belong to the same conversation thread — even if they reside in different accounts — they must be grouped together according to the application-wide threading mode (FR-19, US-13). Threading is determined by `Message-ID` / `In-Reply-To` / `References` headers and is an application-wide concern independent of the Unified Inbox feature itself; this slice ensures the Unified Inbox view correctly consumes that threading.

## Acceptance criteria

- [ ] Two messages from different accounts that share a conversation thread appear grouped together in the Unified Inbox (AC-16).
- [ ] Threading respects the application-wide threading mode setting.
- [ ] Actions on one message in a cross-account thread (e.g. mark read) do not incorrectly propagate to the other account's copy.

## Blocked by

- Blocked by `4-basic-unified-message-list`

## User stories addressed

- US-13 (cross-account conversation grouping)

## Notes

Open question OQ-3 in the epic flags that "by subject only" threading (no Message-ID chain) may produce spurious cross-account merges. This story should implement threading based on Message-ID chains. Whether subject-only threading is safe for cross-account merging is a design decision that may need a HITL review.
