## Parent Feature

#3.16 Per-Message Actions

## What to build

Allow users to edit the subject line of a received message. The operation creates a replacement message on the server (IMAP APPEND with modified subject, then deletion of the original), preserving conversation thread linkage via original message identifiers. The edited message is re-indexed for search. This action is gated behind an "experimental features" preference and is only available for IMAP accounts with write access to non-read-only folders (FR-30 through FR-33, Design Note N-6).

## Acceptance criteria

- [ ] User can edit the subject of a message via an edit dialog
- [ ] Edited subject is displayed in the message list and message view (AC-9)
- [ ] Conversation threading is preserved after subject edit (AC-9)
- [ ] The original message is removed from the server and replaced (AC-9)
- [ ] The edited message is searchable by its new subject
- [ ] Subject editing is only available when experimental features are enabled
- [ ] Subject editing is hidden for POP3 accounts and read-only folders

## Blocked by

None — can start immediately.

## User stories addressed

- US-26 (edit subject line of received message)
- US-27 (replacement message preserves thread linkage)
- US-28 (available only when server supports it)

## Notes

- Open question OQ-3: whether to show an explicit warning dialog explaining server-side implications (beyond the experimental gate) is unresolved. This story implements the experimental gate; adding an additional warning is a UX decision for implementation time.
