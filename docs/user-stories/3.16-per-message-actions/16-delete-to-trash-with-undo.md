## Parent Feature

#3.16 Per-Message Actions

## What to build

The default delete action moves a message to the account's Trash folder (rather than expunging). After deletion, a non-blocking undo action is available for the configurable grace period, reusing the undo infrastructure from the move story. Deleting a message already in the Trash folder permanently expunges it (FR-60, FR-62, FR-64).

## Acceptance criteria

- [ ] Deleting a message from the Inbox moves it to Trash (AC-17)
- [ ] After deletion, an undo option is available for the configured grace period
- [ ] Pressing undo returns the message to its original folder
- [ ] Deleting a message already in Trash permanently expunges it (AC-17)
- [ ] Delete-to-trash works from message list and single-message view

## Blocked by

- Blocked by 11-move-message-with-undo (reuses undo infrastructure)

## User stories addressed

- US-52 (delete moves to Trash by default)
- US-55 (delete from Trash expunges permanently)
- US-56 (undo for non-permanent deletions)
