## Parent Feature

#3.16 Per-Message Actions

## What to build

A "delete permanently" action that immediately and irrecoverably expunges a message from the server, bypassing trash. A confirmation dialog warns the user before proceeding, with a "don't ask again" option to suppress future confirmations (FR-61, FR-63).

## Acceptance criteria

- [ ] "Delete permanently" action is available alongside regular delete
- [ ] A confirmation dialog appears before permanent deletion (AC-17)
- [ ] The "don't ask again" checkbox suppresses future confirmation dialogs
- [ ] After confirmation, the message is hard-deleted and expunged from the server
- [ ] Permanent deletion has no undo (FR-61)
- [ ] The confirmation preference persists across restarts

## Blocked by

- Blocked by 16-delete-to-trash-with-undo

## User stories addressed

- US-53 (delete permanently action)
- US-54 (confirmation dialog with suppress option)
