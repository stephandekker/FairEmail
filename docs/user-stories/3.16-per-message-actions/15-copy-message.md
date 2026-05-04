## Parent Feature

#3.16 Per-Message Actions

## What to build

Allow users to copy one or more messages to any folder within the same account, leaving the original in place. Also support cross-account copies using the same fetch-and-add mechanism as cross-account moves but without removing the original (FR-57, FR-58).

## Acceptance criteria

- [ ] User can copy a message to another folder in the same account via a folder picker
- [ ] The message exists in both the original and target folders after copying (AC-16)
- [ ] User can copy a message to a folder in a different account
- [ ] Cross-account copy does not remove the original message
- [ ] Copy action is available from the same surfaces as the move action

## Blocked by

None — can start immediately.

## User stories addressed

- US-50 (copy to another folder, leaving original in place)
- US-51 (copy to a folder in a different account)
