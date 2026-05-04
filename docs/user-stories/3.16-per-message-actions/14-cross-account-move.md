## Parent Feature

#3.16 Per-Message Actions

## What to build

Extend the move action to support cross-account moves. A cross-account move fetches the raw message from the source account and adds it to the target account as a new message, then removes it from the source. A confirmation dialog warns the user that this is a cross-account operation before proceeding (FR-52).

## Acceptance criteria

- [ ] User can select a folder in a different account from the folder picker
- [ ] A confirmation dialog warns about the cross-account nature of the operation (AC-14)
- [ ] After confirmation, the message appears in the target account's folder (AC-14)
- [ ] The message is removed from the source account after successful transfer (AC-14)
- [ ] Cross-account move works with the undo grace period

## Blocked by

- Blocked by 11-move-message-with-undo

## User stories addressed

- US-45 (move to folder in different account with confirmation)

## Notes

- Open question OQ-6: whether to warn users that cross-account move produces a new server identity is unresolved. This story implements a confirmation dialog that acknowledges the cross-account nature; whether to mention the new-identity detail in the dialog text is a UX decision to be resolved during implementation.
