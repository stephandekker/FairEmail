## Parent Feature

#3.16 Per-Message Actions

## What to build

Allow users to selectively delete individual attachments from a message. A dialog lists all attachments with checkboxes (all pre-selected by default); the user unchecks attachments to keep. Deleting attachments creates a replacement message on the server with removed attachments replaced by empty placeholder stubs that retain the original filename (prefixed to indicate deletion) and content type. Conversation threading is preserved. Available only for IMAP accounts with write access (FR-66 through FR-70, Design Note N-10).

## Acceptance criteria

- [ ] Attachment deletion dialog lists all attachments with checkboxes
- [ ] All attachments are pre-selected by default; user unchecks to keep
- [ ] Deleting 2 of 3 attachments results in 2 stubs and 1 intact attachment on server (AC-19)
- [ ] Placeholder stubs retain original filename (prefixed) and content type
- [ ] Conversation threading is preserved after attachment deletion
- [ ] Action is hidden for POP3 accounts and read-only folders
- [ ] Operation is destructive and not undoable once committed (FR-70)

## Blocked by

None — can start immediately.

## User stories addressed

- US-58 (selectively delete individual attachments)
- US-59 (dialog with checkboxes for granular control)
- US-60 (replacement message preserves threading)
- US-61 (available only when server supports it)

## Notes

- Open question OQ-8: whether to implement an undo grace period for attachment deletion (holding the original locally before committing) is unresolved. This story implements a confirmation dialog without undo, matching the source application.
