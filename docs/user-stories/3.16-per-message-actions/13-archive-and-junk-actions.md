## Parent Feature

#3.16 Per-Message Actions

## What to build

Two specialised move actions: **Archive** moves to the account's designated Archive folder, inheriting all move behaviours (undo, thread scope, auto-read, etc.). **Move to Junk** moves to the account's Junk/Spam folder, with an optional setting to automatically block the sender when junking a message (FR-55, FR-56).

## Acceptance criteria

- [ ] Archive action moves the message to the account's designated Archive folder
- [ ] Archive inherits undo grace period, thread scope, and all move auto-actions
- [ ] Junk action moves the message to the account's Junk/Spam folder
- [ ] With "auto-block sender" enabled, junking a message adds the sender to the block list
- [ ] Both actions are available from the same surfaces as the regular move action

## Blocked by

- Blocked by 11-move-message-with-undo

## User stories addressed

- US-49 (archive behaves as a move to Archive folder)
